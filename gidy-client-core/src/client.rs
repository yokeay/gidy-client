use crate::config::ClientConfig;
use crate::stats::TrafficStats;
use gidy_core::{
    AuthStatus, DataCmd, DpmParams, Frame, FrameCodec,
    build_auth_fragments,
    current_epoch, deobfuscate, derive_auth_key, derive_obfs_key, is_gidy_packet, obfuscate,
};
use quinn::{Endpoint, RecvStream};
use rand::SeedableRng;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub struct GidyClient {
    config: Arc<ClientConfig>,
    psk: [u8; 32],
    obfs_key: [u8; 32],
    #[allow(dead_code)]
    auth_key: [u8; 32],
    stats: Arc<TrafficStats>,
}

pub struct Tunnel {
    quic: Arc<quinn::Connection>,
    stream_id: u16,
    seq: Mutex<u32>,
    dpm: DpmParams,
    codec: FrameCodec,
    obfs_key: [u8; 32],
    stats: Arc<TrafficStats>,
}

impl GidyClient {
    pub fn new(config: ClientConfig, stats: Arc<TrafficStats>) -> Result<Self, String> {
        let psk = config.psk()?;
        let obfs_key = derive_obfs_key(&psk);
        let auth_key = derive_auth_key(&psk);
        Ok(Self {
            config: Arc::new(config),
            psk,
            obfs_key,
            auth_key,
            stats,
        })
    }

    pub async fn connect(&self) -> Result<Connection, String> {
        let _ = rustls::crypto::ring::default_provider().install_default();

        let server_addr = self.config.server_addr.clone();
        let server_name = self.config.server_name.clone();

        let roots = rustls::RootCertStore::empty();
        let mut client_crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        client_crypto
            .dangerous()
            .set_certificate_verifier(Arc::new(NoVerify));

        let mut transport = quinn::TransportConfig::default();
        transport.keep_alive_interval(Some(std::time::Duration::from_secs(15)));

        let quic_client_config = quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto)
            .map_err(|e| format!("quic crypto config: {}", e))?;
        let mut client_config = quinn::ClientConfig::new(Arc::new(quic_client_config));
        client_config.transport_config(Arc::new(transport));

        let endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())
            .map_err(|e| format!("failed to create client endpoint: {}", e))?;

        let addr = server_addr
            .parse()
            .map_err(|e| format!("invalid server addr {}: {}", server_addr, e))?;

        info!("connecting to gidy-server at {}...", addr);
        let quic_conn = endpoint
            .connect_with(client_config, addr, &server_name)
            .map_err(|e| format!("failed to connect: {}", e))?
            .await
            .map_err(|e| format!("connection error: {}", e))?;

        info!("QUIC connection established");

        let (session_id, dpm, matched_epoch) =
            self.authenticate(&quic_conn).await?;

        info!(
            "auth success: session {:02x?}, epoch {}",
            &session_id[..4],
            matched_epoch
        );

        Ok(Connection {
            quic: Arc::new(quic_conn),
            session_id,
            dpm,
            matched_epoch,
            codec: FrameCodec::new(),
            obfs_key: self.obfs_key,
            stats: self.stats.clone(),
            next_stream_id: Mutex::new(1),
        })
    }

    async fn authenticate(
        &self,
        conn: &quinn::Connection,
    ) -> Result<([u8; 16], DpmParams, u64), String> {
        let epoch = current_epoch();
        let dpm = DpmParams::derive(&self.psk, epoch, 4)
            .map_err(|e| format!("dpm derive: {}", e))?;

        let client_challenge = {
            let mut c = [0u8; 32];
            let t = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let seed = t.as_nanos().to_le_bytes();
            let hash = blake3::hash(&seed);
            c.copy_from_slice(hash.as_bytes());
            c
        };

        info!("building auth fragments...");
        let (fragments, _interval) =
            build_auth_fragments(&dpm, &client_challenge, 0x01, epoch, 100);

        let codec = FrameCodec::new();

        for (i, frag) in fragments.iter().enumerate() {
            let encoded = {
                let mut rng = rand_chacha::ChaCha20Rng::from_seed({
                    let mut seed = [0u8; 32];
                    seed[..8].copy_from_slice(&i.to_le_bytes());
                    seed
                });
                codec.encode(frag, &dpm, &mut rng)
            };
            let obfuscated = obfuscate(&self.obfs_key, &encoded);

            let (mut send, _recv) = conn
                .open_bi()
                .await
                .map_err(|e| format!("open bi for auth frag {}: {}", i, e))?;
            send.write_all(&obfuscated)
                .await
                .map_err(|e| format!("write auth frag {}: {}", i, e))?;
            send.finish().unwrap();
        }

        info!("sent {} auth fragments, waiting for response...", fragments.len());

        let auth_deadline =
            tokio::time::Instant::now() + std::time::Duration::from_secs(15);

        loop {
            if tokio::time::Instant::now() > auth_deadline {
                return Err("auth response timeout".into());
            }

            match conn.accept_bi().await {
                Ok((_send, recv)) => {
                    let frame_data = read_stream_full(recv, 4096).await?;

                    let raw = if is_gidy_packet(&frame_data) {
                        deobfuscate(&self.obfs_key, &frame_data)
                            .map_err(|e| format!("deobfuscate auth resp: {}", e))?
                    } else {
                        frame_data.to_vec()
                    };

                    let mut decoded: Option<Frame> = None;
                    let candidate_epochs = [epoch, epoch.saturating_sub(1), epoch + 1];

                    for &cand_epoch in &candidate_epochs {
                        if let Ok(cand_dpm) = DpmParams::derive(&self.psk, cand_epoch, 4) {
                            if let Ok(frame) = codec.decode(&raw, &cand_dpm) {
                                decoded = Some(frame);
                                break;
                            }
                        }
                    }

                    if let Some(Frame::AuthResp {
                        status,
                        assigned_bw_kbps,
                        session_id,
                        server_epoch,
                        ..
                    }) = decoded
                    {
                        if status != AuthStatus::Ok {
                            return Err(format!("auth denied: {:?}", status));
                        }

                        info!(
                            "auth OK: bw={}kbps, epoch={}",
                            assigned_bw_kbps, server_epoch
                        );

                        let matched_dpm =
                            DpmParams::derive(&self.psk, server_epoch, 4)
                                .map_err(|e| format!("dpm server epoch: {}", e))?;

                        return Ok((session_id, matched_dpm, server_epoch));
                    }
                }
                Err(e) => return Err(format!("accept bi during auth: {}", e)),
            }
        }
    }
}

pub struct Connection {
    quic: Arc<quinn::Connection>,
    pub session_id: [u8; 16],
    pub dpm: DpmParams,
    pub matched_epoch: u64,
    codec: FrameCodec,
    obfs_key: [u8; 32],
    stats: Arc<TrafficStats>,
    next_stream_id: Mutex<u16>,
}

impl Connection {
    /// Returns (Tunnel, initial_response_data)
    pub async fn open_tunnel(&self, target: &str) -> Result<(Tunnel, bytes::Bytes), String> {
        let stream_id = {
            let mut id = self.next_stream_id.lock().await;
            let sid = *id;
            *id = id.wrapping_add(1);
            sid
        };

        let frame = Frame::DataReq {
            stream_id,
            seq_num: 0,
            cmd: DataCmd::Connect,
            payload: bytes::Bytes::copy_from_slice(target.as_bytes()),
        };

        let (mut send, recv) = self
            .quic
            .open_bi()
            .await
            .map_err(|e| format!("open bi for tunnel: {}", e))?;

        let encoded = {
            let mut rng = rand_chacha::ChaCha20Rng::from_seed({
                let mut seed = [0u8; 32];
                seed[..2].copy_from_slice(&stream_id.to_le_bytes());
                seed
            });
            self.codec.encode(&frame, &self.dpm, &mut rng)
        };

        info!(
            "open_tunnel: encoded={}B opcodes: frag={:02x} resp={:02x} req={:02x} data_resp={:02x} ping={:02x} rst={:02x} | enc: sid_be={} sn_be={} vsid={} vseq={} | lpmode={} | epoch={}",
            encoded.len(),
            self.dpm.opcodes.auth_frag, self.dpm.opcodes.auth_resp, self.dpm.opcodes.data_req, self.dpm.opcodes.data_resp, self.dpm.opcodes.ping, self.dpm.opcodes.rst,
            self.dpm.encoding.stream_id_be, self.dpm.encoding.seq_num_be, self.dpm.encoding.use_varint_stream_id, self.dpm.encoding.use_varint_seq,
            self.dpm.len_prefix_mode,
            self.matched_epoch,
        );
        info!("open_tunnel: encoded hex[0..64]: {:02x?}", &encoded[..encoded.len().min(64)]);

        let obfuscated = obfuscate(&self.obfs_key, &encoded);

        send.write_all(&obfuscated)
            .await
            .map_err(|e| format!("write connect req: {}", e))?;
        send.finish().unwrap();

        self.stats.add_up(obfuscated.len() as u64);

        let resp_data = read_stream_full(recv, 65536).await?;
        let raw = if is_gidy_packet(&resp_data) {
            deobfuscate(&self.obfs_key, &resp_data)
                .map_err(|e| format!("deobfuscate resp: {}", e))?
        } else {
            resp_data.to_vec()
        };

        let resp_frame = self.codec.decode(&raw, &self.dpm)
            .map_err(|e| format!("decode resp: {}", e))?;

        match resp_frame {
            Frame::DataResp {
                status,
                payload,
                ..
            } => {
                if status != 0 {
                    let msg = String::from_utf8_lossy(&payload);
                    return Err(format!("connect failed: {}", msg));
                }
                self.stats.add_down(raw.len() as u64);

                let init_data = payload;
                Ok((Tunnel {
                    quic: self.quic.clone(),
                    stream_id,
                    seq: Mutex::new(1),
                    dpm: self.dpm.clone(),
                    codec: self.codec.clone(),
                    obfs_key: self.obfs_key,
                    stats: self.stats.clone(),
                }, init_data))
            }
            _ => Err("unexpected response frame".into()),
        }
    }
}

impl Tunnel {
    /// Send data and receive response. Each call opens a new bi-stream.
    pub async fn send(&self, data: &[u8]) -> Result<bytes::Bytes, String> {
        let seq = {
            let mut s = self.seq.lock().await;
            let current = *s;
            *s = s.wrapping_add(1);
            current
        };

        let frame = Frame::DataReq {
            stream_id: self.stream_id,
            seq_num: seq,
            cmd: DataCmd::Data,
            payload: bytes::Bytes::copy_from_slice(data),
        };

        let encoded = {
            let mut rng = rand_chacha::ChaCha20Rng::from_seed({
                let mut seed = [0u8; 32];
                seed[..4].copy_from_slice(&seq.to_le_bytes());
                seed
            });
            self.codec.encode(&frame, &self.dpm, &mut rng)
        };
        let obfuscated = obfuscate(&self.obfs_key, &encoded);

        tracing::debug!("tunnel.send: opening bi-stream stream={} seq={} len={}", self.stream_id, seq, obfuscated.len());
        let (mut send, mut recv) = self
            .quic
            .open_bi()
            .await
            .map_err(|e| format!("tunnel open bi: {}", e))?;

        tracing::debug!("tunnel.send: writing {} bytes", obfuscated.len());
        send.write_all(&obfuscated)
            .await
            .map_err(|e| format!("tunnel send: {}", e))?;
        send.finish().unwrap();
        self.stats.add_up(obfuscated.len() as u64 + data.len() as u64);

        // Read streamed chunks: each chunk is [u32 LE len][obfuscated DataResp frame]
        let mut all_data: Vec<u8> = Vec::new();
        let mut chunk_count = 0u32;
        loop {
            let mut len_buf = [0u8; 4];
            match read_exact(&mut recv, &mut len_buf).await {
                Ok(()) => {}
                Err(e) => {
                    tracing::info!("tunnel.send: read_exact len_buf failed: {} (total so far: {} bytes in {} chunks)", e, all_data.len(), chunk_count);
                    break;
                }
            }
            let chunk_len = u32::from_le_bytes(len_buf) as usize;
            if chunk_len == 0 || chunk_len > 64 * 1024 * 1024 {
                tracing::info!("tunnel.send: bad chunk_len={}, breaking (total so far: {} bytes in {} chunks)", chunk_len, all_data.len(), chunk_count);
                break;
            }

            let mut chunk = vec![0u8; chunk_len];
            read_exact(&mut recv, &mut chunk).await
                .map_err(|e| format!("tunnel read chunk: {}", e))?;

            let raw = if is_gidy_packet(&chunk) {
                deobfuscate(&self.obfs_key, &chunk)
                    .map_err(|e| format!("tunnel deobfuscate: {}", e))?
            } else {
                chunk
            };

            let frame = self.codec.decode(&raw, &self.dpm)
                .map_err(|e| format!("tunnel decode chunk: {}", e))?;

            match frame {
                Frame::DataResp { status, payload, stream_id: sid, seq_num: sn } => {
                    if status != 0 {
                        return Err(format!("data resp error: {}", status));
                    }
                    if payload.is_empty() {
                        tracing::info!("tunnel.send: end-of-stream marker (sid={} sn={}), total {} bytes in {} chunks", sid, sn, all_data.len(), chunk_count);
                        break;
                    }
                    tracing::info!("tunnel.send: chunk #{} wire_len={} payload_len={} sid={} sn={} running_total={}", chunk_count, chunk_len, payload.len(), sid, sn, all_data.len() + payload.len());
                    all_data.extend_from_slice(&payload);
                    chunk_count += 1;
                }
                Frame::Rst { reason, .. } => {
                    return Err(format!("stream reset: reason={}", reason));
                }
                _ => {
                    tracing::info!("tunnel.send: unexpected frame type, ignoring");
                }
            }
        }

        tracing::info!("tunnel.send: DONE received {} bytes total in {} chunks", all_data.len(), chunk_count);
        self.stats.add_down(all_data.len() as u64);
        Ok(bytes::Bytes::from(all_data))
    }

    pub async fn close(&self) -> Result<(), String> {
        let frame = Frame::DataReq {
            stream_id: self.stream_id,
            seq_num: {
                let mut s = self.seq.lock().await;
                let current = *s;
                *s = s.wrapping_add(1);
                current
            },
            cmd: DataCmd::Close,
            payload: bytes::Bytes::new(),
        };

        let encoded = {
            let mut rng = rand_chacha::ChaCha20Rng::from_seed([0xCC; 32]);
            self.codec.encode(&frame, &self.dpm, &mut rng)
        };
        let obfuscated = obfuscate(&self.obfs_key, &encoded);

        let (mut send, _recv) = self
            .quic
            .open_bi()
            .await
            .map_err(|e| format!("tunnel close open bi: {}", e))?;

        let _ = send.write_all(&obfuscated).await;
        let _ = send.finish();
        Ok(())
    }
}

#[derive(Debug)]
struct NoVerify;

impl rustls::client::danger::ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer,
        _intermediates: &[rustls::pki_types::CertificateDer],
        _server_name: &rustls::pki_types::ServerName,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

async fn read_stream_full(mut recv: RecvStream, max_bytes: usize) -> Result<bytes::Bytes, String> {
    let mut buf = Vec::new();
    loop {
        let mut chunk = vec![0u8; 4096];
        match recv.read(&mut chunk).await {
            Ok(Some(n)) => {
                buf.extend_from_slice(&chunk[..n]);
                if buf.len() >= max_bytes {
                    break;
                }
            }
            Ok(None) => break,
            Err(e) => {
                if buf.is_empty() {
                    return Err(format!("read error: {}", e));
                }
                break;
            }
        }
    }
    Ok(bytes::Bytes::from(buf))
}

/// Read exactly `buf.len()` bytes from the stream, handling partial reads.
async fn read_exact(recv: &mut RecvStream, buf: &mut [u8]) -> Result<(), String> {
    let mut offset = 0;
    while offset < buf.len() {
        match recv.read(&mut buf[offset..]).await {
            Ok(Some(n)) if n == 0 && offset == 0 => return Err("stream eof".into()),
            Ok(Some(0)) => return Err("stream finished".into()),
            Ok(Some(n)) => offset += n,
            Ok(None) => return Err("stream finished".into()),
            Err(e) => return Err(format!("read error: {}", e)),
        }
    }
    Ok(())
}

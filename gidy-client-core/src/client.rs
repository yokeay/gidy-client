use crate::config::ClientConfig;
use crate::stats::TrafficStats;
use bytes::{Buf, Bytes};
use gidy_core::{
    Credentials, DpmParams, EntropyEngine, H3_ALPN, KeyChain,
    LogEvent, PSEUDO_HOST_CHECK, Session, SessionLogger, TokenBucket,
    current_epoch,
};
use gidy_core::keys::hmac_challenge;
use parking_lot::Mutex;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex as AsyncMutex;
use tracing::info;

pub struct GidyClient {
    config: Arc<ClientConfig>,
    psk: [u8; 32],
    stats: Arc<TrafficStats>,
}

impl GidyClient {
    pub fn new(config: ClientConfig, stats: Arc<TrafficStats>) -> Result<Self, String> {
        let psk = config.psk()?;
        Ok(Self {
            config: Arc::new(config),
            psk,
            stats,
        })
    }

    pub async fn connect(&self) -> Result<Connection, String> {
        let _ = rustls::crypto::ring::default_provider().install_default();

        let roots = rustls::RootCertStore::empty();
        let mut client_crypto = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        client_crypto
            .dangerous()
            .set_certificate_verifier(Arc::new(NoVerify));
        client_crypto.alpn_protocols = vec![H3_ALPN.to_vec()];

        let mut transport = quinn::TransportConfig::default();
        transport.keep_alive_interval(Some(Duration::from_secs(15)));

        let quic_client_config = quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto)
            .map_err(|e| format!("quic crypto: {}", e))?;
        let mut client_config = quinn::ClientConfig::new(Arc::new(quic_client_config));
        client_config.transport_config(Arc::new(transport));

        let endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap())
            .map_err(|e| format!("client endpoint: {}", e))?;

        let addr: std::net::SocketAddr = self
            .config
            .server_addr
            .parse()
            .map_err(|e| format!("invalid server addr: {}", e))?;

        info!("connecting to {}...", addr);
        let quinn_conn = endpoint
            .connect_with(client_config, addr, &self.config.server_name)
            .map_err(|e| format!("connect: {}", e))?
            .await
            .map_err(|e| format!("QUIC: {}", e))?;

        info!("QUIC connected, setting up h3...");

        let h3_conn = h3_quinn::Connection::new(quinn_conn);
        let (_h3_driver, send_request) = h3::client::builder()
            .build(h3_conn)
            .await
            .map_err(|e| format!("h3 init: {}", e))?;

        tokio::spawn(async move { let _ = _h3_driver; });

        info!("h3 client ready (ALPN: h3)");

        let keychain = match &self.config.keychain_path {
            Some(path) if path.exists() => {
                match KeyChain::load(path) {
                    Ok(kc) => { info!("keychain loaded from {:?}", path); kc }
                    Err(e) => { info!("keychain load failed, creating new: {}", e); KeyChain::new(&self.psk) }
                }
            }
            _ => KeyChain::new(&self.psk),
        };

        let epoch = current_epoch();
        let entropy = EntropyEngine::new();
        let dpm = DpmParams::derive(&self.psk, epoch, entropy.profile_count())
            .unwrap_or_else(|_| DpmParams::derive(&self.psk, 0, 4).unwrap());

        let bandwidth = if self.config.bandwidth_kbps > 0 {
            TokenBucket::new(self.config.bandwidth_kbps)
        } else {
            TokenBucket::unlimited()
        };

        let session_id: String = blake3::hash(format!("{}-{}", addr, epoch).as_bytes())
            .as_bytes()[0..8]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        let logger: Arc<SessionLogger> = Arc::new(match &self.config.log_dir {
            Some(dir) => {
                SessionLogger::new(self.config.gidy_log_level(), dir, &session_id)
                    .unwrap_or_else(|_| SessionLogger::off())
            }
            None => SessionLogger::off(),
        });

        let bw_kbps = self.config.bandwidth_kbps;
        let session = Arc::new(Mutex::new(Session::new(addr, bw_kbps, epoch, dpm.clone())));

        {
            let s = session.lock();
            logger.record(&LogEvent::SessionOpen {
                session_id: session_id.clone(),
                client_addr: addr.to_string(),
                epoch,
                profile: format!("profile-{}", s.dpm.profile_index),
                bw_kbps,
            });
        }

        Ok(Connection {
            send_request: Arc::new(AsyncMutex::new(send_request)),
            psk: self.psk,
            stats: self.stats.clone(),
            keychain: Arc::new(Mutex::new(keychain)),
            dpm: Arc::new(Mutex::new(dpm)),
            entropy: Arc::new(Mutex::new(entropy)),
            bandwidth: Arc::new(Mutex::new(bandwidth)),
            session,
            logger,
            cover_traffic: self.config.cover_traffic,
            keychain_path: self.config.keychain_path.clone(),
        })
    }
}

type H3SendRequest = h3::client::SendRequest<h3_quinn::OpenStreams, Bytes>;
type H3RequestStream = h3::client::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>;

pub struct Connection {
    send_request: Arc<AsyncMutex<H3SendRequest>>,
    psk: [u8; 32],
    stats: Arc<TrafficStats>,
    keychain: Arc<Mutex<KeyChain>>,
    dpm: Arc<Mutex<DpmParams>>,
    entropy: Arc<Mutex<EntropyEngine>>,
    bandwidth: Arc<Mutex<TokenBucket>>,
    session: Arc<Mutex<Session>>,
    logger: Arc<SessionLogger>,
    cover_traffic: bool,
    keychain_path: Option<PathBuf>,
}

impl Connection {
    pub async fn connect_tcp(&self, host: &str, port: u16) -> Result<Tunnel, String> {
        let authority = format!("{}:{}", host, port);
        let psk_hex: String = self.psk.iter().map(|b| format!("{:02x}", b)).collect();
        let creds = Credentials {
            username: "gidy".to_string(),
            password: psk_hex,
        };

        let response_hex: String = {
            let kc = self.keychain.lock();
            let response = hmac_challenge(&kc.current.key, authority.as_bytes());
            response.iter().map(|b| format!("{:02x}", b)).collect()
        };

        let uri = format!("https://{}", authority)
            .parse::<http::Uri>()
            .map_err(|e| format!("uri: {}", e))?;

        let req = http::Request::builder()
            .method(http::Method::CONNECT)
            .uri(uri)
            .header("proxy-authorization", creds.to_header())
            .header("x-gidy-response", &response_hex)
            .header("user-agent", "gidy/2.0.0")
            .body(())
            .map_err(|e| format!("build request: {}", e))?;

        let mut sr = self.send_request.lock().await;
        let mut stream = sr
            .send_request(req)
            .await
            .map_err(|e| format!("send CONNECT: {}", e))?;

        let response = stream
            .recv_response()
            .await
            .map_err(|e| format!("recv response: {}", e))?;

        if response.status() != http::StatusCode::OK {
            let _ = stream.finish().await;
            return Err(format!("CONNECT denied: {}", response.status()));
        }

        self.stats.add_up(authority.len() as u64);

        let stream_id = {
            let mut s = self.session.lock();
            let (id, _) = s.streams.open(authority.clone());
            id
        };

        let session_id = {
            let s = self.session.lock();
            s.id.iter().map(|b| format!("{:02x}", b)).collect()
        };

        self.logger.record(&LogEvent::StreamOpen {
            session_id,
            stream_id: stream_id as u16,
            target: authority.clone(),
        });

        info!("CONNECT {} -> 200 OK", authority);

        Ok(Tunnel {
            stream: Arc::new(AsyncMutex::new(stream)),
            stats: self.stats.clone(),
            dpm: self.dpm.clone(),
            entropy: self.entropy.clone(),
            bandwidth: self.bandwidth.clone(),
            session: self.session.clone(),
            logger: self.logger.clone(),
            cover_traffic: self.cover_traffic,
            stream_id,
        })
    }

    pub async fn health_check(&self) -> Result<bool, String> {
        let psk_hex: String = self.psk.iter().map(|b| format!("{:02x}", b)).collect();
        let creds = Credentials {
            username: "gidy".to_string(),
            password: psk_hex,
        };

        let uri = format!("https://{}", PSEUDO_HOST_CHECK)
            .parse::<http::Uri>()
            .map_err(|e| format!("uri: {}", e))?;

        let req = http::Request::builder()
            .method(http::Method::CONNECT)
            .uri(uri)
            .header("proxy-authorization", creds.to_header())
            .body(())
            .map_err(|e| format!("build request: {}", e))?;

        let mut sr = self.send_request.lock().await;
        let mut stream = sr
            .send_request(req)
            .await
            .map_err(|e| format!("send health: {}", e))?;

        let response = stream
            .recv_response()
            .await
            .map_err(|e| format!("recv response: {}", e))?;

        Ok(response.status() == http::StatusCode::OK)
    }

    pub fn close(&self) {
        {
            let s = self.session.lock();
            let (total_in, total_out) = s.total_bytes();
            let session_id: String = s.id.iter().map(|b| format!("{:02x}", b)).collect();
            self.logger.record(&LogEvent::SessionClose {
                session_id,
                dur_s: s.duration().as_secs(),
                total_streams: s.streams.active_count(),
                total_in,
                total_out,
            });
        }
        self.logger.flush();

        {
            let mut kc = self.keychain.lock();
            let session_id: String = self.session.lock().id.iter().map(|b| format!("{:02x}", b)).collect();
            let transcript = format!("client-session-{}", session_id);
            kc.rotate(transcript.as_bytes());
            if let Some(path) = &self.keychain_path {
                if let Err(e) = kc.save(path) {
                    info!("keychain save failed: {}", e);
                }
            }
        }
    }

    pub fn session(&self) -> &Arc<Mutex<Session>> {
        &self.session
    }

    pub fn logger(&self) -> &Arc<SessionLogger> {
        &self.logger
    }

    pub fn keychain(&self) -> &Arc<Mutex<KeyChain>> {
        &self.keychain
    }
}

pub struct Tunnel {
    stream: Arc<AsyncMutex<H3RequestStream>>,
    stats: Arc<TrafficStats>,
    dpm: Arc<Mutex<DpmParams>>,
    entropy: Arc<Mutex<EntropyEngine>>,
    bandwidth: Arc<Mutex<TokenBucket>>,
    session: Arc<Mutex<Session>>,
    logger: Arc<SessionLogger>,
    cover_traffic: bool,
    stream_id: u64,
}

impl Tunnel {
    pub async fn relay(&self, tcp: tokio::net::TcpStream) -> Result<(), String> {
        let (tcp_read, mut tcp_write) = tcp.into_split();
        let stream = self.stream.clone();
        let stream_reader = stream.clone();
        let stream_writer = stream.clone();

        let dpm = self.dpm.lock().clone();
        let bandwidth_h3 = self.bandwidth.clone();
        let bandwidth_tcp = self.bandwidth.clone();
        let entropy = self.entropy.clone();
        let logger_h3 = self.logger.clone();
        let logger_tcp = self.logger.clone();
        let session_h3 = self.session.clone();
        let session_tcp = self.session.clone();
        let padding_min = dpm.padding_min;
        let stream_id = self.stream_id;
        let cover_traffic = self.cover_traffic;

        // h3 -> TCP: strip DPM padding, bandwidth check
        let h3_to_tcp = tokio::spawn(async move {
            let mut seq: u32 = 0;
            loop {
                let data = {
                    let mut s = stream_reader.lock().await;
                    s.recv_data().await
                };
                match data {
                    Ok(Some(mut buf)) => {
                        let len = buf.remaining();
                        if len == 0 { break; }

                        let chunk = buf.copy_to_bytes(len);
                        let actual_len = len.saturating_sub(padding_min);
                        let actual_data = &chunk[..actual_len.min(len)];

                        let sleep_dur = {
                            let bw = bandwidth_h3.lock();
                            match bw.try_consume(actual_len) {
                                gidy_core::bwctl::ConsumeResult::Delayed(d) => Some(d),
                                gidy_core::bwctl::ConsumeResult::Immediate => None,
                            }
                        };
                        if let Some(d) = sleep_dur {
                            tokio::time::sleep(d).await;
                        }

                        {
                            let mut s = session_h3.lock();
                            s.streams.get_mut(stream_id).map(|r| r.record_in(actual_len));
                        }
                        logger_h3.record_data_in(stream_id as u16, seq, actual_len, None);
                        seq += 1;

                        if tcp_write.write_all(actual_data).await.is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
        });

        // TCP -> h3: add DPM padding + jitter + entropy, bandwidth check
        let mut tcp_read = tcp_read;
        let mut rng = ChaCha20Rng::from_entropy();
        let mut seq: u32 = 0;
        let mut buf = [0u8; 8192];

        loop {
            let n = match tcp_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };

            let actual_data = &buf[..n];

            let sleep_dur = {
                let bw = bandwidth_tcp.lock();
                match bw.try_consume(n) {
                    gidy_core::bwctl::ConsumeResult::Delayed(d) => Some(d),
                    gidy_core::bwctl::ConsumeResult::Immediate => None,
                }
            };
            if let Some(d) = sleep_dur {
                tokio::time::sleep(d).await;
            }

            let jitter = dpm.jitter_us(&mut rng);
            if jitter > 0 {
                tokio::time::sleep(Duration::from_micros(jitter)).await;
            }

            let padding_len = dpm.padding_for(n, &mut rng);

            let entropy_pad = {
                let eg = entropy.lock();
                let profile = eg.active_profile(&dpm);
                eg.padding_for(profile, n + padding_len, &dpm, &mut rng)
            };

            let total_len = n + padding_len + entropy_pad;
            let mut output = Vec::with_capacity(total_len);
            output.extend_from_slice(actual_data);

            let pad_remaining = padding_len + entropy_pad;
            if pad_remaining > 0 {
                let mut pad_bytes = vec![0u8; pad_remaining];
                rand::Rng::fill(&mut rng, &mut pad_bytes[..]);
                output.extend_from_slice(&pad_bytes);
            }

            {
                let mut s = session_tcp.lock();
                s.streams.get_mut(stream_id).map(|r| r.record_out(n));
            }
            logger_tcp.record_data_out(stream_id as u16, seq, n, 0, None);
            seq += 1;

            let data = Bytes::from(output);
            {
                let mut s = stream_writer.lock().await;
                if s.send_data(data).await.is_err() {
                    break;
                }
            }
        }

        let _ = h3_to_tcp.await;

        {
            let mut s = stream.lock().await;
            let _ = s.finish().await;
        }

        // Cover traffic
        if cover_traffic && dpm.cover_enabled {
            let (interval, cover_size) = {
                let eg = entropy.lock();
                let profile = eg.active_profile(&dpm);
                let interval = eg.idle_interval_ms(profile, &mut rng);
                let cover_size = eg.cover_packet_size(profile, &mut rng);
                (interval, cover_size)
            };
            let stream_cover = stream.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(interval)).await;
                    let cover_data = vec![0u8; cover_size];
                    let data = Bytes::from(cover_data);
                    let mut s = stream_cover.lock().await;
                    if s.send_data(data).await.is_err() {
                        break;
                    }
                }
            });
        }

        // Log stream close
        {
            let mut s = self.session.lock();
            if let Some(record) = s.streams.close(stream_id) {
                let session_id: String = s.id.iter().map(|b| format!("{:02x}", b)).collect();
                self.logger.record(&LogEvent::StreamClose {
                    session_id,
                    stream_id: stream_id as u16,
                    dur_ms: record.created_at.elapsed().as_millis() as u64,
                    bytes_in: record.bytes_in,
                    bytes_out: record.bytes_out,
                    close_reason: "completed".into(),
                });
            }
        }

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

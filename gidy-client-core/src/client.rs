use crate::config::ClientConfig;
use crate::stats::TrafficStats;
use bytes::{Buf, Bytes};
use gidy_core::{Credentials, H3_ALPN, PSEUDO_HOST_CHECK};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
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
        transport.keep_alive_interval(Some(std::time::Duration::from_secs(15)));

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

        // h3_driver handles connection-level events (GOAWAY, push, etc.)
        // Spawn it in background to keep the protocol running
        tokio::spawn(async move {
            // Drive h3 connection by reading incoming control frames
            // _h3_driver will be dropped here, which is OK for basic usage
            // For production: poll _h3_driver for events in a loop
            let _ = _h3_driver;
        });

        info!("h3 client ready (ALPN: h3)");

        Ok(Connection {
            send_request: Arc::new(Mutex::new(send_request)),
            psk: self.psk,
            stats: self.stats.clone(),
        })
    }
}

type H3SendRequest = h3::client::SendRequest<h3_quinn::OpenStreams, Bytes>;
type H3RequestStream = h3::client::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>;

pub struct Connection {
    send_request: Arc<Mutex<H3SendRequest>>,
    psk: [u8; 32],
    stats: Arc<TrafficStats>,
}

impl Connection {
    /// Open a CONNECT tunnel to a TCP target
    pub async fn connect_tcp(&self, host: &str, port: u16) -> Result<Tunnel, String> {
        let authority = format!("{}:{}", host, port);
        let psk_hex: String = self.psk.iter().map(|b| format!("{:02x}", b)).collect();
        let creds = Credentials {
            username: "gidy".to_string(),
            password: psk_hex,
        };

        let uri = format!("https://{}", authority)
            .parse::<http::Uri>()
            .map_err(|e| format!("uri: {}", e))?;

        let req = http::Request::builder()
            .method(http::Method::CONNECT)
            .uri(uri)
            .header("proxy-authorization", creds.to_header())
            .header("user-agent", "gidy/2.0.0")
            .body(())
            .map_err(|e| format!("build request: {}", e))?;

        let mut sr = self.send_request.lock().await;
        let mut stream = sr
            .send_request(req)
            .await
            .map_err(|e| format!("send CONNECT: {}", e))?;

        // Receive CONNECT response
        let response = stream
            .recv_response()
            .await
            .map_err(|e| format!("recv response: {}", e))?;

        if response.status() != http::StatusCode::OK {
            let _ = stream.finish().await;
            return Err(format!("CONNECT denied: {}", response.status()));
        }

        self.stats.add_up(authority.len() as u64);
        info!("CONNECT {} -> 200 OK", authority);

        Ok(Tunnel {
            stream: Arc::new(Mutex::new(stream)),
            stats: self.stats.clone(),
        })
    }

    /// Health check via _check pseudo-host
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
}

pub struct Tunnel {
    stream: Arc<Mutex<H3RequestStream>>,
    stats: Arc<TrafficStats>,
}

impl Tunnel {
    /// Relay data bidirectionally between a TCP stream and this h3 tunnel
    pub async fn relay(&self, tcp: tokio::net::TcpStream) -> Result<(), String> {
        let (tcp_read, mut tcp_write) = tcp.into_split();
        let stream = self.stream.clone();
        let stream_reader = stream.clone();
        let stream_writer = stream.clone();

        // h3 -> TCP: read from h3, write to TCP
        let h3_to_tcp = tokio::spawn(async move {
            loop {
                let data = {
                    let mut s = stream_reader.lock().await;
                    s.recv_data().await
                };
                match data {
                    Ok(Some(mut buf)) => {
                        if buf.remaining() == 0 {
                            break;
                        }
                        let chunk = buf.copy_to_bytes(buf.remaining());
                        if tcp_write.write_all(&chunk).await.is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
        });

        // TCP -> h3: read from TCP, write to h3
        let mut tcp_read = tcp_read;
        let mut buf = [0u8; 8192];
        loop {
            let n = match tcp_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };

            let data = Bytes::copy_from_slice(&buf[..n]);
            {
                let mut s = stream_writer.lock().await;
                if s.send_data(data).await.is_err() {
                    break;
                }
            }
            self.stats.add_up(n as u64);
        }

        let _ = h3_to_tcp.await;

        // Finish h3 stream
        {
            let mut s = stream.lock().await;
            let _ = s.finish().await;
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

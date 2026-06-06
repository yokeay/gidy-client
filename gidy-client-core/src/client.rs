use crate::config::ClientConfig;
use crate::stats::TrafficStats;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use bytes::{Buf, Bytes};
use futures_util::{SinkExt, StreamExt};
use gidy_core::{
    Credentials, DpmParams, EntropyEngine, H3_ALPN, KeyChain,
    LogEvent, PSEUDO_HOST_CHECK, Session, SessionLogger, TokenBucket,
    current_epoch,
};
use gidy_core::keys::hmac_challenge;
use hickory_resolver::TokioAsyncResolver;
use hickory_resolver::config::ResolverConfig;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::proto::rr::RecordType;
use parking_lot::Mutex;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex as AsyncMutex;
use tokio_rustls::TlsConnector;
use tokio_tungstenite::{client_async, tungstenite};
use tracing::info;

// WebSocket tunnel frame types
const FRAME_AUTH_REQUEST: u8 = 0x01;
const FRAME_AUTH_RESPONSE: u8 = 0x02;
const FRAME_CONNECT: u8 = 0x03;
const FRAME_CONNECT_OK: u8 = 0x04;
const FRAME_CONNECT_FAIL: u8 = 0x05;
const FRAME_DATA: u8 = 0x06;
const FRAME_CLOSE: u8 = 0x07;

type H3SendRequest = h3::client::SendRequest<h3_quinn::OpenStreams, Bytes>;
type H3RequestStream = h3::client::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>;
type H2SendRequest = h2::client::SendRequest<Bytes>;
type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_rustls::client::TlsStream<tokio::net::TcpStream>,
>;

enum TunnelStream {
    H3(Arc<AsyncMutex<H3RequestStream>>),
    H2 {
        send: h2::SendStream<Bytes>,
        recv: h2::RecvStream,
    },
    WS(WsStream),
    Consumed,
}

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
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        let addr: std::net::SocketAddr = self.config.server_addr
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());

        let transport = if self.config.protocol == "ws" {
            info!("connecting via WebSocket to {}...", self.config.server_addr);
            let ws = establish_ws_connection(&self.config, &self.psk).await?;
            ConnectionTransport::WS(Arc::new(AsyncMutex::new(Some(ws))))
        } else if self.config.protocol == "h2" {
            info!("connecting via H2 to {}...", self.config.server_addr);
            let sr = self.connect_h2().await?;
            ConnectionTransport::H2(Arc::new(AsyncMutex::new(sr)))
        } else {
            let quic_addr: std::net::SocketAddr = self
                .config
                .server_addr
                .parse()
                .map_err(|e| format!("invalid server addr: {}", e))?;
            info!("connecting via QUIC to {}...", quic_addr);
            let sr = self.connect_quic(quic_addr).await?;
            ConnectionTransport::H3(Arc::new(AsyncMutex::new(sr)))
        };

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
            transport,
            config: self.config.clone(),
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

    async fn connect_quic(&self, addr: std::net::SocketAddr) -> Result<H3SendRequest, String> {
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

        info!("h3 client ready");
        Ok(send_request)
    }

    async fn connect_h2(&self) -> Result<H2SendRequest, String> {
        let server_name = rustls::pki_types::ServerName::try_from(self.config.server_name.as_str())
            .map_err(|e| format!("invalid server name {:?}: {}", self.config.server_name, e))?
            .to_owned();

        let mut tls_config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerify))
            .with_no_client_auth();
        tls_config.alpn_protocols = vec![b"h2".to_vec()];
        let connector = TlsConnector::from(Arc::new(tls_config));

        info!("h2: TCP connecting to {}", self.config.server_addr);
        let tcp = TcpStream::connect(&self.config.server_addr)
            .await
            .map_err(|e| format!("TCP connect: {}", e))?;

        let tls = connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| format!("TLS: {}", e))?;

        let (send_request, conn) = h2::client::handshake(tls)
            .await
            .map_err(|e| format!("H2 handshake: {}", e))?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                info!("h2 conn driver error: {}", e);
            }
        });

        info!("h2 client ready");
        Ok(send_request)
    }
}

/// Standalone function to establish a new WS connection (TCP → TLS → WS upgrade → AUTH).
/// Reused by both GidyClient::connect() and Connection::connect_tcp().
async fn establish_ws_connection(
    config: &ClientConfig,
    psk: &[u8; 32],
) -> Result<WsStream, String> {
    let ws_url = config.server_addr.clone();
    let url = url::Url::parse(&ws_url)
        .map_err(|e| format!("ws: invalid URL {}: {}", ws_url, e))?;

    let host = url.host_str().ok_or_else(|| "ws: no host in URL".to_string())?;
    let port = url.port().unwrap_or(443);
    let path = url.path();
    let addr = format!("{}:{}", host, port);
    let server_name = rustls::pki_types::ServerName::try_from(host)
        .map_err(|e| format!("invalid server name: {}", e))?
        .to_owned();

    info!("ws: resolving ECH config for {}", host);

    let initial_ech = match ech_config_from_file(config) {
        Some(ec) => {
            info!("ws: ECH config loaded from config file");
            Some(ec)
        }
        None => fetch_ech_config(host).await,
    };

    // Try ECH connection, with retry on server rejection (ECH key rotation)
    let mut ech_attempt = initial_ech;
    // Try ECH connection, handle rejection gracefully

    let tls = loop {
        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        let tls_config = match ech_attempt {
            Some(ref ec) => {
                info!("ws: ECH enabled, outer SNI will be cloudflare-ech.com");
                rustls::ClientConfig::builder_with_provider(provider)
                    .with_ech(rustls::client::EchMode::Enable(ec.clone()))
                    .map_err(|e| format!("ws: ECH config error: {}", e))?
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(NoVerify))
                    .with_no_client_auth()
            }
            None => {
                info!("ws: ECH not available, using direct SNI");
                rustls::ClientConfig::builder_with_provider(provider)
                    .with_protocol_versions(&[&rustls::version::TLS12, &rustls::version::TLS13])
                    .map_err(|e| format!("ws: protocol versions error: {}", e))?
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(NoVerify))
                    .with_no_client_auth()
            }
        };

        info!("ws: TCP connecting to {}", addr);
        let tcp = TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("ws TCP connect: {}", e))?;
        info!("ws: TCP connected");

        let connector = TlsConnector::from(Arc::new(tls_config));
        match connector.connect(server_name.clone(), tcp).await {
            Ok(tls) => {
                info!("ws: TLS handshake complete");
                break tls;
            }
            Err(io_err) => {
                // Try to extract rustls::Error from the io::Error
                let rustls_err = io_err.get_ref()
                    .and_then(|e| e.downcast_ref::<rustls::Error>());

                let retry_configs = rustls_err.and_then(|e| match e {
                    rustls::Error::PeerIncompatible(
                        rustls::PeerIncompatible::ServerRejectedEncryptedClientHello(rc)
                    ) => rc.clone(),
                    _ => None,
                });

                match retry_configs {
                    Some(configs) if !configs.is_empty() => {
                        // ECH was rejected — the config has expired (CF rotates keys every few hours).
                        // Do NOT fall back to no-ECH because GFW blocks the SNI.
                        // Return a clear error so the user knows to update ech_config_base64.
                        info!("ws: ECH rejected, server provided {} retry config(s) — config expired", configs.len());
                        return Err(
                            "ECH config 已过期，请更新 ech_config_base64。\n\
                             获取方式：在非GFW环境执行 dig HTTPS gidy.eu.cc +short，取 ech= 后的 base64 值".into()
                        );
                    }
                    _ => {
                        // Non-ECH-rejection TLS error (e.g. GFW RST, network issue)
                        // If we were trying with ECH, give a clear message instead of a confusing RST error
                        if ech_attempt.is_some() {
                            return Err(
                                "TLS 连接失败，ECH 可能已过期。请更新 ech_config_base64 后重试。\n\
                                 获取方式：在非GFW环境执行 dig HTTPS gidy.eu.cc +short".into()
                            );
                        } else {
                            return Err(format!("ws TLS: {}", io_err));
                        }
                    }
                }
            }
        }
    };

    let ws_uri = format!("wss://{}:{}{}", host, port, path);
    let ws_request = tungstenite::handshake::client::Request::builder()
        .uri(&ws_uri)
        .header("Host", host)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header("Sec-WebSocket-Key", tungstenite::handshake::client::generate_key())
        .header("Sec-WebSocket-Version", "13")
        .body(())
        .map_err(|e| format!("ws build request: {}", e))?;

    let (ws_stream, _response) = client_async(ws_request, tls)
        .await
        .map_err(|e| format!("ws upgrade: {}", e))?;

    info!("ws: connected, authenticating");

    let psk_hex: String = psk.iter().map(|b| format!("{:02x}", b)).collect();
    let auth_value = format!("Basic {}", BASE64.encode(format!("gidy:{}", psk_hex)));
    let mut auth_frame = vec![FRAME_AUTH_REQUEST];
    auth_frame.extend_from_slice(auth_value.as_bytes());

    let mut ws = ws_stream;
    ws.send(tungstenite::Message::Binary(auth_frame.into()))
        .await
        .map_err(|e| format!("ws send auth: {}", e))?;

    let msg = ws.next().await
        .ok_or_else(|| "ws: connection closed before auth response".to_string())?
        .map_err(|e| format!("ws recv auth: {}", e))?;

    match msg {
        tungstenite::Message::Binary(data) => {
            if data.len() < 2 || data[0] != FRAME_AUTH_RESPONSE {
                return Err(format!("ws: unexpected auth response frame type: {:?}", data.first()));
            }
            if data[1] != 0x00 {
                return Err("ws: authentication failed".into());
            }
        }
        _ => return Err("ws: unexpected non-binary auth response".into()),
    }

    info!("ws: authenticated OK");
    Ok(ws)
}

pub fn ech_config_from_file(config: &ClientConfig) -> Option<rustls::client::EchConfig> {
    let b64 = config.ech_config_base64.as_ref()?;
    let ech_bytes = BASE64.decode(b64.as_bytes()).ok()?;
    info!("ws: decoded ECH config from file ({} bytes)", ech_bytes.len());
    rustls::client::EchConfig::new(
        rustls::pki_types::EchConfigListBytes::from(ech_bytes.as_slice()),
        rustls::crypto::aws_lc_rs::hpke::ALL_SUPPORTED_SUITES,
    ).ok()
}

async fn fetch_ech_config(domain: &str) -> Option<rustls::client::EchConfig> {
    let mut opts = hickory_resolver::config::ResolverOpts::default();
    opts.timeout = Duration::from_secs(3);
    opts.attempts = 1;

    let doh_providers = [
        ("Cloudflare", ResolverConfig::cloudflare_https()),
        ("Google", ResolverConfig::google_https()),
    ];

    let name = match domain.parse::<hickory_resolver::proto::rr::Name>() {
        Ok(n) => n,
        Err(_) => return None,
    };

    for (label, config) in &doh_providers {
        info!("ws: trying {} DoH for HTTPS record of {}", label, domain);
        let resolver = TokioAsyncResolver::new(
            config.clone(),
            opts.clone(),
            TokioConnectionProvider::default(),
        );

        match tokio::time::timeout(Duration::from_secs(5), resolver.lookup(name.clone(), RecordType::HTTPS)).await {
            Ok(Ok(response)) => {
                for rdata in response.iter() {
                    if let hickory_resolver::proto::rr::RData::HTTPS(ref https) = rdata {
                        for (key, value) in https.svc_params() {
                            if let hickory_resolver::proto::rr::rdata::svcb::SvcParamKey::EchConfig = key {
                                if let hickory_resolver::proto::rr::rdata::svcb::SvcParamValue::EchConfig(ref ech) = value {
                                    let ech_bytes: &[u8] = ech.0.as_ref();
                                    info!("ws: found ECH config in DNS ({} bytes) via {}", ech_bytes.len(), label);
                                    match rustls::client::EchConfig::new(
                                        rustls::pki_types::EchConfigListBytes::from(ech_bytes),
                                        rustls::crypto::aws_lc_rs::hpke::ALL_SUPPORTED_SUITES,
                                    ) {
                                        Ok(ec) => return Some(ec),
                                        Err(e) => info!("ws: ECH config parse error: {}", e),
                                    }
                                }
                            }
                        }
                    }
                }
                info!("ws: no ECH config in HTTPS records via {}", label);
            }
            Ok(Err(e)) => info!("ws: {} DoH lookup failed: {}", label, e),
            Err(_) => info!("ws: {} DoH lookup timed out", label),
        }
    }

    info!("ws: ECH not available — all DoH providers failed or no ECH record");
    None
}

enum ConnectionTransport {
    H3(Arc<AsyncMutex<H3SendRequest>>),
    H2(Arc<AsyncMutex<H2SendRequest>>),
    WS(Arc<AsyncMutex<Option<WsStream>>>),
}

pub struct Connection {
    transport: ConnectionTransport,
    config: Arc<ClientConfig>,
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

        let response_hex: String = {
            let kc = self.keychain.lock();
            let response = hmac_challenge(&kc.current.key, authority.as_bytes());
            response.iter().map(|b| format!("{:02x}", b)).collect()
        };

        let stream_id = {
            let mut s = self.session.lock();
            let (id, _) = s.streams.open(authority.clone());
            id
        };

        let session_id_str: String = {
            let s = self.session.lock();
            s.id.iter().map(|b| format!("{:02x}", b)).collect()
        };

        self.logger.record(&LogEvent::StreamOpen {
            session_id: session_id_str,
            stream_id: stream_id as u16,
            target: authority.clone(),
        });

        info!("CONNECT {}...", authority);

        let tunnel_stream = match &self.transport {
            ConnectionTransport::H3(sr) => {
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
                    .header("x-gidy-response", &response_hex)
                    .header("user-agent", "gidy/2.0.0")
                    .body(())
                    .map_err(|e| format!("build request: {}", e))?;

                let mut locked = sr.lock().await;
                let mut stream = locked
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

                TunnelStream::H3(Arc::new(AsyncMutex::new(stream)))
            }
            ConnectionTransport::H2(sr) => {
                let auth_value = format!(
                    "Basic {}",
                    BASE64.encode(format!(":{}", psk_hex))
                );

                let uri = http::Uri::builder()
                    .authority(authority.as_str())
                    .build()
                    .map_err(|e| format!("build h2 uri: {}", e))?;

                let req = http::Request::builder()
                    .method("CONNECT")
                    .uri(uri)
                    .header("proxy-authorization", auth_value)
                    .header("x-gidy-response", &response_hex)
                    .header("user-agent", "gidy/2.0.0")
                    .body(())
                    .map_err(|e| format!("build h2 request: {}", e))?;

                let mut locked = sr.lock().await;
                let (resp_future, send_stream) = locked
                    .send_request(req, false)
                    .map_err(|e| format!("send h2 CONNECT: {}", e))?;

                let response = resp_future.await.map_err(|e| format!("h2 CONNECT response: {}", e))?;

                if response.status() != http::StatusCode::OK {
                    return Err(format!("CONNECT denied: {}", response.status()));
                }

                let recv_stream = response.into_body();
                TunnelStream::H2 { send: send_stream, recv: recv_stream }
            }
            ConnectionTransport::WS(_ws_arc) => {
                // Always create a fresh WS connection for each tunnel.
                // Reusing the initial idle connection can fail if the server
                // has closed it during the idle period between connect() and
                // the first SOCKS5 request.
                info!("ws: creating new connection for tunnel to {}", authority);
                let mut ws = establish_ws_connection(&self.config, &self.psk).await?;

                // CONNECT frame: 0x03 + authority
                let mut connect_frame = vec![FRAME_CONNECT];
                connect_frame.extend_from_slice(authority.as_bytes());
                ws.send(tungstenite::Message::Binary(connect_frame.into()))
                    .await
                    .map_err(|e| format!("ws send CONNECT: {}", e))?;

                // Wait for CONNECT_OK or CONNECT_FAIL
                let msg = ws.next().await
                    .ok_or_else(|| "ws: closed before connect response".to_string())?
                    .map_err(|e| format!("ws recv connect: {}", e))?;

                match msg {
                    tungstenite::Message::Binary(data) => {
                        if data.is_empty() {
                            return Err("ws: empty connect response".into());
                        }
                        match data[0] {
                            FRAME_CONNECT_OK => {
                                info!("ws CONNECT {} -> OK", authority);
                            }
                            FRAME_CONNECT_FAIL => {
                                let status = data.get(1).copied().unwrap_or(0);
                                return Err(format!("ws CONNECT denied: status 0x{:02x}", status));
                            }
                            _ => return Err(format!("ws: unexpected frame type: 0x{:02x}", data[0])),
                        }
                    }
                    _ => return Err("ws: unexpected non-binary connect response".into()),
                }

                TunnelStream::WS(ws)
            }
        };

        self.stats.add_up(authority.len() as u64);
        info!("CONNECT {} -> OK", authority);

        Ok(Tunnel {
            stream: Arc::new(AsyncMutex::new(tunnel_stream)),
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

        match &self.transport {
            ConnectionTransport::H3(sr) => {
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

                let mut locked = sr.lock().await;
                let mut stream = locked
                    .send_request(req)
                    .await
                    .map_err(|e| format!("send health: {}", e))?;

                let response = stream
                    .recv_response()
                    .await
                    .map_err(|e| format!("recv response: {}", e))?;

                Ok(response.status() == http::StatusCode::OK)
            }
            ConnectionTransport::H2(sr) => {
                let auth_value = format!(
                    "Basic {}",
                    BASE64.encode(format!(":{}", psk_hex))
                );
                let uri = http::Uri::builder()
                    .authority(PSEUDO_HOST_CHECK)
                    .build()
                    .map_err(|e| format!("build h2 health uri: {}", e))?;
                let req = http::Request::builder()
                    .method("CONNECT")
                    .uri(uri)
                    .header("proxy-authorization", auth_value)
                    .body(())
                    .map_err(|e| format!("build h2 health: {}", e))?;

                let mut locked = sr.lock().await;
                let (resp_future, _send) = locked
                    .send_request(req, true)
                    .map_err(|e| format!("send h2 health: {}", e))?;

                let response = resp_future.await.map_err(|e| format!("h2 health response: {}", e))?;
                Ok(response.status() == http::StatusCode::OK)
            }
            ConnectionTransport::WS(ws_arc) => {
                let mut locked = ws_arc.lock().await;
                match locked.as_mut() {
                    Some(ws) => {
                        // CONNECT frame with authority = "_check"
                        let mut frame = vec![FRAME_CONNECT];
                        frame.extend_from_slice(PSEUDO_HOST_CHECK.as_bytes());
                        ws.send(tungstenite::Message::Binary(frame.into()))
                            .await
                            .map_err(|e| format!("ws send health: {}", e))?;

                        let msg = ws.next().await
                            .ok_or_else(|| "ws: closed before health response".to_string())?
                            .map_err(|e| format!("ws recv health: {}", e))?;

                        match msg {
                            tungstenite::Message::Binary(data) if !data.is_empty() => {
                                Ok(data[0] == FRAME_CONNECT_OK)
                            }
                            _ => Ok(false),
                        }
                    }
                    None => {
                        // Initial WS stream taken by a tunnel; assume healthy
                        Ok(true)
                    }
                }
            }
        }
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
    stream: Arc<AsyncMutex<TunnelStream>>,
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
    pub async fn relay(&self, tcp: tokio::net::TcpStream, prefix: Option<&[u8]>) -> Result<(), String> {
        let dpm = self.dpm.lock().clone();
        let padding_min = dpm.padding_min;
        let stream_id = self.stream_id;
        let cover_traffic = self.cover_traffic;

        // Determine transport type without holding the lock
        let transport_type = {
            let guard = self.stream.lock().await;
            match &*guard {
                TunnelStream::H3(_) => 0u8,
                TunnelStream::H2 { .. } => 1u8,
                TunnelStream::WS(_) => 2u8,
                TunnelStream::Consumed => 3u8,
            }
            // guard dropped here
        };

        match transport_type {
            0 => {
                if let Some(pre) = prefix {
                    tracing::warn!("h3 relay ignoring prefix {} bytes (unexpected)", pre.len());
                }
                self.relay_h3(tcp, dpm, padding_min, stream_id, cover_traffic).await
            }
            1 => {
                if let Some(pre) = prefix {
                    tracing::warn!("h2 relay ignoring prefix {} bytes (unexpected)", pre.len());
                }
                self.relay_h2(tcp, dpm, padding_min, stream_id).await
            }
            2 => {
                self.relay_ws(tcp, dpm, padding_min, stream_id, prefix).await
            }
            _ => Err("tunnel already consumed".into()),
        }
    }

    async fn relay_ws(
        &self,
        tcp: tokio::net::TcpStream,
        dpm: DpmParams,
        padding_min: usize,
        stream_id: u64,
        prefix: Option<&[u8]>,
    ) -> Result<(), String> {
        // Take ownership of the WS stream
        let ws = {
            let mut guard = self.stream.lock().await;
            let old = std::mem::replace(&mut *guard, TunnelStream::Consumed);
            match old {
                TunnelStream::WS(ws) => ws,
                _ => return Err("expected WS tunnel stream".into()),
            }
        };

        let (tcp_read, mut tcp_write) = tcp.into_split();

        // Split WS stream into independent read/write halves to avoid lock contention
        let (ws_sink, ws_stream) = ws.split();
        let ws_reader = Arc::new(AsyncMutex::new(ws_stream));
        let ws_writer = Arc::new(AsyncMutex::new(ws_sink));

        // Send prefix data (leftover from SOCKS5 read) through WS first
        // Format: [0x06][actual data]
        if let Some(pre) = prefix {
            if !pre.is_empty() {
                tracing::debug!("tcp→ws: sending prefix {} bytes", pre.len());
                let mut frame = Vec::with_capacity(1 + pre.len());
                frame.push(FRAME_DATA);
                frame.extend_from_slice(pre);
                let mut locked = ws_writer.lock().await;
                locked.send(tungstenite::Message::Binary(frame.into()))
                    .await
                    .map_err(|e| format!("ws send prefix: {}", e))?;
            }
        }

        tracing::info!("ws relay started");
        let bandwidth_down = self.bandwidth.clone();
        let bandwidth_up = self.bandwidth.clone();
        let stats_down = self.stats.clone();
        let stats_up = self.stats.clone();
        let entropy = self.entropy.clone();
        let logger_down = self.logger.clone();
        let logger_up = self.logger.clone();
        let session_down = self.session.clone();
        let session_up = self.session.clone();

        // WS -> TCP: read DATA frames, strip padding, write to tcp
        let ws_to_tcp = tokio::spawn(async move {
            let mut seq: u32 = 0;
            loop {
                let msg = ws_reader.lock().await.next().await;
                match msg {
                    Some(Ok(tungstenite::Message::Binary(data))) => {
                        if data.len() < 1 { break; }
                        match data[0] {
                            FRAME_DATA => {
                                // Format: [0x06][2-byte length BE][data][padding]
                                if data.len() < 3 { continue; }
                                let actual_len = u16::from_be_bytes([data[1], data[2]]) as usize;
                                if actual_len > data.len() - 3 { continue; }
                                let actual_data = &data[3..3 + actual_len];

                                let sleep_dur = {
                                    let bw = bandwidth_down.lock();
                                    match bw.try_consume(actual_len) {
                                        gidy_core::bwctl::ConsumeResult::Delayed(d) => Some(d),
                                        gidy_core::bwctl::ConsumeResult::Immediate => None,
                                    }
                                };
                                if let Some(d) = sleep_dur { tokio::time::sleep(d).await; }

                                {
                                    let mut s = session_down.lock();
                                    s.streams.get_mut(stream_id).map(|r| r.record_in(actual_len));
                                }
                                stats_down.add_down(actual_len as u64);
                                logger_down.record_data_in(stream_id as u16, seq, actual_len, None);
                                seq += 1;

                                if tcp_write.write_all(actual_data).await.is_err() { break; }
                            }
                            FRAME_CLOSE => break,
                            _ => continue,
                        }
                    }
                    Some(Ok(tungstenite::Message::Close(_))) => break,
                    Some(Ok(_)) => continue, // text/ping/pong — ignore
                    Some(Err(e)) => { tracing::warn!("ws recv error: {}", e); break; }
                    None => break,
                }
            }
        });

        // TCP -> WS: read from tcp, add DPM padding, send DATA frame
        let mut tcp_read = tcp_read;
        let mut rng = ChaCha20Rng::from_entropy();
        let mut seq: u32 = 0;
        let mut buf = [0u8; 8192];

        loop {
            let n = match tokio::time::timeout(Duration::from_secs(15), tcp_read.read(&mut buf)).await {
                Ok(Ok(0)) => { tracing::info!("ws relay: TCP EOF"); break; }
                Ok(Ok(n)) => { tracing::debug!("ws relay: TCP read {} bytes", n); n }
                Ok(Err(e)) => { tracing::warn!("ws relay: TCP read error: {}", e); break; }
                Err(_) => { tracing::warn!("ws relay: TCP read timeout"); break; }
            };

            let sleep_dur = {
                let bw = bandwidth_up.lock();
                match bw.try_consume(n) {
                    gidy_core::bwctl::ConsumeResult::Delayed(d) => Some(d),
                    gidy_core::bwctl::ConsumeResult::Immediate => None,
                }
            };
            if let Some(d) = sleep_dur { tokio::time::sleep(d).await; }

            let jitter = dpm.jitter_us(&mut rng);
            if jitter > 0 { tokio::time::sleep(Duration::from_micros(jitter)).await; }

            // TCP -> WS: send [0x06][actual data], no padding
            let mut frame = Vec::with_capacity(1 + n);
            frame.push(FRAME_DATA);
            frame.extend_from_slice(&buf[..n]);

            {
                let mut s = session_up.lock();
                s.streams.get_mut(stream_id).map(|r| r.record_out(n));
            }
            stats_up.add_up(n as u64);
            logger_up.record_data_out(stream_id as u16, seq, n, 0, None);
            seq += 1;

            {
                let mut locked = ws_writer.lock().await;
                if locked.send(tungstenite::Message::Binary(frame.into())).await.is_err() { break; }
            }
        }

        // Send CLOSE frame
        {
            let mut locked = ws_writer.lock().await;
            let close_frame = vec![FRAME_CLOSE];
            let _ = locked.send(tungstenite::Message::Binary(close_frame.into())).await;
            let _ = locked.close().await;
        }

        let _ = ws_to_tcp.await;
        self.log_stream_close(stream_id);
        Ok(())
    }

    async fn relay_h3(
        &self,
        tcp: tokio::net::TcpStream,
        dpm: DpmParams,
        padding_min: usize,
        stream_id: u64,
        cover_traffic: bool,
    ) -> Result<(), String> {
        let stream = self.stream.clone();
        let stream_reader = stream.clone();
        let stream_writer = stream.clone();
        let (tcp_read, mut tcp_write) = tcp.into_split();

        let bandwidth_h3 = self.bandwidth.clone();
        let bandwidth_tcp = self.bandwidth.clone();
        let entropy = self.entropy.clone();
        let logger_h3 = self.logger.clone();
        let logger_tcp = self.logger.clone();
        let session_h3 = self.session.clone();
        let session_tcp = self.session.clone();

        let h3_to_tcp = tokio::spawn(async move {
            let mut seq: u32 = 0;
            loop {
                let data = {
                    let mut guard = stream_reader.lock().await;
                    match &mut *guard {
                        TunnelStream::H3(s) => s.lock().await.recv_data().await,
                        _ => break,
                    }
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
                        if let Some(d) = sleep_dur { tokio::time::sleep(d).await; }

                        {
                            let mut s = session_h3.lock();
                            s.streams.get_mut(stream_id).map(|r| r.record_in(actual_len));
                        }
                        logger_h3.record_data_in(stream_id as u16, seq, actual_len, None);
                        seq += 1;

                        if tcp_write.write_all(actual_data).await.is_err() { break; }
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
        });

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

            let sleep_dur = {
                let bw = bandwidth_tcp.lock();
                match bw.try_consume(n) {
                    gidy_core::bwctl::ConsumeResult::Delayed(d) => Some(d),
                    gidy_core::bwctl::ConsumeResult::Immediate => None,
                }
            };
            if let Some(d) = sleep_dur { tokio::time::sleep(d).await; }

            let jitter = dpm.jitter_us(&mut rng);
            if jitter > 0 { tokio::time::sleep(Duration::from_micros(jitter)).await; }

            let padding_len = dpm.padding_for(n, &mut rng);
            let entropy_pad = {
                let eg = entropy.lock();
                let profile = eg.active_profile(&dpm);
                eg.padding_for(profile, n + padding_len, &dpm, &mut rng)
            };

            let total_len = n + padding_len + entropy_pad;
            let mut output = Vec::with_capacity(total_len);
            output.extend_from_slice(&buf[..n]);
            if padding_len + entropy_pad > 0 {
                let mut pad_bytes = vec![0u8; padding_len + entropy_pad];
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
                let mut guard = stream_writer.lock().await;
                match &mut *guard {
                    TunnelStream::H3(s) => {
                        if s.lock().await.send_data(data).await.is_err() { break; }
                    }
                    _ => break,
                }
            }
        }

        let _ = h3_to_tcp.await;

        {
            let mut guard = stream.lock().await;
            if let TunnelStream::H3(s) = &mut *guard {
                let _ = s.lock().await.finish().await;
            }
        }

        if cover_traffic && dpm.cover_enabled {
            let (interval, cover_size) = {
                let eg = self.entropy.lock();
                let profile = eg.active_profile(&dpm);
                let interval = eg.idle_interval_ms(profile, &mut rng);
                let cover_size = eg.cover_packet_size(profile, &mut rng);
                (interval, cover_size)
            };
            let stream_cover = stream.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(interval)).await;
                    let data = Bytes::from(vec![0u8; cover_size]);
                    let mut guard = stream_cover.lock().await;
                    match &mut *guard {
                        TunnelStream::H3(s) => {
                            if s.lock().await.send_data(data).await.is_err() { break; }
                        }
                        _ => break,
                    }
                }
            });
        }

        self.log_stream_close(stream_id);
        Ok(())
    }

    async fn relay_h2(
        &self,
        tcp: tokio::net::TcpStream,
        dpm: DpmParams,
        padding_min: usize,
        stream_id: u64,
    ) -> Result<(), String> {
        let (send_stream, recv_stream) = {
            let mut guard = self.stream.lock().await;
            let old = std::mem::replace(&mut *guard, TunnelStream::Consumed);
            match old {
                TunnelStream::H2 { send, recv } => (send, recv),
                _ => return Err("expected H2 tunnel stream".into()),
            }
        };

        let (tcp_read, mut tcp_write) = tcp.into_split();
        let bandwidth_down = self.bandwidth.clone();
        let bandwidth_up = self.bandwidth.clone();
        let entropy = self.entropy.clone();
        let logger_down = self.logger.clone();
        let logger_up = self.logger.clone();
        let session_down = self.session.clone();
        let session_up = self.session.clone();

        let down_task = tokio::spawn(async move {
            let mut recv_stream = recv_stream;
            let mut seq: u32 = 0;
            loop {
                match recv_stream.data().await {
                    Some(Ok(data)) => {
                        let _ = recv_stream.flow_control().release_capacity(data.len());
                        let len = data.len();
                        let actual_len = len.saturating_sub(padding_min);

                        let sleep_dur = {
                            let bw = bandwidth_down.lock();
                            match bw.try_consume(actual_len) {
                                gidy_core::bwctl::ConsumeResult::Delayed(d) => Some(d),
                                gidy_core::bwctl::ConsumeResult::Immediate => None,
                            }
                        };
                        if let Some(d) = sleep_dur { tokio::time::sleep(d).await; }

                        {
                            let mut s = session_down.lock();
                            s.streams.get_mut(stream_id).map(|r| r.record_in(actual_len));
                        }
                        logger_down.record_data_in(stream_id as u16, seq, actual_len, None);
                        seq += 1;

                        if tcp_write.write_all(&data[..actual_len.min(len)]).await.is_err() { break; }
                    }
                    Some(Err(e)) => { tracing::warn!("h2 recv error: {}", e); break; }
                    None => break,
                }
            }
        });

        let mut send_stream = send_stream;
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

            let sleep_dur = {
                let bw = bandwidth_up.lock();
                match bw.try_consume(n) {
                    gidy_core::bwctl::ConsumeResult::Delayed(d) => Some(d),
                    gidy_core::bwctl::ConsumeResult::Immediate => None,
                }
            };
            if let Some(d) = sleep_dur { tokio::time::sleep(d).await; }

            let jitter = dpm.jitter_us(&mut rng);
            if jitter > 0 { tokio::time::sleep(Duration::from_micros(jitter)).await; }

            let padding_len = dpm.padding_for(n, &mut rng);
            let entropy_pad = {
                let eg = entropy.lock();
                let profile = eg.active_profile(&dpm);
                eg.padding_for(profile, n + padding_len, &dpm, &mut rng)
            };

            let pad_total = padding_len + entropy_pad;
            let mut output = Vec::with_capacity(n + pad_total);
            output.extend_from_slice(&buf[..n]);
            if pad_total > 0 {
                let mut pb = vec![0u8; pad_total];
                rand::Rng::fill(&mut rng, &mut pb[..]);
                output.extend_from_slice(&pb);
            }

            {
                let mut s = session_up.lock();
                s.streams.get_mut(stream_id).map(|r| r.record_out(n));
            }
            logger_up.record_data_out(stream_id as u16, seq, n, 0, None);
            seq += 1;

            send_stream.reserve_capacity(output.len());
            if send_stream.send_data(Bytes::from(output), false).is_err() { break; }
        }

        let _ = send_stream.send_data(Bytes::new(), true);
        let _ = down_task.await;
        self.log_stream_close(stream_id);
        Ok(())
    }

    fn log_stream_close(&self, stream_id: u64) {
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
}

#[derive(Debug)]
pub struct NoVerify;

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
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}

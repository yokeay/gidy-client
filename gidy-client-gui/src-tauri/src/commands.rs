use gidy_client_core::client::GidyClient;
use gidy_client_core::config::ClientConfig;
use gidy_client_core::proxy::Socks5Server;
use gidy_client_core::proxy::HttpProxyServer;
use gidy_client_core::proxy::{LogEntry, new_log_buffer, SharedLogBuffer};
use gidy_client_core::stats::TrafficStats;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

// Re-export for lib.rs
pub use proxy_state::ProxyState;
mod proxy_state {
    use std::sync::Arc;

    pub struct ProxyState {
        pub running: bool,
        pub connected: bool,
        pub error: Option<String>,
        pub config: super::ClientConfig,
        pub stats: Arc<super::TrafficStats>,
        pub shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
        pub start_time: Option<std::time::Instant>,
        /// Shared log buffer between SOCKS5 and HTTP proxy servers.
        pub connection_logs: super::SharedLogBuffer,
    }

    impl Default for ProxyState {
        fn default() -> Self {
            Self {
                running: false,
                connected: false,
                error: None,
                config: super::ClientConfig {
                    psk_hex: "4f3915417e21b4d3c54bb378c1fc66657b7a02626e688198438ad7a12b58270a".into(),
                    server_addr: "wss://gidy.eu.cc/ws".into(),
                    server_name: "gidy.eu.cc".into(),
                    listen_addr: "127.0.0.1:5555".parse().unwrap(),
                    log_level: "info".into(),
                    bandwidth_kbps: 0,
                    log_level_gidy: "basic".into(),
                    log_dir: None,
                    keychain_path: None,
                    cover_traffic: false,
                    protocol: "ws".into(),
                    ech_config_base64: Some("AEX+DQBBTgAgACCYo9Q9bckgxEVSsy1OpdbHyORBRUmFm2bD328SjAxhSAAEAAEAAQASY2xvdWRmbGFyZS1lY2guY29tAAA=".into()),
                    ech_token: Some("23ba5c1e97d380d288b2fd6e4cc5114e".into()),
                },
                stats: super::TrafficStats::new(),
                shutdown_tx: None,
                start_time: None,
                connection_logs: super::new_log_buffer(),
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GuiConfig {
    pub psk_hex: String,
    pub server_addr: String,
    pub server_port: u16,
    pub server_name: String,
    pub ws_url: String,
    pub ech_config_base64: String,
    pub ech_token: String,
    pub socks5_addr: String,
    pub socks5_port: u16,
    pub http_addr: String,
    pub http_port: u16,
    pub protocol: String,
    pub mode: String,
    pub auto_start: bool,
    pub auto_connect: bool,
    pub minimize_to_tray: bool,
    pub log_retention_days: u32,
    pub theme: String,
    pub theme_color: String,
    pub log_level: String,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            psk_hex: "4f3915417e21b4d3c54bb378c1fc66657b7a02626e688198438ad7a12b58270a".into(),
            server_addr: "gidy.eu.cc".into(),
            server_port: 443,
            server_name: "gidy.eu.cc".into(),
            ws_url: "wss://gidy.eu.cc/ws".into(),
            ech_config_base64: "AEX+DQBBTgAgACCYo9Q9bckgxEVSsy1OpdbHyORBRUmFm2bD328SjAxhSAAEAAEAAQASY2xvdWRmbGFyZS1lY2guY29tAAA=".into(),
            ech_token: "23ba5c1e97d380d288b2fd6e4cc5114e".into(),
            socks5_addr: "127.0.0.1".into(),
            socks5_port: 5555,
            http_addr: "127.0.0.1".into(),
            http_port: 5556,
            protocol: "ws".into(),
            mode: "global".into(),
            auto_start: false,
            auto_connect: false,
            minimize_to_tray: true,
            log_retention_days: 7,
            theme: "dark".into(),
            theme_color: "blue".into(),
            log_level: "info".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyStatus {
    pub running: bool,
    pub connected: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatsSnapshot {
    pub bytes_up: u64,
    pub bytes_down: u64,
    pub speed_up_kbps: f64,
    pub speed_down_kbps: f64,
    pub uptime_secs: u64,
    pub active_connections: u32,
}

use crate::AppState;

#[tauri::command]
pub async fn connect(state: tauri::State<'_, AppState>) -> Result<ProxyStatus, String> {
    // Phase 1: read config under lock, then drop
    let (config, stats, connection_logs) = {
        let proxy = state.proxy.lock();
        if proxy.running {
            return Ok(ProxyStatus {
                running: true,
                connected: proxy.connected,
                error: proxy.error.clone(),
            });
        }
        if proxy.config.psk_hex.len() != 64 {
            return Err("PSK must be 64 hex characters".to_string());
        }
        (proxy.config.clone(), proxy.stats.clone(), proxy.connection_logs.clone())
    };

    // Phase 2: async connect without holding lock
    let client = GidyClient::new(config.clone(), stats.clone())
        .map_err(|e| e.to_string())?;
    let conn = client.connect().await
        .map_err(|e| format!("connect: {}", e))?;

    // Phase 3: update state under lock
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    {
        let mut proxy = state.proxy.lock();
        proxy.running = true;
        proxy.connected = true;
        proxy.error = None;
        proxy.shutdown_tx = Some(shutdown_tx);
        proxy.start_time = Some(std::time::Instant::now());
    }

    // SOCKS5 proxy on config.listen_addr (default 127.0.0.1:5555)
    // HTTP  proxy on 127.0.0.1:5556 (for Windows system proxy)
    let socks5_addr = config.listen_addr.to_string();
    let http_proxy_addr = "127.0.0.1:5556".to_string();

    // Set Windows system proxy → HTTP proxy port
    {
        let sys_proxy = state.sys_proxy.lock();
        if let Err(e) = sys_proxy.set("127.0.0.1", 5556) {
            tracing::warn!("failed to set system proxy: {}", e);
        }
    }

    // Clone conn for HTTP proxy (Connection is Arc-wrapped internally)
    let conn_arc = Arc::new(conn);
    let stats_clone = stats.clone();

    let (socks5_shutdown_tx, socks5_shutdown_rx) = tokio::sync::oneshot::channel();
    let (http_shutdown_tx, http_shutdown_rx) = tokio::sync::oneshot::channel();

    // SOCKS5 server task
    let socks5_conn = conn_arc.clone();
    let socks5_stats = stats.clone();
    let socks5_logs = connection_logs.clone();
    tokio::spawn(async move {
        let socks5 = Socks5Server::from_arc(socks5_addr, socks5_conn, socks5_stats, socks5_logs);
        tokio::select! {
            result = socks5.run() => {
                if let Err(e) = result {
                    tracing::error!("socks5 server: {}", e);
                }
            }
            _ = socks5_shutdown_rx => {
                tracing::info!("socks5 shutdown requested");
            }
        }
    });

    // HTTP CONNECT proxy server task
    let http_logs = connection_logs;
    tokio::spawn(async move {
        let http_proxy = HttpProxyServer::from_arc(http_proxy_addr, conn_arc, stats_clone, http_logs);
        tokio::select! {
            result = http_proxy.run() => {
                if let Err(e) = result {
                    tracing::error!("http proxy server: {}", e);
                }
            }
            _ = http_shutdown_rx => {
                tracing::info!("http proxy shutdown requested");
            }
        }
    });

    // Wait for main shutdown signal, then signal both sub-servers
    tokio::spawn(async move {
        let _ = shutdown_rx.await;
        let _ = socks5_shutdown_tx.send(());
        let _ = http_shutdown_tx.send(());
        tracing::info!("proxy servers shutdown complete");
    });

    Ok(ProxyStatus {
        running: true,
        connected: true,
        error: None,
    })
}

#[tauri::command]
pub async fn disconnect(state: tauri::State<'_, AppState>) -> Result<ProxyStatus, String> {
    // Clear system proxy first
    {
        let sys_proxy = state.sys_proxy.lock();
        if let Err(e) = sys_proxy.clear() {
            tracing::warn!("failed to clear system proxy: {}", e);
        }
    }

    let mut proxy = state.proxy.lock();
    proxy.running = false;
    proxy.connected = false;
    proxy.start_time = None;

    if let Some(tx) = proxy.shutdown_tx.take() {
        let _ = tx.send(());
    }

    Ok(ProxyStatus {
        running: false,
        connected: false,
        error: None,
    })
}

#[tauri::command]
pub async fn get_stats(state: tauri::State<'_, AppState>) -> Result<StatsSnapshot, String> {
    let proxy = state.proxy.lock();
    let snap = proxy.stats.snapshot();
    Ok(StatsSnapshot {
        bytes_up: snap.bytes_up,
        bytes_down: snap.bytes_down,
        speed_up_kbps: snap.speed_up_kbps,
        speed_down_kbps: snap.speed_down_kbps,
        uptime_secs: snap.uptime_secs,
        active_connections: 0,
    })
}

#[tauri::command]
pub async fn get_config(state: tauri::State<'_, AppState>) -> Result<GuiConfig, String> {
    let proxy = state.proxy.lock();
    let cfg = &proxy.config;

    // For WS protocol, server_addr is the full URL (e.g. wss://gidy.eu.cc/ws)
    // For QUIC/H2/H3, server_addr is host:port
    let (server_addr, server_port) = if cfg.protocol == "ws" {
        // WS: extract host from URL for display, port defaults to 443
        let url_host = cfg.server_addr
            .trim_start_matches("wss://")
            .trim_start_matches("ws://")
            .split('/').next().unwrap_or("gidy.eu.cc");
        // Strip port if present in host
        let host = url_host.split(':').next().unwrap_or("gidy.eu.cc");
        (host.to_string(), 443u16)
    } else {
        let host = cfg.server_addr.split(':').next().unwrap_or("gidy.eu.cc").to_string();
        let port = cfg.server_addr.split(':').nth(1).and_then(|p| p.parse().ok()).unwrap_or(4434u16);
        (host, port)
    };

    let ech_b64 = cfg.ech_config_base64.clone().unwrap_or_default();
    let ech_token = cfg.ech_token.clone().unwrap_or_default();

    Ok(GuiConfig {
        psk_hex: cfg.psk_hex.clone(),
        server_addr,
        server_port,
        server_name: cfg.server_name.clone(),
        ws_url: if cfg.protocol == "ws" { cfg.server_addr.clone() } else { "wss://gidy.eu.cc/ws".into() },
        ech_config_base64: ech_b64,
        ech_token,
        socks5_addr: cfg.listen_addr.to_string().split(':').next().unwrap_or("127.0.0.1").to_string(),
        socks5_port: cfg.listen_addr.port(),
        http_addr: "127.0.0.1".into(),
        http_port: 5556,
        protocol: cfg.protocol.clone(),
        mode: "global".into(),
        auto_start: false,
        auto_connect: false,
        minimize_to_tray: true,
        log_retention_days: 7,
        theme: "dark".into(),
        theme_color: "blue".into(),
        log_level: cfg.log_level.clone(),
    })
}

#[tauri::command]
pub async fn update_config(
    state: tauri::State<'_, AppState>,
    config: GuiConfig,
) -> Result<GuiConfig, String> {
    let mut proxy = state.proxy.lock();

    // Prevent config changes while running
    if proxy.running {
        return Err("Cannot update config while proxy is running".into());
    }

    let listen_addr: SocketAddr = format!("{}:{}", config.socks5_addr, config.socks5_port)
        .parse()
        .map_err(|e| format!("invalid listen addr: {}", e))?;

    // WS protocol uses full URL as server_addr; others use host:port
    let server_addr = if config.protocol == "ws" {
        config.ws_url.clone()
    } else {
        format!("{}:{}", config.server_addr, config.server_port)
    };

    let ech_config_base64 = if config.ech_config_base64.is_empty() {
        None
    } else {
        Some(config.ech_config_base64.clone())
    };

    let ech_token = if config.ech_token.is_empty() {
        None
    } else {
        Some(config.ech_token.clone())
    };

    proxy.config = ClientConfig {
        psk_hex: config.psk_hex.clone(),
        server_addr,
        server_name: config.server_name.clone(),
        listen_addr,
        log_level: config.log_level.clone(),
        bandwidth_kbps: 0,
        log_level_gidy: "basic".into(),
        log_dir: None,
        keychain_path: None,
        cover_traffic: false,
        protocol: config.protocol.clone(),
        ech_config_base64,
        ech_token,
    };

    Ok(config)
}

#[tauri::command]
pub async fn get_status(state: tauri::State<'_, AppState>) -> Result<ProxyStatus, String> {
    let proxy = state.proxy.lock();
    Ok(ProxyStatus {
        running: proxy.running,
        connected: proxy.connected,
        error: proxy.error.clone(),
    })
}

#[tauri::command]
pub async fn generate_psk() -> Result<String, String> {
    let psk = gidy_core::generate_psk();
    Ok(psk.iter().map(|b| format!("{:02x}", b)).collect())
}

/// Fetch fresh ECH config from the server API.
/// If the proxy is running, route the request through the local SOCKS5 proxy
/// (so it goes through the ECH tunnel and bypasses GFW).
/// If the proxy is not running, try direct connection (works in non-GFW environments).
#[tauri::command]
pub async fn refresh_ech(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let (server_addr, ech_token, proxy_running, socks5_port) = {
        let proxy = state.proxy.lock();
        (
            proxy.config.server_addr.clone(),
            proxy.config.ech_token.clone(),
            proxy.running,
            proxy.config.listen_addr.port(),
        )
    };

    let ech_token = ech_token.ok_or_else(|| "未配置 ECH Token，请在设置中填写".to_string())?;

    let url = url::Url::parse(&server_addr)
        .map_err(|e| format!("invalid URL: {}", e))?;
    let host = url.host_str().ok_or_else(|| "no host in URL".to_string())?;

    let api_url = format!("https://{}/ech-config?token={}", host, ech_token);
    tracing::info!("refresh_ech: fetching {}", api_url);

    // Build reqwest client — route through SOCKS5 proxy if proxy is running
    let mut client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15));

    if proxy_running {
        let socks5_proxy = format!("socks5://127.0.0.1:{}", socks5_port);
        tracing::info!("refresh_ech: routing through SOCKS5 proxy {}", socks5_proxy);
        let proxy = reqwest::Proxy::all(&socks5_proxy)
            .map_err(|e| format!("proxy config error: {}", e))?;
        client_builder = client_builder.proxy(proxy);
    }

    let client = client_builder
        .build()
        .map_err(|e| format!("HTTP client build: {}", e))?;

    let resp = client.get(&api_url)
        .send().await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body = resp.text().await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let ech_base64 = body.trim();

    // Try parsing as JSON first
    let ech_base64 = if ech_base64.starts_with('{') {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(ech_base64) {
            // Support nested JSON like {"domains":{"gidy.eu.cc":{"ech_config_base64":"..."}}}
            // or flat {"ech_config_base64":"..."}
            val.get("ech_config_base64")
                .or_else(|| val.get("ech"))
                .or_else(|| val.get("ech_config"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    // Try nested: domains.<name>.ech_config_base64
                    val.get("domains")
                        .and_then(|d| d.as_object())
                        .and_then(|obj| obj.values().next())
                        .and_then(|v| v.get("ech_config_base64"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or(ech_base64.to_string())
                })
        } else {
            ech_base64.to_string()
        }
    } else {
        ech_base64.to_string()
    };

    // Validate base64
    base64::engine::general_purpose::STANDARD.decode(&ech_base64)
        .map_err(|e| format!("返回数据无效 (base64 解码失败): {}", e))?;

    // Update config
    {
        let mut proxy = state.proxy.lock();
        proxy.config.ech_config_base64 = Some(ech_base64.clone());
    }

    tracing::info!("refresh_ech: got new ECH config ({} bytes)", ech_base64.len());
    Ok(ech_base64)
}

/// Return recent connection log entries (most recent last).
#[tauri::command]
pub async fn get_connection_logs(state: tauri::State<'_, AppState>) -> Result<Vec<LogEntry>, String> {
    let proxy = state.proxy.lock();
    let logs = proxy.connection_logs.lock();
    Ok(logs.clone())
}

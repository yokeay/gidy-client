use gidy_client_core::client::GidyClient;
use gidy_client_core::config::ClientConfig;
use gidy_client_core::proxy::Socks5Server;
use gidy_client_core::stats::TrafficStats;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

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
    }

    impl Default for ProxyState {
        fn default() -> Self {
            Self {
                running: false,
                connected: false,
                error: None,
                config: super::ClientConfig {
                    psk_hex: "4f3915417e21b4d3c54bb378c1fc66657b7a02626e688198438ad7a12b58270a".into(),
                    server_addr: "gidy.eu.cc:443".into(),
                    server_name: "gidy.eu.cc".into(),
                    listen_addr: "127.0.0.1:1080".parse().unwrap(),
                    log_level: "info".into(),
                    bandwidth_kbps: 0,
                    log_level_gidy: "basic".into(),
                    log_dir: None,
                    keychain_path: None,
                    cover_traffic: false,
                    protocol: "h2".into(),
                },
                stats: super::TrafficStats::new(),
                shutdown_tx: None,
                start_time: None,
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
            socks5_addr: "127.0.0.1".into(),
            socks5_port: 1080,
            http_addr: "127.0.0.1".into(),
            http_port: 8080,
            protocol: "h2".into(),
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
    let (config, stats) = {
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
        (proxy.config.clone(), proxy.stats.clone())
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

    let listen_str = config.listen_addr.to_string();
    tokio::spawn(async move {
        let socks5 = Socks5Server::new(listen_str, conn, stats);
        tokio::select! {
            result = socks5.run() => {
                if let Err(e) = result {
                    tracing::error!("socks5 server: {}", e);
                }
            }
            _ = shutdown_rx => {
                tracing::info!("proxy shutdown requested");
            }
        }
    });

    Ok(ProxyStatus {
        running: true,
        connected: true,
        error: None,
    })
}

#[tauri::command]
pub async fn disconnect(state: tauri::State<'_, AppState>) -> Result<ProxyStatus, String> {
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
    Ok(GuiConfig {
        psk_hex: cfg.psk_hex.clone(),
        server_addr: cfg.server_addr.split(':').next().unwrap_or("").to_string(),
        server_port: cfg.server_addr.split(':').nth(1).and_then(|p| p.parse().ok()).unwrap_or(4434),
        server_name: cfg.server_name.clone(),
        socks5_addr: cfg.listen_addr.to_string().split(':').next().unwrap_or("127.0.0.1").to_string(),
        socks5_port: cfg.listen_addr.port(),
        http_addr: "127.0.0.1".into(),
        http_port: 8080,
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

    let server_addr = format!("{}:{}", config.server_addr, config.server_port);
    let listen_addr: SocketAddr = format!("{}:{}", config.socks5_addr, config.socks5_port)
        .parse()
        .map_err(|e| format!("invalid listen addr: {}", e))?;

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

use std::sync::Arc;
use gidy_client_core::config::ClientConfig;
use gidy_client_core::stats::TrafficStats;

#[derive(Clone)]
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
            psk_hex: String::from("0000000000000000000000000000000000000000000000000000000000000000"),
            server_addr: String::from("127.0.0.1"),
            server_port: 4433,
            server_name: String::from("localhost"),
            socks5_addr: String::from("127.0.0.1"),
            socks5_port: 1080,
            http_addr: String::from("127.0.0.1"),
            http_port: 8080,
            protocol: String::from("gidy"),
            mode: String::from("global"),
            auto_start: false,
            auto_connect: false,
            minimize_to_tray: false,
            log_retention_days: 7,
            theme: String::from("dark"),
            theme_color: String::from("blue"),
            log_level: String::from("info"),
        }
    }
}

impl GuiConfig {
    pub fn to_client_config(&self) -> ClientConfig {
        ClientConfig {
            psk_hex: self.psk_hex.clone(),
            server_addr: format!("{}:{}", self.server_addr, self.server_port),
            listen_addr: format!("{}:{}", self.socks5_addr, self.socks5_port).parse().unwrap(),
            server_name: self.server_name.clone(),
            log_level: self.log_level.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ProxyStatus {
    pub running: bool,
    pub connected: bool,
    pub error: Option<String>,
}

// Re-export TrafficSnapshot from core as StatsSnapshot for convenience
pub use gidy_client_core::stats::TrafficSnapshot as StatsSnapshot;

pub struct ProxyState {
    pub running: bool,
    pub connected: bool,
    pub error: Option<String>,
    pub config: GuiConfig,
    pub stats: Arc<TrafficStats>,
    pub start_time: Option<std::time::Instant>,
}

impl ProxyState {
    pub fn new() -> Self {
        Self {
            running: false,
            connected: false,
            error: None,
            config: GuiConfig::default(),
            stats: TrafficStats::new(),
            start_time: None,
        }
    }

    pub fn status(&self) -> ProxyStatus {
        ProxyStatus {
            running: self.running,
            connected: self.connected,
            error: self.error.clone(),
        }
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        self.stats.snapshot()
    }
}

pub fn generate_psk() -> String {
    let key = gidy_core::generate_psk();
    hex::encode(key)
}

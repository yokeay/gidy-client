use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub psk_hex: String,

    #[serde(default = "default_server_addr")]
    pub server_addr: String,

    #[serde(default = "default_listen_addr")]
    pub listen_addr: SocketAddr,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default = "default_server_name")]
    pub server_name: String,

    #[serde(default)]
    pub bandwidth_kbps: u32,

    #[serde(default = "default_log_level_gidy")]
    pub log_level_gidy: String,

    pub log_dir: Option<PathBuf>,

    pub keychain_path: Option<PathBuf>,

    #[serde(default)]
    pub cover_traffic: bool,
}

fn default_server_addr() -> String { "127.0.0.1:443".into() }
fn default_listen_addr() -> SocketAddr { "127.0.0.1:1080".parse().unwrap() }
fn default_log_level() -> String { "info".into() }
fn default_server_name() -> String { "gidy.example.com".into() }
fn default_log_level_gidy() -> String { "basic".into() }

impl ClientConfig {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read config {}: {}", path, e))?;
        toml::from_str(&content)
            .map_err(|e| format!("failed to parse config: {}", e))
    }

    pub fn psk(&self) -> Result<[u8; 32], String> {
        gidy_core::validate_psk_hex(&self.psk_hex)
            .map_err(|e| e.to_string())
    }

    pub fn gidy_log_level(&self) -> gidy_core::LogLevel {
        match self.log_level_gidy.as_str() {
            "off" => gidy_core::LogLevel::Off,
            "basic" => gidy_core::LogLevel::Basic,
            "detail" => gidy_core::LogLevel::Detail,
            "full" => gidy_core::LogLevel::Full,
            _ => gidy_core::LogLevel::Basic,
        }
    }
}

pub fn generate_default_config() -> ClientConfig {
    ClientConfig {
        psk_hex: "0000000000000000000000000000000000000000000000000000000000000000".into(),
        server_addr: default_server_addr(),
        listen_addr: default_listen_addr(),
        log_level: default_log_level(),
        server_name: default_server_name(),
        bandwidth_kbps: 0,
        log_level_gidy: default_log_level_gidy(),
        log_dir: None,
        keychain_path: None,
        cover_traffic: false,
    }
}

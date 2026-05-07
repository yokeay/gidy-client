use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// 预共享密钥 (hex encoded, 64 chars = 32 bytes)
    pub psk_hex: String,

    /// gidy-server 地址
    #[serde(default = "default_server_addr")]
    pub server_addr: String,

    /// 本地 SOCKS5 代理监听地址
    #[serde(default = "default_listen_addr")]
    pub listen_addr: SocketAddr,

    /// 日志级别
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// 服务器名称 (SNI)
    #[serde(default = "default_server_name")]
    pub server_name: String,
}

fn default_server_addr() -> String {
    "127.0.0.1:443".into()
}

fn default_listen_addr() -> SocketAddr {
    "127.0.0.1:1080".parse().unwrap()
}

fn default_log_level() -> String {
    "info".into()
}

fn default_server_name() -> String {
    "gidy.example.com".into()
}

impl ClientConfig {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read config {}: {}", path, e))?;
        toml::from_str(&content)
            .map_err(|e| format!("failed to parse config: {}", e))
    }

    pub fn psk(&self) -> Result<[u8; 32], String> {
        let hex = self.psk_hex.trim();
        if hex.len() != 64 {
            return Err(format!("psk_hex must be 64 hex characters, got {}", hex.len()));
        }
        let mut psk = [0u8; 32];
        let bytes = hex_decode(hex)?;
        psk.copy_from_slice(&bytes);
        Ok(psk)
    }
}

pub fn generate_default_config() -> ClientConfig {
    ClientConfig {
        psk_hex: "0000000000000000000000000000000000000000000000000000000000000000".into(),
        server_addr: default_server_addr(),
        listen_addr: default_listen_addr(),
        log_level: default_log_level(),
        server_name: default_server_name(),
    }
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| format!("invalid hex byte at {}: {}", i, e))
        })
        .collect()
}

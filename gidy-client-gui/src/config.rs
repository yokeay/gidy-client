use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    pub psk_hex: String,
    pub server_addr: String,
    pub server_name: String,
    pub listen_addr: String,
    pub log_level: String,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            psk_hex: String::new(),
            server_addr: "49.12.243.33:4433".into(),
            server_name: "localhost".into(),
            listen_addr: "127.0.0.1:1080".into(),
            log_level: "info".into(),
        }
    }
}

impl GuiConfig {
    pub fn load() -> Self {
        let config_path = std::env::current_exe()
            .ok()
            .map(|p| {
                let mut p = p.parent().map(|d| d.to_path_buf()).unwrap_or_default();
                p.push("gidy-client.toml");
                p
            })
            .unwrap_or_else(|| std::path::PathBuf::from("gidy-client.toml"));

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(s) => match toml::from_str(&s) {
                    Ok(c) => return c,
                    Err(e) => tracing::warn!("config parse: {}, using defaults", e),
                },
                Err(e) => tracing::warn!("config read: {}, using defaults", e),
            }
        }

        let default = Self::default();
        if let Ok(s) = toml::to_string_pretty(&default) {
            let _ = std::fs::write(&config_path, s);
        }
        default
    }

    #[allow(dead_code)]
    pub fn psk(&self) -> Result<[u8; 32], String> {
        let hex = self.psk_hex.trim();
        if hex.len() != 64 {
            return Err("PSK must be 64 hex characters".into());
        }
        let mut psk = [0u8; 32];
        for i in 0..32 {
            psk[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|e| format!("psk hex: {}", e))?;
        }
        Ok(psk)
    }
}

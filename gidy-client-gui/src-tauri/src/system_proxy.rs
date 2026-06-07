//! System proxy settings – platform-specific implementations.
//!
//! - Windows: sets/clears Internet Options proxy via registry + WinInet notify
//! - macOS:   sets/clears HTTP/HTTPS/SOCKS proxy via `networksetup`
//! - Linux:   no-op (user configures proxy manually or via DE settings)

use std::sync::Arc;
use parking_lot::Mutex;

/// Proxy settings to apply to the system.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SystemProxy {
    pub host: String,
    pub port: u16,
    pub enabled: bool,
}

impl SystemProxy {
    #[allow(dead_code)]
    pub fn proxy_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Manages the system proxy settings.
pub struct SystemProxyManager {
    /// The proxy string that was set, so we can restore/clear it.
    current: Arc<Mutex<Option<String>>>,
}

impl SystemProxyManager {
    pub fn new() -> Self {
        Self {
            current: Arc::new(Mutex::new(None)),
        }
    }

    /// Enable system proxy with the given host:port.
    pub fn set(&self, host: &str, port: u16) -> Result<(), String> {
        let proxy_str = format!("{}:{}", host, port);
        self.set_platform(host, port)?;
        *self.current.lock() = Some(proxy_str);
        tracing::info!("system proxy set: {}:{}", host, port);
        Ok(())
    }

    /// Disable system proxy.
    pub fn clear(&self) -> Result<(), String> {
        self.clear_platform()?;
        *self.current.lock() = None;
        tracing::info!("system proxy cleared");
        Ok(())
    }

    // ── Windows ──────────────────────────────────────────────────────
    #[cfg(target_os = "windows")]
    fn set_platform(&self, host: &str, port: u16) -> Result<(), String> {
        let key_path = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

        let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            .open_subkey_with_flags(key_path, winreg::enums::KEY_WRITE)
            .or_else(|_| {
                winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
                    .create_subkey(key_path)
                    .map(|(k, _)| k)
            })
            .map_err(|e| format!("open registry key: {}", e))?;

        key.set_value("ProxyEnable", &1u32)
            .map_err(|e| format!("set ProxyEnable: {}", e))?;

        let proxy_str = format!("{}:{}", host, port);
        key.set_value("ProxyServer", &proxy_str)
            .map_err(|e| format!("set ProxyServer: {}", e))?;

        Self::notify_wininet();
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn clear_platform(&self) -> Result<(), String> {
        let key_path = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

        let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            .open_subkey_with_flags(key_path, winreg::enums::KEY_WRITE)
            .map_err(|e| format!("open registry key: {}", e))?;

        key.set_value("ProxyEnable", &0u32)
            .map_err(|e| format!("set ProxyEnable: {}", e))?;

        Self::notify_wininet();
        Ok(())
    }

    /// Notify Windows to refresh proxy settings via WinInet.
    #[cfg(target_os = "windows")]
    fn notify_wininet() {
        #[link(name = "wininet")]
        extern "system" {
            fn InternetSetOptionA(
                internet: isize,
                option: u32,
                buffer: *const core::ffi::c_void,
                buflen: u32,
            ) -> i32;
        }

        const INTERNET_OPTION_SETTINGS_CHANGED: u32 = 39;
        const INTERNET_OPTION_REFRESH: u32 = 37;

        unsafe {
            InternetSetOptionA(0, INTERNET_OPTION_SETTINGS_CHANGED, std::ptr::null(), 0);
            InternetSetOptionA(0, INTERNET_OPTION_REFRESH, std::ptr::null(), 0);
        }
    }

    // ── macOS ────────────────────────────────────────────────────────
    #[cfg(target_os = "macos")]
    fn set_platform(&self, host: &str, port: u16) -> Result<(), String> {
        // Try common network services; iterate all if specific ones fail
        let services = Self::get_network_services()?;

        for svc in &services {
            // HTTP proxy
            let _ = Self::run_networksetup(&[
                "-setwebproxy", svc, host, &port.to_string(), &"0".to_string(),
            ]);
            let _ = Self::run_networksetup(&["-setwebproxystate", svc, "on"]);

            // HTTPS proxy
            let _ = Self::run_networksetup(&[
                "-setsecurewebproxy", svc, host, &port.to_string(), &"0".to_string(),
            ]);
            let _ = Self::run_networksetup(&["-setsecurewebproxystate", svc, "on"]);

            // SOCKS proxy on the SOCKS5 port (port param = http_port, socks = original)
            // We use the same host:port for simplicity
            let _ = Self::run_networksetup(&[
                "-setsocksfirewallproxy", svc, host, &port.to_string(),
            ]);
            let _ = Self::run_networksetup(&["-setsocksfirewallproxystate", svc, "on"]);
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn clear_platform(&self) -> Result<(), String> {
        let services = Self::get_network_services()?;

        for svc in &services {
            let _ = Self::run_networksetup(&["-setwebproxystate", svc, "off"]);
            let _ = Self::run_networksetup(&["-setsecurewebproxystate", svc, "off"]);
            let _ = Self::run_networksetup(&["-setsocksfirewallproxystate", svc, "off"]);
        }
        Ok(())
    }

    /// Get list of network services on macOS.
    #[cfg(target_os = "macos")]
    fn get_network_services() -> Result<Vec<String>, String> {
        let output = std::process::Command::new("networksetup")
            .args(["-listallnetworkservices"])
            .output()
            .map_err(|e| format!("run networksetup: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let services: Vec<String> = stdout
            .lines()
            .skip(1) // First line is header: "An asterisk (*) denotes..."
            .filter(|l| !l.is_empty() && !l.starts_with('*'))
            .map(|l| l.trim().to_string())
            .collect();

        if services.is_empty() {
            // Fallback to common service names
            return Ok(vec!["Wi-Fi".to_string(), "Ethernet".to_string()]);
        }
        Ok(services)
    }

    /// Run networksetup command on macOS.
    #[cfg(target_os = "macos")]
    fn run_networksetup(args: &[&str]) -> Result<(), String> {
        std::process::Command::new("networksetup")
            .args(args)
            .output()
            .map_err(|e| format!("networksetup {}: {}", args.join(" "), e))?;
        Ok(())
    }

    // ── Linux ────────────────────────────────────────────────────────
    #[cfg(target_os = "linux")]
    fn set_platform(&self, host: &str, port: u16) -> Result<(), String> {
        // Try GNOME gsettings first
        let http_url = format!("http://{}:{}", host, port);
        let result = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy", "mode", "manual"])
            .output();

        if let Ok(_) = result {
            let _ = std::process::Command::new("gsettings")
                .args(["set", "org.gnome.system.proxy.http", "host", host])
                .output();
            let _ = std::process::Command::new("gsettings")
                .args(["set", "org.gnome.system.proxy.http", "port", &port.to_string()])
                .output();
            let _ = std::process::Command::new("gsettings")
                .args(["set", "org.gnome.system.proxy.https", "host", host])
                .output();
            let _ = std::process::Command::new("gsettings")
                .args(["set", "org.gnome.system.proxy.https", "port", &port.to_string()])
                .output();
            let _ = std::process::Command::new("gsettings")
                .args(["set", "org.gnome.system.proxy.socks", "host", host])
                .output();
            let _ = std::process::Command::new("gsettings")
                .args(["set", "org.gnome.system.proxy.socks", "port", &port.to_string()])
                .output();
        } else {
            // Not GNOME — log a hint; the user must configure proxy manually
            tracing::warn!(
                "Linux system proxy: could not auto-configure (not GNOME?). \
                 Please set HTTP/HTTPS/SOCKS proxy to {}:{} manually.",
                host, port
            );
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn clear_platform(&self) -> Result<(), String> {
        let result = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy", "mode", "none"])
            .output();

        if result.is_err() {
            tracing::warn!(
                "Linux system proxy: could not auto-clear. \
                 Please disable proxy settings manually."
            );
        }
        Ok(())
    }
}

impl Drop for SystemProxyManager {
    fn drop(&mut self) {
        let _ = self.clear();
    }
}

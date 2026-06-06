//! Windows system proxy settings via registry.
//! Sets/clears the Internet Options proxy for the current user.

use std::sync::Arc;
use parking_lot::Mutex;

/// Proxy settings to apply to the system.
#[derive(Debug, Clone)]
pub struct SystemProxy {
    pub host: String,
    pub port: u16,
    pub enabled: bool,
}

impl SystemProxy {
    pub fn proxy_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Manages the Windows system proxy settings.
/// On `set`, writes proxy enable=1 and proxy server to registry.
/// On `clear`, writes proxy enable=0.
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

        // Registry path for Internet Settings
        let key_path = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

        // Open or create the key
        let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            .open_subkey_with_flags(key_path, winreg::enums::KEY_WRITE)
            .or_else(|_| {
                winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
                    .create_subkey(key_path)
                    .map(|(k, _)| k)
            })
            .map_err(|e| format!("open registry key: {}", e))?;

        // Set ProxyEnable = 1
        key.set_value("ProxyEnable", &1u32)
            .map_err(|e| format!("set ProxyEnable: {}", e))?;

        // Set ProxyServer = "host:port"
        key.set_value("ProxyServer", &proxy_str)
            .map_err(|e| format!("set ProxyServer: {}", e))?;

        // Notify the system that proxy settings changed
        Self::notify_change();

        *self.current.lock() = Some(proxy_str);
        tracing::info!("system proxy set: {}:{}", host, port);
        Ok(())
    }

    /// Disable system proxy.
    pub fn clear(&self) -> Result<(), String> {
        let key_path = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

        let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            .open_subkey_with_flags(key_path, winreg::enums::KEY_WRITE)
            .map_err(|e| format!("open registry key: {}", e))?;

        // Set ProxyEnable = 0
        key.set_value("ProxyEnable", &0u32)
            .map_err(|e| format!("set ProxyEnable: {}", e))?;

        // Notify the system
        Self::notify_change();

        *self.current.lock() = None;
        tracing::info!("system proxy cleared");
        Ok(())
    }

    /// Tell Windows to refresh proxy settings.
    /// This calls InternetSetOption with INTERNET_OPTION_SETTINGS_CHANGED
    /// and INTERNET_OPTION_REFRESH to make changes take effect immediately.
    fn notify_change() {
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
            // INTERNET_OPTION_SETTINGS_CHANGED — notify that settings have changed
            InternetSetOptionA(0, INTERNET_OPTION_SETTINGS_CHANGED, std::ptr::null(), 0);
            // INTERNET_OPTION_REFRESH — refresh the current settings
            InternetSetOptionA(0, INTERNET_OPTION_REFRESH, std::ptr::null(), 0);
        }
    }
}

impl Drop for SystemProxyManager {
    fn drop(&mut self) {
        // Ensure proxy is cleared when the manager is dropped
        let _ = self.clear();
    }
}

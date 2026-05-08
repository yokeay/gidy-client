// Windows system proxy configuration via WinINet API.
// On non-Windows platforms these are no-ops.

#[cfg(target_os = "windows")]
mod imp {
    use windows::Win32::Networking::WinInet::{
        InternetSetOptionW, INTERNET_OPTION_PER_CONNECTION_OPTION,
        INTERNET_OPTION_PROXY_SETTINGS_CHANGED, INTERNET_OPTION_REFRESH,
        INTERNET_PER_CONN_FLAGS, INTERNET_PER_CONN_OPTIONW,
        INTERNET_PER_CONN_OPTION_LISTW, INTERNET_PER_CONN_PROXY_SERVER,
        PROXY_TYPE_DIRECT,
    };

    pub fn set_proxy(proxy_addr: &str) {
        unsafe {
            let proxy_w: Vec<u16> = proxy_addr
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mut option: INTERNET_PER_CONN_OPTIONW = std::mem::zeroed();
            option.dwOption = INTERNET_PER_CONN_PROXY_SERVER;
            option.Value.pszValue = windows::core::PWSTR(proxy_w.as_ptr() as *mut _);

            let mut list: INTERNET_PER_CONN_OPTION_LISTW = std::mem::zeroed();
            list.dwSize = std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32;
            list.dwOptionCount = 1;
            list.pOptions = &mut option;

            let _ = InternetSetOptionW(
                None,
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                Some(&list as *const _ as *const std::ffi::c_void),
                std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
            );

            let _ = InternetSetOptionW(None, INTERNET_OPTION_PROXY_SETTINGS_CHANGED, None, 0);
            let _ = InternetSetOptionW(None, INTERNET_OPTION_REFRESH, None, 0);
        }

        tracing::info!("Windows system proxy set to: {}", proxy_addr);
    }

    pub fn clear_proxy() {
        unsafe {
            let mut option: INTERNET_PER_CONN_OPTIONW = std::mem::zeroed();
            option.dwOption = INTERNET_PER_CONN_FLAGS;
            option.Value.dwValue = PROXY_TYPE_DIRECT;

            let mut list: INTERNET_PER_CONN_OPTION_LISTW = std::mem::zeroed();
            list.dwSize = std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32;
            list.dwOptionCount = 1;
            list.pOptions = &mut option;

            let _ = InternetSetOptionW(
                None,
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                Some(&list as *const _ as *const std::ffi::c_void),
                std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
            );

            let _ = InternetSetOptionW(None, INTERNET_OPTION_PROXY_SETTINGS_CHANGED, None, 0);
            let _ = InternetSetOptionW(None, INTERNET_OPTION_REFRESH, None, 0);
        }

        tracing::info!("Windows system proxy cleared");
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
mod imp {
    pub fn set_proxy(_proxy_addr: &str) {
        tracing::info!("set_proxy is no-op on non-Windows");
    }

    pub fn clear_proxy() {
        tracing::info!("clear_proxy is no-op on non-Windows");
    }
}

#[allow(unused_imports)]
pub use imp::*;

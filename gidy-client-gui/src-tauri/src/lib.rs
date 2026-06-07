mod commands;
mod system_proxy;

use commands::ProxyState;
use parking_lot::Mutex;
use std::sync::Arc;
use system_proxy::SystemProxyManager;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

#[cfg(target_os = "windows")]
fn apply_rounded_corners(hwnd: isize) {
    // DwmSetWindowAttribute: DWMWA_WINDOW_CORNER_PREFERENCE=33, DWMWCP_ROUND=2
    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: isize,
            attr: u32,
            pv_attr: *const core::ffi::c_void,
            cb_attr: u32,
        ) -> i32;
    }
    unsafe {
        let pref: u32 = 2; // DWMWCP_ROUND
        DwmSetWindowAttribute(
            hwnd,
            33, // DWMWA_WINDOW_CORNER_PREFERENCE
            &pref as *const u32 as *const core::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

pub struct AppState {
    pub proxy: Arc<Mutex<ProxyState>>,
    pub sys_proxy: Arc<Mutex<SystemProxyManager>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            proxy: Arc::new(Mutex::new(ProxyState::default())),
            sys_proxy: Arc::new(Mutex::new(SystemProxyManager::new())),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::get_stats,
            commands::get_config,
            commands::update_config,
            commands::get_status,
            commands::generate_psk,
            commands::refresh_ech,
            commands::get_connection_logs,
        ])
        .setup(|app| {
            let _handle = app.handle().clone();

            // Set WebView2 background to fully transparent so CSS border-radius works
            if let Some(win) = app.get_webview_window("main") {
                // rgba(0,0,0,0) — fully transparent
                let _ = win.set_background_color(Some((0_u8, 0_u8, 0_u8, 0_u8).into()));

                // Apply rounded corners via DWM on Windows 11+
                #[cfg(target_os = "windows")]
                if let Ok(hwnd) = win.hwnd() {
                    apply_rounded_corners(hwnd.0 as *mut _ as isize);
                }            }

            // Build tray menu
            let show_item = MenuItem::with_id(app, "show", "回主界面", true, None::<&str>)?;
            let toggle_item = MenuItem::with_id(app, "toggle", "断开连接", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出软件", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &toggle_item, &quit_item])?;

            // Load icon
            let icon = Image::from_path("icons/32x32.png")
                .or_else(|_| Image::from_path("icons/128x128.png"))
                .ok();

            let mut builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("Gidy-Client")
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "toggle" => {
                            // Emit event to frontend to handle connect/disconnect
                            let _ = app.emit("tray-toggle-connection", ());
                        }
                        "quit" => {
                            // Directly exit the process from tray menu
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // Left-click: show window
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(win) = tray.app_handle().get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                });

            if let Some(icon) = icon {
                builder = builder.icon(icon);
            }

            builder.build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

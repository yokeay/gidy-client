mod commands;

use commands::ProxyState;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

pub struct AppState {
    pub proxy: Arc<Mutex<ProxyState>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            proxy: Arc::new(Mutex::new(ProxyState::default())),
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
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::get_stats,
            commands::get_config,
            commands::update_config,
            commands::get_status,
            commands::generate_psk,
        ])
        .setup(|app| {
            let _handle = app.handle().clone();

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
                            // Emit quit event so frontend can clean up
                            let _ = app.emit("tray-quit", ());
                            std::process::exit(0);
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

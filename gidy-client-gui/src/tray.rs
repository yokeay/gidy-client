#[cfg(any(target_os = "windows", target_os = "macos"))]
mod imp {
    use tray_icon::{
        TrayIcon, TrayIconBuilder, TrayIconEvent,
        menu::{Menu, MenuEvent, MenuId, MenuItem},
    };

    const TRAY_ICON_PNG: &[u8] = include_bytes!("../assets/tray-icon.png");

    fn load_tray_icon() -> tray_icon::Icon {
        let img = image::load_from_memory(TRAY_ICON_PNG)
            .expect("Failed to decode tray icon");
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        tray_icon::Icon::from_rgba(rgba.into_raw(), w, h)
            .expect("Failed to create tray icon")
    }

    pub struct SystemTray {
        tray: Option<TrayIcon>,
        menu_show_id: MenuId,
        menu_exit_id: MenuId,
    }

    #[derive(Debug, PartialEq)]
    pub enum TrayEvent {
        None,
        ShowWindow,
        Exit,
    }

    impl SystemTray {
        pub fn new(show_label: &str, exit_label: &str, tooltip: &str) -> Self {
            let icon = load_tray_icon();

            let menu_show = MenuItem::new(show_label.to_string(), true, None);
            let menu_exit = MenuItem::new(exit_label.to_string(), true, None);
            let menu_show_id = menu_show.id().clone();
            let menu_exit_id = menu_exit.id().clone();

            let menu = Menu::new();
            menu.append(&menu_show).ok();
            menu.append(&menu_exit).ok();

            let tray = TrayIconBuilder::new()
                .with_icon(icon)
                .with_tooltip(tooltip)
                .with_menu(Box::new(menu))
                .build()
                .ok();

            // Initially hidden - shown when window minimizes to tray
            if let Some(ref t) = tray {
                t.set_visible(false).ok();
            }

            Self {
                tray,
                menu_show_id,
                menu_exit_id,
            }
        }

        pub fn set_visible(&mut self, visible: bool) {
            if let Some(ref t) = self.tray {
                t.set_visible(visible).ok();
            }
        }

        pub fn poll_events(&self) -> TrayEvent {
            // Check menu events
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == self.menu_show_id {
                    return TrayEvent::ShowWindow;
                }
                if event.id == self.menu_exit_id {
                    return TrayEvent::Exit;
                }
            }

            // Check tray icon events (double-click to show)
            if let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                    return TrayEvent::ShowWindow;
                }
            }

            TrayEvent::None
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod imp {
    pub struct SystemTray;

    #[derive(Debug, PartialEq)]
    pub enum TrayEvent {
        None,
        #[allow(dead_code)]
        ShowWindow,
        #[allow(dead_code)]
        Exit,
    }

    impl SystemTray {
        pub fn new(_show_label: &str, _exit_label: &str, _tooltip: &str) -> Self {
            Self
        }

        pub fn set_visible(&mut self, _visible: bool) {}

        pub fn poll_events(&self) -> TrayEvent {
            TrayEvent::None
        }
    }
}

pub use imp::*;

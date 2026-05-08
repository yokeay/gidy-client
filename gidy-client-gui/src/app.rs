use std::sync::Arc;
use parking_lot::Mutex;
use egui::{Align2, Vec2, StrokeKind};

use gidy_client_core::client::GidyClient;
use gidy_client_core::proxy::Socks5Server;

use crate::sidebar::{NavPage, sidebar_ui};
use crate::theme::{AppTheme, ThemeMode, ThemeColor};
use crate::i18n::{I18n, Lang};
use crate::proxy_state::{GuiConfig, ProxyState};
use crate::dashboard::{DashboardPage, dashboard_ui};
use crate::system_config::{ConfigForm, system_config_ui};
use crate::traffic_monitor::{TrafficMonitorPage, traffic_monitor_ui};
use crate::user_settings::{SettingsForm, user_settings_ui};
use crate::about::about_ui;

pub struct GidyApp {
    page: NavPage,
    theme: AppTheme,
    i18n: I18n,
    proxy_state: Arc<Mutex<ProxyState>>,

    config_form: ConfigForm,
    settings_form: SettingsForm,
    dashboard: DashboardPage,
    traffic_monitor: TrafficMonitorPage,

    rt: tokio::runtime::Runtime,
    proxy_handle: Option<tokio::task::JoinHandle<()>>,

    toast: Option<(String, f64)>,
}

impl GidyApp {
    pub fn new() -> Self {
        let config = GuiConfig::default();
        let proxy_state = Arc::new(Mutex::new(ProxyState::new()));
        Self {
            page: NavPage::Dashboard,
            theme: AppTheme::new(ThemeMode::Dark, ThemeColor::Blue),
            i18n: I18n::new(),
            config_form: ConfigForm::from_config(&config),
            settings_form: SettingsForm::from_config(&config),
            dashboard: DashboardPage::new(),
            traffic_monitor: TrafficMonitorPage::new(),
            rt: tokio::runtime::Runtime::new().expect("tokio runtime"),
            proxy_handle: None,
            proxy_state,
            toast: None,
        }
    }

    fn start_proxy(&mut self) {
        let state = self.proxy_state.clone();
        let config = self.config_form.to_config().to_client_config();
        let listen_addr = format!("{}:{}",
            self.config_form.socks5_addr,
            self.config_form.socks5_port,
        );
        let stats = {
            let mut s = state.lock();
            s.running = true;
            s.connected = false;
            s.error = None;
            s.stats.clone()
        };

        let state_clone = state.clone();
        let handle = self.rt.spawn(async move {
            let client = match GidyClient::new(config.clone(), stats.clone()) {
                Ok(c) => c,
                Err(e) => {
                    let mut s = state_clone.lock();
                    s.running = false;
                    s.error = Some(e);
                    return;
                }
            };

            let conn = match client.connect().await {
                Ok(c) => c,
                Err(e) => {
                    let mut s = state_clone.lock();
                    s.running = false;
                    s.error = Some(e);
                    return;
                }
            };

            {
                let mut s = state_clone.lock();
                s.connected = true;
                s.start_time = Some(std::time::Instant::now());
                s.error = None;
            }

            let proxy = Socks5Server::new(listen_addr, conn, stats);
            if let Err(e) = proxy.run().await {
                let mut s = state_clone.lock();
                s.error = Some(e);
            }

            let mut s = state_clone.lock();
            s.running = false;
            s.connected = false;
        });

        self.proxy_handle = Some(handle);
    }

    fn stop_proxy(&mut self) {
        if let Some(h) = self.proxy_handle.take() {
            h.abort();
        }
        let mut s = self.proxy_state.lock();
        s.running = false;
        s.connected = false;
    }
}

impl eframe::App for GidyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.style_mut(|style| self.theme.apply_to_style(style));

        // Poll stats for chart
        {
            let state = self.proxy_state.lock();
            if state.running {
                let snap = state.snapshot();
                drop(state);
                self.dashboard.update_chart(&snap);
                self.traffic_monitor.update_chart(&snap);
            }
        }

        // Toast timer
        if let Some((_, remaining)) = &mut self.toast {
            *remaining -= ctx.input(|i| i.unstable_dt.max(0.016)) as f64;
            if *remaining <= 0.0 {
                self.toast = None;
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let available = ui.available_size();

                // Sidebar
                let sidebar_w = 180.0;
                let sidebar_rect = egui::Rect::from_min_size(
                    ui.cursor().left_top(),
                    Vec2::new(sidebar_w, available.y),
                );
                let mut sidebar_ui_inner = ui.child_ui(
                    sidebar_rect,
                    egui::Layout::top_down(egui::Align::Min),
                    None,
                );
                let next_page = sidebar_ui(&mut sidebar_ui_inner, self.page, &self.theme, &self.i18n);
                if next_page != self.page {
                    self.page = next_page;
                }

                // Divider
                ui.painter().line_segment(
                    [egui::pos2(sidebar_w, 0.0), egui::pos2(sidebar_w, available.y)],
                    egui::Stroke::new(1.0, self.theme.border),
                );

                // Content area
                let content_rect = egui::Rect::from_min_max(
                    egui::pos2(sidebar_w + 1.0, 0.0),
                    egui::pos2(available.x, available.y),
                );
                let mut content = ui.child_ui(
                    content_rect.shrink(24.0),
                    egui::Layout::top_down(egui::Align::Min),
                    None,
                );

                // Top bar
                let top_h = 36.0;
                let top_rect = egui::Rect::from_min_size(
                    content.cursor().left_top(),
                    Vec2::new(content.available_width(), top_h),
                );
                content.painter().text(
                    top_rect.left_center(),
                    Align2::LEFT_CENTER,
                    self.page.label(&self.i18n),
                    egui::FontId::proportional(16.0),
                    self.theme.fg,
                );

                // Language toggle
                let lang = self.i18n.lang();
                let lang_label = match lang { Lang::Zh => "EN", Lang::En => "ZH" };
                let lang_btn = egui::Rect::from_center_size(
                    top_rect.right_center() + Vec2::new(-28.0, 0.0),
                    Vec2::new(40.0, 24.0),
                );
                content.painter().rect_filled(lang_btn, egui::CornerRadius::same(6), self.theme.card_bg);
                content.painter().rect_stroke(lang_btn, egui::CornerRadius::same(6), egui::Stroke::new(1.0, self.theme.border), StrokeKind::Inside);
                content.painter().text(
                    lang_btn.center(),
                    Align2::CENTER_CENTER,
                    lang_label,
                    egui::FontId::proportional(11.0),
                    self.theme.muted,
                );
                if content.allocate_rect(lang_btn, egui::Sense::click()).clicked() {
                    self.i18n.set_lang(lang.toggle());
                }

                content.advance_cursor_after_rect(egui::Rect::from_min_size(
                    top_rect.left_top(),
                    Vec2::new(content.available_width(), top_h + 8.0),
                ));

                // Scrollable content
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(&mut content, |ui| {
                        let status = self.proxy_state.lock().status();
                        let snap = self.proxy_state.lock().snapshot();

                        let mut on_toggle: Option<bool> = None;
                        let mut on_save_config: bool = false;
                        let mut on_generate_psk: bool = false;
                        let mut on_save_settings: bool = false;
                        let mut on_theme_mode: Option<ThemeMode> = None;
                        let mut on_theme_color: Option<ThemeColor> = None;

                        match self.page {
                            NavPage::Dashboard => {
                                let _connect_label = if status.running {
                                    self.i18n.t("dashboard.stop")
                                } else {
                                    self.i18n.t("dashboard.start")
                                };
                                dashboard_ui(
                                    ui, &status, &snap, &mut self.dashboard.chart,
                                    _connect_label, &self.theme, &self.i18n, &mut on_toggle,
                                );
                            }
                            NavPage::SystemConfig => {
                                system_config_ui(
                                    ui, &mut self.config_form, &self.theme, &self.i18n,
                                    &mut on_save_config, &mut on_generate_psk,
                                );
                            }
                            NavPage::TrafficMonitor => {
                                traffic_monitor_ui(
                                    ui, &snap, &mut self.traffic_monitor.chart,
                                    &self.traffic_monitor.log, &self.theme, &self.i18n,
                                );
                            }
                            NavPage::UserSettings => {
                                user_settings_ui(
                                    ui, &mut self.settings_form, &mut self.theme, &self.i18n,
                                    &mut on_save_settings, &mut on_theme_mode, &mut on_theme_color,
                                );
                            }
                            NavPage::About => {
                                about_ui(ui, &self.theme, &self.i18n);
                            }
                        }

                        // Handle actions
                        if let Some(start) = on_toggle {
                            if start {
                                self.start_proxy();
                            } else {
                                self.stop_proxy();
                            }
                        }

                        if on_save_config {
                            let c = self.config_form.to_config();
                            self.proxy_state.lock().config = c;
                            self.config_form.toast = Some(self.i18n.t("systemConfig.saved").to_string());
                        }

                        if on_generate_psk {
                            self.config_form.psk_hex = crate::proxy_state::generate_psk();
                        }

                        if on_theme_mode.is_some() || on_theme_color.is_some() {
                            if let Some(m) = on_theme_mode {
                                if m != self.theme.mode {
                                    self.theme.toggle_mode();
                                }
                            }
                            if let Some(c) = on_theme_color {
                                self.theme.set_color(c);
                            }
                            let mut cfg = self.proxy_state.lock();
                            cfg.config.theme = match self.theme.mode {
                                ThemeMode::Dark => "dark".to_string(),
                                ThemeMode::Light => "light".to_string(),
                            };
                            cfg.config.theme_color = match self.theme.color {
                                ThemeColor::Blue => "blue".to_string(),
                                ThemeColor::Emerald => "emerald".to_string(),
                                ThemeColor::Purple => "purple".to_string(),
                                ThemeColor::Orange => "orange".to_string(),
                                ThemeColor::Rose => "rose".to_string(),
                            };
                        }

                        if on_save_settings {
                            let mut cfg = self.proxy_state.lock();
                            self.settings_form.apply_to(&mut cfg.config);
                            self.settings_form.toast = Some(self.i18n.t("userSettings.saved").to_string());
                        }
                    });

                // Toast overlay
                if let Some((ref msg, _)) = self.toast {
                    let toast_rect = egui::Rect::from_center_size(
                        egui::pos2(available.x / 2.0, available.y - 40.0),
                        Vec2::new(300.0, 36.0),
                    );
                    ui.painter().rect_filled(toast_rect, egui::CornerRadius::same(8), self.theme.accent);
                    ui.painter().text(
                        toast_rect.center(),
                        Align2::CENTER_CENTER,
                        msg.as_str(),
                        egui::FontId::proportional(13.0),
                        egui::Color32::WHITE,
                    );
                }
            });

        ctx.request_repaint_after(std::time::Duration::from_millis(1000));
    }
}

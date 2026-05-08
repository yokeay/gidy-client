use crate::config::GuiConfig;
use crate::lang::{Lang, TextKey};
use crate::proxy::ProxyManager;
use crate::theme::{self, ACCENT_BLUE, GREEN_ON, RED_OFF};
use crate::tray::{SystemTray, TrayEvent};
use egui::{Color32, RichText, Stroke, Vec2};
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;

#[derive(PartialEq)]
enum Tab {
    Config,
    Monitor,
}

#[derive(PartialEq)]
enum CloseAction {
    None,
    ShowDialog,
    MinimizeToTray,
    ForceExit,
}

pub struct GidyApp {
    config: GuiConfig,
    proxy_mgr: Arc<ProxyManager>,
    rt: Arc<Runtime>,

    // UI state
    tab: Tab,
    psk_input: String,
    server_addr: String,
    server_name: String,
    listen_addr: String,
    log_level: String,
    status_message: String,
    status_error: bool,

    // i18n
    lang: Lang,

    // Log buffer
    log_lines: Vec<(String, Color32)>,

    last_stats_update: Instant,
    stats_text: String,
    connected_cache: bool,
    running_cache: bool,

    // Config dirty flag
    config_dirty: bool,

    // System tray
    system_tray: SystemTray,
    close_action: CloseAction,

    // Window rect for centering dialog
    main_rect: Option<egui::Rect>,
}

impl GidyApp {
    pub fn new(config: GuiConfig, proxy_mgr: Arc<ProxyManager>, rt: Runtime) -> Self {
        let psk = config.psk_hex.clone();
        let server_addr = config.server_addr.clone();
        let server_name = config.server_name.clone();
        let listen_addr = config.listen_addr.clone();
        let log_level = config.log_level.clone();
        let lang = config.language;

        let system_tray = SystemTray::new(
            lang.text(TextKey::TrayShowWindow),
            lang.text(TextKey::TrayExit),
            lang.text(TextKey::TrayTooltip),
        );

        Self {
            config,
            proxy_mgr,
            rt: Arc::new(rt),
            tab: Tab::Monitor,
            psk_input: psk,
            server_addr,
            server_name,
            listen_addr,
            log_level,
            status_message: lang.text(TextKey::Ready).into(),
            status_error: false,
            lang,
            log_lines: Vec::new(),
            last_stats_update: Instant::now(),
            stats_text: String::new(),
            connected_cache: false,
            running_cache: false,
            config_dirty: false,
            system_tray,
            close_action: CloseAction::None,
            main_rect: None,
        }
    }

    fn add_log(&mut self, msg: &str, color: Color32) {
        let ts = chrono::Local::now().format("%H:%M:%S");
        self.log_lines
            .push((format!("[{}] {}", ts, msg), color));
        tracing::info!("{}", msg);
        if self.log_lines.len() > 500 {
            self.log_lines.remove(0);
        }
    }

    fn toggle_proxy(&mut self) {
        if self.running_cache {
            let msg = self.lang.text(TextKey::Stopping).to_string();
            self.add_log(&msg, Color32::from_gray(180));
            let mgr = self.proxy_mgr.clone();
            let rt = self.rt.clone();
            rt.spawn(async move {
                mgr.stop().await;
            });
            self.running_cache = false;
            self.connected_cache = false;
            self.status_message = self.lang.text(TextKey::Stopped).into();
            self.status_error = false;
            self.add_log(self.lang.text(TextKey::ProxyStopped), Color32::from_gray(180));
        } else {
            self.config.psk_hex = self.psk_input.clone();
            self.config.server_addr = self.server_addr.clone();
            self.config.server_name = self.server_name.clone();
            self.config.listen_addr = self.listen_addr.clone();
            self.config.log_level = self.log_level.clone();
            self.config.language = self.lang;
            self.config_dirty = false;

            let mgr = self.proxy_mgr.clone();
            let rt = self.rt.clone();
            let updated_config = self.config.clone();
            rt.spawn(async move {
                mgr.update_config(updated_config).await;
                match mgr.start().await {
                    Ok(()) => {
                        tracing::info!("proxy started");
                    }
                    Err(e) => {
                        tracing::error!("proxy start failed: {}", e);
                    }
                }
            });

            self.add_log(self.lang.text(TextKey::Starting), ACCENT_BLUE);
            self.status_message = self.lang.text(TextKey::Connecting).into();
            self.status_error = false;
            self.running_cache = true;
            self.connected_cache = true;
            self.add_log(self.lang.text(TextKey::ProxyStarted), Color32::from_rgb(80, 220, 120));
        }
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes >= 1_073_741_824 {
            format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
        } else if bytes >= 1_048_576 {
            format!("{:.2} MB", bytes as f64 / 1_048_576.0)
        } else if bytes >= 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    fn format_speed(kbps: f64) -> String {
        if kbps >= 1000.0 {
            format!("{:.1} Mbps", kbps / 1000.0)
        } else {
            format!("{:.0} Kbps", kbps)
        }
    }

    fn render_status_indicator(&mut self, ui: &mut egui::Ui) {
        let desired_size = Vec2::splat(180.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        let center = rect.center();
        let radius = rect.width() / 2.0;

        if response.clicked() {
            self.toggle_proxy();
        }

        // Simple color states
        let circle_color = if self.running_cache || self.connected_cache {
            GREEN_ON
        } else {
            // Ready state: moonlit white
            Color32::from_gray(210)
        };

        let painter = ui.painter();

        // Shadow (dark circle offset slightly)
        painter.circle_filled(
            center + Vec2::new(0.0, radius * 0.08),
            radius,
            Color32::from_black_alpha(80),
        );

        // Main circle with subtle edge
        painter.circle_filled(center, radius, circle_color);
        painter.circle_stroke(
            center,
            radius - 1.0,
            Stroke::new(1.0, Color32::from_black_alpha(30)),
        );

        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    fn render_config_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);

        ui.label(RichText::new(self.lang.text(TextKey::PskLabel)).color(Color32::from_gray(200)));
        let psk_resp = ui.add_sized(
            [ui.available_width(), 32.0],
            egui::TextEdit::singleline(&mut self.psk_input)
                .password(true)
                .hint_text(self.lang.text(TextKey::PskHint)),
        );
        if psk_resp.changed() {
            self.config_dirty = true;
        }
        ui.add_space(4.0);

        ui.label(RichText::new(self.lang.text(TextKey::ServerAddrLabel)).color(Color32::from_gray(200)));
        let addr_resp = ui.add_sized(
            [ui.available_width(), 32.0],
            egui::TextEdit::singleline(&mut self.server_addr).hint_text("ip:port"),
        );
        if addr_resp.changed() {
            self.config_dirty = true;
        }
        ui.add_space(4.0);

        ui.label(RichText::new(self.lang.text(TextKey::SniLabel)).color(Color32::from_gray(200)));
        let name_resp = ui.add_sized(
            [ui.available_width(), 32.0],
            egui::TextEdit::singleline(&mut self.server_name).hint_text("gidy.example.com"),
        );
        if name_resp.changed() {
            self.config_dirty = true;
        }
        ui.add_space(4.0);

        ui.label(RichText::new(self.lang.text(TextKey::ListenAddrLabel)).color(Color32::from_gray(200)));
        let listen_resp = ui.add_sized(
            [ui.available_width(), 32.0],
            egui::TextEdit::singleline(&mut self.listen_addr).hint_text("127.0.0.1:1080"),
        );
        if listen_resp.changed() {
            self.config_dirty = true;
        }
        ui.add_space(4.0);

        ui.label(RichText::new(self.lang.text(TextKey::LogLevelLabel)).color(Color32::from_gray(200)));
        egui::ComboBox::from_id_salt("log_level")
            .selected_text(&self.log_level)
            .show_ui(ui, |ui| {
                for level in &["trace", "debug", "info", "warn", "error"] {
                    let resp = ui.selectable_value(
                        &mut self.log_level,
                        level.to_string(),
                        *level,
                    );
                    if resp.changed() {
                        self.config_dirty = true;
                    }
                }
            });

        ui.add_space(8.0);
        ui.label(RichText::new(self.lang.text(TextKey::Language)).color(Color32::from_gray(200)));
        egui::ComboBox::from_id_salt("language")
            .selected_text(match self.lang {
                Lang::Zh => "中文",
                Lang::En => "English",
            })
            .show_ui(ui, |ui| {
                for (label, val) in &[("English", Lang::En), ("中文", Lang::Zh)] {
                    let resp = ui.selectable_value(&mut self.lang, *val, *label);
                    if resp.changed() {
                        self.config_dirty = true;
                        self.status_message = self.lang.text(TextKey::Ready).into();
                        self.add_log("Language changed", Color32::from_gray(180));
                    }
                }
            });
    }

    fn render_monitor_tab(&mut self, ui: &mut egui::Ui) {
        if self.last_stats_update.elapsed().as_millis() > 500 {
            let snapshot = self.proxy_mgr.stats().snapshot();
            self.connected_cache = snapshot.connected;
            self.stats_text = format!(
                "↑ {}  ↓ {}",
                Self::format_speed(snapshot.speed_up_kbps),
                Self::format_speed(snapshot.speed_down_kbps),
            );
            self.last_stats_update = Instant::now();
        }

        ui.add_space(8.0);

        let speed_color = if self.connected_cache {
            GREEN_ON
        } else {
            Color32::from_gray(120)
        };
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(&self.stats_text)
                    .size(22.0)
                    .color(speed_color),
            );
        });
        ui.add_space(4.0);

        let snapshot = self.proxy_mgr.stats().snapshot();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(self.lang.text(TextKey::TotalUpload))
                    .size(13.0)
                    .color(Color32::from_gray(180)),
            );
            ui.label(
                RichText::new(Self::format_bytes(snapshot.bytes_up))
                    .size(13.0)
                    .color(Color32::from_gray(220)),
            );
            ui.add_space(20.0);
            ui.label(
                RichText::new(self.lang.text(TextKey::TotalDownload))
                    .size(13.0)
                    .color(Color32::from_gray(180)),
            );
            ui.label(
                RichText::new(Self::format_bytes(snapshot.bytes_down))
                    .size(13.0)
                    .color(Color32::from_gray(220)),
            );
        });
        ui.add_space(2.0);
        ui.label(
            RichText::new(format!("{}{}", self.lang.text(TextKey::Uptime), snapshot.uptime_secs))
                .size(13.0)
                .color(Color32::from_gray(150)),
        );

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        ui.label(
            RichText::new(self.lang.text(TextKey::EventLog))
                .size(13.0)
                .color(Color32::from_gray(180)),
        );

        let log_height = ui.available_height() - 16.0;
        egui::ScrollArea::vertical()
            .max_height(log_height)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for (line, color) in &self.log_lines {
                    ui.label(
                        RichText::new(line)
                            .size(12.0)
                            .color(*color),
                    );
                }
            });
    }
}

impl eframe::App for GidyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Tray event polling (runs even when hidden via repaint loop)
        match self.system_tray.poll_events() {
            TrayEvent::ShowWindow => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                self.system_tray.set_visible(false);
                self.add_log("Restored from tray", Color32::from_gray(180));
            }
            TrayEvent::Exit => {
                self.close_action = CloseAction::ForceExit;
                self.add_log("Exit requested from tray", Color32::from_gray(180));
            }
            TrayEvent::None => {}
        }

        // Close interception — only show dialog if NOT force exit
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.close_action == CloseAction::ForceExit {
                // Let it close
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.close_action = CloseAction::ShowDialog;
                self.add_log("Window close requested", Color32::from_gray(180));
            }
        }

        // Handle close action
        match self.close_action {
            CloseAction::ForceExit => {
                self.system_tray.set_visible(false);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                self.close_action = CloseAction::None;
                return;
            }
            CloseAction::MinimizeToTray => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                self.system_tray.set_visible(true);
                self.close_action = CloseAction::None;
                self.add_log("Minimized to tray", Color32::from_gray(180));
                // Keep repaint loop alive for tray events
                ctx.request_repaint();
            }
            CloseAction::None | CloseAction::ShowDialog => {}
        }

        // Track main window rect for centering dialog
        self.main_rect = ctx.input(|i| i.viewport().outer_rect);

        theme::apply_theme(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(self.lang.text(TextKey::Title))
                        .size(16.0)
                        .color(Color32::from_gray(200)),
                );
            });
            ui.add_space(4.0);

            if !self.status_message.is_empty() {
                let color = if self.status_error {
                    RED_OFF
                } else if self.connected_cache {
                    GREEN_ON
                } else {
                    Color32::from_gray(180)
                };
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(&self.status_message).size(12.0).color(color));
                });
            }

            ui.add_space(16.0);
            ui.vertical_centered(|ui| {
                self.render_status_indicator(ui);
            });
            ui.add_space(16.0);

            // Tabs
            ui.horizontal(|ui| {
                let config_text = if self.tab == Tab::Config {
                    RichText::new(format!("⚙ {}", self.lang.text(TextKey::ConfigTab))).color(ACCENT_BLUE)
                } else {
                    RichText::new(format!("⚙ {}", self.lang.text(TextKey::ConfigTab))).color(Color32::from_gray(160))
                };
                let monitor_text = if self.tab == Tab::Monitor {
                    RichText::new(format!("📊 {}", self.lang.text(TextKey::MonitorTab))).color(ACCENT_BLUE)
                } else {
                    RichText::new(format!("📊 {}", self.lang.text(TextKey::MonitorTab))).color(Color32::from_gray(160))
                };

                if ui.selectable_label(self.tab == Tab::Config, config_text).clicked() {
                    self.tab = Tab::Config;
                }
                if ui.selectable_label(self.tab == Tab::Monitor, monitor_text).clicked() {
                    self.tab = Tab::Monitor;
                }

                if self.config_dirty && self.tab != Tab::Config {
                    ui.add_space(4.0);
                    ui.label(RichText::new("●").size(10.0).color(Color32::from_rgb(255, 180, 60)));
                }
            });

            ui.separator();

            match self.tab {
                Tab::Config => self.render_config_tab(ui),
                Tab::Monitor => self.render_monitor_tab(ui),
            }
        });

        // --- Close dialog centered on main window ---
        if self.close_action == CloseAction::ShowDialog {
            let dialog_size = egui::vec2(300.0, 120.0);

            let mut dialog_builder = egui::ViewportBuilder::default()
                .with_title(self.lang.text(TextKey::CloseDialogTitle))
                .with_inner_size(dialog_size)
                .with_resizable(false);

            // Center on main window
            if let Some(main_rect) = self.main_rect {
                let dialog_center_x = main_rect.min.x + main_rect.width() / 2.0;
                let dialog_center_y = main_rect.min.y + main_rect.height() / 2.0;
                dialog_builder = dialog_builder.with_position(egui::pos2(
                    dialog_center_x - dialog_size.x / 2.0,
                    dialog_center_y - dialog_size.y / 2.0,
                ));
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("close_dialog"),
                dialog_builder,
                |ctx, _class| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.add_space(12.0);
                        ui.vertical_centered(|ui| {
                            ui.label(self.lang.text(TextKey::CloseDialogText));
                        });
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let no_text = self.lang.text(TextKey::CloseDialogNo).to_string();
                                let yes_text = self.lang.text(TextKey::CloseDialogYes).to_string();
                                if ui.button(&no_text).clicked() {
                                    self.add_log("Clicked: Exit", RED_OFF);
                                    self.close_action = CloseAction::ForceExit;
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                                if ui.button(&yes_text).clicked() {
                                    self.add_log("Clicked: Minimize to tray", Color32::from_gray(180));
                                    self.close_action = CloseAction::MinimizeToTray;
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                            });
                        });
                    });
                },
            );
        }

        // Keep repaint loop alive when hidden for tray events
        if !ctx.input(|i| i.viewport().focused.unwrap_or(true)) {
            ctx.request_repaint();
        }
    }
}

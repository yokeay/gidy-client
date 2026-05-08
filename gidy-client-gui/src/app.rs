use crate::config::GuiConfig;
use crate::lang::{Lang, TextKey};
use crate::proxy::ProxyManager;
use crate::theme::{self, ACCENT_BLUE, GREEN_GLOW, GREEN_ON, RED_GLOW, RED_OFF};
use crate::tray::{SystemTray, TrayEvent};
use egui::{Color32, Pos2, RichText, Stroke, Vec2};
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
    Exit,
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

    // Animation
    glow_phase: f32,
    last_stats_update: Instant,
    stats_text: String,
    connected_cache: bool,
    running_cache: bool,

    // Config dirty flag
    config_dirty: bool,

    // System tray
    system_tray: SystemTray,
    close_action: CloseAction,
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
            glow_phase: 0.0,
            last_stats_update: Instant::now(),
            stats_text: String::new(),
            connected_cache: false,
            running_cache: false,
            config_dirty: false,
            system_tray,
            close_action: CloseAction::None,
        }
    }

    fn add_log(&mut self, msg: &str, color: Color32) {
        self.log_lines.push((msg.to_string(), color));
        if self.log_lines.len() > 500 {
            self.log_lines.remove(0);
        }
    }

    fn toggle_proxy(&mut self) {
        if self.running_cache {
            self.add_log(self.lang.text(TextKey::Stopping), Color32::from_gray(180));
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
            // Apply config changes
            self.config.psk_hex = self.psk_input.clone();
            self.config.server_addr = self.server_addr.clone();
            self.config.server_name = self.server_name.clone();
            self.config.listen_addr = self.listen_addr.clone();
            self.config.log_level = self.log_level.clone();
            self.config.language = self.lang;
            self.config_dirty = false;

            self.add_log(self.lang.text(TextKey::Starting), ACCENT_BLUE);
            self.status_message = self.lang.text(TextKey::Connecting).into();
            self.status_error = false;

            let mgr = self.proxy_mgr.clone();
            let rt = self.rt.clone();
            rt.spawn(async move {
                match mgr.start().await {
                    Ok(()) => {
                        tracing::info!("proxy started");
                    }
                    Err(e) => {
                        tracing::error!("proxy start failed: {}", e);
                    }
                }
            });
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

    fn render_power_button(&mut self, ui: &mut egui::Ui) {
        let desired_size = Vec2::splat(180.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        let center = rect.center();
        let radius = rect.width() / 2.0;

        if response.clicked() {
            self.toggle_proxy();
        }

        // Glow phase animation
        self.glow_phase += ui.ctx().input(|i| i.unstable_dt) * 3.0;

        let (main_color, glow_color) = if self.running_cache || self.connected_cache {
            (GREEN_ON, GREEN_GLOW)
        } else {
            (RED_OFF, RED_GLOW)
        };

        let painter = ui.painter();

        // Outer glow rings
        for i in 0..3 {
            let phase = self.glow_phase + i as f32 * 2.1;
            let alpha = (phase.sin() * 0.5 + 0.5) as u8;
            let glow_r = radius + 8.0 + i as f32 * 10.0;
            let glow_a = ((alpha as f32 / 255.0) * 0.25 * (1.0 - i as f32 * 0.25)) as u8;
            let g = Color32::from_rgba_premultiplied(
                glow_color.r(),
                glow_color.g(),
                glow_color.b(),
                glow_a,
            );
            painter.circle_filled(center, glow_r, g);
        }

        // Outer ring
        painter.circle_stroke(
            center,
            radius + 4.0,
            Stroke::new(2.5, Color32::from_rgba_premultiplied(255, 255, 255, 40)),
        );

        // Main circle - gradient effect with concentric circles
        let base = Color32::from_rgba_premultiplied(20, 20, 24, 230);
        painter.circle_filled(center, radius, base);

        // Inner fill circles for depth
        for i in 0..8 {
            let r = radius - i as f32 * 6.0;
            if r <= 0.0 {
                break;
            }
            let t = i as f32 / 8.0;
            let a = (40.0 - t * 30.0) as u8;
            painter.circle_filled(
                center,
                r,
                Color32::from_rgba_premultiplied(
                    main_color.r(),
                    main_color.g(),
                    main_color.b(),
                    a,
                ),
            );
        }

        // Inner highlight ring
        painter.circle_stroke(
            center,
            radius - 6.0,
            Stroke::new(
                1.5,
                Color32::from_rgba_premultiplied(255, 255, 255, 15),
            ),
        );

        // Center button
        let inner_radius = radius - 18.0;
        if response.is_pointer_button_down_on() {
            painter.circle_filled(
                center,
                inner_radius,
                Color32::from_rgba_premultiplied(
                    main_color.r(),
                    main_color.g(),
                    main_color.b(),
                    180,
                ),
            );
        } else {
            painter.circle_filled(
                center,
                inner_radius,
                Color32::from_rgba_premultiplied(
                    main_color.r(),
                    main_color.g(),
                    main_color.b(),
                    140,
                ),
            );
        }

        // Inner highlight (top-left light reflection)
        let hl_center = center + Vec2::new(-inner_radius * 0.25, -inner_radius * 0.25);
        let hl_r = inner_radius * 0.45;
        painter.circle_filled(
            hl_center,
            hl_r,
            Color32::from_rgba_premultiplied(255, 255, 255, 30),
        );

        // Power icon (simple vertical line + arc)
        let icon_color = if response.hovered() {
            Color32::WHITE
        } else {
            Color32::from_gray(230)
        };
        let icon_size = 18.0;

        // Vertical line
        let line_top = Pos2::new(center.x, center.y - icon_size * 0.7);
        let line_bot = Pos2::new(center.x, center.y + icon_size * 0.3);
        painter.line_segment(
            [line_top, line_bot],
            Stroke::new(3.0, icon_color),
        );

        // Arc at top (simplified as small circle ring)
        let arc_center = center + Vec2::new(0.0, -icon_size * 0.7);
        painter.circle_stroke(
            arc_center,
            icon_size * 0.35,
            Stroke::new(2.5, icon_color),
        );

        // Hover cursor
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
            egui::TextEdit::singleline(&mut self.server_name).hint_text("localhost"),
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
                        // Update status message immediately on language change
                        self.status_message = self.lang.text(TextKey::Ready).into();
                    }
                }
            });
    }

    fn render_monitor_tab(&mut self, ui: &mut egui::Ui) {
        // Update stats periodically
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

        // Speed display
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

        // Total stats
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

        // Log window
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
        // ---- Close interception ----
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_action = CloseAction::ShowDialog;
        }

        // ---- Tray event polling ----
        match self.system_tray.poll_events() {
            TrayEvent::ShowWindow => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                self.system_tray.set_visible(false);
            }
            TrayEvent::Exit => {
                self.close_action = CloseAction::Exit;
            }
            TrayEvent::None => {}
        }

        // ---- Handle close action ----
        match self.close_action {
            CloseAction::Exit => {
                self.system_tray.set_visible(false);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                self.close_action = CloseAction::None;
                return;
            }
            CloseAction::MinimizeToTray => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                self.system_tray.set_visible(true);
                self.close_action = CloseAction::None;
            }
            CloseAction::None | CloseAction::ShowDialog => {}
        }

        // ---- Theme ----
        theme::apply_theme(ctx);

        // ---- Main UI ----
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

            // Status line
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

            // Power button - centered
            ui.vertical_centered(|ui| {
                self.render_power_button(ui);
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

                if ui
                    .selectable_label(self.tab == Tab::Config, config_text)
                    .clicked()
                {
                    self.tab = Tab::Config;
                }
                if ui
                    .selectable_label(self.tab == Tab::Monitor, monitor_text)
                    .clicked()
                {
                    self.tab = Tab::Monitor;
                }

                // Config dirty indicator
                if self.config_dirty && self.tab != Tab::Config {
                    ui.add_space(4.0);
                    ui.label(RichText::new("●").size(10.0).color(Color32::from_rgb(255, 180, 60)));
                }
            });

            ui.separator();

            // Tab content in remaining space
            match self.tab {
                Tab::Config => self.render_config_tab(ui),
                Tab::Monitor => self.render_monitor_tab(ui),
            }
        });

        // ---- Close dialog (modal) ----
        if self.close_action == CloseAction::ShowDialog {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("close_dialog"),
                egui::ViewportBuilder::default()
                    .with_title(self.lang.text(TextKey::CloseDialogTitle))
                    .with_inner_size([300.0, 120.0])
                    .with_resizable(false),
                |ctx, _class| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.add_space(12.0);
                        ui.vertical_centered(|ui| {
                            ui.label(self.lang.text(TextKey::CloseDialogText));
                        });
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(self.lang.text(TextKey::CloseDialogNo)).clicked() {
                                    self.close_action = CloseAction::Exit;
                                }
                                if ui.button(self.lang.text(TextKey::CloseDialogYes)).clicked() {
                                    self.close_action = CloseAction::MinimizeToTray;
                                }
                            });
                        });
                    });
                },
            );
        }
    }
}

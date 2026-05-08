use crate::config::GuiConfig;
use crate::lang::{Lang, TextKey};
use crate::proxy::ProxyManager;
use crate::theme::{self, ACCENT_BLUE, CHART_DOWN, CHART_DOWN_FILL, CHART_GRID, CHART_UP, CHART_UP_FILL, GREEN_ON, RED_OFF};
use crate::tray::{SystemTray, TrayEvent};
use egui::{Color32, RichText, Stroke, Vec2};
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    TrafficMonitor,
    Settings,
    SystemConfig,
    About,
}

#[derive(PartialEq)]
enum CloseAction {
    None,
    ShowDialog,
    MinimizeToTray,
    ForceExit,
}

const CHART_MAX_POINTS: usize = 60;
const CHART_WIDTH: f32 = 360.0;
const CHART_HEIGHT: f32 = 140.0;

pub struct GidyApp {
    config: GuiConfig,
    proxy_mgr: Arc<ProxyManager>,
    rt: Arc<Runtime>,

    // UI state
    psk_input: String,
    server_addr: String,
    server_name: String,
    listen_addr: String,
    log_level: String,
    status_error: bool,

    // i18n
    lang: Lang,

    // Tabs
    current_tab: Tab,

    // Chart data
    chart_up_history: Vec<f64>,
    chart_down_history: Vec<f64>,
    last_chart_push: Instant,

    // System config
    auto_start: bool,
    auto_connect: bool,
    minimize_to_tray: bool,

    // Logo
    logo_texture: Option<egui::TextureHandle>,

    // Log buffer
    log_lines: Vec<(String, Color32)>,

    // Stats
    last_stats_update: Instant,
    stats_text: String,
    connected_cache: bool,
    running_cache: bool,

    // Config dirty tracking
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
        let auto_start = config.auto_start;
        let auto_connect = config.auto_connect;
        let minimize_to_tray = config.minimize_to_tray;

        let system_tray = SystemTray::new(
            lang.text(TextKey::TrayShowWindow),
            lang.text(TextKey::TrayExit),
            lang.text(TextKey::TrayTooltip),
        );

        let mut app = Self {
            config,
            proxy_mgr,
            rt: Arc::new(rt),
            psk_input: psk,
            server_addr,
            server_name,
            listen_addr,
            log_level,
            status_error: false,
            lang,
            current_tab: Tab::TrafficMonitor,
            chart_up_history: Vec::with_capacity(CHART_MAX_POINTS),
            chart_down_history: Vec::with_capacity(CHART_MAX_POINTS),
            last_chart_push: Instant::now(),
            auto_start,
            auto_connect,
            minimize_to_tray,
            logo_texture: None,
            log_lines: Vec::new(),
            last_stats_update: Instant::now(),
            stats_text: String::new(),
            connected_cache: false,
            running_cache: false,
            config_dirty: false,
            system_tray,
            close_action: CloseAction::None,
            main_rect: None,
        };

        app.add_log("gidy client started", Color32::from_gray(180));

        if app.auto_start {
            app.add_log("Auto-start enabled, starting proxy...", ACCENT_BLUE);
            app.start_proxy();
        }

        app
    }

    fn ensure_logo(&mut self, ctx: &egui::Context) {
        if self.logo_texture.is_some() {
            return;
        }
        let logo_bytes = include_bytes!("../logo.png");
        if let Ok(img) = image::load_from_memory(logo_bytes) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let pixels = rgba.into_raw();
            let color_image =
                egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);
            self.logo_texture =
                Some(ctx.load_texture("logo", color_image, egui::TextureOptions::default()));
        } else {
            tracing::warn!("Failed to load logo from embedded bytes");
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

    fn start_proxy(&mut self) {
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
        self.status_error = false;
        self.running_cache = true;
        self.connected_cache = true;
        self.add_log(self.lang.text(TextKey::ProxyStarted), GREEN_ON);
    }

    fn stop_proxy(&mut self) {
        let msg = self.lang.text(TextKey::Stopping).to_string();
        self.add_log(&msg, Color32::from_gray(180));
        let mgr = self.proxy_mgr.clone();
        let rt = self.rt.clone();
        rt.spawn(async move {
            mgr.stop().await;
        });
        self.running_cache = false;
        self.connected_cache = false;
        self.status_error = false;
        self.add_log(self.lang.text(TextKey::ProxyStopped), Color32::from_gray(180));
    }

    fn toggle_proxy(&mut self) {
        if self.running_cache {
            self.stop_proxy();
        } else {
            self.start_proxy();
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
        } else if kbps < 10.0 {
            format!("{:.1} Kbps", kbps)
        } else {
            format!("{:.0} Kbps", kbps)
        }
    }

    fn format_uptime(secs: u64) -> String {
        if secs >= 3600 {
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            format!("{:02}:{:02}:{:02}", h, m, s)
        } else if secs >= 60 {
            let m = secs / 60;
            let s = secs % 60;
            format!("{:02}:{:02}", m, s)
        } else {
            format!("{}s", secs)
        }
    }

    fn refresh_stats(&mut self) {
        if self.last_stats_update.elapsed().as_millis() > 500 {
            let snapshot = self.proxy_mgr.stats().snapshot();
            self.connected_cache = snapshot.connected;
            self.stats_text = format!(
                "{} {}  {} {}",
                self.lang.text(TextKey::UploadRate),
                Self::format_speed(snapshot.speed_up_kbps),
                self.lang.text(TextKey::DownloadRate),
                Self::format_speed(snapshot.speed_down_kbps),
            );
            self.last_stats_update = Instant::now();

            // Update chart history (~1 sample per second)
            if self.last_chart_push.elapsed().as_millis() > 900 {
                self.chart_up_history.push(snapshot.speed_up_kbps);
                self.chart_down_history.push(snapshot.speed_down_kbps);
                if self.chart_up_history.len() > CHART_MAX_POINTS {
                    self.chart_up_history.remove(0);
                }
                if self.chart_down_history.len() > CHART_MAX_POINTS {
                    self.chart_down_history.remove(0);
                }
                self.last_chart_push = Instant::now();
            }
        }
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        let tabs = [
            (Tab::TrafficMonitor, self.lang.text(TextKey::TrafficMonitor)),
            (Tab::Settings, self.lang.text(TextKey::Settings)),
            (Tab::SystemConfig, self.lang.text(TextKey::SystemConfig)),
            (Tab::About, self.lang.text(TextKey::About)),
        ];

        let spacing = ui.spacing_mut();
        spacing.item_spacing.x = 0.0;

        ui.horizontal(|ui| {
            for (tab, label) in &tabs {
                let selected = self.current_tab == *tab;
                let text = if selected {
                    RichText::new(*label).size(13.0).color(Color32::from_gray(240))
                } else {
                    RichText::new(*label).size(13.0).color(Color32::from_gray(150))
                };

                let resp = ui.add(
                    egui::Button::new(text)
                        .fill(Color32::TRANSPARENT)
                        .min_size(Vec2::new(80.0, 30.0)),
                );

                if selected {
                    let rect = resp.rect;
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + 8.0, rect.max.y - 2.0),
                            egui::vec2(rect.width() - 16.0, 2.0),
                        ),
                        4.0,
                        ACCENT_BLUE,
                    );
                }

                if resp.clicked() {
                    self.current_tab = *tab;
                }
            }
        });

        ui.reset_style();
        ui.separator();
    }

    fn render_chart(&self, ui: &mut egui::Ui) {
        let desired_size = Vec2::new(CHART_WIDTH, CHART_HEIGHT);
        let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        let painter = ui.painter();

        // Background
        painter.rect_filled(
            rect,
            6.0,
            Color32::from_rgba_premultiplied(20, 20, 26, 180),
        );
        painter.rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, Color32::from_rgba_premultiplied(60, 60, 70, 100)),
            egui::StrokeKind::Inside,
        );

        let inner = rect.shrink(4.0);
        let x_min = inner.min.x;
        let x_max = inner.max.x;
        let y_min = inner.min.y;
        let y_max = inner.max.y;
        let _y_mid = (y_min + y_max) / 2.0;

        // Find max value for scaling
        let mut max_val: f64 = 10.0;
        for v in self.chart_up_history.iter().chain(self.chart_down_history.iter()) {
            if *v > max_val {
                max_val = *v;
            }
        }
        if max_val < 1.0 {
            max_val = 1.0;
        }
        // Round up to nice number
        let nice_max = if max_val <= 5.0 {
            5.0
        } else if max_val <= 10.0 {
            10.0
        } else if max_val <= 50.0 {
            50.0
        } else if max_val <= 100.0 {
            100.0
        } else if max_val <= 500.0 {
            500.0
        } else if max_val <= 1000.0 {
            1000.0
        } else {
            ((max_val / 1000.0).ceil() + 0.5) * 1000.0
        };

        let scale = (y_max - y_min - 4.0) as f64 / nice_max;

        // Grid lines
        for i in 0..=4 {
            let frac = i as f32 / 4.0;
            let y = y_max - (y_max - y_min) * frac;
            painter.line_segment(
                [egui::pos2(x_min, y), egui::pos2(x_max, y)],
                Stroke::new(0.5, CHART_GRID),
            );
        }

        // Draw filled areas and lines
        let n = self.chart_up_history.len();
        if n < 2 {
            return;
        }

        let x_step = (x_max - x_min) / (n.max(2) as f32 - 1.0);

        // Helper: data -> screen points
        let to_points = |data: &[f64]| -> Vec<egui::Pos2> {
            data.iter()
                .enumerate()
                .map(|(i, v)| {
                    let x = x_max - (n - 1 - i) as f32 * x_step;
                    let y = y_max - ((*v).min(nice_max) * scale) as f32 - 2.0;
                    egui::pos2(x, y)
                })
                .collect()
        };

        let up_pts = to_points(&self.chart_up_history);
        let down_pts = to_points(&self.chart_down_history);

        // Upload fill (green, semi-transparent)
        {
            let mut fill_pts = up_pts.clone();
            fill_pts.push(egui::pos2(fill_pts.last().unwrap().x, y_max));
            fill_pts.push(egui::pos2(fill_pts.first().unwrap().x, y_max));
            if fill_pts.len() >= 3 {
                painter.add(egui::Shape::convex_polygon(
                    fill_pts,
                    CHART_UP_FILL,
                    Stroke::NONE,
                ));
            }
        }

        // Download fill (blue, semi-transparent)
        {
            let mut fill_pts = down_pts.clone();
            fill_pts.push(egui::pos2(fill_pts.last().unwrap().x, y_max));
            fill_pts.push(egui::pos2(fill_pts.first().unwrap().x, y_max));
            if fill_pts.len() >= 3 {
                painter.add(egui::Shape::convex_polygon(
                    fill_pts,
                    CHART_DOWN_FILL,
                    Stroke::NONE,
                ));
            }
        }

        // Upload line
        if up_pts.len() >= 2 {
            for w in up_pts.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(1.5, CHART_UP));
            }
        }

        // Download line
        if down_pts.len() >= 2 {
            for w in down_pts.windows(2) {
                painter.line_segment([w[0], w[1]], Stroke::new(1.5, CHART_DOWN));
            }
        }

        // Legend
        painter.circle_filled(
            egui::pos2(x_min + 8.0, y_min + 14.0),
            3.0,
            CHART_UP,
        );
        painter.text(
            egui::pos2(x_min + 16.0, y_min + 8.0),
            egui::Align2::LEFT_TOP,
            "Upload",
            egui::FontId::monospace(10.0),
            Color32::from_gray(200),
        );
        painter.circle_filled(
            egui::pos2(x_min + 70.0, y_min + 14.0),
            3.0,
            CHART_DOWN,
        );
        painter.text(
            egui::pos2(x_min + 78.0, y_min + 8.0),
            egui::Align2::LEFT_TOP,
            "Download",
            egui::FontId::monospace(10.0),
            Color32::from_gray(200),
        );
    }

    fn render_traffic_tab(&mut self, ui: &mut egui::Ui) {
        let snapshot = self.proxy_mgr.stats().snapshot();

        // Speed display
        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            let speed_color = if self.connected_cache {
                GREEN_ON
            } else {
                Color32::from_gray(120)
            };
            ui.label(
                RichText::new(format!(
                    "{} {}  {} {}",
                    self.lang.text(TextKey::UploadRate),
                    Self::format_speed(snapshot.speed_up_kbps),
                    self.lang.text(TextKey::DownloadRate),
                    Self::format_speed(snapshot.speed_down_kbps),
                ))
                .size(16.0)
                .color(speed_color),
            );
        });

        ui.add_space(8.0);

        // Chart
        ui.vertical_centered(|ui| {
            self.render_chart(ui);
        });

        ui.add_space(6.0);

        // Traffic totals
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{}{}",
                    self.lang.text(TextKey::TotalUpload),
                    Self::format_bytes(snapshot.bytes_up)
                ))
                .size(12.0)
                .color(Color32::from_gray(200)),
            );
            ui.add_space(16.0);
            ui.label(
                RichText::new(format!(
                    "{}{}",
                    self.lang.text(TextKey::TotalDownload),
                    Self::format_bytes(snapshot.bytes_down)
                ))
                .size(12.0)
                .color(Color32::from_gray(200)),
            );
        });

        ui.add_space(2.0);

        // Uptime
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "{}{}",
                    self.lang.text(TextKey::Uptime),
                    Self::format_uptime(snapshot.uptime_secs)
                ))
                .size(12.0)
                .color(Color32::from_gray(180)),
            );
        });

        ui.add_space(4.0);

        // Status + Start/Stop button
        ui.horizontal(|ui| {
            let status_text = if self.connected_cache {
                self.lang.text(TextKey::Connected)
            } else if self.running_cache {
                self.lang.text(TextKey::Starting)
            } else if self.status_error {
                self.lang.text(TextKey::Disconnected)
            } else {
                self.lang.text(TextKey::Ready)
            };
            let status_color = if self.connected_cache {
                GREEN_ON
            } else if self.running_cache {
                Color32::from_rgb(255, 180, 60)
            } else if self.status_error {
                RED_OFF
            } else {
                Color32::from_gray(210)
            };

            ui.label(
                RichText::new(format!("{} {}", self.lang.text(TextKey::Status), status_text))
                    .size(12.0)
                    .color(status_color),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let btn_text = if self.running_cache {
                    self.lang.text(TextKey::Stop)
                } else {
                    self.lang.text(TextKey::Start)
                };
                let btn_color = if self.running_cache {
                    RED_OFF
                } else {
                    GREEN_ON
                };
                if ui
                    .add(
                        egui::Button::new(RichText::new(btn_text).size(13.0).color(Color32::WHITE))
                            .fill(btn_color)
                            .min_size(Vec2::new(80.0, 28.0))
                            .corner_radius(6),
                    )
                    .clicked()
                {
                    self.toggle_proxy();
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(2.0);

        // Event log header
        ui.label(
            RichText::new(self.lang.text(TextKey::EventLog))
                .size(12.0)
                .color(Color32::from_gray(180)),
        );

        // Event log
        let log_height = ui.available_height() - 4.0;
        egui::ScrollArea::vertical()
            .max_height(log_height)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for (line, color) in &self.log_lines {
                    ui.label(
                        RichText::new(line)
                            .size(11.0)
                            .color(*color),
                    );
                }
            });
    }

    fn render_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);

        ui.label(RichText::new(self.lang.text(TextKey::PskLabel)).color(Color32::from_gray(200)));
        let psk_resp = ui.add_sized(
            [ui.available_width(), 28.0],
            egui::TextEdit::singleline(&mut self.psk_input)
                .password(true)
                .hint_text(self.lang.text(TextKey::PskHint)),
        );
        if psk_resp.changed() {
            self.config_dirty = true;
        }
        ui.add_space(6.0);

        ui.label(RichText::new(self.lang.text(TextKey::ServerAddrLabel)).color(Color32::from_gray(200)));
        let addr_resp = ui.add_sized(
            [ui.available_width(), 28.0],
            egui::TextEdit::singleline(&mut self.server_addr).hint_text("ip:port"),
        );
        if addr_resp.changed() {
            self.config_dirty = true;
        }
        ui.add_space(6.0);

        ui.label(RichText::new(self.lang.text(TextKey::SniLabel)).color(Color32::from_gray(200)));
        let name_resp = ui.add_sized(
            [ui.available_width(), 28.0],
            egui::TextEdit::singleline(&mut self.server_name).hint_text("gidy.example.com"),
        );
        if name_resp.changed() {
            self.config_dirty = true;
        }
        ui.add_space(6.0);

        ui.label(RichText::new(self.lang.text(TextKey::ListenAddrLabel)).color(Color32::from_gray(200)));
        let listen_resp = ui.add_sized(
            [ui.available_width(), 28.0],
            egui::TextEdit::singleline(&mut self.listen_addr).hint_text("127.0.0.1:1080"),
        );
        if listen_resp.changed() {
            self.config_dirty = true;
        }
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new(self.lang.text(TextKey::LogLevelLabel)).color(Color32::from_gray(200)));
            egui::ComboBox::from_id_salt("log_level")
                .selected_text(&self.log_level)
                .width(80.0)
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

            ui.add_space(12.0);

            ui.label(RichText::new(self.lang.text(TextKey::Language)).color(Color32::from_gray(200)));
            egui::ComboBox::from_id_salt("language")
                .selected_text(match self.lang {
                    Lang::Zh => "中文",
                    Lang::En => "English",
                })
                .width(80.0)
                .show_ui(ui, |ui| {
                    for (label, val) in &[("English", Lang::En), ("中文", Lang::Zh)] {
                        let resp = ui.selectable_value(&mut self.lang, *val, *label);
                        if resp.changed() {
                            self.config_dirty = true;
                        }
                    }
                });
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new(self.lang.text(TextKey::Save)).color(Color32::WHITE))
                        .fill(ACCENT_BLUE)
                        .min_size(Vec2::new(80.0, 30.0))
                        .corner_radius(6),
                )
                .clicked()
            {
                self.config.psk_hex = self.psk_input.clone();
                self.config.server_addr = self.server_addr.clone();
                self.config.server_name = self.server_name.clone();
                self.config.listen_addr = self.listen_addr.clone();
                self.config.log_level = self.log_level.clone();
                self.config.language = self.lang;
                self.config.auto_start = self.auto_start;
                self.config.auto_connect = self.auto_connect;
                self.config.minimize_to_tray = self.minimize_to_tray;
                self.config_dirty = false;

                // Persist config
                if let Ok(s) = toml::to_string_pretty(&self.config) {
                    let config_path = std::env::current_exe()
                        .ok()
                        .map(|p| {
                            let mut p = p.parent().map(|d| d.to_path_buf()).unwrap_or_default();
                            p.push("gidy-client.toml");
                            p
                        })
                        .unwrap_or_else(|| std::path::PathBuf::from("gidy-client.toml"));
                    let _ = std::fs::write(&config_path, s);
                }

                self.add_log("Settings saved", Color32::from_gray(180));
            }

            if ui
                .add(
                    egui::Button::new(RichText::new(self.lang.text(TextKey::Cancel)).color(Color32::from_gray(200)))
                        .fill(Color32::TRANSPARENT)
                        .min_size(Vec2::new(80.0, 30.0))
                        .stroke(Stroke::new(1.0, Color32::from_gray(100)))
                        .corner_radius(6),
                )
                .clicked()
            {
                self.psk_input = self.config.psk_hex.clone();
                self.server_addr = self.config.server_addr.clone();
                self.server_name = self.config.server_name.clone();
                self.listen_addr = self.config.listen_addr.clone();
                self.log_level = self.config.log_level.clone();
                self.lang = self.config.language;
                self.auto_start = self.config.auto_start;
                self.auto_connect = self.config.auto_connect;
                self.minimize_to_tray = self.config.minimize_to_tray;
                self.config_dirty = false;
            }
        });
    }

    fn render_system_config_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);

        ui.label(
            RichText::new("System")
                .size(14.0)
                .color(Color32::from_gray(230)),
        );
        ui.add_space(4.0);

        egui::Frame::NONE
            .fill(Color32::from_rgba_premultiplied(30, 30, 36, 120))
            .corner_radius(8)
            .inner_margin(egui::vec2(12.0, 8.0))
            .show(ui, |ui| {
                ui.add_space(4.0);

                let auto_start_resp = ui.checkbox(&mut self.auto_start, self.lang.text(TextKey::AutoStartBoot));
                if auto_start_resp.changed() {
                    self.config.auto_start = self.auto_start;
                    self.save_config_file();
                }

                ui.add_space(4.0);

                let auto_conn_resp = ui.checkbox(&mut self.auto_connect, self.lang.text(TextKey::AutoConnect));
                if auto_conn_resp.changed() {
                    self.config.auto_connect = self.auto_connect;
                    self.save_config_file();
                }

                ui.add_space(4.0);

                let tray_resp = ui.checkbox(&mut self.minimize_to_tray, self.lang.text(TextKey::MinimizeToTray));
                if tray_resp.changed() {
                    self.config.minimize_to_tray = self.minimize_to_tray;
                    self.save_config_file();
                }

                ui.add_space(4.0);
            });
    }

    fn save_config_file(&mut self) {
        self.config.auto_start = self.auto_start;
        self.config.auto_connect = self.auto_connect;
        self.config.minimize_to_tray = self.minimize_to_tray;

        if let Ok(s) = toml::to_string_pretty(&self.config) {
            let config_path = std::env::current_exe()
                .ok()
                .map(|p| {
                    let mut p = p.parent().map(|d| d.to_path_buf()).unwrap_or_default();
                    p.push("gidy-client.toml");
                    p
                })
                .unwrap_or_else(|| std::path::PathBuf::from("gidy-client.toml"));
            let _ = std::fs::write(&config_path, s);
        }
    }

    fn render_about_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);

        ui.vertical_centered(|ui| {
            // Logo
            if let Some(ref texture) = self.logo_texture {
                let size = Vec2::splat(120.0);
                ui.image(egui::ImageSource::Texture(
                    egui::load::SizedTexture::new(texture.id(), size),
                ));
            } else {
                // Placeholder if logo not loaded
                let (rect, _) = ui.allocate_exact_size(
                    Vec2::splat(120.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(
                    rect,
                    12.0,
                    Color32::from_rgba_premultiplied(40, 40, 48, 150),
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "gidy",
                    egui::FontId::proportional(24.0),
                    Color32::from_gray(180),
                );
            }

            ui.add_space(12.0);

            ui.label(
                RichText::new(self.lang.text(TextKey::Title))
                    .size(20.0)
                    .color(Color32::from_gray(240)),
            );

            ui.add_space(4.0);

            ui.label(
                RichText::new(format!("{} {}", self.lang.text(TextKey::Version), env!("CARGO_PKG_VERSION")))
                    .size(14.0)
                    .color(Color32::from_gray(180)),
            );

            ui.add_space(8.0);

            ui.label(
                RichText::new(self.lang.text(TextKey::AboutDescription))
                    .size(13.0)
                    .color(Color32::from_gray(160)),
            );
        });
    }
}

impl eframe::App for GidyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Ensure logo texture is loaded
        self.ensure_logo(ctx);

        // Tray event polling
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

        // Close interception
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.close_action == CloseAction::ForceExit {
                // Let it close
            } else if self.minimize_to_tray {
                // Config says minimize to tray, skip dialog
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.close_action = CloseAction::MinimizeToTray;
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.close_action = CloseAction::ShowDialog;
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
                ctx.request_repaint();
            }
            CloseAction::None | CloseAction::ShowDialog => {}
        }

        // Track main window rect
        self.main_rect = ctx.input(|i| i.viewport().outer_rect);

        // Refresh stats periodically
        self.refresh_stats();

        theme::apply_theme(ctx);

        // --- Main UI ---
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tab bar
            self.render_tab_bar(ui);

            // Tab content
            match self.current_tab {
                Tab::TrafficMonitor => self.render_traffic_tab(ui),
                Tab::Settings => self.render_settings_tab(ui),
                Tab::SystemConfig => self.render_system_config_tab(ui),
                Tab::About => self.render_about_tab(ui),
            }
        });

        // --- Close dialog ---
        if self.close_action == CloseAction::ShowDialog {
            let dialog_size = egui::vec2(300.0, 120.0);

            let mut dialog_builder = egui::ViewportBuilder::default()
                .with_title(self.lang.text(TextKey::CloseDialogTitle))
                .with_inner_size(dialog_size)
                .with_resizable(false);

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
                                    self.close_action = CloseAction::ForceExit;
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                                if ui.button(&yes_text).clicked() {
                                    self.close_action = CloseAction::MinimizeToTray;
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                            });
                        });
                    });
                },
            );
        }

        // Keep repaint alive when hidden for tray events
        if !ctx.input(|i| i.viewport().focused.unwrap_or(true)) {
            ctx.request_repaint();
        }
    }
}

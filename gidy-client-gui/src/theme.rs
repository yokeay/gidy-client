use egui::{Color32, CornerRadius, Stroke, Style, Visuals};

pub fn setup_fonts(ctx: &egui::Context) {
    #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
    let mut fonts = egui::FontDefinitions::default();

    #[cfg(target_os = "windows")]
    {
        let cjk_paths = [
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
            "C:\\Windows\\Fonts\\simsun.ttc",
        ];
        for path in &cjk_paths {
            if let Ok(data) = std::fs::read(path) {
                fonts.font_data.insert(
                    "cjk".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(data)),
                );
                fonts
                    .families
                    .get_mut(&egui::FontFamily::Proportional)
                    .unwrap()
                    .insert(0, "cjk".to_owned());
                tracing::info!("Loaded CJK font from {}", path);
                break;
            }
        }
    }

    ctx.set_fonts(fonts);
}

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = Style::default();

    // Glass-dark visual style
    style.visuals = Visuals {
        dark_mode: true,
        override_text_color: Some(Color32::from_gray(230)),
        window_corner_radius: CornerRadius::same(12),
        window_shadow: egui::epaint::Shadow {
            offset: [0, 8],
            blur: 32,
            spread: 0,
            color: Color32::from_black_alpha(160),
        },
        window_fill: Color32::from_rgba_premultiplied(18, 18, 22, 210),
        panel_fill: Color32::from_rgba_premultiplied(24, 24, 28, 180),
        faint_bg_color: Color32::from_rgba_premultiplied(32, 32, 36, 120),
        extreme_bg_color: Color32::from_rgba_premultiplied(12, 12, 14, 230),
        code_bg_color: Color32::from_rgba_premultiplied(20, 20, 24, 200),
        warn_fg_color: Color32::from_rgb(255, 180, 60),
        error_fg_color: Color32::from_rgb(255, 80, 80),
        hyperlink_color: Color32::from_rgb(120, 180, 255),
        selection: egui::style::Selection {
            bg_fill: Color32::from_rgba_premultiplied(80, 80, 120, 100),
            stroke: Stroke::new(1.0, Color32::from_rgb(120, 140, 200)),
        },
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgba_premultiplied(40, 40, 48, 120),
                weak_bg_fill: Color32::from_rgba_premultiplied(30, 30, 36, 80),
                bg_stroke: Stroke::new(1.0, Color32::from_rgba_premultiplied(60, 60, 70, 150)),
                corner_radius: CornerRadius::same(8),
                fg_stroke: Stroke::new(1.0, Color32::from_gray(200)),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgba_premultiplied(40, 40, 48, 150),
                weak_bg_fill: Color32::from_rgba_premultiplied(30, 30, 36, 100),
                bg_stroke: Stroke::new(1.0, Color32::from_rgba_premultiplied(70, 70, 80, 180)),
                corner_radius: CornerRadius::same(8),
                fg_stroke: Stroke::new(1.0, Color32::from_gray(180)),
                expansion: 1.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgba_premultiplied(55, 55, 65, 200),
                weak_bg_fill: Color32::from_rgba_premultiplied(40, 40, 48, 150),
                bg_stroke: Stroke::new(1.0, Color32::from_rgb(140, 160, 220)),
                corner_radius: CornerRadius::same(8),
                fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
                expansion: 2.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgba_premultiplied(50, 50, 60, 220),
                weak_bg_fill: Color32::from_rgba_premultiplied(40, 40, 48, 180),
                bg_stroke: Stroke::new(2.0, Color32::from_rgb(100, 180, 100)),
                corner_radius: CornerRadius::same(8),
                fg_stroke: Stroke::new(2.0, Color32::from_gray(255)),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgba_premultiplied(45, 45, 55, 200),
                weak_bg_fill: Color32::from_rgba_premultiplied(35, 35, 42, 150),
                bg_stroke: Stroke::new(1.0, Color32::from_rgb(120, 160, 210)),
                corner_radius: CornerRadius::same(8),
                fg_stroke: Stroke::new(1.5, Color32::from_gray(220)),
                expansion: 0.0,
            },
        },
        ..Visuals::dark()
    };

    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    style.animation_time = 0.2;

    ctx.set_style(style);
}

/// Green gradient colors for active state
pub const GREEN_ON: Color32 = Color32::from_rgb(80, 220, 120);
pub const RED_OFF: Color32 = Color32::from_rgb(220, 80, 80);

/// Accent color
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(100, 160, 240);

/// Chart line colors
pub const CHART_UP: Color32 = GREEN_ON;
pub const CHART_DOWN: Color32 = ACCENT_BLUE;
pub const CHART_UP_FILL: Color32 = Color32::from_rgba_premultiplied(80, 220, 120, 40);
pub const CHART_DOWN_FILL: Color32 = Color32::from_rgba_premultiplied(100, 160, 240, 40);
pub const CHART_GRID: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 15);

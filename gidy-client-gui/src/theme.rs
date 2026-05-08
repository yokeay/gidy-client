use egui::{Color32, CornerRadius, Stroke, Style, Visuals};

#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode { Light, Dark }

#[derive(Clone, Copy, PartialEq)]
pub enum ThemeColor { Blue, Emerald, Purple, Orange, Rose }

impl ThemeColor {
    pub fn accent(self) -> Color32 {
        match self {
            ThemeColor::Blue => Color32::from_rgb(59, 130, 246),
            ThemeColor::Emerald => Color32::from_rgb(16, 185, 129),
            ThemeColor::Purple => Color32::from_rgb(168, 85, 247),
            ThemeColor::Orange => Color32::from_rgb(249, 115, 22),
            ThemeColor::Rose => Color32::from_rgb(244, 63, 94),
        }
    }

    pub fn accent_light(self) -> Color32 {
        match self {
            ThemeColor::Blue => Color32::from_rgb(37, 99, 235),
            ThemeColor::Emerald => Color32::from_rgb(5, 150, 105),
            ThemeColor::Purple => Color32::from_rgb(147, 51, 234),
            ThemeColor::Orange => Color32::from_rgb(234, 88, 12),
            ThemeColor::Rose => Color32::from_rgb(225, 29, 72),
        }
    }
}

pub struct AppTheme {
    pub mode: ThemeMode,
    pub color: ThemeColor,
    pub bg: Color32,
    pub card_bg: Color32,
    pub fg: Color32,
    pub muted: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub chart_up: Color32,
    pub chart_down: Color32,
    pub banner_from: Color32,
    pub banner_to: Color32,
}

impl AppTheme {
    pub fn new(mode: ThemeMode, color: ThemeColor) -> Self {
        match mode {
            ThemeMode::Dark => Self {
                mode, color,
                bg: Color32::from_rgb(15, 15, 17),
                card_bg: Color32::from_rgb(26, 26, 30),
                fg: Color32::from_rgb(228, 228, 231),
                muted: Color32::from_rgb(140, 140, 150),
                border: Color32::from_rgb(45, 45, 52),
                accent: color.accent(),
                chart_up: Color32::from_rgb(16, 185, 129),
                chart_down: Color32::from_rgb(59, 130, 246),
                banner_from: Color32::from_rgb(5, 150, 105),
                banner_to: Color32::from_rgb(16, 185, 129),
            },
            ThemeMode::Light => Self {
                mode, color,
                bg: Color32::from_rgb(245, 245, 245),
                card_bg: Color32::from_rgb(255, 255, 255),
                fg: Color32::from_rgb(26, 26, 26),
                muted: Color32::from_rgb(140, 140, 150),
                border: Color32::from_rgb(225, 225, 230),
                accent: color.accent_light(),
                chart_up: Color32::from_rgb(5, 150, 105),
                chart_down: Color32::from_rgb(37, 99, 235),
                banner_from: Color32::from_rgb(5, 150, 105),
                banner_to: Color32::from_rgb(16, 185, 129),
            },
        }
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
        *self = Self::new(self.mode, self.color);
    }

    pub fn set_color(&mut self, c: ThemeColor) {
        self.color = c;
        *self = Self::new(self.mode, c);
    }

    pub fn banner_gradient(&self) -> (Color32, Color32) {
        (self.banner_from, self.banner_to)
    }

    pub fn apply_to_style(&self, style: &mut Style) {
        style.visuals = Visuals {
            dark_mode: self.mode == ThemeMode::Dark,
            override_text_color: Some(self.fg),
            panel_fill: self.bg,
            window_fill: self.bg,
            extreme_bg_color: self.bg,
            window_corner_radius: CornerRadius::same(8),
            window_shadow: Default::default(),
            window_stroke: Stroke::new(0.0, Color32::TRANSPARENT),
            menu_corner_radius: CornerRadius::same(8),
            ..Visuals::default()
        };
        style.visuals.widgets.noninteractive.bg_fill = self.card_bg;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.fg);
        style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);
        style.visuals.widgets.inactive.bg_fill = self.card_bg;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.fg);
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
        style.visuals.widgets.hovered.bg_fill = rgba_mix(self.accent, 30);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.fg);
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
        style.visuals.widgets.active.bg_fill = rgba_mix(self.accent, 50);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.fg);
        style.visuals.widgets.active.corner_radius = CornerRadius::same(6);
        style.visuals.selection.bg_fill = rgba_mix(self.accent, 60);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
    }
}

fn rgba_mix(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(c.r(), c.g(), c.b(), alpha)
}

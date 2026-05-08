use egui::{Align2, Color32, CornerRadius, Rect, RichText, Stroke, StrokeKind, Vec2};
use crate::i18n::I18n;
use crate::proxy_state::GuiConfig;
use crate::theme::{AppTheme, ThemeMode, ThemeColor};

pub struct SettingsForm {
    pub auto_start: bool,
    pub auto_connect: bool,
    pub minimize_to_tray: bool,
    pub log_retention_days: String,
    pub toast: Option<String>,
}

impl SettingsForm {
    pub fn from_config(c: &GuiConfig) -> Self {
        Self {
            auto_start: c.auto_start,
            auto_connect: c.auto_connect,
            minimize_to_tray: c.minimize_to_tray,
            log_retention_days: c.log_retention_days.to_string(),
            toast: None,
        }
    }

    pub fn apply_to(&self, c: &mut GuiConfig) {
        c.auto_start = self.auto_start;
        c.auto_connect = self.auto_connect;
        c.minimize_to_tray = self.minimize_to_tray;
        c.log_retention_days = self.log_retention_days.parse().unwrap_or(7);
    }
}

pub fn user_settings_ui(
    ui: &mut egui::Ui,
    form: &mut SettingsForm,
    theme: &mut AppTheme,
    i18n: &I18n,
    on_save: &mut bool,
    on_theme_mode_change: &mut Option<ThemeMode>,
    on_theme_color_change: &mut Option<ThemeColor>,
) {
    ui.heading(RichText::new(i18n.t("userSettings.title")).color(theme.fg).size(18.0));
    ui.add_space(16.0);

    let available = ui.available_size();

    // Basic Settings card
    let card_rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(available.x - 16.0, 240.0));
    ui.painter().rect_filled(card_rect, CornerRadius::same(10), theme.card_bg);
    ui.painter().rect_stroke(card_rect, CornerRadius::same(10), Stroke::new(1.0, theme.border), StrokeKind::Inside);

    let mut card = ui.child_ui(card_rect.shrink(16.0), egui::Layout::top_down(egui::Align::Min), None);
    card.label(RichText::new(i18n.t("userSettings.basicSettings")).color(theme.fg).size(14.0));
    card.add_space(16.0);

    // Auto-start
    toggle_row(&mut card, i18n.t("userSettings.autoStart"), &mut form.auto_start, theme);
    card.add_space(12.0);

    // Auto-connect
    toggle_row(&mut card, i18n.t("userSettings.autoConnect"), &mut form.auto_connect, theme);
    card.add_space(12.0);

    // Minimize to tray
    toggle_row(&mut card, i18n.t("userSettings.minimizeToTray"), &mut form.minimize_to_tray, theme);
    card.add_space(16.0);

    // Log retention
    card.label(RichText::new(i18n.t("userSettings.logRetention")).color(theme.muted).size(12.0));
    card.add_space(4.0);
    card.horizontal(|ui| {
        let field_rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(60.0, 28.0));
        ui.painter().rect_filled(field_rect, CornerRadius::same(6), theme.bg);
        ui.painter().rect_stroke(field_rect, CornerRadius::same(6), Stroke::new(1.0, theme.border), StrokeKind::Inside);
        let mut child = ui.child_ui(field_rect.shrink(6.0), egui::Layout::left_to_right(egui::Align::Center), None);
        child.add(egui::TextEdit::singleline(&mut form.log_retention_days)
            .font(egui::FontId::proportional(13.0))
            .text_color(theme.fg)
            .frame(false)
            .desired_width(40.0));
        ui.advance_cursor_after_rect(Rect::from_min_size(field_rect.left_top(), Vec2::new(field_rect.width(), field_rect.height() + 4.0)));
        ui.label(RichText::new(i18n.t("userSettings.days")).color(theme.muted).size(12.0));
    });

    ui.advance_cursor_after_rect(Rect::from_min_size(card_rect.left_top(), Vec2::new(available.x, card_rect.height() + 16.0)));

    // Theme Settings card
    let theme_card = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(available.x - 16.0, 160.0));
    ui.painter().rect_filled(theme_card, CornerRadius::same(10), theme.card_bg);
    ui.painter().rect_stroke(theme_card, CornerRadius::same(10), Stroke::new(1.0, theme.border), StrokeKind::Inside);

    let mut tcard = ui.child_ui(theme_card.shrink(16.0), egui::Layout::top_down(egui::Align::Min), None);
    tcard.label(RichText::new(i18n.t("userSettings.themeMode")).color(theme.fg).size(13.0));
    tcard.add_space(8.0);

    // Theme mode segmented toggle
    let mode_rect = Rect::from_min_size(tcard.cursor().left_top(), Vec2::new(160.0, 32.0));
    tcard.painter().rect_filled(mode_rect, CornerRadius::same(8), theme.bg);
    tcard.painter().rect_stroke(mode_rect, CornerRadius::same(8), Stroke::new(1.0, theme.border), StrokeKind::Inside);

    let light_rect = Rect::from_min_max(
        mode_rect.left_top(),
        egui::pos2(mode_rect.center().x, mode_rect.bottom()),
    );
    let dark_rect = Rect::from_min_max(
        egui::pos2(mode_rect.center().x, mode_rect.top()),
        mode_rect.right_bottom(),
    );

    let is_dark = theme.mode == ThemeMode::Dark;
    if is_dark {
        tcard.painter().rect_filled(dark_rect.shrink(2.0), CornerRadius::same(6), theme.accent);
    } else {
        tcard.painter().rect_filled(light_rect.shrink(2.0), CornerRadius::same(6), theme.accent);
    }

    tcard.painter().text(light_rect.center(), Align2::CENTER_CENTER,
        i18n.t("userSettings.light"), egui::FontId::proportional(12.0),
        if !is_dark { Color32::WHITE } else { theme.muted });
    tcard.painter().text(dark_rect.center(), Align2::CENTER_CENTER,
        i18n.t("userSettings.dark"), egui::FontId::proportional(12.0),
        if is_dark { Color32::WHITE } else { theme.muted });

    if tcard.allocate_rect(light_rect, egui::Sense::click()).clicked() {
        *on_theme_mode_change = Some(ThemeMode::Light);
    }
    if tcard.allocate_rect(dark_rect, egui::Sense::click()).clicked() {
        *on_theme_mode_change = Some(ThemeMode::Dark);
    }
    tcard.advance_cursor_after_rect(Rect::from_min_size(mode_rect.left_top(), Vec2::new(160.0, 32.0 + 16.0)));

    // Theme color
    tcard.label(RichText::new(i18n.t("userSettings.themeColor")).color(theme.fg).size(13.0));
    tcard.add_space(8.0);
    tcard.horizontal(|ui| {
        let colors = [
            ThemeColor::Blue,
            ThemeColor::Emerald,
            ThemeColor::Purple,
            ThemeColor::Orange,
            ThemeColor::Rose,
        ];
        for &c in &colors {
            let is_active = theme.color == c;
            let dot_r = if is_active { 12.0 } else { 10.0 };
            let dot_rect = Rect::from_center_size(ui.cursor().center_top() + Vec2::new(0.0, 14.0), Vec2::new(dot_r * 2.0, dot_r * 2.0));

            ui.painter().circle(dot_rect.center(), dot_r, c.accent(), if is_active {
                Stroke::new(2.0, theme.fg)
            } else {
                Stroke::new(0.0, Color32::TRANSPARENT)
            });

            let resp = ui.allocate_rect(dot_rect.expand(6.0), egui::Sense::click());
            if resp.clicked() {
                *on_theme_color_change = Some(c);
            }
        }
    });

    ui.advance_cursor_after_rect(Rect::from_min_size(theme_card.left_top(), Vec2::new(available.x, theme_card.height() + 24.0)));

    // Version info
    ui.label(RichText::new(i18n.t("userSettings.updateCheck")).color(theme.fg).size(13.0));
    ui.add_space(4.0);
    ui.label(RichText::new(format!("{}: v0.2.3", i18n.t("userSettings.currentVersion"))).color(theme.muted).size(12.0));
    ui.add_space(16.0);

    // Save button
    let btn_rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(160.0, 36.0));
    ui.painter().rect_filled(btn_rect, CornerRadius::same(8), theme.accent);
    ui.painter().text(btn_rect.center(), Align2::CENTER_CENTER,
        i18n.t("userSettings.saveConfig"), egui::FontId::proportional(13.0), Color32::WHITE);
    if ui.allocate_rect(btn_rect, egui::Sense::click()).clicked() {
        *on_save = true;
    }
    ui.advance_cursor_after_rect(Rect::from_min_size(btn_rect.left_top(), Vec2::new(160.0, 36.0 + 8.0)));

    // Toast
    if let Some(ref msg) = form.toast {
        ui.label(RichText::new(msg).color(theme.chart_up).size(12.0));
    }
}

fn toggle_row(ui: &mut egui::Ui, label: &str, value: &mut bool, theme: &AppTheme) {
    ui.horizontal(|ui| {
        let toggle_w = 40.0;
        let toggle_h = 22.0;
        let toggle_rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(toggle_w, toggle_h));

        let track_color = if *value { theme.accent } else { theme.border };
        ui.painter().rect_filled(toggle_rect, CornerRadius::same(11), track_color);

        let knob_r = toggle_h / 2.0 - 2.0;
        let knob_x = if *value {
            toggle_rect.right() - knob_r - 2.0
        } else {
            toggle_rect.left() + knob_r + 2.0
        };
        ui.painter().circle(
            egui::pos2(knob_x, toggle_rect.center().y),
            knob_r,
            Color32::WHITE,
            Stroke::new(0.0, Color32::TRANSPARENT),
        );

        let resp = ui.allocate_rect(toggle_rect, egui::Sense::click());
        if resp.clicked() {
            *value = !*value;
        }

        ui.add_space(8.0);
        ui.label(RichText::new(label).color(theme.fg).size(13.0));
    });
}

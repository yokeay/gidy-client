use egui::{Align2, Rect, RichText, CornerRadius, Stroke, StrokeKind, Vec2};
use crate::i18n::I18n;
use crate::theme::AppTheme;

pub fn about_ui(ui: &mut egui::Ui, theme: &AppTheme, i18n: &I18n) {
    let available = ui.available_size();

    ui.heading(RichText::new(i18n.t("nav.about")).color(theme.fg).size(18.0));
    ui.add_space(24.0);

    // Center card
    let card_w = 400.0;
    let card_h = 260.0;
    let card_x = (available.x - card_w) / 2.0;
    let card_rect = Rect::from_min_size(
        ui.cursor().left_top() + Vec2::new(card_x, 0.0),
        Vec2::new(card_w, card_h),
    );
    ui.painter().rect_filled(card_rect, CornerRadius::same(12), theme.card_bg);
    ui.painter().rect_stroke(card_rect, CornerRadius::same(12), Stroke::new(1.0, theme.border), StrokeKind::Inside);

    // Logo text
    ui.painter().text(
        card_rect.center_top() + Vec2::new(0.0, 32.0),
        Align2::CENTER_CENTER,
        "gidy client",
        egui::FontId::proportional(24.0),
        theme.accent,
    );

    ui.painter().text(
        card_rect.center_top() + Vec2::new(0.0, 64.0),
        Align2::CENTER_CENTER,
        format!("v0.2.3"),
        egui::FontId::proportional(14.0),
        theme.muted,
    );

    // Description
    ui.painter().text(
        card_rect.center_top() + Vec2::new(0.0, 96.0),
        Align2::CENTER_CENTER,
        "A fast and lightweight proxy client using QUIC transport\nwith protocol morphing for enhanced privacy.",
        egui::FontId::proportional(12.0),
        theme.muted,
    );

    // Tech stack
    ui.painter().text(
        card_rect.center_top() + Vec2::new(0.0, 150.0),
        Align2::CENTER_CENTER,
        "Built with Rust · egui/eframe · gidy protocol",
        egui::FontId::proportional(12.0),
        theme.muted,
    );

    // Divider
    let div_y = card_rect.top() + 180.0;
    ui.painter().line_segment(
        [egui::pos2(card_rect.left() + 40.0, div_y), egui::pos2(card_rect.right() - 40.0, div_y)],
        Stroke::new(1.0, theme.border),
    );

    // Footer
    ui.painter().text(
        card_rect.center_top() + Vec2::new(0.0, 210.0),
        Align2::CENTER_CENTER,
        "MIT License · Copyright 2024",
        egui::FontId::proportional(11.0),
        theme.muted,
    );

    ui.advance_cursor_after_rect(Rect::from_min_size(card_rect.left_top(), Vec2::new(available.x, card_h + 24.0)));
}

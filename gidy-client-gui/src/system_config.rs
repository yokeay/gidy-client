use egui::{Align2, Color32, CornerRadius, Rect, Stroke, StrokeKind, Vec2};
use crate::i18n::I18n;
use crate::proxy_state::GuiConfig;
use crate::theme::AppTheme;

pub struct ConfigForm {
    pub psk_hex: String,
    pub server_addr: String,
    pub server_port: String,
    pub server_name: String,
    pub socks5_addr: String,
    pub socks5_port: String,
    pub http_addr: String,
    pub http_port: String,
    pub protocol: String,
    pub mode: String,
    pub toast: Option<String>,
}

impl ConfigForm {
    pub fn from_config(c: &GuiConfig) -> Self {
        Self {
            psk_hex: c.psk_hex.clone(),
            server_addr: c.server_addr.clone(),
            server_port: c.server_port.to_string(),
            server_name: c.server_name.clone(),
            socks5_addr: c.socks5_addr.clone(),
            socks5_port: c.socks5_port.to_string(),
            http_addr: c.http_addr.clone(),
            http_port: c.http_port.to_string(),
            protocol: c.protocol.clone(),
            mode: c.mode.clone(),
            toast: None,
        }
    }

    pub fn to_config(&self) -> GuiConfig {
        GuiConfig {
            psk_hex: self.psk_hex.clone(),
            server_addr: self.server_addr.clone(),
            server_port: self.server_port.parse().unwrap_or(4433),
            server_name: self.server_name.clone(),
            socks5_addr: self.socks5_addr.clone(),
            socks5_port: self.socks5_port.parse().unwrap_or(1080),
            http_addr: self.http_addr.clone(),
            http_port: self.http_port.parse().unwrap_or(8080),
            protocol: self.protocol.clone(),
            mode: self.mode.clone(),
            ..GuiConfig::default()
        }
    }
}

pub fn system_config_ui(
    ui: &mut egui::Ui,
    form: &mut ConfigForm,
    theme: &AppTheme,
    i18n: &I18n,
    on_save: &mut bool,
    on_generate_psk: &mut bool,
) {
    ui.heading(RichText::new(i18n.t("systemConfig.title")).color(theme.fg).size(18.0));
    ui.add_space(16.0);

    let available = ui.available_size();
    let col_w = (available.x - 24.0) / 2.0;

    // Left column: Proxy Server
    section_card(ui, i18n.t("systemConfig.proxyServer"), theme, col_w, |ui| {
        text_field_row(ui, i18n.t("systemConfig.serverAddr"), &mut form.server_addr, theme, i18n);
        text_field_row(ui, i18n.t("systemConfig.serverPort"), &mut form.server_port, theme, i18n);
        text_field_row(ui, i18n.t("systemConfig.psk"), &mut form.psk_hex, theme, i18n);
        ui.add_space(4.0);
        ui.label(RichText::new(i18n.t("systemConfig.pskHint")).color(theme.muted).size(11.0));
        ui.add_space(8.0);
        if theme_button(ui, i18n.t("systemConfig.generatePsk"), theme.accent, theme) {
            *on_generate_psk = true;
        }
        ui.add_space(12.0);

        // Protocol dropdown
        ui.label(RichText::new(i18n.t("systemConfig.protocol")).color(theme.muted).size(12.0));
        egui::ComboBox::from_id_salt("protocol_select")
            .selected_text(&form.protocol)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut form.protocol, "gidy".to_string(), "gidy");
            });
    });

    ui.advance_cursor_after_rect(egui::Rect::from_min_size(
        ui.cursor().left_top(),
        Vec2::new(col_w + 12.0, 0.0),
    ));

    // Right column: Local Proxy
    section_card(ui, i18n.t("systemConfig.localProxy"), theme, col_w, |ui| {
        text_field_row(ui, i18n.t("systemConfig.socks5Addr"), &mut form.socks5_addr, theme, i18n);
        text_field_row(ui, i18n.t("systemConfig.socks5Port"), &mut form.socks5_port, theme, i18n);
        text_field_row(ui, i18n.t("systemConfig.httpAddr"), &mut form.http_addr, theme, i18n);
        text_field_row(ui, i18n.t("systemConfig.httpPort"), &mut form.http_port, theme, i18n);

        ui.add_space(12.0);
        ui.label(RichText::new(i18n.t("systemConfig.mode")).color(theme.muted).size(12.0));

        // Mode segmented toggle
        let mode_rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(180.0, 32.0));
        draw_segmented_toggle(
            ui,
            mode_rect,
            &["global".to_string(), "pac".to_string()],
            &[i18n.t("systemConfig.globalMode"), i18n.t("systemConfig.pacMode")],
            &mut form.mode,
            theme,
        );
        ui.advance_cursor_after_rect(mode_rect.expand2(Vec2::new(0.0, 8.0)));
    });

    ui.add_space(20.0);

    // Connection note
    let note_w = available.x - 24.0;
    ui.label(RichText::new(i18n.t("systemConfig.connectionNote")).color(theme.muted).size(12.0));
    ui.add_space(6.0);
    let note_rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(note_w, 50.0));
    ui.painter().rect_filled(note_rect, CornerRadius::same(8), theme.card_bg);
    ui.painter().rect_stroke(note_rect, CornerRadius::same(8), Stroke::new(1.0, theme.border), StrokeKind::Inside);

    let mut note_ui = ui.child_ui(note_rect.shrink(12.0), egui::Layout::top_down(egui::Align::Min), None);
    note_ui.label(RichText::new(i18n.t("systemConfig.connectionNoteText")).color(theme.muted).size(11.0));
    ui.advance_cursor_after_rect(Rect::from_min_size(note_rect.left_top(), Vec2::new(note_w, note_rect.height() + 16.0)));

    // Save button
    ui.add_space(8.0);
    if theme_button(ui, i18n.t("systemConfig.saveAndConnect"), theme.accent, theme) {
        *on_save = true;
    }

    // Toast
    if let Some(ref msg) = form.toast {
        ui.add_space(12.0);
        ui.label(RichText::new(msg).color(theme.chart_up).size(12.0));
    }
}

fn section_card<R>(ui: &mut egui::Ui, title: &str, theme: &AppTheme, width: f32, add_contents: impl FnOnce(&mut egui::Ui) -> R) {
    let desired_h = 320.0;
    let rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(width, desired_h));
    ui.painter().rect_filled(rect, CornerRadius::same(10), theme.card_bg);
    ui.painter().rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, theme.border), StrokeKind::Inside);

    let mut child = ui.child_ui(rect.shrink(16.0), egui::Layout::top_down(egui::Align::Min), None);
    child.label(RichText::new(title).color(theme.fg).size(14.0));
    child.add_space(12.0);
    add_contents(&mut child);
    ui.advance_cursor_after_rect(Rect::from_min_size(rect.left_top(), Vec2::new(width, 0.0)));
}

fn text_field_row(ui: &mut egui::Ui, label: &str, value: &mut String, theme: &AppTheme, _i18n: &I18n) {
    ui.label(RichText::new(label).color(theme.muted).size(12.0));
    ui.add_space(2.0);
    let rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(240.0, 28.0));
    ui.painter().rect_filled(rect, CornerRadius::same(6), theme.bg);
    ui.painter().rect_stroke(rect, CornerRadius::same(6), Stroke::new(1.0, theme.border), StrokeKind::Inside);
    let mut child = ui.child_ui(rect.shrink(8.0), egui::Layout::left_to_right(egui::Align::Center), None);
    let _resp = child.add(egui::TextEdit::singleline(value)
        .font(egui::FontId::proportional(13.0))
        .text_color(theme.fg)
        .frame(false));
    ui.advance_cursor_after_rect(Rect::from_min_size(rect.left_top(), Vec2::new(rect.width(), rect.height() + 6.0)));
}

fn draw_segmented_toggle(
    ui: &mut egui::Ui,
    rect: Rect,
    values: &[String],
    labels: &[&str],
    selected: &mut String,
    theme: &AppTheme,
) {
    ui.painter().rect_filled(rect, CornerRadius::same(8), theme.bg);
    ui.painter().rect_stroke(rect, CornerRadius::same(8), Stroke::new(1.0, theme.border), StrokeKind::Inside);

    let n = values.len() as f32;
    let seg_w = rect.width() / n;

    for (i, (val, label)) in values.iter().zip(labels.iter()).enumerate() {
        let seg_rect = Rect::from_min_max(
            egui::pos2(rect.left() + seg_w * i as f32, rect.top()),
            egui::pos2(rect.left() + seg_w * (i + 1) as f32, rect.bottom()),
        );

        if *val == *selected {
            ui.painter().rect_filled(seg_rect.shrink(2.0), CornerRadius::same(6), theme.accent);
        }

        ui.painter().text(
            seg_rect.center(),
            Align2::CENTER_CENTER,
            *label,
            egui::FontId::proportional(12.0),
            if *val == *selected { Color32::WHITE } else { theme.muted },
        );

        let resp = ui.allocate_rect(seg_rect, egui::Sense::click());
        if resp.clicked() {
            *selected = val.clone();
        }
    }
}

fn theme_button(ui: &mut egui::Ui, label: &str, color: Color32, _theme: &AppTheme) -> bool {
    let btn_rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(160.0, 36.0));
    ui.painter().rect_filled(btn_rect, CornerRadius::same(8), color);
    ui.painter().text(
        btn_rect.center(),
        Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        Color32::WHITE,
    );
    let resp = ui.allocate_rect(btn_rect, egui::Sense::click());
    ui.advance_cursor_after_rect(Rect::from_min_size(btn_rect.left_top(), Vec2::new(btn_rect.width(), btn_rect.height() + 4.0)));
    resp.clicked()
}

use egui::RichText;

use egui::{Align2, Rect, RichText, CornerRadius, Stroke, StrokeKind, Vec2};
use crate::i18n::{I18n, format_bytes, format_speed, format_uptime};
use crate::proxy_state::StatsSnapshot;
use crate::speed_chart::SpeedChart;
use crate::theme::AppTheme;

pub struct ConnectionLogEntry {
    pub time: String,
    pub target: String,
    pub entry_type: String,
    pub size: u64,
    pub duration: String,
}

pub struct TrafficMonitorPage {
    pub chart: SpeedChart,
    pub log: Vec<ConnectionLogEntry>,
}

impl TrafficMonitorPage {
    pub fn new() -> Self {
        Self {
            chart: SpeedChart::new(),
            log: Vec::new(),
        }
    }

    pub fn update_chart(&mut self, snap: &StatsSnapshot) {
        self.chart.push(snap.speed_up_kbps, snap.speed_down_kbps);
    }
}

pub fn traffic_monitor_ui(
    ui: &mut egui::Ui,
    snapshot: &StatsSnapshot,
    chart: &mut SpeedChart,
    log: &[ConnectionLogEntry],
    theme: &AppTheme,
    i18n: &I18n,
) {
    ui.heading(RichText::new(i18n.t("trafficMonitor.title")).color(theme.fg).size(18.0));
    ui.add_space(16.0);

    let available = ui.available_size();

    // Stats cards row
    let card_w = (available.x - 48.0) / 4.0;
    let card_h = 72.0;
    let up_speed = format_speed(snapshot.speed_up_kbps);
    let down_speed = format_speed(snapshot.speed_down_kbps);
    let up_total = format_bytes(snapshot.bytes_up);
    let down_total = format_bytes(snapshot.bytes_down);
    let cards = [
        (i18n.t("trafficMonitor.upload"), up_speed.as_str(), theme.chart_up),
        (i18n.t("trafficMonitor.download"), down_speed.as_str(), theme.chart_down),
        (i18n.t("trafficMonitor.totalUpload"), up_total.as_str(), theme.fg),
        (i18n.t("trafficMonitor.totalDownload"), down_total.as_str(), theme.fg),
    ];

    for (i, &(label, val, color)) in cards.iter().enumerate() {
        let pos = ui.cursor().left_top() + Vec2::new((card_w + 16.0) * i as f32, 0.0);
        let rect = Rect::from_min_size(pos, Vec2::new(card_w, card_h));
        ui.painter().rect_filled(rect, CornerRadius::same(10), theme.card_bg);
        ui.painter().rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, theme.border), StrokeKind::Inside);
        ui.painter().text(
            rect.left_top() + Vec2::new(12.0, 12.0),
            Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(11.0),
            theme.muted,
        );
        ui.painter().text(
            rect.left_top() + Vec2::new(12.0, 34.0),
            Align2::LEFT_TOP,
            val,
            egui::FontId::proportional(18.0),
            color,
        );
    }
    ui.advance_cursor_after_rect(Rect::from_min_size(
        ui.cursor().left_top(),
        Vec2::new(available.x, card_h + 16.0),
    ));

    // Uptime bar
    let uptime_text = format!("{}: {}", i18n.t("trafficMonitor.uptime"), format_uptime(snapshot.uptime_secs));
    ui.painter().text(
        ui.cursor().left_top(),
        Align2::LEFT_TOP,
        uptime_text,
        egui::FontId::proportional(12.0),
        theme.muted,
    );
    ui.add_space(20.0);

    // Speed chart
    let chart_area = Rect::from_min_size(
        ui.cursor().left_top(),
        Vec2::new(available.x - 16.0, 200.0),
    );
    ui.painter().rect_filled(chart_area, CornerRadius::same(10), theme.card_bg);
    ui.painter().rect_stroke(chart_area, CornerRadius::same(10), Stroke::new(1.0, theme.border), StrokeKind::Inside);
    let mut chart_ui = ui.child_ui(chart_area.shrink(12.0), egui::Layout::top_down(egui::Align::Min), None);
    chart.ui(&mut chart_ui, theme.chart_up, theme.chart_down);
    ui.advance_cursor_after_rect(Rect::from_min_size(
        chart_area.left_top(),
        Vec2::new(available.x, chart_area.height() + 20.0),
    ));

    // Connection log
    ui.label(RichText::new(i18n.t("trafficMonitor.connectionLog")).color(theme.fg).size(14.0));
    ui.add_space(8.0);

    // Table header
    let header_rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(available.x - 16.0, 32.0));
    ui.painter().rect_filled(header_rect, CornerRadius::same(6), theme.bg);
    let cols = [
        (i18n.t("trafficMonitor.time"), 0.0, 0.20),
        (i18n.t("trafficMonitor.target"), 0.20, 0.40),
        (i18n.t("trafficMonitor.type"), 0.60, 0.15),
        (i18n.t("trafficMonitor.size"), 0.75, 0.10),
        (i18n.t("trafficMonitor.duration"), 0.85, 0.15),
    ];
    for &(label, x, _w) in &cols {
        ui.painter().text(
            header_rect.left_top() + Vec2::new(available.x * x + 12.0, 8.0),
            Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(11.0),
            theme.muted,
        );
    }
    ui.advance_cursor_after_rect(Rect::from_min_size(header_rect.left_top(), Vec2::new(available.x, 32.0)));

    // Table rows
    if log.is_empty() {
        let empty_rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(available.x - 16.0, 60.0));
        ui.painter().text(
            empty_rect.center(),
            Align2::CENTER_CENTER,
            i18n.t("trafficMonitor.noData"),
            egui::FontId::proportional(13.0),
            theme.muted,
        );
        ui.advance_cursor_after_rect(Rect::from_min_size(empty_rect.left_top(), Vec2::new(available.x, 60.0)));
    } else {
        for entry in log.iter().take(20) {
            let row_rect = Rect::from_min_size(ui.cursor().left_top(), Vec2::new(available.x - 16.0, 28.0));
            ui.painter().line_segment(
                [row_rect.left_bottom(), row_rect.right_bottom()],
                Stroke::new(1.0, theme.border.gamma_multiply(0.3)),
            );
            let cols = [
                (&entry.time, 0.0),
                (&entry.target, 0.20),
                (&entry.entry_type, 0.60),
                (&format_bytes(entry.size), 0.75),
                (&entry.duration, 0.85),
            ];
            for (val, x) in &cols {
                ui.painter().text(
                    row_rect.left_top() + Vec2::new(available.x * x + 12.0, 6.0),
                    Align2::LEFT_TOP,
                    *val,
                    egui::FontId::proportional(11.0),
                    theme.fg,
                );
            }
            ui.advance_cursor_after_rect(Rect::from_min_size(row_rect.left_top(), Vec2::new(available.x, 28.0)));
        }
    }
}

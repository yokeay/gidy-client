use egui::{Align2, Color32, Rect, CornerRadius, Stroke, StrokeKind, Vec2};
use crate::i18n::{I18n, format_bytes, format_speed, format_uptime};
use crate::proxy_state::{ProxyStatus, StatsSnapshot};
use crate::speed_chart::{SpeedChart, draw_gradient_header};
use crate::theme::AppTheme;

pub struct DashboardPage {
    pub chart: SpeedChart,
}

impl DashboardPage {
    pub fn new() -> Self {
        Self { chart: SpeedChart::new() }
    }

    pub fn update_chart(&mut self, snap: &StatsSnapshot) {
        self.chart.push(snap.speed_up_kbps, snap.speed_down_kbps);
    }
}

pub fn dashboard_ui(
    ui: &mut egui::Ui,
    status: &ProxyStatus,
    snapshot: &StatsSnapshot,
    chart: &mut SpeedChart,
    _connect_label: &str,
    theme: &AppTheme,
    i18n: &I18n,
    on_toggle: &mut Option<bool>,
) {
    let available = ui.available_size();
    let banner_h = 160.0;

    // Gradient banner
    let banner_rect = Rect::from_min_size(
        ui.cursor().left_top(),
        Vec2::new(available.x, banner_h),
    );
    draw_gradient_header(ui.painter(), banner_rect, theme.banner_from, theme.banner_to);

    // Status indicator in banner
    let status_color = if status.running && status.connected {
        Color32::from_rgb(16, 185, 129)
    } else if status.running {
        Color32::from_rgb(249, 115, 22)
    } else {
        Color32::from_rgb(244, 63, 94)
    };
    let status_ring_r = 28.0;
    let ring_center = banner_rect.left_top() + Vec2::new(40.0, 44.0);
    ui.painter().circle(
        ring_center,
        status_ring_r,
        Color32::TRANSPARENT,
        Stroke::new(3.0, status_color.gamma_multiply(0.4)),
    );
    ui.painter().circle(
        ring_center,
        status_ring_r * 0.7,
        status_color,
        Stroke::new(0.0, Color32::TRANSPARENT),
    );

    // Status text
    let status_text = if status.running && status.connected {
        i18n.t("dashboard.connected")
    } else if status.running {
        i18n.t("common.loading")
    } else {
        i18n.t("dashboard.disconnected")
    };
    ui.painter().text(
        ring_center + Vec2::new(status_ring_r + 16.0, -8.0),
        Align2::LEFT_CENTER,
        status_text,
        egui::FontId::proportional(18.0),
        Color32::WHITE,
    );

    // Uptime
    let uptime = format_uptime(snapshot.uptime_secs);
    ui.painter().text(
        ring_center + Vec2::new(status_ring_r + 16.0, 14.0),
        Align2::LEFT_CENTER,
        format!("{} · {} {}", i18n.t("dashboard.serviceUptime"), uptime, ""),
        egui::FontId::proportional(12.0),
        Color32::WHITE.gamma_multiply(0.7),
    );

    // Start/Stop button
    let btn_pos = banner_rect.right_top() + Vec2::new(-100.0, 36.0);
    let btn_rect = Rect::from_center_size(btn_pos, Vec2::new(80.0, 36.0));
    ui.painter().rect_filled(btn_rect, CornerRadius::same(8), Color32::WHITE.gamma_multiply(0.2));
    let label = if status.running {
        i18n.t("dashboard.stop")
    } else {
        i18n.t("dashboard.start")
    };
    ui.painter().text(
        btn_rect.center(),
        Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(14.0),
        Color32::WHITE,
    );

    let btn_resp = ui.allocate_rect(btn_rect, egui::Sense::click());
    if btn_resp.clicked() {
        *on_toggle = Some(!status.running);
    }

    ui.advance_cursor_after_rect(Rect::from_min_size(
        ui.cursor().left_top(),
        Vec2::new(available.x, banner_h + 16.0),
    ));

    // Speed cards
    let card_w = (available.x - 32.0) / 2.0;
    let card_h = 80.0;

    // Upload card
    let up_rect = ui.cursor().left_top();
    ui.painter().rect_filled(
        Rect::from_min_size(up_rect, Vec2::new(card_w, card_h)),
        CornerRadius::same(10),
        theme.card_bg,
    );
    ui.painter().rect_stroke(
        Rect::from_min_size(up_rect, Vec2::new(card_w, card_h)),
        CornerRadius::same(10),
        Stroke::new(1.0, theme.border),
        StrokeKind::Inside,
    );
    ui.painter().text(
        up_rect + Vec2::new(16.0, 16.0),
        Align2::LEFT_TOP,
        i18n.t("dashboard.uploadSpeed"),
        egui::FontId::proportional(12.0),
        theme.muted,
    );
    ui.painter().text(
        up_rect + Vec2::new(16.0, 36.0),
        Align2::LEFT_TOP,
        format_speed(snapshot.speed_up_kbps),
        egui::FontId::proportional(22.0),
        theme.chart_up,
    );
    ui.painter().text(
        up_rect + Vec2::new(16.0, 60.0),
        Align2::LEFT_TOP,
        format!("{} {}", i18n.t("dashboard.totalUpload"), format_bytes(snapshot.bytes_up)),
        egui::FontId::proportional(11.0),
        theme.muted,
    );

    // Download card
    let down_rect = up_rect + Vec2::new(card_w + 16.0, 0.0);
    ui.painter().rect_filled(
        Rect::from_min_size(down_rect, Vec2::new(card_w, card_h)),
        CornerRadius::same(10),
        theme.card_bg,
    );
    ui.painter().rect_stroke(
        Rect::from_min_size(down_rect, Vec2::new(card_w, card_h)),
        CornerRadius::same(10),
        Stroke::new(1.0, theme.border),
        StrokeKind::Inside,
    );
    ui.painter().text(
        down_rect + Vec2::new(16.0, 16.0),
        Align2::LEFT_TOP,
        i18n.t("dashboard.downloadSpeed"),
        egui::FontId::proportional(12.0),
        theme.muted,
    );
    ui.painter().text(
        down_rect + Vec2::new(16.0, 36.0),
        Align2::LEFT_TOP,
        format_speed(snapshot.speed_down_kbps),
        egui::FontId::proportional(22.0),
        theme.chart_down,
    );
    ui.painter().text(
        down_rect + Vec2::new(16.0, 60.0),
        Align2::LEFT_TOP,
        format!("{} {}", i18n.t("dashboard.totalDownload"), format_bytes(snapshot.bytes_down)),
        egui::FontId::proportional(11.0),
        theme.muted,
    );

    ui.advance_cursor_after_rect(Rect::from_min_size(up_rect, Vec2::new(available.x, card_h + 16.0)));

    // Speed chart
    let chart_area = Rect::from_min_size(
        ui.cursor().left_top(),
        Vec2::new(available.x - 16.0, 220.0),
    );
    ui.painter().rect_filled(
        chart_area.expand(0.0),
        CornerRadius::same(10),
        theme.card_bg,
    );
    ui.painter().rect_stroke(
        chart_area.expand(0.0),
        CornerRadius::same(10),
        Stroke::new(1.0, theme.border),
        StrokeKind::Inside,
    );

    let mut chart_ui = ui.child_ui(chart_area.shrink(12.0), egui::Layout::top_down(egui::Align::Min), None);
    chart.ui(&mut chart_ui, theme.chart_up, theme.chart_down);

    ui.advance_cursor_after_rect(Rect::from_min_size(
        chart_area.left_top(),
        Vec2::new(available.x, chart_area.height() + 16.0),
    ));

    // Bottom stats row
    let stat_w = (available.x - 48.0) / 3.0;
    let stat_h: f32 = 48.0;
    let dns_elapsed = "-".to_string();
    let uptime_str = format_uptime(snapshot.uptime_secs);
    let proxy_conn = if snapshot.connected { "1" } else { "0" };
    let stats = [
        (i18n.t("dashboard.dnsElapsed"), dns_elapsed.as_str(), theme.muted),
        (i18n.t("dashboard.serviceUptime"), uptime_str.as_str(), theme.muted),
        (i18n.t("dashboard.proxyConnections"), proxy_conn, theme.muted),
    ];

    for (i, &(label, val, _)) in stats.iter().enumerate() {
        let pos = ui.cursor().left_top() + Vec2::new((stat_w + 16.0) * i as f32, 0.0);
        let rect = Rect::from_min_size(pos, Vec2::new(stat_w, stat_h));
        ui.painter().rect_filled(rect, CornerRadius::same(8), theme.card_bg);
        ui.painter().rect_stroke(rect, CornerRadius::same(8), Stroke::new(1.0, theme.border), StrokeKind::Inside);
        ui.painter().text(
            rect.left_top() + Vec2::new(12.0, 8.0),
            Align2::LEFT_TOP,
            label,
            egui::FontId::proportional(11.0),
            theme.muted,
        );
        ui.painter().text(
            rect.left_top() + Vec2::new(12.0, 26.0),
            Align2::LEFT_TOP,
            val,
            egui::FontId::proportional(14.0),
            theme.fg,
        );
    }

    ui.advance_cursor_after_rect(Rect::from_min_size(
        ui.cursor().left_top(),
        Vec2::new(available.x, stat_h + 16.0),
    ));

    // Version footer
    ui.painter().text(
        ui.cursor().left_top() + Vec2::new(0.0, 4.0),
        Align2::LEFT_TOP,
        format!("{} v0.2.3", i18n.t("dashboard.version")),
        egui::FontId::proportional(11.0),
        theme.muted,
    );
}

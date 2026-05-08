use egui::{Align2, Rect, CornerRadius, Stroke, Vec2};
use crate::theme::AppTheme;
use crate::i18n::I18n;

#[derive(Clone, Copy, PartialEq)]
pub enum NavPage {
    Dashboard,
    SystemConfig,
    TrafficMonitor,
    UserSettings,
    About,
}

impl NavPage {
    pub fn label(self, i18n: &I18n) -> &str {
        match self {
            NavPage::Dashboard => i18n.t("nav.dashboard"),
            NavPage::SystemConfig => i18n.t("nav.systemConfig"),
            NavPage::TrafficMonitor => i18n.t("nav.trafficMonitor"),
            NavPage::UserSettings => i18n.t("nav.userSettings"),
            NavPage::About => i18n.t("nav.about"),
        }
    }

    fn icon(self) -> &'static str {
        match self {
            NavPage::Dashboard => "◉",
            NavPage::SystemConfig => "⚙",
            NavPage::TrafficMonitor => "↕",
            NavPage::UserSettings => "☰",
            NavPage::About => "ℹ",
        }
    }

    fn all() -> &'static [NavPage] {
        &[
            NavPage::Dashboard,
            NavPage::SystemConfig,
            NavPage::TrafficMonitor,
            NavPage::UserSettings,
            NavPage::About,
        ]
    }
}

pub fn sidebar_ui(ui: &mut egui::Ui, active: NavPage, theme: &AppTheme, i18n: &I18n) -> NavPage {
    let mut next_page = active;
    let sidebar_w = 180.0;
    let _available_h = ui.available_height();

    // Sidebar background
    let sidebar_rect = ui.max_rect();
    ui.painter().rect_filled(sidebar_rect, CornerRadius::ZERO, theme.bg);

    // Logo area
    let logo_rect = Rect::from_min_size(
        sidebar_rect.left_top(),
        Vec2::new(sidebar_w, 72.0),
    );
    ui.painter().text(
        logo_rect.center(),
        Align2::CENTER_CENTER,
        "gidy client",
        egui::FontId::proportional(20.0),
        theme.accent,
    );

    // Separator
    let sep_y = logo_rect.bottom();
    ui.painter().line_segment(
        [egui::pos2(sidebar_rect.left() + 16.0, sep_y), egui::pos2(sidebar_rect.left() + sidebar_w - 16.0, sep_y)],
        Stroke::new(1.0, theme.border),
    );

    // Nav items
    let item_h = 40.0;
    let item_area = Rect::from_min_max(
        egui::pos2(sidebar_rect.left(), sep_y + 8.0),
        egui::pos2(sidebar_rect.left() + sidebar_w, sep_y + 8.0 + (NavPage::all().len() as f32 * (item_h + 2.0))),
    );

    let mut ui = ui.child_ui(item_area, egui::Layout::top_down(egui::Align::Min), None);

    for &page in NavPage::all() {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(sidebar_w - 16.0, item_h),
            egui::Sense::click(),
        );

        let is_active = page == active;
        if is_active {
            ui.painter().rect_filled(
                rect.expand2(Vec2::new(2.0, 0.0)),
                CornerRadius::same(8),
                theme.accent.gamma_multiply(0.15),
            );
        } else if response.hovered() {
            ui.painter().rect_filled(
                rect,
                CornerRadius::same(8),
                theme.card_bg,
            );
        }

        let text_color = if is_active { theme.accent } else { theme.muted };
        let icon_pos = rect.left_center() + Vec2::new(16.0, 0.0);
        ui.painter().text(
            icon_pos,
            Align2::LEFT_CENTER,
            page.icon(),
            egui::FontId::proportional(14.0),
            text_color,
        );
        ui.painter().text(
            icon_pos + Vec2::new(20.0, 0.0),
            Align2::LEFT_CENTER,
            page.label(i18n),
            egui::FontId::proportional(13.0),
            text_color,
        );

        if response.clicked() {
            next_page = page;
        }
    }

    next_page
}

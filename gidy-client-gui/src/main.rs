mod app;
mod dashboard;
mod system_config;
mod traffic_monitor;
mod user_settings;
mod about;
mod sidebar;
mod speed_chart;
mod theme;
mod i18n;
mod proxy_state;

use app::GidyApp;

fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 680.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "gidy client",
        options,
        Box::new(|_cc| Ok(Box::new(GidyApp::new()))),
    )
}

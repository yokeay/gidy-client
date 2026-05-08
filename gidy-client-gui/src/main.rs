#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod config;
mod lang;
mod proxy;
mod theme;
mod tray;
mod win_proxy;

use std::sync::Arc;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::GuiConfig::load();

    // Set up file logging
    let log_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let log_path = log_dir.join("gidy-client.log");
    let (non_blocking, _guard) = if let Ok(f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        tracing_appender::non_blocking(f)
    } else {
        tracing_appender::non_blocking(std::io::stderr())
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.log_level)),
        )
        .with_writer(non_blocking)
        .with_target(true)
        .with_thread_ids(true)
        .init();

    tracing::info!("gidy-client-gui starting, config: {:?}", config);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    let _rt_guard = rt.enter();

    let proxy_mgr = Arc::new(proxy::ProxyManager::new(config.clone()));
    let app = app::GidyApp::new(config, proxy_mgr, rt);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 560.0])
            .with_min_inner_size([420.0, 360.0])
            .with_resizable(true)
            .with_decorations(true)
            .with_transparent(true),
        ..Default::default()
    };

    eframe::run_native(
        "gidy-client",
        native_options,
        Box::new(|cc| {
            theme::setup_fonts(&cc.egui_ctx);
            theme::apply_theme(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| format!("eframe error: {}", e))?;

    Ok(())
}

mod win_proxy;

use clap::{Parser, Subcommand};
use gidy_client_core::{ClientConfig, GidyClient, Socks5Server, TrafficStats};
use tracing::{info, error};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "gidy-client")]
#[command(version = "0.1.0")]
#[command(about = "gidy proxy client - Linux CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the SOCKS5 proxy
    Run {
        /// Path to config file (TOML)
        #[arg(short, long, default_value = "gidy-client.toml")]
        config: String,
    },
    /// Generate a default config file
    GenConfig {
        /// Output path
        #[arg(short, long, default_value = "gidy-client.toml")]
        output: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { config: config_path } => {
            let config = ClientConfig::from_file(&config_path)
                .unwrap_or_else(|e| {
                    eprintln!("failed to load config {}: {}", config_path, e);
                    std::process::exit(1);
                });

            let filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&config.log_level));
            tracing_subscriber::fmt().with_env_filter(filter).init();

            if let Err(e) = run(config).await {
                error!("fatal: {}", e);
                std::process::exit(1);
            }
        }
        Commands::GenConfig { output } => {
            let cfg = gidy_client_core::generate_default_config();
            let toml_str = toml::to_string_pretty(&cfg)
                .expect("serialize default config");
            std::fs::write(&output, toml_str)
                .unwrap_or_else(|e| {
                    eprintln!("failed to write {}: {}", output, e);
                    std::process::exit(1);
                });
            println!("default config written to {}", output);
        }
    }
}

async fn run(config: ClientConfig) -> Result<(), String> {

    let listen_addr = config.listen_addr.to_string();
    info!("gidy-client v{} starting", env!("CARGO_PKG_VERSION"));
    info!("server: {}", config.server_addr);
    info!("proxy listen: {}", listen_addr);

    // Ctrl+C handler to clean up system proxy on exit
    tokio::spawn(async {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            info!("ctrl+c received, clearing system proxy...");
            win_proxy::clear_proxy();
            std::process::exit(0);
        }
    });

    let stats = TrafficStats::new();

    // Spawn stats reporter once
    let stats_clone = stats.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            let snap = stats_clone.snapshot();
            info!(
                "stats: ↑{} ↓{} | {:.1} kbps↑ {:.1} kbps↓ | uptime {}s",
                format_bytes(snap.bytes_up),
                format_bytes(snap.bytes_down),
                snap.speed_up_kbps,
                snap.speed_down_kbps,
                snap.uptime_secs,
            );
        }
    });

    loop {
        info!("connecting to gidy-server...");
        let client = GidyClient::new(config.clone(), stats.clone())?;

        match client.connect().await {
            Ok(conn) => {
                info!("connected, starting SOCKS5 proxy...");

                // Auto-configure Windows system proxy
                let proxy_addr = format!("socks=127.0.0.1:{}", config.listen_addr.port());
                win_proxy::set_proxy(&proxy_addr);

                let proxy = Socks5Server::new(listen_addr.clone(), conn, stats.clone());

                match proxy.run().await {
                    Err(e) => {
                        error!("proxy error: {}", e);
                        win_proxy::clear_proxy();
                        info!("reconnecting in 5 seconds...");
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                    Ok(()) => {
                        info!("proxy stopped normally");
                        win_proxy::clear_proxy();
                    }
                }
            }
            Err(e) => {
                error!("connection failed: {}", e);
                win_proxy::clear_proxy();
                info!("retrying in 5 seconds...");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

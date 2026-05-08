use crate::config::GuiConfig;
use gidy_client_core::client::GidyClient;
use gidy_client_core::config::ClientConfig;
use gidy_client_core::proxy::Socks5Server;
use gidy_client_core::stats::TrafficStats;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ProxyState {
    pub running: bool,
    pub connected: bool,
    pub session_id: Option<[u8; 16]>,
    pub error: Option<String>,
}

pub struct ProxyManager {
    config: Mutex<GuiConfig>,
    state: Mutex<ProxyState>,
    stats: Arc<TrafficStats>,
    shutdown_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

impl ProxyManager {
    pub fn new(config: GuiConfig) -> Self {
        Self {
            config: Mutex::new(config),
            state: Mutex::new(ProxyState {
                running: false,
                connected: false,
                session_id: None,
                error: None,
            }),
            stats: TrafficStats::new(),
            shutdown_tx: Mutex::new(None),
        }
    }

    pub fn stats(&self) -> Arc<TrafficStats> {
        self.stats.clone()
    }

    #[allow(dead_code)]
    pub async fn update_config(&self, config: GuiConfig) {
        *self.config.lock().await = config;
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        if state.running {
            return Ok(());
        }

        let config = self.config.lock().await.clone();
        let listen_addr: SocketAddr = config
            .listen_addr
            .parse()
            .map_err(|e| format!("invalid listen_addr: {}", e))?;

        // Validate PSK format
        if config.psk_hex.len() != 64 {
            return Err("PSK must be 64 hex characters".into());
        }

        let client_config = ClientConfig {
            psk_hex: config.psk_hex.clone(),
            server_addr: config.server_addr.clone(),
            server_name: config.server_name.clone(),
            log_level: config.log_level.clone(),
            listen_addr,
        };

        let client = GidyClient::new(client_config, self.stats.clone())
            .map_err(|e| format!("client: {}", e))?;

        let conn = client.connect().await.map_err(|e| {
            state.error = Some(e.clone());
            format!("connect: {}", e)
        })?;

        state.connected = true;
        state.session_id = Some(conn.session_id);
        state.error = None;
        state.running = true;

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        let listen_str = config.listen_addr.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            let socks5 = Socks5Server::new(listen_str, conn, stats);
            tokio::select! {
                result = socks5.run() => {
                    if let Err(e) = result {
                        tracing::error!("socks5 server: {}", e);
                    }
                }
                _ = shutdown_rx => {
                    tracing::info!("proxy shutdown requested");
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) {
        let mut state = self.state.lock().await;
        state.running = false;
        state.connected = false;
        state.session_id = None;

        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(());
        }

        // Clear Windows proxy settings
        #[cfg(target_os = "windows")]
        {
            crate::win_proxy::clear_proxy();
        }
    }
}

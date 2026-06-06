import { invoke } from "@tauri-apps/api/core";

export interface ProxyStatus {
  running: boolean;
  connected: boolean;
  error: string | null;
}

export interface StatsSnapshot {
  bytes_up: number;
  bytes_down: number;
  speed_up_kbps: number;
  speed_down_kbps: number;
  uptime_secs: number;
  active_connections: number;
}

export interface GuiConfig {
  psk_hex: string;
  server_addr: string;
  server_port: number;
  server_name: string;
  ws_url: string;
  ech_config_base64: string;
  ech_token: string;
  socks5_addr: string;
  socks5_port: number;
  http_addr: string;
  http_port: number;
  protocol: string;
  mode: string;
  auto_start: boolean;
  auto_connect: boolean;
  minimize_to_tray: boolean;
  log_retention_days: number;
  theme: string;
  theme_color: string;
  log_level: string;
}

export async function connect(): Promise<ProxyStatus> {
  return invoke<ProxyStatus>("connect");
}

export async function disconnect(): Promise<ProxyStatus> {
  return invoke<ProxyStatus>("disconnect");
}

export async function getStats(): Promise<StatsSnapshot> {
  return invoke<StatsSnapshot>("get_stats");
}

export async function getConfig(): Promise<GuiConfig> {
  return invoke<GuiConfig>("get_config");
}

export async function updateConfig(config: GuiConfig): Promise<GuiConfig> {
  return invoke<GuiConfig>("update_config", { config });
}

export async function getStatus(): Promise<ProxyStatus> {
  return invoke<ProxyStatus>("get_status");
}

export async function generatePsk(): Promise<string> {
  return invoke<string>("generate_psk");
}

export async function refreshEch(): Promise<string> {
  return invoke<string>("refresh_ech");
}

export interface ConnectionLogEntry {
  target: string;
  protocol: string;
  connected_at: string;
}

export async function getConnectionLogs(): Promise<ConnectionLogEntry[]> {
  return invoke<ConnectionLogEntry[]>("get_connection_logs");
}

export function formatBytes(bytes: number): string {
  if (bytes >= 1_073_741_824)
    return (bytes / 1_073_741_824).toFixed(2) + " GB";
  if (bytes >= 1_048_576) return (bytes / 1_048_576).toFixed(2) + " MB";
  if (bytes >= 1024) return (bytes / 1024).toFixed(2) + " KB";
  return bytes + " B";
}

export function formatSpeed(kbps: number): string {
  if (kbps >= 1000) return (kbps / 1000).toFixed(1) + " Mbps";
  return kbps.toFixed(1) + " Kbps";
}

export function formatUptime(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  if (m > 0) return `${m}:${String(s).padStart(2, "0")}`;
  return `${s}s`;
}

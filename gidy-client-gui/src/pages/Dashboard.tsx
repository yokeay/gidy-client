import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { ArrowUp, ArrowDown, Activity, Server, Zap } from "lucide-react";
import {
  getStats,
  getStatus,
  connect,
  disconnect,
  formatBytes,
  formatSpeed,
  formatUptime,
  StatsSnapshot,
} from "../api";

export default function Dashboard() {
  const { t } = useTranslation();
  const [status, setStatus] = useState({
    running: false,
    connected: false,
    error: null as string | null,
  });
  const [stats, setStats] = useState<StatsSnapshot>({
    bytes_up: 0,
    bytes_down: 0,
    speed_up_kbps: 0,
    speed_down_kbps: 0,
    uptime_secs: 0,
    active_connections: 0,
  });
  const [loading, setLoading] = useState(false);
  const [connectError, setConnectError] = useState<string | null>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | undefined>(undefined);

  const refreshStats = useCallback(async () => {
    try {
      const [s, st] = await Promise.all([getStats(), getStatus()]);
      setStats(s);
      setStatus(st);
    } catch {}
  }, []);

  useEffect(() => {
    refreshStats();
    pollRef.current = setInterval(refreshStats, 1000);
    return () => clearInterval(pollRef.current);
  }, [refreshStats]);

  const handleToggle = async () => {
    setLoading(true);
    setConnectError(null);
    try {
      if (status.running) {
        const result = await disconnect();
        setStatus(result);
      } else {
        const result = await connect();
        setStatus(result);
      }
    } catch (e) {
      setConnectError(String(e));
    }
    setLoading(false);
  };

  return (
    <div className="space-y-5">
      {/* Connection Card */}
      <div className="bg-card rounded-2xl border border-border p-8">
        <div className="flex items-center gap-8">
          {/* Big connect button */}
          <div className="flex-shrink-0 flex flex-col items-center gap-3">
            <button
              onClick={handleToggle}
              disabled={loading}
              className={`relative w-24 h-24 rounded-full flex items-center justify-center transition-all duration-300 disabled:opacity-50 ${
                status.running
                  ? "bg-foreground text-background shadow-[0_0_32px_rgba(0,0,0,0.25)] hover:opacity-90"
                  : "bg-muted border-2 border-border text-muted-foreground hover:border-foreground/40 hover:text-foreground"
              }`}
            >
              {status.running && (
                <span className="absolute inset-0 rounded-full animate-ping bg-foreground/20" />
              )}
              <Zap
                size={32}
                strokeWidth={1.75}
                className={status.running ? "relative z-10" : ""}
              />
            </button>
            <span className="text-xs font-medium text-muted-foreground">
              {loading
                ? t("common.loading")
                : status.running
                ? t("dashboard.stop")
                : t("dashboard.start")}
            </span>
          </div>

          {/* Status info */}
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-1">
              <span className="relative flex h-2.5 w-2.5">
                {status.running && (
                  <span className="absolute inline-flex h-full w-full rounded-full bg-foreground opacity-30 animate-ping" />
                )}
                <span
                  className={`relative inline-flex h-2.5 w-2.5 rounded-full ${
                    status.running ? "bg-foreground" : "bg-muted-foreground/40"
                  }`}
                />
              </span>
              <span className="text-xs uppercase tracking-widest text-muted-foreground">
                gidy client · v0.2.7
              </span>
            </div>
            <h2 className="text-3xl font-bold tabular mb-1">
              {status.running
                ? t("dashboard.connected")
                : t("dashboard.disconnected")}
            </h2>
            <p className="text-sm text-muted-foreground">
              {status.running
                ? `${t("dashboard.serviceUptime")}: ${formatUptime(stats.uptime_secs)}`
                : t("dashboard.clickToConnect")}
            </p>
          </div>

          {/* Quick stats */}
          <div className="flex-shrink-0 flex gap-6 text-right">
            <div>
              <div className="flex items-center gap-1.5 justify-end mb-1">
                <ArrowUp size={12} className="text-muted-foreground" />
                <span className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  {t("dashboard.uploadSpeed")}
                </span>
              </div>
              <p className="text-xl font-semibold tabular">
                {formatSpeed(stats.speed_up_kbps)}
              </p>
            </div>
            <div>
              <div className="flex items-center gap-1.5 justify-end mb-1">
                <ArrowDown size={12} className="text-muted-foreground" />
                <span className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  {t("dashboard.downloadSpeed")}
                </span>
              </div>
              <p className="text-xl font-semibold tabular">
                {formatSpeed(stats.speed_down_kbps)}
              </p>
            </div>
          </div>
        </div>
      </div>

      {connectError && (
        <div className="bg-destructive/10 border border-destructive/30 rounded-2xl p-4 text-sm text-destructive">
          {connectError}
        </div>
      )}

      {/* Traffic cards */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-card rounded-2xl border border-border p-6">
          <div className="flex items-center justify-between mb-3">
            <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
              {t("dashboard.uploadSpeed")}
            </span>
            <ArrowUp size={14} strokeWidth={1.75} className="text-muted-foreground" />
          </div>
          <p className="text-2xl font-semibold text-foreground tabular">
            {formatSpeed(stats.speed_up_kbps)}
          </p>
          <p className="text-xs text-muted-foreground mt-2 tabular">
            {t("dashboard.totalUpload")}: {formatBytes(stats.bytes_up)}
          </p>
        </div>
        <div className="bg-card rounded-2xl border border-border p-6">
          <div className="flex items-center justify-between mb-3">
            <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
              {t("dashboard.downloadSpeed")}
            </span>
            <ArrowDown size={14} strokeWidth={1.75} className="text-muted-foreground" />
          </div>
          <p className="text-2xl font-semibold text-foreground tabular">
            {formatSpeed(stats.speed_down_kbps)}
          </p>
          <p className="text-xs text-muted-foreground mt-2 tabular">
            {t("dashboard.totalDownload")}: {formatBytes(stats.bytes_down)}
          </p>
        </div>
      </div>

      {/* Bottom metric cards */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-card rounded-2xl border border-border p-6">
          <div className="flex items-center justify-between mb-3">
            <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
              {t("dashboard.proxyConnections")}
            </span>
            <Activity size={14} strokeWidth={1.75} className="text-muted-foreground" />
          </div>
          <p className="text-xl font-semibold tabular">
            {stats.active_connections}
          </p>
        </div>
        <div className="bg-card rounded-2xl border border-border p-6">
          <div className="flex items-center justify-between mb-3">
            <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
              {t("dashboard.dnsElapsed")}
            </span>
            <Server size={14} strokeWidth={1.75} className="text-muted-foreground" />
          </div>
          <p className="text-xl font-semibold tabular">12ms</p>
        </div>
        <div className="bg-card rounded-2xl border border-border p-6">
          <div className="flex items-center justify-between mb-3">
            <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
              {t("dashboard.serviceUptime")}
            </span>
            <Activity size={14} strokeWidth={1.75} className="text-muted-foreground" />
          </div>
          <p className="text-xl font-semibold tabular">
            {formatUptime(stats.uptime_secs)}
          </p>
        </div>
      </div>
    </div>
  );
}

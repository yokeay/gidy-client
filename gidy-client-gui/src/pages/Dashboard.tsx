import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Play, Square, ArrowUp, ArrowDown, Activity, Server } from "lucide-react";
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
      {/* Banner */}
      <div className="relative overflow-hidden rounded-2xl bg-foreground text-background p-7">
        <div className="absolute right-6 top-1/2 -translate-y-1/2 opacity-10">
          <Server size={140} strokeWidth={1.2} />
        </div>
        <div className="relative z-10 flex items-center justify-between gap-6">
          <div className="min-w-0">
            <p className="text-xs uppercase tracking-widest opacity-60 mb-1.5">
              gidy client
            </p>
            <h2 className="text-3xl font-bold tabular">
              {t("dashboard.version")} v0.2.7
            </h2>
            <p className="opacity-70 mt-2 text-sm">
              {status.connected
                ? t("dashboard.connected")
                : t("dashboard.disconnected")}
            </p>
          </div>
          <button
            onClick={handleToggle}
            disabled={loading}
            className="shrink-0 flex items-center gap-2 px-6 py-3 rounded-xl bg-background/10 hover:bg-background/20 backdrop-blur transition-colors font-medium text-sm disabled:opacity-50"
          >
            {status.running ? (
              <>
                <Square size={15} fill="currentColor" />
                {t("dashboard.stop")}
              </>
            ) : (
              <>
                <Play size={15} fill="currentColor" />
                {t("dashboard.start")}
              </>
            )}
          </button>
        </div>
      </div>

      {connectError && (
        <div className="bg-destructive/10 border border-destructive/30 rounded-2xl p-4 text-sm text-destructive">
          {connectError}
        </div>
      )}

      {/* Speed cards */}
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

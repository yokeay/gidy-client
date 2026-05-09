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
  const [status, setStatus] = useState({ running: false, connected: false, error: null as string | null });
  const [stats, setStats] = useState<StatsSnapshot>({
    bytes_up: 0, bytes_down: 0, speed_up_kbps: 0,
    speed_down_kbps: 0, uptime_secs: 0, active_connections: 0,
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
    <div className="space-y-6">
      {/* Banner */}
      <div className="relative overflow-hidden rounded-xl bg-gradient-to-r from-emerald-600 to-emerald-500 p-6 text-white">
        <div className="absolute right-4 top-1/2 -translate-y-1/2 opacity-20">
          <Server size={120} />
        </div>
        <div className="relative z-10 flex items-center justify-between">
          <div>
            <p className="text-emerald-100 text-sm mb-1">gidy client</p>
            <h2 className="text-2xl font-bold">
              {t("dashboard.version")} v0.2.5
            </h2>
            <p className="text-emerald-100 mt-2 text-sm">
              {status.connected
                ? t("dashboard.connected")
                : t("dashboard.disconnected")}
            </p>
          </div>
          <button
            onClick={handleToggle}
            disabled={loading}
            className="relative z-10 flex items-center gap-2 px-6 py-3 rounded-xl bg-white/20 hover:bg-white/30 backdrop-blur transition-all font-medium text-sm disabled:opacity-50"
          >
            {status.running ? (
              <>
                <Square size={16} fill="currentColor" />
                {t("dashboard.stop")}
              </>
            ) : (
              <>
                <Play size={16} fill="currentColor" />
                {t("dashboard.start")}
              </>
            )}
          </button>
        </div>
      </div>

      {connectError && (
        <div className="bg-destructive/10 border border-destructive/30 rounded-xl p-4 text-sm text-destructive">
          {connectError}
        </div>
      )}

      {/* Speed cards */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-card rounded-xl border border-border p-4">
          <div className="flex items-center gap-2 text-muted-foreground text-xs mb-2">
            <ArrowUp size={14} className="text-chart-up" />
            {t("dashboard.uploadSpeed")}
          </div>
          <p className="text-2xl font-bold text-foreground">
            {formatSpeed(stats.speed_up_kbps)}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            {t("dashboard.totalUpload")}: {formatBytes(stats.bytes_up)}
          </p>
        </div>
        <div className="bg-card rounded-xl border border-border p-4">
          <div className="flex items-center gap-2 text-muted-foreground text-xs mb-2">
            <ArrowDown size={14} className="text-chart-down" />
            {t("dashboard.downloadSpeed")}
          </div>
          <p className="text-2xl font-bold text-foreground">
            {formatSpeed(stats.speed_down_kbps)}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            {t("dashboard.totalDownload")}: {formatBytes(stats.bytes_down)}
          </p>
        </div>
      </div>

      {/* Bottom metric cards */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-card rounded-xl border border-border p-4">
          <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1">
            <Activity size={14} />
            {t("dashboard.proxyConnections")}
          </div>
          <p className="text-lg font-semibold">{stats.active_connections}</p>
        </div>
        <div className="bg-card rounded-xl border border-border p-4">
          <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1">
            <Server size={14} />
            {t("dashboard.dnsElapsed")}
          </div>
          <p className="text-lg font-semibold">12ms</p>
        </div>
        <div className="bg-card rounded-xl border border-border p-4">
          <div className="flex items-center gap-2 text-muted-foreground text-xs mb-1">
            <Activity size={14} />
            {t("dashboard.serviceUptime")}
          </div>
          <p className="text-lg font-semibold">{formatUptime(stats.uptime_secs)}</p>
        </div>
      </div>
    </div>
  );
}

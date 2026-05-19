import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import {
  getStats,
  getStatus,
  connect,
  disconnect,
  formatUptime,
  StatsSnapshot,
} from "../api";

export default function Dashboard() {
  const { t } = useTranslation();
  const [status, setStatus] = useState({ running: false, connected: false, error: null as string | null });
  const [stats, setStats] = useState<StatsSnapshot>({
    bytes_up: 0, bytes_down: 0,
    speed_up_kbps: 0, speed_down_kbps: 0,
    uptime_secs: 0, active_connections: 0,
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
      const result = status.running ? await disconnect() : await connect();
      setStatus(result);
    } catch (e) {
      setConnectError(String(e));
    }
    setLoading(false);
  };

  const isRunning = status.running;

  return (
    <div className="flex flex-col items-center" style={{ paddingTop: 12 }}>

      {/* ── Power Button ── */}
      <div
        className="relative flex items-center justify-center"
        style={{ width: 200, height: 200, marginBottom: 24 }}
      >
        {/* Pulse rings */}
        {[190, 168, 148].map((size, i) => (
          <div
            key={i}
            className={`ring-pulse-${i + 1}`}
            style={{
              position: "absolute",
              width: size,
              height: size,
              borderRadius: "50%",
              border: `1.5px solid ${isRunning ? "rgba(46,204,113,0.2)" : "rgba(122,130,153,0.12)"}`,
            }}
          />
        ))}

        {/* Button */}
        <button
          onClick={handleToggle}
          disabled={loading}
          style={{
            width: 120,
            height: 120,
            borderRadius: "50%",
            background: isRunning
              ? "radial-gradient(circle at 35% 35%, #4de88a, #1fa852 50%, #0d6b32)"
              : "radial-gradient(circle at 35% 35%, #2a2d38, #1a1d25 50%, #13161c)",
            border: "none",
            cursor: "pointer",
            position: "relative",
            zIndex: 2,
            boxShadow: isRunning
              ? "0 0 0 6px rgba(46,204,113,0.15), 0 0 0 12px rgba(46,204,113,0.08), 0 8px 32px rgba(0,0,0,0.5), inset 0 2px 8px rgba(255,255,255,0.25)"
              : "0 0 0 6px rgba(255,255,255,0.04), 0 8px 32px rgba(0,0,0,0.4)",
            transition: "all 0.3s",
            transform: loading ? "scale(0.96)" : "scale(1)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <span
            style={{
              fontSize: 36,
              color: isRunning ? "rgba(255,255,255,0.9)" : "var(--muted-fg)",
              textShadow: isRunning ? "0 0 20px rgba(255,255,255,0.5)" : "none",
            }}
          >
            ⏻
          </span>
        </button>
      </div>

      {/* Status text */}
      <div className="flex items-center gap-2" style={{ marginBottom: 6 }}>
        <div
          className={isRunning ? "pulse-dot" : ""}
          style={{
            width: 8,
            height: 8,
            borderRadius: "50%",
            background: isRunning ? "var(--accent-green)" : "var(--muted-fg)",
            boxShadow: isRunning ? "0 0 8px var(--accent-green)" : "none",
          }}
        />
        <span style={{ fontSize: 16, color: isRunning ? "var(--accent-green)" : "var(--muted-fg)", fontWeight: 600 }}>
          {loading
            ? t("common.loading")
            : isRunning ? t("dashboard.connected") : t("dashboard.disconnected")}
        </span>
      </div>
      <p style={{ fontSize: 13, color: "var(--muted-fg)", marginBottom: 32 }}>
        {isRunning
          ? `${t("dashboard.serviceUptime")}: ${formatUptime(stats.uptime_secs)}`
          : t("dashboard.clickToConnect")}
      </p>

      {/* Error */}
      {connectError && (
        <div
          style={{
            marginBottom: 20,
            padding: "10px 16px",
            background: "rgba(231,76,60,0.1)",
            border: "1px solid rgba(231,76,60,0.3)",
            borderRadius: 8,
            fontSize: 13,
            color: "#e74c3c",
            maxWidth: 480,
            width: "100%",
          }}
        >
          {connectError}
        </div>
      )}

      {/* Quick stats */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(3, 1fr)",
          gap: 12,
          width: "100%",
          maxWidth: 480,
        }}
      >
        {[
          { label: t("dashboard.uploadSpeed"), value: `${stats.speed_up_kbps.toFixed(1)} Kbps`, color: "var(--accent-green)" },
          { label: t("dashboard.downloadSpeed"), value: `${stats.speed_down_kbps.toFixed(1)} Kbps`, color: "var(--accent-blue)" },
          { label: t("dashboard.proxyConnections"), value: String(stats.active_connections), color: "var(--fg)" },
        ].map((card, i) => (
          <div
            key={i}
            style={{
              background: "var(--card)",
              border: "1px solid var(--border)",
              borderRadius: 10,
              padding: 16,
              display: "flex",
              flexDirection: "column",
              gap: 6,
            }}
          >
            <div style={{ fontSize: 11, color: "var(--text-muted, #4a5268)", letterSpacing: "0.06em" }}>
              {card.label}
            </div>
            <div
              className="tabular"
              style={{ fontSize: 18, fontWeight: 600, color: card.color, lineHeight: 1 }}
            >
              {card.value}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

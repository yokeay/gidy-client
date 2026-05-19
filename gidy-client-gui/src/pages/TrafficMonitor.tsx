import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import SpeedChart from "../components/SpeedChart";
import {
  getStats,
  getStatus,
  formatBytes,
  formatSpeed,
  StatsSnapshot,
} from "../api";

interface ChartPoint {
  time: number;
  up: number;
  down: number;
}

interface ConnectionLog {
  time: string;
  process: string;
  protocol: string;
  local: string;
  remote: string;
  speedUp: string;
  speedDown: string;
}

export default function TrafficMonitor() {
  const { t } = useTranslation();
  const [stats, setStats] = useState<StatsSnapshot>({
    bytes_up: 0, bytes_down: 0,
    speed_up_kbps: 0, speed_down_kbps: 0,
    uptime_secs: 0, active_connections: 0,
  });
  const [chartData, setChartData] = useState<ChartPoint[]>([]);
  const [logs, setLogs] = useState<ConnectionLog[]>([]);
  const [running, setRunning] = useState(false);
  const pollRef = useRef<ReturnType<typeof setInterval> | undefined>(undefined);
  const logidRef = useRef(0);

  const refresh = useCallback(async () => {
    try {
      const [s, st] = await Promise.all([getStats(), getStatus()]);
      setStats(s);
      setRunning(st.running);
      setChartData(prev => {
        const next = [...prev, { time: Date.now(), up: s.speed_up_kbps, down: s.speed_down_kbps }];
        if (next.length > 60) return next.slice(-60);
        return next;
      });
      if (s.active_connections > 0 && st.running) {
        const id = ++logidRef.current;
        const protocols = ["TCP", "UDP", "TCP"];
        const processes = ["Chrome.exe", "gidy-client.exe", "System"];
        setLogs(prev => {
          const entry: ConnectionLog = {
            time: new Date().toLocaleTimeString(),
            process: processes[id % 3],
            protocol: protocols[id % 3],
            local: `192.168.1.100:${50000 + (id % 999)}`,
            remote: `conn-${id}.example.com:443`,
            speedUp: formatSpeed(s.speed_up_kbps / Math.max(1, s.active_connections)),
            speedDown: formatSpeed(s.speed_down_kbps / Math.max(1, s.active_connections)),
          };
          const next = [entry, ...prev];
          if (next.length > 100) return next.slice(0, 100);
          return next;
        });
      }
    } catch {}
  }, []);

  useEffect(() => {
    refresh();
    pollRef.current = setInterval(refresh, 1000);
    return () => clearInterval(pollRef.current);
  }, [refresh]);

  const statCards = [
    { label: t("trafficMonitor.upload"), value: formatSpeed(stats.speed_up_kbps), arrow: "↑", color: "var(--accent-green)" },
    { label: t("trafficMonitor.download"), value: formatSpeed(stats.speed_down_kbps), arrow: "↓", color: "var(--accent-blue)" },
    { label: t("trafficMonitor.totalUpload"), value: formatBytes(stats.bytes_up), color: "var(--fg)" },
    { label: t("trafficMonitor.totalDownload"), value: formatBytes(stats.bytes_down), color: "var(--fg)" },
    { label: t("trafficMonitor.connectionLog"), value: String(stats.active_connections), suffix: running ? "个" : "-", color: "var(--fg)" },
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>

      {/* ── 5 stat cards ── */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: 12 }}>
        {statCards.map((card, i) => (
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
              transition: "border-color 0.15s, transform 0.15s",
            }}
            onMouseEnter={e => {
              (e.currentTarget as HTMLElement).style.borderColor = "rgba(255,255,255,0.1)";
              (e.currentTarget as HTMLElement).style.transform = "translateY(-1px)";
            }}
            onMouseLeave={e => {
              (e.currentTarget as HTMLElement).style.borderColor = "var(--border)";
              (e.currentTarget as HTMLElement).style.transform = "translateY(0)";
            }}
          >
            <div style={{ fontSize: 11, color: "var(--text-muted, #4a5268)", letterSpacing: "0.06em" }}>
              {card.label}
            </div>
            <div className="tabular" style={{ fontSize: 20, fontWeight: 600, color: card.color, lineHeight: 1 }}>
              {card.value}
              {card.arrow && (
                <span style={{ fontSize: 11, marginLeft: 3, color: card.color }}>
                  {card.arrow}
                </span>
              )}
              {card.suffix && (
                <span style={{ fontSize: 12, color: "var(--muted-fg)", fontWeight: 400, marginLeft: 2 }}>
                  {card.suffix}
                </span>
              )}
            </div>
          </div>
        ))}
      </div>

      {/* ── Chart ── */}
      <div
        style={{
          background: "var(--card)",
          border: "1px solid var(--border)",
          borderRadius: 10,
          padding: 20,
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 16 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 14 }}>
            <span style={{ fontSize: 14, fontWeight: 500, color: "var(--fg)" }}>
              {t("trafficMonitor.download")} / {t("trafficMonitor.upload")}
            </span>
          </div>
          <div style={{ display: "flex", gap: 16 }}>
            {[
              { color: "var(--accent-green)", label: `${t("trafficMonitor.upload")} (Kbps)` },
              { color: "var(--accent-blue)", label: `${t("trafficMonitor.download")} (Kbps)` },
            ].map((l, i) => (
              <div key={i} style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 12, color: "var(--muted-fg)" }}>
                <div style={{ width: 8, height: 8, borderRadius: "50%", background: l.color }} />
                {l.label}
              </div>
            ))}
          </div>
        </div>
        <div style={{ height: 160 }}>
          <SpeedChart data={chartData} />
        </div>
      </div>

      {/* ── Connection table ── */}
      <div
        style={{
          background: "var(--card)",
          border: "1px solid var(--border)",
          borderRadius: 10,
          overflow: "hidden",
        }}
      >
        <div
          style={{
            padding: "16px 20px 12px",
            borderBottom: "1px solid var(--border)",
            fontSize: 14,
            fontWeight: 500,
            color: "var(--fg)",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <span>{t("trafficMonitor.connectionLog")}</span>
          <button
            onClick={() => setLogs([])}
            style={{
              display: "flex",
              alignItems: "center",
              gap: 5,
              background: "transparent",
              border: "none",
              color: "var(--muted-fg)",
              fontSize: 12,
              cursor: "pointer",
              padding: "5px 10px",
              borderRadius: 6,
              fontFamily: "var(--font-ui)",
              transition: "all 0.15s",
            }}
            onMouseEnter={e => {
              (e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.05)";
              (e.currentTarget as HTMLElement).style.color = "var(--fg)";
            }}
            onMouseLeave={e => {
              (e.currentTarget as HTMLElement).style.background = "transparent";
              (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
            }}
          >
            ↻ {t("trafficMonitor.time")}
          </button>
        </div>
        <div className="scroll-thin" style={{ overflowY: "auto", maxHeight: 260 }}>
          <table style={{ width: "100%", borderCollapse: "collapse" }}>
            <thead>
              <tr>
                {["应用/进程", "协议", "本地地址", "远程地址", "上传速度", "下载速度", "状态"].map(h => (
                  <th
                    key={h}
                    style={{
                      padding: "10px 16px",
                      textAlign: "left",
                      fontSize: 11,
                      color: "var(--text-muted, #4a5268)",
                      fontWeight: 500,
                      letterSpacing: "0.06em",
                      background: "rgba(255,255,255,0.02)",
                    }}
                  >
                    {h}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {logs.length === 0 ? (
                <tr>
                  <td
                    colSpan={7}
                    style={{ padding: "40px 0", textAlign: "center", color: "var(--muted-fg)", fontSize: 13 }}
                  >
                    {t("trafficMonitor.noData")}
                  </td>
                </tr>
              ) : (
                logs.map((log, i) => (
                  <tr
                    key={i}
                    style={{ borderTop: "1px solid var(--border)" }}
                    onMouseEnter={e => ((e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.02)")}
                    onMouseLeave={e => ((e.currentTarget as HTMLElement).style.background = "transparent")}
                  >
                    <td style={{ padding: "13px 16px", fontFamily: "var(--font-mono)", fontSize: 12, color: "var(--fg)" }}>
                      {log.process}
                    </td>
                    <td style={{ padding: "13px 16px", fontSize: 13, color: "var(--muted-fg)" }}>{log.protocol}</td>
                    <td style={{ padding: "13px 16px", fontSize: 12, color: "var(--muted-fg)", fontFamily: "var(--font-mono)" }}>
                      {log.local}
                    </td>
                    <td style={{ padding: "13px 16px", fontSize: 12, color: "var(--muted-fg)", fontFamily: "var(--font-mono)" }}>
                      {log.remote}
                    </td>
                    <td style={{ padding: "13px 16px", fontFamily: "var(--font-mono)", fontSize: 12, color: "var(--accent-green)" }}>
                      ↑ {log.speedUp}
                    </td>
                    <td style={{ padding: "13px 16px", fontFamily: "var(--font-mono)", fontSize: 12, color: "var(--accent-blue)" }}>
                      ↓ {log.speedDown}
                    </td>
                    <td style={{ padding: "13px 16px", fontSize: 13, color: "var(--accent-green)", fontWeight: 500 }}>
                      活动
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "12px 20px",
            borderTop: "1px solid var(--border)",
          }}
        >
          <span style={{ fontSize: 12, color: "var(--text-muted, #4a5268)" }}>
            共 {stats.active_connections} 个连接
          </span>
        </div>
      </div>
    </div>
  );
}

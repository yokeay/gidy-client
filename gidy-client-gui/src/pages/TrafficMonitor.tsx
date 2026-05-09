import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { ArrowUp, ArrowDown } from "lucide-react";
import Speedometer from "../components/Speedometer";
import SpeedChart from "../components/SpeedChart";
import { getStats, getStatus, formatBytes, formatSpeed, StatsSnapshot } from "../api";

interface ChartPoint {
  time: number;
  up: number;
  down: number;
}

interface ConnectionLog {
  time: string;
  target: string;
  type: string;
  size: string;
  duration: string;
}

export default function TrafficMonitor() {
  const { t } = useTranslation();
  const [stats, setStats] = useState<StatsSnapshot>({
    bytes_up: 0, bytes_down: 0, speed_up_kbps: 0,
    speed_down_kbps: 0, uptime_secs: 0, active_connections: 0,
  });
  const [chartData, setChartData] = useState<ChartPoint[]>([]);
  const [logs, setLogs] = useState<ConnectionLog[]>([]);
  const [running, setRunning] = useState(false);
  const pollRef = useRef<ReturnType<typeof setInterval> | undefined>(undefined);
  const logidRef = useRef(0);

  const totalSpeed = stats.speed_up_kbps + stats.speed_down_kbps;

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
        const types = ["HTTP", "HTTPS", "SOCKS5"];
        setLogs(prev => {
          const entry: ConnectionLog = {
            time: new Date().toLocaleTimeString(),
            target: `conn-${id}.example.com`,
            type: types[id % 3],
            size: formatBytes(s.bytes_up + s.bytes_down),
            duration: `${s.uptime_secs}s`,
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

  return (
    <div className="space-y-6">
      {/* Speedometer */}
      <div className="bg-card rounded-xl border border-border p-6 flex flex-col items-center">
        {running ? (
          <Speedometer speedKbps={totalSpeed} maxKbps={2000} />
        ) : (
          <Speedometer speedKbps={0} maxKbps={2000} />
        )}
        <div className="flex items-center gap-8 mt-4">
          <div className="flex items-center gap-2 text-sm">
            <span className="w-3 h-3 rounded-full bg-chart-up" />
            <span className="text-muted-foreground">{t("trafficMonitor.upload")}</span>
            <span className="font-mono font-semibold text-chart-up">{formatSpeed(stats.speed_up_kbps)}</span>
          </div>
          <div className="flex items-center gap-2 text-sm">
            <span className="w-3 h-3 rounded-full bg-chart-down" />
            <span className="text-muted-foreground">{t("trafficMonitor.download")}</span>
            <span className="font-mono font-semibold text-chart-down">{formatSpeed(stats.speed_down_kbps)}</span>
          </div>
        </div>
      </div>

      {/* Speed cards */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-card rounded-xl border border-border p-4">
          <div className="flex items-center gap-2 text-xs text-muted-foreground mb-2">
            <ArrowUp size={14} className="text-chart-up" />
            {t("trafficMonitor.upload")}
          </div>
          <p className="text-2xl font-bold font-mono">{formatSpeed(stats.speed_up_kbps)}</p>
          <p className="text-xs text-muted-foreground mt-2">
            {t("trafficMonitor.totalUpload")}: {formatBytes(stats.bytes_up)}
          </p>
        </div>
        <div className="bg-card rounded-xl border border-border p-4">
          <div className="flex items-center gap-2 text-xs text-muted-foreground mb-2">
            <ArrowDown size={14} className="text-chart-down" />
            {t("trafficMonitor.download")}
          </div>
          <p className="text-2xl font-bold font-mono">{formatSpeed(stats.speed_down_kbps)}</p>
          <p className="text-xs text-muted-foreground mt-2">
            {t("trafficMonitor.totalDownload")}: {formatBytes(stats.bytes_down)}
          </p>
        </div>
      </div>

      {/* Speed Chart */}
      <SpeedChart data={chartData} />

      {/* Connection Log Table */}
      <div className="bg-card rounded-xl border border-border p-4">
        <h3 className="text-sm font-semibold mb-3">{t("trafficMonitor.connectionLog")}</h3>
        <div className="overflow-auto max-h-64">
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b border-border text-muted-foreground">
                <th className="text-left py-2 font-medium">{t("trafficMonitor.time")}</th>
                <th className="text-left py-2 font-medium">{t("trafficMonitor.target")}</th>
                <th className="text-left py-2 font-medium">{t("trafficMonitor.type")}</th>
                <th className="text-left py-2 font-medium">{t("trafficMonitor.size")}</th>
                <th className="text-left py-2 font-medium">{t("trafficMonitor.duration")}</th>
              </tr>
            </thead>
            <tbody>
              {logs.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-8 text-center text-muted-foreground">
                    {t("trafficMonitor.noData")}
                  </td>
                </tr>
              ) : (
                logs.map((log, i) => (
                  <tr key={i} className="border-b border-border/50 hover:bg-muted/50">
                    <td className="py-2 font-mono">{log.time}</td>
                    <td className="py-2">{log.target}</td>
                    <td className="py-2">{log.type}</td>
                    <td className="py-2 font-mono">{log.size}</td>
                    <td className="py-2 font-mono">{log.duration}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { ArrowUp, ArrowDown, Database, HardDrive } from "lucide-react";
import Speedometer from "../components/Speedometer";
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
  target: string;
  type: string;
  size: string;
  duration: string;
}

interface KpiCardProps {
  label: string;
  value: string;
  Icon: typeof ArrowUp;
}

function KpiCard({ label, value, Icon }: KpiCardProps) {
  return (
    <div className="bg-card rounded-2xl border border-border p-5">
      <div className="flex items-center justify-between mb-3">
        <span className="text-[11px] uppercase tracking-wider text-muted-foreground">
          {label}
        </span>
        <Icon size={14} strokeWidth={1.75} className="text-muted-foreground" />
      </div>
      <p className="text-2xl font-semibold tabular text-foreground">{value}</p>
    </div>
  );
}

export default function TrafficMonitor() {
  const { t } = useTranslation();
  const [stats, setStats] = useState<StatsSnapshot>({
    bytes_up: 0,
    bytes_down: 0,
    speed_up_kbps: 0,
    speed_down_kbps: 0,
    uptime_secs: 0,
    active_connections: 0,
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
      setChartData((prev) => {
        const next = [
          ...prev,
          { time: Date.now(), up: s.speed_up_kbps, down: s.speed_down_kbps },
        ];
        if (next.length > 60) return next.slice(-60);
        return next;
      });
      if (s.active_connections > 0 && st.running) {
        const id = ++logidRef.current;
        const types = ["HTTP", "HTTPS", "SOCKS5"];
        setLogs((prev) => {
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
    <div className="space-y-5">
      {/* Top 4 KPI cards */}
      <div className="grid grid-cols-4 gap-4">
        <KpiCard
          label={t("trafficMonitor.upload")}
          value={formatSpeed(stats.speed_up_kbps)}
          Icon={ArrowUp}
        />
        <KpiCard
          label={t("trafficMonitor.download")}
          value={formatSpeed(stats.speed_down_kbps)}
          Icon={ArrowDown}
        />
        <KpiCard
          label={t("trafficMonitor.totalUpload")}
          value={formatBytes(stats.bytes_up)}
          Icon={HardDrive}
        />
        <KpiCard
          label={t("trafficMonitor.totalDownload")}
          value={formatBytes(stats.bytes_down)}
          Icon={Database}
        />
      </div>

      {/* Speedometer + Chart side-by-side */}
      <div className="grid grid-cols-3 gap-4">
        <div className="col-span-1 bg-card rounded-2xl border border-border p-5 flex flex-col items-center justify-center">
          <Speedometer
            speedKbps={running ? totalSpeed : 0}
            maxKbps={2000}
            upKbps={running ? stats.speed_up_kbps : 0}
            downKbps={running ? stats.speed_down_kbps : 0}
          />
        </div>
        <div className="col-span-2 bg-card rounded-2xl border border-border p-5 h-[280px]">
          <SpeedChart data={chartData} />
        </div>
      </div>

      {/* Connection Log */}
      <div className="bg-card rounded-2xl border border-border">
        <div className="flex items-center justify-between px-5 py-4 border-b border-border">
          <h3 className="text-sm font-semibold">
            {t("trafficMonitor.connectionLog")}
          </h3>
          <span className="text-xs text-muted-foreground tabular">
            {logs.length}
          </span>
        </div>
        <div className="scroll-thin overflow-auto max-h-64">
          <table className="w-full text-xs">
            <thead className="sticky top-0 bg-card">
              <tr className="text-muted-foreground border-b border-border">
                <th className="text-left py-2.5 px-5 font-medium uppercase tracking-wider text-[10px]">
                  {t("trafficMonitor.time")}
                </th>
                <th className="text-left py-2.5 px-3 font-medium uppercase tracking-wider text-[10px]">
                  {t("trafficMonitor.target")}
                </th>
                <th className="text-left py-2.5 px-3 font-medium uppercase tracking-wider text-[10px]">
                  {t("trafficMonitor.type")}
                </th>
                <th className="text-left py-2.5 px-3 font-medium uppercase tracking-wider text-[10px]">
                  {t("trafficMonitor.size")}
                </th>
                <th className="text-left py-2.5 px-5 font-medium uppercase tracking-wider text-[10px]">
                  {t("trafficMonitor.duration")}
                </th>
              </tr>
            </thead>
            <tbody>
              {logs.length === 0 ? (
                <tr>
                  <td
                    colSpan={5}
                    className="py-12 text-center text-muted-foreground"
                  >
                    {t("trafficMonitor.noData")}
                  </td>
                </tr>
              ) : (
                logs.map((log, i) => (
                  <tr
                    key={i}
                    className="border-b border-border/50 hover:bg-muted/40 transition-colors"
                  >
                    <td className="py-2.5 px-5 tabular">{log.time}</td>
                    <td className="py-2.5 px-3">{log.target}</td>
                    <td className="py-2.5 px-3">
                      <span className="inline-flex items-center px-2 py-0.5 rounded-md bg-muted text-[10px] font-medium">
                        {log.type}
                      </span>
                    </td>
                    <td className="py-2.5 px-3 tabular">{log.size}</td>
                    <td className="py-2.5 px-5 tabular">{log.duration}</td>
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

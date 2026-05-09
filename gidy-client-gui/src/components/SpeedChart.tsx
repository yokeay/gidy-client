import { useMemo } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
} from "recharts";
import { useTranslation } from "react-i18next";

interface SpeedChartProps {
  data: { time: number; up: number; down: number }[];
}

export default function SpeedChart({ data }: SpeedChartProps) {
  const { t } = useTranslation();

  const chartData = useMemo(() => {
    if (data.length === 0) {
      const now = Date.now();
      return Array.from({ length: 30 }, (_, i) => ({
        time: now - (29 - i) * 1000,
        up: 0,
        down: 0,
      }));
    }
    return data;
  }, [data]);

  return (
    <div className="w-full h-48 bg-card rounded-xl border border-border p-4">
      <div className="flex items-center gap-6 mb-2">
        <div className="flex items-center gap-2 text-xs">
          <span className="w-3 h-3 rounded-full bg-chart-up" />
          <span className="text-muted-foreground">{t("trafficMonitor.upload")}</span>
        </div>
        <div className="flex items-center gap-2 text-xs">
          <span className="w-3 h-3 rounded-full bg-chart-down" />
          <span className="text-muted-foreground">{t("trafficMonitor.download")}</span>
        </div>
      </div>
      <ResponsiveContainer width="100%" height="85%">
        <AreaChart data={chartData}>
          <defs>
            <linearGradient id="upGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="var(--chart-up)" stopOpacity={0.3} />
              <stop offset="100%" stopColor="var(--chart-up)" stopOpacity={0} />
            </linearGradient>
            <linearGradient id="downGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="var(--chart-down)" stopOpacity={0.3} />
              <stop offset="100%" stopColor="var(--chart-down)" stopOpacity={0} />
            </linearGradient>
          </defs>
          <XAxis
            dataKey="time"
            tick={false}
            axisLine={false}
            tickLine={false}
          />
          <YAxis
            tick={false}
            axisLine={false}
            tickLine={false}
            width={0}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: "var(--card)",
              border: "1px solid var(--border)",
              borderRadius: "8px",
              fontSize: "12px",
            }}
            labelFormatter={() => ""}
          />
          <Area
            type="monotone"
            dataKey="up"
            stroke="var(--chart-up)"
            fill="url(#upGrad)"
            strokeWidth={2}
            dot={false}
          />
          <Area
            type="monotone"
            dataKey="down"
            stroke="var(--chart-down)"
            fill="url(#downGrad)"
            strokeWidth={2}
            dot={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}

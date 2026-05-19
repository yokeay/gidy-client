import { useMemo } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Tooltip,
  CartesianGrid,
} from "recharts";

interface SpeedChartProps {
  data: { time: number; up: number; down: number }[];
}

export default function SpeedChart({ data }: SpeedChartProps) {
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
    <div className="w-full h-full flex flex-col">
      <div className="flex-1 min-h-0">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart
            data={chartData}
            margin={{ top: 4, right: 4, bottom: 4, left: 0 }}
          >
            <defs>
              <linearGradient id="upGrad" x1="0" y1="0" x2="0" y2="1">
                <stop
                  offset="0%"
                  stopColor="var(--chart-up)"
                  stopOpacity={0.18}
                />
                <stop
                  offset="100%"
                  stopColor="var(--chart-up)"
                  stopOpacity={0}
                />
              </linearGradient>
              <linearGradient id="downGrad" x1="0" y1="0" x2="0" y2="1">
                <stop
                  offset="0%"
                  stopColor="var(--chart-down)"
                  stopOpacity={0.28}
                />
                <stop
                  offset="100%"
                  stopColor="var(--chart-down)"
                  stopOpacity={0}
                />
              </linearGradient>
            </defs>
            <CartesianGrid
              strokeDasharray="2 4"
              stroke="var(--border)"
              vertical={false}
            />
            <XAxis dataKey="time" tick={false} axisLine={false} tickLine={false} />
            <YAxis tick={false} axisLine={false} tickLine={false} width={0} />
            <Tooltip
              contentStyle={{
                backgroundColor: "var(--card)",
                border: "1px solid var(--border)",
                borderRadius: "10px",
                fontSize: "12px",
                fontFamily: "var(--font-mono)",
              }}
              labelFormatter={() => ""}
            />
            <Area
              type="monotone"
              dataKey="down"
              stroke="var(--chart-down)"
              fill="url(#downGrad)"
              strokeWidth={2}
              dot={false}
            />
            <Area
              type="monotone"
              dataKey="up"
              stroke="var(--chart-up)"
              fill="url(#upGrad)"
              strokeWidth={2}
              strokeDasharray="4 3"
              dot={false}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

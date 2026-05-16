import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { formatSpeed } from "../api";

interface SpeedometerProps {
  speedKbps: number;
  maxKbps?: number;
}

export default function Speedometer({
  speedKbps,
  maxKbps = 1000,
}: SpeedometerProps) {
  const { t } = useTranslation();

  const ratio = Math.max(0, Math.min(speedKbps / maxKbps, 1));
  const angle = ratio * 270;
  const startAngle = -225;
  const endAngle = startAngle + angle;

  const cx = 160;
  const cy = 150;
  const r = 110;

  const polarToCartesian = (
    cx: number,
    cy: number,
    r: number,
    angleDeg: number,
  ) => {
    const rad = (angleDeg * Math.PI) / 180;
    return { x: cx + r * Math.cos(rad), y: cy + r * Math.sin(rad) };
  };

  const describeArc = (start: number, end: number) => {
    const s = polarToCartesian(cx, cy, r, start);
    const e = polarToCartesian(cx, cy, r, end);
    const large = end - start > 180 ? 1 : 0;
    return `M ${s.x} ${s.y} A ${r} ${r} 0 ${large} 1 ${e.x} ${e.y}`;
  };

  // Major ticks every 45° (7 ticks across 270°) + minor ticks
  const majorTicks: ReactNode[] = [];
  const minorTicks: ReactNode[] = [];
  for (let deg = -225; deg <= 45; deg += 15) {
    const isMajor = (deg + 225) % 45 === 0;
    const inner = polarToCartesian(cx, cy, r - (isMajor ? 14 : 8), deg);
    const outer = polarToCartesian(cx, cy, r - 2, deg);
    const arr = isMajor ? majorTicks : minorTicks;
    arr.push(
      <line
        key={deg}
        x1={inner.x}
        y1={inner.y}
        x2={outer.x}
        y2={outer.y}
        stroke="currentColor"
        strokeWidth={isMajor ? 1.5 : 1}
        className={
          isMajor ? "text-muted-foreground" : "text-muted-foreground/40"
        }
        strokeLinecap="round"
      />,
    );
  }

  return (
    <div className="flex flex-col items-center w-full">
      <svg viewBox="0 0 320 280" className="w-full max-w-sm h-auto">
        {/* Background arc */}
        <path
          d={describeArc(-225, 45)}
          fill="none"
          stroke="currentColor"
          strokeWidth="8"
          className="text-muted"
          strokeLinecap="round"
        />
        {/* Active arc */}
        {angle > 0 && (
          <path
            d={describeArc(-225, endAngle)}
            fill="none"
            stroke="currentColor"
            strokeWidth="8"
            strokeLinecap="round"
            className="text-foreground"
          />
        )}
        {/* Ticks */}
        {minorTicks}
        {majorTicks}
        {/* Center text */}
        <text
          x={cx}
          y={cy + 8}
          textAnchor="middle"
          className="fill-foreground tabular"
          fontSize="32"
          fontWeight="700"
        >
          {formatSpeed(speedKbps)}
        </text>
        <text
          x={cx}
          y={cy + 32}
          textAnchor="middle"
          className="fill-muted-foreground"
          fontSize="11"
          letterSpacing="0.08em"
        >
          {t("trafficMonitor.realtime").toUpperCase()}
        </text>
      </svg>
    </div>
  );
}

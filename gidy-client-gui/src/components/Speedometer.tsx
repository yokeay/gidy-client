import { useTranslation } from "react-i18next";
import { formatSpeed } from "../api";

interface SpeedometerProps {
  speedKbps: number;
  maxKbps?: number;
}

export default function Speedometer({ speedKbps, maxKbps = 1000 }: SpeedometerProps) {
  const { t } = useTranslation();

  const angle = Math.min((speedKbps / maxKbps) * 270, 270);
  const startAngle = -225;
  const endAngle = startAngle + angle;

  const cx = 160, cy = 140, r = 120;

  const polarToCartesian = (cx: number, cy: number, r: number, angleDeg: number) => {
    const rad = (angleDeg * Math.PI) / 180;
    return { x: cx + r * Math.cos(rad), y: cy + r * Math.sin(rad) };
  };

  const describeArc = (start: number, end: number) => {
    const s = polarToCartesian(cx, cy, r, start);
    const e = polarToCartesian(cx, cy, r, end);
    const large = end - start > 180 ? 1 : 0;
    return `M ${s.x} ${s.y} A ${r} ${r} 0 ${large} 1 ${e.x} ${e.y}`;
  };

  // Ticks every 30 degrees
  const ticks = [];
  for (let deg = -225; deg <= 45; deg += 45) {
    const inner = polarToCartesian(cx, cy, r - 14, deg);
    const outer = polarToCartesian(cx, cy, r - 2, deg);
    ticks.push(
      <line
        key={deg}
        x1={inner.x} y1={inner.y} x2={outer.x} y2={outer.y}
        stroke="currentColor" strokeWidth={2} className="text-muted-foreground/50"
      />
    );
  }

  // Needle
  const needleEnd = polarToCartesian(cx, cy, r - 30, endAngle);
  const needleBase1 = polarToCartesian(cx, cy, 6, endAngle + 90);
  const needleBase2 = polarToCartesian(cx, cy, 6, endAngle - 90);

  return (
    <div className="flex flex-col items-center">
      <svg viewBox="0 0 320 300" className="w-full max-w-xs h-auto">
        {/* Background arc */}
        <path
          d={describeArc(-225, 45)}
          fill="none"
          stroke="currentColor"
          strokeWidth="10"
          className="text-muted-foreground/20"
          strokeLinecap="round"
        />
        {/* Active arc */}
        {angle > 0 && (
          <path
            d={describeArc(-225, endAngle)}
            fill="none"
            stroke="var(--chart-up)"
            strokeWidth="10"
            strokeLinecap="round"
          />
        )}
        {/* Ticks */}
        {ticks}
        {/* Needle */}
        <polygon
          points={`${needleEnd.x},${needleEnd.y} ${needleBase1.x},${needleBase1.y} ${needleBase2.x},${needleBase2.y}`}
          fill="currentColor"
          className="text-foreground"
        />
        <circle cx={cx} cy={cy} r="8" fill="currentColor" className="text-foreground" />
        {/* Center speed text */}
        <text x={cx} y={cy + 40} textAnchor="middle" className="fill-foreground text-3xl font-bold" fontSize="28">
          {formatSpeed(speedKbps)}
        </text>
        <text x={cx} y={cy + 62} textAnchor="middle" className="fill-muted-foreground text-xs" fontSize="12">
          {t("trafficMonitor.realtime")}
        </text>
      </svg>
    </div>
  );
}

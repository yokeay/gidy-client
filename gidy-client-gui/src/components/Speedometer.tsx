import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { formatSpeed } from "../api";

interface SpeedometerProps {
  speedKbps: number;
  maxKbps?: number;
  upKbps?: number;
  downKbps?: number;
}

export default function Speedometer({
  speedKbps,
  maxKbps = 1000,
  upKbps,
  downKbps,
}: SpeedometerProps) {
  const { t } = useTranslation();

  const ratio = Math.max(0, Math.min(speedKbps / maxKbps, 1));
  const angle = ratio * 270;
  const startAngle = -225;
  const endAngle = startAngle + angle;

  const cx = 160;
  const cy = 150;
  const r = 110;
  const rInner = 94; // inner arc for down

  const polarToCartesian = (
    centerX: number,
    centerY: number,
    radius: number,
    angleDeg: number,
  ) => {
    const rad = (angleDeg * Math.PI) / 180;
    return { x: centerX + radius * Math.cos(rad), y: centerY + radius * Math.sin(rad) };
  };

  const describeArc = (radius: number, start: number, end: number) => {
    const s = polarToCartesian(cx, cy, radius, start);
    const e = polarToCartesian(cx, cy, radius, end);
    const large = end - start > 180 ? 1 : 0;
    return `M ${s.x} ${s.y} A ${radius} ${radius} 0 ${large} 1 ${e.x} ${e.y}`;
  };

  // Down arc ratio (inner ring)
  const downRatio = downKbps !== undefined ? Math.max(0, Math.min(downKbps / maxKbps, 1)) : 0;
  const downAngle = downRatio * 270;
  const downEndAngle = startAngle + downAngle;

  // Up arc ratio (outer ring)
  const upRatio = upKbps !== undefined ? Math.max(0, Math.min(upKbps / maxKbps, 1)) : ratio;
  const upAngle = upRatio * 270;
  const upEndAngle = startAngle + upAngle;

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

  const showDualRing = upKbps !== undefined && downKbps !== undefined;

  return (
    <div className="flex flex-col items-center w-full">
      <svg viewBox="0 0 320 280" className="w-full max-w-sm h-auto">
        <defs>
          <linearGradient id="upArcGrad" gradientUnits="userSpaceOnUse" x1="50" y1="40" x2="270" y2="260">
            <stop offset="0%" stopColor="var(--chart-up)" stopOpacity="0.6" />
            <stop offset="100%" stopColor="var(--chart-up)" />
          </linearGradient>
          <linearGradient id="downArcGrad" gradientUnits="userSpaceOnUse" x1="50" y1="40" x2="270" y2="260">
            <stop offset="0%" stopColor="var(--chart-down)" stopOpacity="0.6" />
            <stop offset="100%" stopColor="var(--chart-down)" />
          </linearGradient>
        </defs>

        {/* Background arc (outer) */}
        <path
          d={describeArc(r, -225, 45)}
          fill="none"
          stroke="currentColor"
          strokeWidth="8"
          className="text-muted"
          strokeLinecap="round"
        />

        {showDualRing ? (
          <>
            {/* Inner background arc */}
            <path
              d={describeArc(rInner, -225, 45)}
              fill="none"
              stroke="currentColor"
              strokeWidth="6"
              className="text-muted"
              strokeLinecap="round"
            />
            {/* Upload arc (outer) */}
            {upAngle > 0 && (
              <path
                d={describeArc(r, -225, upEndAngle)}
                fill="none"
                stroke="url(#upArcGrad)"
                strokeWidth="8"
                strokeLinecap="round"
              />
            )}
            {/* Download arc (inner) */}
            {downAngle > 0 && (
              <path
                d={describeArc(rInner, -225, downEndAngle)}
                fill="none"
                stroke="url(#downArcGrad)"
                strokeWidth="6"
                strokeLinecap="round"
              />
            )}
          </>
        ) : (
          /* Single arc mode */
          angle > 0 && (
            <path
              d={describeArc(r, -225, endAngle)}
              fill="none"
              stroke="currentColor"
              strokeWidth="8"
              strokeLinecap="round"
              className="text-foreground"
            />
          )
        )}

        {/* Ticks */}
        {minorTicks}
        {majorTicks}

        {/* Center speed text */}
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

      {/* Legend for dual ring */}
      {showDualRing && (
        <div className="flex items-center gap-5 -mt-2">
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <span className="w-3 h-1.5 rounded-full" style={{ background: "var(--chart-up)" }} />
            <span>{t("trafficMonitor.upload")}</span>
            <span className="tabular font-medium text-foreground ml-1">
              {formatSpeed(upKbps ?? 0)}
            </span>
          </div>
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <span className="w-3 h-1.5 rounded-full" style={{ background: "var(--chart-down)" }} />
            <span>{t("trafficMonitor.download")}</span>
            <span className="tabular font-medium text-foreground ml-1">
              {formatSpeed(downKbps ?? 0)}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}

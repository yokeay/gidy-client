import { useState, useEffect, useRef } from "react";
import { getStats, getStatus, formatBytes, formatSpeed } from "../api";

interface LogEntry {
  id: number;
  ts: string;
  level: "INFO" | "WARN" | "ERROR" | "DEBUG";
  message: string;
}

let logIdSeq = 0;

function makeLog(level: LogEntry["level"], message: string): LogEntry {
  const now = new Date();
  const ts = now.toTimeString().slice(0, 8) + "." + String(now.getMilliseconds()).padStart(3, "0");
  return { id: ++logIdSeq, ts, level, message };
}

const LEVEL_COLOR: Record<LogEntry["level"], string> = {
  INFO:  "var(--accent-green)",
  WARN:  "#f39c12",
  ERROR: "#e74c3c",
  DEBUG: "var(--muted-fg)",
};

export default function LogViewer() {
  const [logs, setLogs] = useState<LogEntry[]>(() => [
    makeLog("INFO", "gidy-client 日志系统初始化"),
    makeLog("INFO", "加载配置文件成功"),
  ]);
  const [filter, setFilter] = useState<"ALL" | LogEntry["level"]>("ALL");
  const [autoscroll, setAutoscroll] = useState(true);
  const bottomRef = useRef<HTMLDivElement>(null);
  const prevRunning = useRef(false);

  useEffect(() => {
    const push = (entry: LogEntry) => {
      setLogs(prev => {
        const next = [...prev, entry];
        return next.length > 500 ? next.slice(-500) : next;
      });
    };

    const poll = async () => {
      try {
        const [st, stats] = await Promise.all([getStatus(), getStats()]);

        if (st.running && !prevRunning.current) {
          push(makeLog("INFO", "代理服务已启动，开始监听本地端口"));
        }
        if (!st.running && prevRunning.current) {
          push(makeLog("INFO", "代理服务已停止"));
        }
        prevRunning.current = st.running;

        if (st.running) {
          if (stats.speed_up_kbps > 0 || stats.speed_down_kbps > 0) {
            push(makeLog("DEBUG",
              `流量统计 ↑${formatSpeed(stats.speed_up_kbps)}  ↓${formatSpeed(stats.speed_down_kbps)}  ` +
              `总↑${formatBytes(stats.bytes_up)}  总↓${formatBytes(stats.bytes_down)}  ` +
              `连接数 ${stats.active_connections}`
            ));
          }
          if (stats.active_connections > 0) {
            push(makeLog("INFO",
              `活动连接 ${stats.active_connections} 个  运行时长 ${stats.uptime_secs}s`
            ));
          }
        }

        if (st.error) {
          push(makeLog("ERROR", `代理错误：${st.error}`));
        }
      } catch (e) {
        push(makeLog("WARN", `状态轮询失败：${e}`));
      }
    };

    const id = setInterval(poll, 2000);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    if (autoscroll) {
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs, autoscroll]);

  const visible = filter === "ALL" ? logs : logs.filter(l => l.level === filter);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", gap: 14 }}>
      {/* Toolbar */}
      <div style={{ display: "flex", alignItems: "center", gap: 10, flexWrap: "wrap" }}>
        {(["ALL", "INFO", "DEBUG", "WARN", "ERROR"] as const).map(lv => (
          <button
            key={lv}
            onClick={() => setFilter(lv)}
            style={{
              padding: "5px 14px",
              borderRadius: 6,
              fontSize: 12,
              fontWeight: 500,
              cursor: "pointer",
              border: filter === lv ? "1px solid rgba(46,204,113,0.5)" : "1px solid var(--border)",
              background: filter === lv ? "rgba(46,204,113,0.12)" : "var(--card)",
              color: filter === lv ? "var(--accent-green)" : "var(--muted-fg)",
              fontFamily: "var(--font-mono)",
              transition: "all 0.15s",
            }}
          >
            {lv}
          </button>
        ))}
        <div style={{ marginLeft: "auto", display: "flex", alignItems: "center", gap: 10 }}>
          <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 12, color: "var(--muted-fg)", cursor: "pointer", userSelect: "none" }}>
            <input
              type="checkbox"
              checked={autoscroll}
              onChange={e => setAutoscroll(e.target.checked)}
              style={{ accentColor: "var(--accent-green)" }}
            />
            自动滚动
          </label>
          <button
            onClick={() => setLogs([])}
            style={{
              padding: "5px 14px",
              borderRadius: 6,
              fontSize: 12,
              background: "var(--card)",
              border: "1px solid var(--border)",
              color: "var(--muted-fg)",
              cursor: "pointer",
              fontFamily: "var(--font-ui)",
              transition: "all 0.15s",
            }}
            onMouseEnter={e => {
              (e.currentTarget as HTMLElement).style.borderColor = "rgba(231,76,60,0.4)";
              (e.currentTarget as HTMLElement).style.color = "#e74c3c";
            }}
            onMouseLeave={e => {
              (e.currentTarget as HTMLElement).style.borderColor = "var(--border)";
              (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
            }}
          >
            清空
          </button>
        </div>
      </div>

      {/* Log body */}
      <div
        className="scroll-thin"
        style={{
          flex: 1,
          overflowY: "auto",
          background: "#0a0c0f",
          border: "1px solid var(--border)",
          borderRadius: 10,
          padding: "14px 0",
          fontFamily: "var(--font-mono)",
          fontSize: 12,
          lineHeight: 1.7,
          minHeight: 0,
        }}
      >
        {visible.length === 0 ? (
          <div style={{ textAlign: "center", color: "var(--muted-fg)", paddingTop: 60 }}>暂无日志</div>
        ) : (
          visible.map(log => (
            <div
              key={log.id}
              style={{
                display: "flex",
                gap: 12,
                padding: "2px 18px",
                transition: "background 0.1s",
              }}
              onMouseEnter={e => ((e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.02)")}
              onMouseLeave={e => ((e.currentTarget as HTMLElement).style.background = "transparent")}
            >
              <span style={{ color: "#4a5268", flexShrink: 0 }}>{log.ts}</span>
              <span
                style={{
                  color: LEVEL_COLOR[log.level],
                  fontWeight: 600,
                  flexShrink: 0,
                  width: 42,
                }}
              >
                {log.level}
              </span>
              <span style={{ color: "#c8ccd8", wordBreak: "break-all" }}>{log.message}</span>
            </div>
          ))
        )}
        <div ref={bottomRef} />
      </div>

      {/* Footer */}
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: 11, color: "var(--text-muted, #4a5268)" }}>
        <span>共 {visible.length} 条{filter !== "ALL" ? `（已筛选 ${filter}）` : ""}</span>
        <span>实时刷新 · 2s</span>
      </div>
    </div>
  );
}

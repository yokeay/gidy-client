import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { getStatus, getStats, formatUptime } from "../api";

interface SidebarProps {
  theme: "light" | "dark";
  onToggleTheme: () => void;
  onToggleLang: () => void;
  themeColor: string;
  currentLang: "zh" | "en";
  currentPath: string;
}

const navTop = [
  { path: "/dashboard", icon: "⚙", labelZh: "系统配置", labelEn: "Config" },
  { path: "/traffic-monitor", icon: "⌥", labelZh: "流量监测", labelEn: "Traffic" },
];

const navBottom = [
  { path: "/about", icon: "☰", labelZh: "日志", labelEn: "Logs" },
  { path: "/user-settings", icon: "⚙", labelZh: "设置", labelEn: "Settings" },
];

export default function Sidebar({ onToggleLang, currentLang, currentPath }: SidebarProps) {
  const navigate = useNavigate();
  const [running, setRunning] = useState(false);
  const [uptime, setUptime] = useState(0);

  useEffect(() => {
    const poll = async () => {
      try {
        const [st, stats] = await Promise.all([getStatus(), getStats()]);
        setRunning(st.running);
        setUptime(stats.uptime_secs);
      } catch {}
    };
    poll();
    const id = setInterval(poll, 2000);
    return () => clearInterval(id);
  }, []);

  const isActive = (path: string) => {
    if (path === "/dashboard") {
      return currentPath === "/" || currentPath === "/dashboard";
    }
    return currentPath === path;
  };

  const NavItem = ({ path, icon, labelZh, labelEn }: { path: string; icon: string; labelZh: string; labelEn: string }) => {
    const active = isActive(path);
    const label = currentLang === "zh" ? labelZh : labelEn;
    return (
      <div
        onClick={() => navigate(path)}
        className="flex items-center gap-[10px] rounded-[8px] cursor-pointer mb-[2px]"
        style={{
          padding: "10px 12px",
          fontSize: 14,
          fontWeight: active ? 500 : 400,
          color: active ? "var(--accent-green)" : "var(--muted-fg)",
          background: active
            ? "linear-gradient(135deg, rgba(46,204,113,0.2), rgba(46,204,113,0.08))"
            : "transparent",
          border: active ? "1px solid rgba(46,204,113,0.25)" : "1px solid transparent",
          transition: "all 0.15s",
          userSelect: "none",
        }}
        onMouseEnter={e => {
          if (!active) {
            (e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.04)";
            (e.currentTarget as HTMLElement).style.color = "var(--fg)";
          }
        }}
        onMouseLeave={e => {
          if (!active) {
            (e.currentTarget as HTMLElement).style.background = "transparent";
            (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
          }
        }}
      >
        <span style={{ fontSize: 16 }}>{icon}</span>
        <span>{label}</span>
      </div>
    );
  };

  return (
    <aside
      className="flex flex-col shrink-0"
      style={{
        width: "var(--sidebar-width)",
        background: "var(--bg-sidebar)",
        borderRight: "1px solid var(--border)",
        padding: "16px 10px",
      }}
    >
      {/* Top nav */}
      <div>
        {navTop.map(item => <NavItem key={item.path} {...item} />)}
      </div>

      {/* Bottom section */}
      <div
        className="mt-auto"
        style={{ paddingTop: 12, borderTop: "1px solid var(--border)" }}
      >
        {navBottom.map(item => <NavItem key={item.path} {...item} />)}

        {/* Lang toggle */}
        <div
          onClick={onToggleLang}
          className="flex items-center gap-[10px] rounded-[8px] cursor-pointer mb-[2px]"
          style={{
            padding: "10px 12px",
            fontSize: 14,
            color: "var(--muted-fg)",
            transition: "all 0.15s",
            userSelect: "none",
          }}
          onMouseEnter={e => {
            (e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.04)";
            (e.currentTarget as HTMLElement).style.color = "var(--fg)";
          }}
          onMouseLeave={e => {
            (e.currentTarget as HTMLElement).style.background = "transparent";
            (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
          }}
        >
          <span style={{ fontSize: 16 }}>🌐</span>
          <span>{currentLang === "zh" ? "English" : "中文"}</span>
        </div>

        {/* Connection status indicator */}
        <div style={{ marginTop: 16, padding: "0 12px" }}>
          <div className="flex items-center gap-[7px]" style={{ marginBottom: 4 }}>
            <div
              className="pulse-dot rounded-full"
              style={{
                width: 8,
                height: 8,
                background: running ? "var(--accent-green)" : "var(--muted-fg)",
                boxShadow: running ? "0 0 8px var(--accent-green)" : "none",
              }}
            />
            <span
              style={{
                fontSize: 13,
                color: running ? "var(--accent-green)" : "var(--muted-fg)",
                fontWeight: 500,
              }}
            >
              {running
                ? (currentLang === "zh" ? "连接正常" : "Connected")
                : (currentLang === "zh" ? "未连接" : "Disconnected")}
            </span>
          </div>
          <div
            className="tabular"
            style={{ fontSize: 11, color: "var(--text-muted, #4a5268)" }}
          >
            {currentLang === "zh" ? "运行时间" : "Uptime"} {formatUptime(uptime)}
          </div>
        </div>
      </div>
    </aside>
  );
}

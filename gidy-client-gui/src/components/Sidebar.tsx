import React, { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { LayoutDashboard, Settings2, BarChart3, ScrollText, Settings } from "lucide-react";
import { getStatus, getStats, formatUptime } from "../api";

interface SidebarProps {
  currentPath: string;
  currentLang?: "zh" | "en";
}

export default function Sidebar({ currentPath }: SidebarProps) {
  const navigate = useNavigate();
  const { t } = useTranslation();
  const [running, setRunning] = useState(false);
  const [uptime, setUptime] = useState(0);

  const NAV_ITEMS = [
    { path: "/dashboard", icon: <LayoutDashboard size={16} />, label: t("nav.overview") },
    { path: "/proxy-config", icon: <Settings2 size={16} />, label: t("nav.config") },
    { path: "/traffic-monitor", icon: <BarChart3 size={16} />, label: t("nav.trafficMonitor") },
    { path: "/logs", icon: <ScrollText size={16} />, label: t("nav.logs") },
  ];

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

  const isActive = (path: string) =>
    path === "/dashboard"
      ? currentPath === "/" || currentPath === "/dashboard"
      : currentPath === path;

  const NavItem = ({ path, icon, label }: { path: string; icon: React.ReactNode; label: string }) => {

    const active = isActive(path);
    return (
      <div
        onClick={() => navigate(path)}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
          padding: "10px 12px",
          borderRadius: 8,
          marginBottom: 2,
          fontSize: 14,
          fontWeight: active ? 500 : 400,
          color: active ? "var(--accent-green)" : "var(--muted-fg)",
          background: active
            ? "linear-gradient(135deg,rgba(46,204,113,0.2),rgba(46,204,113,0.08))"
            : "transparent",
          border: active ? "1px solid rgba(46,204,113,0.25)" : "1px solid transparent",
          cursor: "pointer",
          userSelect: "none",
          transition: "all 0.15s",
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
        <span style={{ display: "flex", alignItems: "center" }}>{icon}</span>
        <span>{label}</span>
      </div>
    );
  };

  const settingsActive = currentPath === "/user-settings";

  return (
    <aside
      style={{
        width: "var(--sidebar-width)",
        background: "var(--bg-sidebar)",
        borderRight: "1px solid var(--border)",
        display: "flex",
        flexDirection: "column",
        flexShrink: 0,
        padding: "16px 10px 0",
      }}
    >
      {/* Top nav — 3 items */}
      <div style={{ flex: 1 }}>
        {NAV_ITEMS.map(item => <NavItem key={item.path} {...item} />)}
      </div>

      {/* Bottom area */}
      <div style={{ paddingBottom: 16 }}>
        {/* Settings button */}
        <div
          onClick={() => navigate("/user-settings")}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 10,
            padding: "10px 12px",
            borderRadius: 8,
            marginBottom: 2,
            fontSize: 14,
            fontWeight: settingsActive ? 500 : 400,
            color: settingsActive ? "var(--accent-green)" : "var(--muted-fg)",
            background: settingsActive
              ? "linear-gradient(135deg,rgba(46,204,113,0.2),rgba(46,204,113,0.08))"
              : "transparent",
            border: settingsActive ? "1px solid rgba(46,204,113,0.25)" : "1px solid transparent",
            cursor: "pointer",
            userSelect: "none",
            transition: "all 0.15s",
          }}
          onMouseEnter={e => {
            if (!settingsActive) {
              (e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.04)";
              (e.currentTarget as HTMLElement).style.color = "var(--fg)";
            }
          }}
          onMouseLeave={e => {
            if (!settingsActive) {
              (e.currentTarget as HTMLElement).style.background = "transparent";
              (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
            }
          }}
        >
          <span style={{ display: "flex", alignItems: "center" }}><Settings size={16} /></span>
          <span>{t("sidebar.settings")}</span>
        </div>

        {/* Divider */}
        <div style={{ height: 1, background: "var(--border)", margin: "10px 4px 12px" }} />

        {/* Connection status */}
        <div style={{ padding: "0 12px", textAlign: "center" }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: 7, marginBottom: 4 }}>
            <div
              className={running ? "pulse-dot" : ""}
              style={{
                width: 8,
                height: 8,
                borderRadius: "50%",
                flexShrink: 0,
                background: running ? "var(--accent-green)" : "var(--muted-fg)",
                boxShadow: running ? "0 0 8px var(--accent-green)" : "none",
              }}
            />
            <span
              style={{
                fontSize: 13,
                fontWeight: 500,
                color: running ? "var(--accent-green)" : "var(--muted-fg)",
              }}
            >
              {running ? t("sidebar.connected") : t("sidebar.disconnected")}
            </span>
          </div>
          <div
            className="tabular"
            style={{ fontSize: 11, color: "var(--text-muted,#4a5268)", textAlign: "center" }}
          >
            {t("sidebar.uptime")} {formatUptime(uptime)}
          </div>
        </div>
      </div>
    </aside>
  );
}

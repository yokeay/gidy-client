import { useState, useEffect } from "react";
import { Routes, Route, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Sidebar from "./components/Sidebar";
import Dashboard from "./pages/Dashboard";
import SystemConfig from "./pages/SystemConfig";
import TrafficMonitor from "./pages/TrafficMonitor";
import UserSettings from "./pages/UserSettings";
import About from "./pages/About";

function Titlebar() {
  const appWindow = getCurrentWindow();

  return (
    <div
      className="titlebar-drag flex items-center shrink-0 px-[18px] gap-[10px]"
      style={{
        height: "var(--titlebar-height)",
        background: "var(--bg-secondary, #13161c)",
        borderBottom: "1px solid var(--border)",
      }}
    >
      <div
        className="titlebar-no-drag flex items-center justify-center shrink-0 rounded-[6px] font-bold text-black"
        style={{
          width: 28, height: 28,
          fontSize: 15,
          background: "var(--accent-green)",
          boxShadow: "0 0 12px rgba(46,204,113,0.4)",
          letterSpacing: "-1px",
        }}
      >
        G
      </div>
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: 13,
          fontWeight: 500,
          color: "var(--fg)",
          letterSpacing: "0.02em",
        }}
      >
        gidy-client
      </span>

      <div className="titlebar-no-drag ml-auto flex gap-1">
        {[
          { label: "─", action: () => appWindow.minimize(), danger: false },
          { label: "□", action: () => appWindow.toggleMaximize(), danger: false },
          { label: "✕", action: () => appWindow.close(), danger: true },
        ].map(btn => (
          <button
            key={btn.label}
            onClick={btn.action}
            style={{
              width: 32, height: 32,
              borderRadius: 6,
              border: "none",
              background: "transparent",
              color: "var(--muted-fg)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 14,
              transition: "background 0.15s, color 0.15s",
            }}
            onMouseEnter={e => {
              if (btn.danger) {
                (e.currentTarget as HTMLElement).style.background = "#e74c3c";
                (e.currentTarget as HTMLElement).style.color = "#fff";
              } else {
                (e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.08)";
                (e.currentTarget as HTMLElement).style.color = "var(--fg)";
              }
            }}
            onMouseLeave={e => {
              (e.currentTarget as HTMLElement).style.background = "transparent";
              (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
            }}
          >
            {btn.label}
          </button>
        ))}
      </div>
    </div>
  );
}

function PageTitle({ title }: { title: string }) {
  return (
    <div style={{ marginBottom: 28 }}>
      <div style={{ fontSize: 22, fontWeight: 600, color: "var(--fg)", marginBottom: 6, letterSpacing: "-0.02em" }}>
        {title}
      </div>
      <div
        style={{
          width: 28, height: 2.5,
          background: "var(--accent-green)",
          borderRadius: 2,
          boxShadow: "0 0 8px var(--accent-green)",
        }}
      />
    </div>
  );
}

export default function App() {
  const { t, i18n } = useTranslation();
  const [theme] = useState<"dark">("dark");
  const location = useLocation();

  useEffect(() => {
    document.documentElement.classList.add("dark");
  }, []);

  const toggleLang = () => {
    i18n.changeLanguage(i18n.language === "zh" ? "en" : "zh");
  };

  const getPageTitle = () => {
    const p = location.pathname;
    if (p === "/" || p === "/dashboard") return t("nav.systemConfig");
    if (p === "/system-config") return t("nav.systemConfig");
    if (p === "/traffic-monitor") return t("nav.trafficMonitor");
    if (p === "/user-settings") return t("nav.userSettings");
    if (p === "/about") return t("nav.about");
    return "";
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100vh",
        width: "100vw",
        overflow: "hidden",
        background: "var(--bg)",
      }}
    >
      <Titlebar />
      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        <Sidebar
          theme={theme}
          onToggleTheme={() => {}}
          onToggleLang={toggleLang}
          themeColor="green"
          currentLang={i18n.language as "zh" | "en"}
          currentPath={location.pathname}
        />
        <main
          className="scroll-thin"
          style={{
            flex: 1,
            overflowY: "auto",
            padding: "32px 36px",
            background: "var(--bg)",
          }}
        >
          <PageTitle title={getPageTitle()} />
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/system-config" element={<SystemConfig />} />
            <Route path="/traffic-monitor" element={<TrafficMonitor />} />
            <Route
              path="/user-settings"
              element={
                <UserSettings
                  theme={theme}
                  themeColor="green"
                  onThemeChange={() => {}}
                  onThemeColorChange={() => {}}
                />
              }
            />
            <Route path="/about" element={<About />} />
          </Routes>
        </main>
      </div>
    </div>
  );
}

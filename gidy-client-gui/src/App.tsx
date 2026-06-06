import { useState, useEffect, useCallback } from "react";
import { Routes, Route, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import Sidebar from "./components/Sidebar";
import Dashboard from "./pages/Dashboard";
import SystemConfig from "./pages/SystemConfig";
import ProxyConfig from "./pages/ProxyConfig";
import TrafficMonitor from "./pages/TrafficMonitor";
import UserSettings from "./pages/UserSettings";
import LogViewer from "./pages/LogViewer";

const APP_VERSION = "v0.3.0";

// ── Close Confirm Dialog ──────────────────────────────────────────────────────
function CloseDialog({
  onConfirm, onMinimize, onDismiss,
}: {
  onConfirm: () => void;
  onMinimize: () => void;
  onDismiss: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div
      style={{
        position: "fixed", inset: 0, zIndex: 9999,
        background: "rgba(0,0,0,0.6)", backdropFilter: "blur(6px)",
        display: "flex", alignItems: "center", justifyContent: "center",
      }}
      onClick={onDismiss}
    >
      <div
        onClick={e => e.stopPropagation()}
        style={{
          background: "var(--bg-secondary,#13161c)",
          border: "1px solid var(--border)",
          borderRadius: 14,
          padding: "28px 32px",
          width: 340,
          boxShadow: "0 24px 80px rgba(0,0,0,0.6)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 12 }}>
          <div style={{
            width: 32, height: 32, borderRadius: 8,
            background: "rgba(231,76,60,0.15)",
            border: "1px solid rgba(231,76,60,0.3)",
            display: "flex", alignItems: "center", justifyContent: "center",
            fontSize: 16,
          }}>⚡</div>
          <span style={{ fontSize: 16, fontWeight: 600, color: "var(--fg)" }}>{t("closeDialog.title")}</span>
        </div>
        <p style={{ fontSize: 13, color: "var(--muted-fg)", lineHeight: 1.7, marginBottom: 24 }}>
          {t("closeDialog.desc")}
        </p>
        <div style={{ display: "flex", gap: 10 }}>
          <button
            onClick={onMinimize}
            style={{
              flex: 1, padding: "9px 0", borderRadius: 8,
              background: "var(--card)", border: "1px solid var(--border)",
              color: "var(--muted-fg)", fontSize: 13, fontWeight: 500,
              cursor: "pointer", fontFamily: "var(--font-ui)", transition: "all 0.15s",
            }}
            onMouseEnter={e => {
              (e.currentTarget as HTMLElement).style.borderColor = "rgba(255,255,255,0.15)";
              (e.currentTarget as HTMLElement).style.color = "var(--fg)";
            }}
            onMouseLeave={e => {
              (e.currentTarget as HTMLElement).style.borderColor = "var(--border)";
              (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
            }}
          >
            {t("closeDialog.minimize")}
          </button>
          <button
            onClick={onConfirm}
            style={{
              flex: 1, padding: "9px 0", borderRadius: 8,
              background: "#e74c3c", border: "none",
              color: "#fff", fontSize: 13, fontWeight: 600,
              cursor: "pointer", fontFamily: "var(--font-ui)",
              boxShadow: "0 0 12px rgba(231,76,60,0.3)",
              transition: "all 0.15s",
            }}
            onMouseEnter={e => ((e.currentTarget as HTMLElement).style.boxShadow = "0 0 20px rgba(231,76,60,0.5)")}
            onMouseLeave={e => ((e.currentTarget as HTMLElement).style.boxShadow = "0 0 12px rgba(231,76,60,0.3)")}
          >
            {t("closeDialog.quit")}
          </button>
        </div>
      </div>
    </div>
  );
}

// ── Titlebar ──────────────────────────────────────────────────────────────────
function Titlebar({ onClose }: { onClose: () => void }) {
  const appWindow = getCurrentWindow();

  return (
    <div
      className="titlebar-drag"
      style={{
        height: "var(--titlebar-height)",
        background: "var(--bg-secondary,#13161c)",
        borderBottom: "1px solid var(--border)",
        display: "flex",
        alignItems: "center",
        padding: "0 16px",
        gap: 10,
        flexShrink: 0,
      }}
    >
      {/* Logo + title */}
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <div
          className="titlebar-no-drag"
          style={{
            width: 26, height: 26, borderRadius: 6,
            background: "var(--accent-green)",
            boxShadow: "0 0 10px rgba(46,204,113,0.4)",
            display: "flex", alignItems: "center", justifyContent: "center",
            fontSize: 13, fontWeight: 700, color: "#000",
            letterSpacing: "-0.5px", flexShrink: 0,
          }}
        >
          G
        </div>
        <div style={{ display: "flex", alignItems: "baseline", gap: 6 }}>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: 13, fontWeight: 600,
            color: "var(--fg)", letterSpacing: "0.02em",
          }}>
            Gidy-Client
          </span>
          <span style={{
            fontFamily: "var(--font-mono)", fontSize: 10,
            color: "var(--muted-fg)", letterSpacing: "0.04em",
          }}>
            {APP_VERSION}
          </span>
        </div>
      </div>

      {/* Window controls — right side, only minimize + close */}
      <div className="titlebar-no-drag" style={{ marginLeft: "auto", display: "flex", gap: 4 }}>
        <button
          onClick={() => appWindow.minimize()}
          title="最小化"
          style={{
            width: 32, height: 28, borderRadius: 6,
            border: "none", background: "transparent",
            color: "var(--muted-fg)", cursor: "pointer",
            display: "flex", alignItems: "center", justifyContent: "center",
            fontSize: 12, transition: "background 0.15s, color 0.15s",
          }}
          onMouseEnter={e => {
            (e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.08)";
            (e.currentTarget as HTMLElement).style.color = "var(--fg)";
          }}
          onMouseLeave={e => {
            (e.currentTarget as HTMLElement).style.background = "transparent";
            (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
          }}
        >
          ─
        </button>
        <button
          onClick={onClose}
          title="关闭"
          style={{
            width: 32, height: 28, borderRadius: 6,
            border: "none", background: "transparent",
            color: "var(--muted-fg)", cursor: "pointer",
            display: "flex", alignItems: "center", justifyContent: "center",
            fontSize: 12, transition: "background 0.15s, color 0.15s",
          }}
          onMouseEnter={e => {
            (e.currentTarget as HTMLElement).style.background = "#e74c3c";
            (e.currentTarget as HTMLElement).style.color = "#fff";
          }}
          onMouseLeave={e => {
            (e.currentTarget as HTMLElement).style.background = "transparent";
            (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
          }}
        >
          ✕
        </button>
      </div>
    </div>
  );
}

// ── Page title ────────────────────────────────────────────────────────────────
function PageTitle({ title }: { title: string }) {
  return (
    <div style={{ marginBottom: 28 }}>
      <div style={{ fontSize: 20, fontWeight: 600, color: "var(--fg)", marginBottom: 6, letterSpacing: "-0.02em", display: "inline-block" }}>
        {title}
      </div>
      <div style={{
        height: 2.5, background: "var(--accent-green)",
        borderRadius: 2, boxShadow: "0 0 8px var(--accent-green)",
        width: "fit-content",
      }} />
    </div>
  );
}

// ── App ───────────────────────────────────────────────────────────────────────
export default function App() {
  const { i18n, t } = useTranslation();
  const location = useLocation();
  const [showCloseDialog, setShowCloseDialog] = useState(false);

  useEffect(() => {
    document.documentElement.classList.add("dark");
  }, []);

  // Listen for tray quit event from Rust
  useEffect(() => {
    const unlisten = listen("tray-quit", () => {
      setShowCloseDialog(true);
    });
    return () => { unlisten.then(fn => fn()); };
  }, []);

  const handleClose = useCallback(() => {
    setShowCloseDialog(true);
  }, []);

  const handleConfirmQuit = useCallback(async () => {
    setShowCloseDialog(false);
    // destroy() forcefully closes the window and exits the process
    const appWindow = getCurrentWindow();
    await appWindow.destroy();
  }, []);

  const handleMinimizeToTray = useCallback(async () => {
    setShowCloseDialog(false);
    const appWindow = getCurrentWindow();
    await appWindow.hide();
  }, []);

  const handleDismissDialog = useCallback(() => {
    setShowCloseDialog(false);
  }, []);

  const getPageTitle = () => {
    const p = location.pathname;
    if (p === "/" || p === "/dashboard") return t("nav.overview");
    if (p === "/proxy-config") return t("nav.config");
    if (p === "/traffic-monitor") return t("nav.trafficMonitor");
    if (p === "/logs") return t("nav.logs");
    if (p === "/user-settings") return t("nav.settings");
    return "";
  };

  return (
    <div style={{
      display: "flex", flexDirection: "column",
      height: "100vh", width: "100vw",
      overflow: "hidden", background: "var(--bg)",
    }}>
      <Titlebar onClose={handleClose} />

      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        <Sidebar
          currentPath={location.pathname}
          currentLang={i18n.language as "zh" | "en"}
        />
        <main
          className="scroll-thin"
          style={{
            flex: 1, overflowY: "auto",
            padding: "28px 32px 48px",
            background: "var(--bg)",
            display: "flex",
            flexDirection: "column",
          }}
        >
          <PageTitle title={getPageTitle()} />
          <div style={{ flex: 1, minHeight: 0 }}>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/dashboard" element={<Dashboard />} />
              <Route path="/system-config" element={<SystemConfig />} />
              <Route path="/proxy-config" element={<ProxyConfig />} />
              <Route path="/traffic-monitor" element={<TrafficMonitor />} />
              <Route path="/logs" element={<LogViewer />} />
              <Route
                path="/user-settings"
                element={
                  <UserSettings
                    theme="dark"
                    themeColor="green"
                    onThemeChange={() => {}}
                    onThemeColorChange={() => {}}
                  />
                }
              />
            </Routes>
          </div>
        </main>
      </div>

      {showCloseDialog && (
        <CloseDialog
          onConfirm={handleConfirmQuit}
          onMinimize={handleMinimizeToTray}
          onDismiss={handleDismissDialog}
        />
      )}
    </div>
  );
}

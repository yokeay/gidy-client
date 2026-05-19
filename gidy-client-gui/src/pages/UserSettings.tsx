import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { getConfig, updateConfig, GuiConfig } from "../api";

interface UserSettingsProps {
  theme: "light" | "dark";
  themeColor: string;
  onThemeChange: (t: "light" | "dark") => void;
  onThemeColorChange: (c: string) => void;
}

const APP_VERSION = "v0.2.8";

function SettingsItem({
  num,
  title,
  sub,
  right,
}: {
  num: string;
  title: string;
  sub?: React.ReactNode;
  right?: React.ReactNode;
}) {
  return (
    <div
      style={{
        background: "var(--card)",
        border: "1px solid var(--border)",
        borderRadius: 10,
        padding: "20px 24px",
        display: "flex",
        alignItems: "center",
        gap: 18,
        transition: "border-color 0.15s",
      }}
      onMouseEnter={e =>
        ((e.currentTarget as HTMLElement).style.borderColor = "rgba(255,255,255,0.1)")
      }
      onMouseLeave={e =>
        ((e.currentTarget as HTMLElement).style.borderColor = "var(--border)")
      }
    >
      {/* Number badge */}
      <div
        style={{
          width: 28,
          height: 28,
          borderRadius: "50%",
          border: "1.5px solid var(--accent-green)",
          color: "var(--accent-green)",
          fontFamily: "var(--font-mono)",
          fontSize: 13,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexShrink: 0,
          fontWeight: 600,
        }}
      >
        {num}
      </div>
      {/* Info */}
      <div style={{ flex: 1 }}>
        <div style={{ fontSize: 14, fontWeight: 500, color: "var(--fg)", marginBottom: sub ? 3 : 0 }}>
          {title}
        </div>
        {sub && (
          <div style={{ fontSize: 12, color: "var(--text-muted, #4a5268)", fontFamily: "var(--font-mono)" }}>
            {sub}
          </div>
        )}
      </div>
      {/* Right action */}
      {right}
    </div>
  );
}

function GreenBtn({ onClick, children }: { onClick?: () => void; children: React.ReactNode }) {
  return (
    <button
      onClick={onClick}
      style={{
        background: "var(--accent-green)",
        color: "#000",
        border: "none",
        borderRadius: 8,
        padding: "9px 18px",
        fontSize: 13,
        fontWeight: 600,
        cursor: "pointer",
        whiteSpace: "nowrap",
        boxShadow: "0 0 12px rgba(46,204,113,0.25)",
        transition: "all 0.15s",
        flexShrink: 0,
        fontFamily: "var(--font-ui)",
      }}
      onMouseEnter={e =>
        ((e.currentTarget as HTMLElement).style.boxShadow = "0 0 20px rgba(46,204,113,0.4)")
      }
      onMouseLeave={e =>
        ((e.currentTarget as HTMLElement).style.boxShadow = "0 0 12px rgba(46,204,113,0.25)")
      }
    >
      {children}
    </button>
  );
}

function ToggleSwitch({ value, onChange }: { value: boolean; onChange: () => void }) {
  return (
    <button
      onClick={onChange}
      style={{
        position: "relative",
        width: 48,
        height: 24,
        borderRadius: 12,
        background: value ? "var(--accent-green)" : "rgba(255,255,255,0.12)",
        border: "none",
        cursor: "pointer",
        transition: "background 0.2s",
        flexShrink: 0,
      }}
    >
      <span
        style={{
          position: "absolute",
          top: 2,
          left: value ? 26 : 2,
          width: 20,
          height: 20,
          borderRadius: "50%",
          background: "#fff",
          boxShadow: "0 1px 4px rgba(0,0,0,0.3)",
          transition: "left 0.2s",
          display: "block",
        }}
      />
    </button>
  );
}

export default function UserSettings({ theme, themeColor, onThemeChange, onThemeColorChange }: UserSettingsProps) {
  const { t, i18n } = useTranslation();
  const [config, setConfig] = useState<GuiConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [kernelPath, setKernelPath] = useState("/usr/local/bin/gidy-core");

  useEffect(() => {
    getConfig()
      .then(c => {
        setConfig(c);
        onThemeChange(c.theme as "light" | "dark");
        onThemeColorChange(c.theme_color);
      })
      .catch(() => {});
  }, []);

  const handleToggle = (key: keyof GuiConfig) => {
    if (!config) return;
    setConfig({ ...config, [key]: !config[key] });
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      const updated = { ...config, theme, theme_color: themeColor };
      await updateConfig(updated);
      setConfig(updated);
      setMessage(t("userSettings.saved"));
      setTimeout(() => setMessage(null), 2000);
    } catch {
      setMessage(t("common.error"));
    }
    setSaving(false);
  };

  if (!config) return null;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 14, maxWidth: 680, width: "100%" }}>

      {/* ① Version check */}
      <SettingsItem
        num="①"
        title={t("userSettings.updateCheck")}
        sub={`${t("userSettings.currentVersion")}：${APP_VERSION}`}
        right={<GreenBtn onClick={() => {}}>{t("userSettings.checkUpdate")}</GreenBtn>}
      />

      {/* ② Kernel path */}
      <SettingsItem
        num="②"
        title={t("userSettings.basicSettings")}
        sub={
          <div style={{ marginTop: 10, display: "flex", alignItems: "center", gap: 0, background: "var(--bg-secondary, #13161c)", border: "1px solid var(--border)", borderRadius: 7, overflow: "hidden", width: "100%", maxWidth: 400 }}>
            <input
              type="text"
              value={kernelPath}
              onChange={e => setKernelPath(e.target.value)}
              style={{
                flex: 1,
                padding: "9px 14px",
                fontFamily: "var(--font-mono)",
                fontSize: 12,
                color: "var(--muted-fg)",
                background: "transparent",
                border: "none",
                outline: "none",
              }}
            />
            <button
              style={{
                padding: "9px 14px",
                background: "transparent",
                border: "none",
                borderLeft: "1px solid var(--border)",
                color: "var(--muted-fg)",
                fontSize: 12,
                cursor: "pointer",
                whiteSpace: "nowrap",
                fontFamily: "var(--font-ui)",
              }}
            >
              浏览
            </button>
          </div>
        }
      />

      {/* ③ Auto start + Auto connect + Minimize to tray */}
      <SettingsItem
        num="③"
        title={t("userSettings.autoStart")}
        sub={t("userSettings.autoConnect")}
        right={
          <div style={{ display: "flex", flexDirection: "column", gap: 10, alignItems: "flex-end" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              <span style={{ fontSize: 12, color: "var(--muted-fg)" }}>{t("userSettings.autoStart")}</span>
              <ToggleSwitch value={config.auto_start} onChange={() => handleToggle("auto_start")} />
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              <span style={{ fontSize: 12, color: "var(--muted-fg)" }}>{t("userSettings.autoConnect")}</span>
              <ToggleSwitch value={config.auto_connect} onChange={() => handleToggle("auto_connect")} />
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
              <span style={{ fontSize: 12, color: "var(--muted-fg)" }}>{t("userSettings.minimizeToTray")}</span>
              <ToggleSwitch value={config.minimize_to_tray} onChange={() => handleToggle("minimize_to_tray")} />
            </div>
          </div>
        }
      />

      {/* ④ Language + About */}
      <SettingsItem
        num="④"
        title="Language / 语言"
        sub={
          <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
            {["zh", "en"].map(lang => (
              <button
                key={lang}
                onClick={() => i18n.changeLanguage(lang)}
                style={{
                  padding: "6px 16px",
                  borderRadius: 6,
                  fontSize: 12,
                  cursor: "pointer",
                  border: i18n.language === lang
                    ? "1px solid rgba(46,204,113,0.5)"
                    : "1px solid var(--border)",
                  background: i18n.language === lang
                    ? "rgba(46,204,113,0.15)"
                    : "transparent",
                  color: i18n.language === lang ? "var(--accent-green)" : "var(--muted-fg)",
                  fontFamily: "var(--font-ui)",
                  transition: "all 0.15s",
                }}
              >
                {lang === "zh" ? "中文" : "English"}
              </button>
            ))}
          </div>
        }
        right={
          <a
            href="https://github.com/yokeay/gidy-client"
            target="_blank"
            rel="noreferrer"
            style={{ fontSize: 13, color: "var(--accent-blue)", textDecoration: "none" }}
          >
            github.com/yokeay/gidy-client ↗
          </a>
        }
      />

      {/* Log retention */}
      <div
        style={{
          background: "var(--card)",
          border: "1px solid var(--border)",
          borderRadius: 10,
          padding: "16px 24px",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: 18,
        }}
      >
        <span style={{ fontSize: 14, color: "var(--fg)" }}>{t("userSettings.logRetention")}</span>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <input
            type="number"
            value={config.log_retention_days}
            onChange={e => setConfig({ ...config, log_retention_days: parseInt(e.target.value) || 7 })}
            style={{
              width: 64,
              background: "var(--muted)",
              border: "1px solid var(--border)",
              borderRadius: 6,
              padding: "6px 8px",
              fontSize: 13,
              color: "var(--fg)",
              textAlign: "center",
              fontFamily: "var(--font-mono)",
              outline: "none",
            }}
          />
          <span style={{ fontSize: 12, color: "var(--muted-fg)" }}>{t("userSettings.days")}</span>
        </div>
      </div>

      {/* Save */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "flex-end", gap: 16 }}>
        {message && (
          <span style={{ fontSize: 12, color: "var(--accent-green)" }}>{message}</span>
        )}
        <GreenBtn onClick={handleSave}>
          {saving ? t("common.loading") : t("userSettings.saveConfig")}
        </GreenBtn>
      </div>
    </div>
  );
}

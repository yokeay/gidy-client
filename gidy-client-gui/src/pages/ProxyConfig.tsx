import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import {
  getConfig,
  updateConfig,
  generatePsk,
  GuiConfig,
} from "../api";

function FieldGroup({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 7 }}>
      <div style={{ fontSize: 12, color: "var(--muted-fg)", fontWeight: 500, letterSpacing: "0.05em" }}>
        {label}
      </div>
      {children}
    </div>
  );
}

function FieldRow({
  value,
  type = "text",
  onChange,
  actionIcon,
  onAction,
  readOnly = false,
  placeholder,
}: {
  value: string;
  type?: string;
  onChange?: (v: string) => void;
  actionIcon?: string;
  onAction?: () => void;
  readOnly?: boolean;
  placeholder?: string;
}) {
  return (
    <div
      style={{
        display: "flex", alignItems: "center",
        background: "var(--card)", border: "1px solid var(--border)",
        borderRadius: 8, overflow: "hidden", transition: "border-color 0.15s",
      }}
      onMouseEnter={e => ((e.currentTarget as HTMLElement).style.borderColor = "rgba(255,255,255,0.12)")}
      onMouseLeave={e => ((e.currentTarget as HTMLElement).style.borderColor = "var(--border)")}
    >
      <input
        type={type}
        value={value}
        readOnly={readOnly}
        placeholder={placeholder}
        onChange={e => onChange?.(e.target.value)}
        style={{
          flex: 1, background: "transparent", border: "none", outline: "none",
          padding: "12px 14px", fontFamily: "var(--font-mono)", fontSize: 13,
          color: "var(--fg)", letterSpacing: type === "password" ? "0.15em" : "0.04em",
        }}
      />
      {actionIcon && (
        <button
          onClick={onAction}
          style={{
            width: 44, height: 44, background: "transparent",
            border: "none", borderLeft: "1px solid var(--border)",
            cursor: "pointer", display: "flex", alignItems: "center", justifyContent: "center",
            color: "var(--text-muted, #4a5268)", fontSize: 15, transition: "all 0.15s", flexShrink: 0,
          }}
          onMouseEnter={e => {
            (e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.04)";
            (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
          }}
          onMouseLeave={e => {
            (e.currentTarget as HTMLElement).style.background = "transparent";
            (e.currentTarget as HTMLElement).style.color = "var(--text-muted, #4a5268)";
          }}
        >
          {actionIcon}
        </button>
      )}
    </div>
  );
}

export default function ProxyConfig() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [config, setConfig] = useState<GuiConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [showPsk, setShowPsk] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  useEffect(() => {
    getConfig().then(setConfig).catch(() => {});
  }, []);

  const handleChange = (key: keyof GuiConfig, value: string | number | boolean) => {
    if (!config) return;
    setConfig({ ...config, [key]: value });
    setMessage(null);
  };

  const handleGeneratePsk = async () => {
    try {
      const psk = await generatePsk();
      setConfig(prev => (prev ? { ...prev, psk_hex: psk } : prev));
    } catch {}
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      await updateConfig(config);
      setMessage({ type: "success", text: t("proxyConfig.saved") });
      // Auto navigate to overview after save
      setTimeout(() => navigate("/dashboard"), 800);
    } catch {
      setMessage({ type: "error", text: t("proxyConfig.saveFailed") });
    }
    setSaving(false);
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).catch(() => {});
  };

  if (!config) return null;

  const serverDisplay = `${config.server_addr}:${config.server_port}`;
  const localDisplay = `${config.socks5_addr}:${config.socks5_port}`;
  const httpDisplay = `${config.http_addr}:${config.http_port}`;

  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", paddingTop: 12 }}>
      <div style={{ display: "flex", flexDirection: "column", gap: 20, width: "100%", maxWidth: 640 }}>

        {/* PSK */}
        <FieldGroup label={t("proxyConfig.psk")}>
          <div style={{ display: "flex", gap: 8 }}>
            <div style={{ flex: 1 }}>
              <FieldRow
                value={config.psk_hex}
                type={showPsk ? "text" : "password"}
                onChange={v => handleChange("psk_hex", v)}
                actionIcon={showPsk ? "🙈" : "👁"}
                onAction={() => setShowPsk(p => !p)}
              />
            </div>
            <button
              onClick={handleGeneratePsk}
              style={{
                padding: "0 14px", background: "var(--card)",
                border: "1px solid var(--border)", borderRadius: 8,
                color: "var(--muted-fg)", fontSize: 12, cursor: "pointer",
                whiteSpace: "nowrap", fontFamily: "var(--font-ui)",
              }}
              onMouseEnter={e => {
                (e.currentTarget as HTMLElement).style.borderColor = "rgba(46,204,113,0.4)";
                (e.currentTarget as HTMLElement).style.color = "var(--accent-green)";
              }}
              onMouseLeave={e => {
                (e.currentTarget as HTMLElement).style.borderColor = "var(--border)";
                (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
              }}
            >
              {t("proxyConfig.generatePsk")}
            </button>
          </div>
        </FieldGroup>

        {/* Server address */}
        <FieldGroup label={t("proxyConfig.serverAddr")}>
          <FieldRow
            value={serverDisplay}
            readOnly
            actionIcon="⧉"
            onAction={() => copyToClipboard(serverDisplay)}
          />
        </FieldGroup>

        {/* Server Name */}
        <FieldGroup label={t("proxyConfig.serverName")}>
          <FieldRow
            value={config.server_name}
            onChange={v => handleChange("server_name", v)}
            actionIcon="⧉"
            onAction={() => copyToClipboard(config.server_name)}
          />
        </FieldGroup>

        {/* SOCKS5 */}
        <FieldGroup label={t("proxyConfig.socks5Addr")}>
          <FieldRow
            value={localDisplay}
            readOnly
            actionIcon="⧉"
            onAction={() => copyToClipboard(localDisplay)}
          />
        </FieldGroup>

        {/* HTTP proxy */}
        <FieldGroup label={t("proxyConfig.httpAddr")}>
          <FieldRow
            value={httpDisplay}
            readOnly
            actionIcon="⧉"
            onAction={() => copyToClipboard(httpDisplay)}
          />
        </FieldGroup>

        {/* Protocol toggle */}
        <FieldGroup label={t("proxyConfig.protocol")}>
          <div style={{ display: "flex", gap: 8 }}>
            {["quic", "h2"].map(p => (
              <button
                key={p}
                onClick={() => handleChange("protocol", p)}
                style={{
                  flex: 1, padding: "10px 0", borderRadius: 8,
                  fontSize: 13, fontWeight: 500, cursor: "pointer",
                  border: config.protocol === p
                    ? "1px solid rgba(46,204,113,0.5)"
                    : "1px solid var(--border)",
                  background: config.protocol === p
                    ? "linear-gradient(135deg, rgba(46,204,113,0.2), rgba(46,204,113,0.08))"
                    : "var(--card)",
                  color: config.protocol === p ? "var(--accent-green)" : "var(--muted-fg)",
                  transition: "all 0.15s", fontFamily: "var(--font-mono)", letterSpacing: "0.04em",
                }}
              >
                {p.toUpperCase()}
              </button>
            ))}
          </div>
        </FieldGroup>

        {/* Mode toggle */}
        <FieldGroup label={t("proxyConfig.mode")}>
          <div style={{ display: "flex", gap: 8 }}>
            {["global", "pac"].map(m => (
              <button
                key={m}
                onClick={() => handleChange("mode", m)}
                style={{
                  flex: 1, padding: "10px 0", borderRadius: 8,
                  fontSize: 13, fontWeight: 500, cursor: "pointer",
                  border: config.mode === m
                    ? "1px solid rgba(46,204,113,0.5)"
                    : "1px solid var(--border)",
                  background: config.mode === m
                    ? "linear-gradient(135deg, rgba(46,204,113,0.2), rgba(46,204,113,0.08))"
                    : "var(--card)",
                  color: config.mode === m ? "var(--accent-green)" : "var(--muted-fg)",
                  transition: "all 0.15s", fontFamily: "var(--font-ui)",
                }}
              >
                {m === "global" ? t("proxyConfig.globalMode") : t("proxyConfig.pacMode")}
              </button>
            ))}
          </div>
        </FieldGroup>

        {/* Save */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "flex-end", gap: 16, paddingTop: 4 }}>
          {message && (
            <span style={{ fontSize: 12, color: message.type === "success" ? "var(--accent-green)" : "#e74c3c" }}>
              {message.text}
            </span>
          )}
          <button
            onClick={handleSave}
            disabled={saving}
            style={{
              padding: "9px 24px", background: "var(--accent-green)", color: "#000",
              border: "none", borderRadius: 8, fontSize: 13, fontWeight: 600,
              cursor: "pointer", boxShadow: "0 0 12px rgba(46,204,113,0.25)",
              transition: "all 0.15s", opacity: saving ? 0.5 : 1, fontFamily: "var(--font-ui)",
            }}
            onMouseEnter={e => {
              if (!saving) (e.currentTarget as HTMLElement).style.boxShadow = "0 0 20px rgba(46,204,113,0.4)";
            }}
            onMouseLeave={e => {
              (e.currentTarget as HTMLElement).style.boxShadow = "0 0 12px rgba(46,204,113,0.25)";
            }}
          >
            {saving ? t("common.loading") : t("proxyConfig.save")}
          </button>
        </div>
      </div>
    </div>
  );
}

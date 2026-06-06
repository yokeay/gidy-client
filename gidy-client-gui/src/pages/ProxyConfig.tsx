import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import {
  getConfig,
  updateConfig,
  generatePsk,
  refreshEch,
  GuiConfig,
} from "../api";

type Protocol = "ws" | "h2" | "h3" | "quic";
type Tab = "protocol" | "rules" | "mode";
type ProxyMode = "rule" | "global" | "direct";

const DEFAULT_PORTS: Record<Protocol, number> = { ws: 443, h2: 443, h3: 4434, quic: 4434 };

const PROTOCOLS: { key: Protocol; label: string }[] = [
  { key: "ws", label: "WS" },
  { key: "h2", label: "H2" },
  { key: "h3", label: "H3" },
  { key: "quic", label: "QUIC" },
];

const TAB_ITEMS: { key: Tab; labelKey: string }[] = [
  { key: "protocol", labelKey: "proxyConfig.tabProtocol" },
  { key: "rules", labelKey: "proxyConfig.tabRules" },
  { key: "mode", labelKey: "proxyConfig.tabMode" },
];

export default function ProxyConfig() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [config, setConfig] = useState<GuiConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [showPsk, setShowPsk] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<Tab>("protocol");
  const [activeProtocol, setActiveProtocol] = useState<Protocol>("ws");
  const [proxyMode, setProxyMode] = useState<ProxyMode>("global");

  useEffect(() => { getConfig().then(c => { setConfig(c); setActiveProtocol(c.protocol as Protocol); setProxyMode(c.mode as ProxyMode || "global"); }).catch(() => {}); }, []);

  const handleChange = (key: keyof GuiConfig, value: string | number | boolean) => {
    if (!config) return;
    setConfig({ ...config, [key]: value });
    setMessage(null);
  };

  const handleProtocolSelect = (proto: Protocol) => {
    if (!config) return;
    setActiveProtocol(proto);
    setConfig({ ...config, protocol: proto, server_port: DEFAULT_PORTS[proto] });
    setMessage(null);
  };

  const handleGeneratePsk = async () => {
    try { const psk = await generatePsk(); setConfig(prev => prev ? { ...prev, psk_hex: psk } : prev); } catch {}
  };

  const handleRefreshEch = async () => {
    setLoading(true);
    setMessage(null);
    try {
      const newEch = await refreshEch();
      setConfig(prev => prev ? { ...prev, ech_config_base64: newEch } : prev);
      setMessage({ type: "success", text: "✅ ECH 配置已刷新" });
    } catch (e) {
      setMessage({ type: "error", text: `❌ ${String(e)}` });
    }
    setLoading(false);
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      await updateConfig({ ...config, mode: proxyMode });
      setMessage({ type: "success", text: t("proxyConfig.saved") });
      setTimeout(() => navigate("/dashboard"), 800);
    } catch { setMessage({ type: "error", text: t("proxyConfig.saveFailed") }); }
    setSaving(false);
  };

  const copyToClipboard = (text: string) => { navigator.clipboard.writeText(text).catch(() => {}); };

  if (!config) return null;

  const activeIdx = PROTOCOLS.findIndex(p => p.key === activeProtocol);
  const activeTabIdx = TAB_ITEMS.findIndex(tb => tb.key === activeTab);

  return (
    <div style={{ display: "flex", flexDirection: "column", position: "relative" }}>

      {/* ── Loading overlay for ECH refresh ── */}
      {loading && (
        <div style={{
          position: "fixed", inset: 0, zIndex: 9999,
          background: "rgba(0,0,0,0.45)", display: "flex",
          alignItems: "center", justifyContent: "center",
        }}>
          <div style={{
            background: "var(--card)", border: "1px solid var(--border)",
            borderRadius: 12, padding: "28px 40px",
            display: "flex", flexDirection: "column", alignItems: "center", gap: 14,
          }}>
            <div style={{ fontSize: 14, color: "var(--fg)" }}>正在刷新 ECH 配置…</div>
            <div style={{ width: 180, height: 3, background: "var(--border)", borderRadius: 2, overflow: "hidden" }}>
              <div style={{ width: "40%", height: "100%", background: "var(--accent-green)", borderRadius: 2,
                animation: "slide 1.2s ease-in-out infinite" }} />
            </div>
          </div>
          <style>{`@keyframes slide { 0%,100%{transform:translateX(-100%)} 50%{transform:translateX(250%)} }`}</style>
        </div>
      )}

      {/* ── Top row: TabBar + Save ── */}
      <div style={{ display: "flex", alignItems: "center", gap: 16, marginBottom: 24 }}>
        {/* TabBar */}
        <div style={{
          display: "flex", alignItems: "center",
          background: "var(--card)", border: "1px solid var(--border)",
          borderRadius: 10, padding: 4, position: "relative",
        }}>
          {/* Sliding tab highlight */}
          <div style={{
            position: "absolute", top: 4, left: 4 + activeTabIdx * 88, width: 84, height: 34,
            background: "linear-gradient(135deg, rgba(46,204,113,0.22), rgba(46,204,113,0.08))",
            border: "1px solid rgba(46,204,113,0.5)", borderRadius: 8,
            transition: "left 0.25s cubic-bezier(0.4,0,0.2,1)",
            pointerEvents: "none",
          }} />
          {TAB_ITEMS.map(tb => (
            <button
              key={tb.key}
              onClick={() => setActiveTab(tb.key)}
              style={{
                width: 84, height: 34, borderRadius: 8, border: "none",
                background: "transparent", cursor: "pointer", position: "relative", zIndex: 1,
                fontSize: 13, fontWeight: 600, fontFamily: "var(--font-ui)",
                color: activeTab === tb.key ? "var(--accent-green)" : "var(--muted-fg)",
                transition: "color 0.15s",
              }}
            >
              {t(tb.labelKey as any)}
            </button>
          ))}
        </div>

        <div style={{ flex: 1 }} />

        {/* Toast */}
        {message && (
          <span style={{
            fontSize: 12, color: message.type === "success" ? "var(--accent-green)" : "#e74c3c",
            transition: "all 0.3s",
          }}>
            {message.text}
          </span>
        )}

        {/* Save */}
        <button
          onClick={handleSave}
          disabled={saving}
          style={{
            padding: "8px 22px", background: "var(--accent-green)", color: "#000",
            border: "none", borderRadius: 8, fontSize: 13, fontWeight: 600,
            cursor: "pointer", boxShadow: "0 0 12px rgba(46,204,113,0.25)",
            transition: "all 0.15s", opacity: saving ? 0.5 : 1, fontFamily: "var(--font-ui)",
            flexShrink: 0,
          }}
          onMouseEnter={e => { if (!saving) (e.currentTarget as HTMLElement).style.boxShadow = "0 0 20px rgba(46,204,113,0.4)"; }}
          onMouseLeave={e => { (e.currentTarget as HTMLElement).style.boxShadow = "0 0 12px rgba(46,204,113,0.25)"; }}
        >
          {saving ? t("common.loading") : t("common.save")}
        </button>
      </div>

      {/* ── Tab content ── */}

      {/* Protocol tab */}
      {activeTab === "protocol" && (
        <div style={{ display: "flex", gap: 0 }}>
          {/* Left: vertical protocol menu */}
          <div style={{
            width: 80, flexShrink: 0,
            background: "var(--card)", border: "1px solid var(--border)",
            borderRadius: 10, padding: 4, position: "relative",
          }}>
            {/* Sliding highlight for vertical menu */}
            <div style={{
              position: "absolute", top: 4 + activeIdx * 46, left: 4,
              width: 72, height: 42,
              background: "linear-gradient(135deg, rgba(46,204,113,0.22), rgba(46,204,113,0.08))",
              border: "1px solid rgba(46,204,113,0.5)", borderRadius: 8,
              transition: "top 0.25s cubic-bezier(0.4,0,0.2,1)",
              pointerEvents: "none",
            }} />
            {PROTOCOLS.map(p => (
              <button
                key={p.key}
                onClick={() => handleProtocolSelect(p.key)}
                style={{
                  height: 42, borderRadius: 8, border: "none",
                  background: "transparent", cursor: "pointer", position: "relative", zIndex: 1,
                  fontSize: 13, fontWeight: 600, fontFamily: "var(--font-mono)", letterSpacing: "0.08em",
                  color: activeProtocol === p.key ? "var(--accent-green)" : "var(--muted-fg)",
                  transition: "color 0.15s",
                }}
              >
                {p.label}
              </button>
            ))}
          </div>

          {/* Right: protocol config form */}
          <div style={{ flex: 1, paddingLeft: 24, display: "flex", flexDirection: "column", gap: 18, maxWidth: 540, paddingBottom: 32 }}>
            {/* PSK (shared) */}
            <FieldGroup label={t("proxyConfig.psk")}>
              <div style={{ display: "flex", gap: 8 }}>
                <div style={{ flex: 1 }}>
                  <InputRow value={config.psk_hex} type={showPsk ? "text" : "password"}
                    onChange={v => handleChange("psk_hex", v)}
                    actionIcon={showPsk ? "🙈" : "👁"} onAction={() => setShowPsk(p => !p)} />
                </div>
                <SmallBtn onClick={handleGeneratePsk}>{t("proxyConfig.generatePsk")}</SmallBtn>
              </div>
            </FieldGroup>

            {/* WS config */}
            {activeProtocol === "ws" && (
              <>
                <FieldGroup label={t("proxyConfig.wsUrl")}>
                  <InputRow value={config.ws_url} onChange={v => handleChange("ws_url", v)}
                    placeholder="wss://gidy.eu.cc/ws" actionIcon="⧉" onAction={() => copyToClipboard(config.ws_url)} />
                </FieldGroup>
                <FieldGroup label={t("proxyConfig.serverName")}>
                  <InputRow value={config.server_name} onChange={v => handleChange("server_name", v)}
                    actionIcon="⧉" onAction={() => copyToClipboard(config.server_name)} />
                </FieldGroup>
                <FieldGroup label={t("proxyConfig.echConfig")}>
                  <div style={{ display: "flex", gap: 8 }}>
                    <div style={{ flex: 1 }}>
                      <InputRow value={config.ech_config_base64} onChange={v => handleChange("ech_config_base64", v)}
                        placeholder="AEX+DQB..." actionIcon="⧉" onAction={() => copyToClipboard(config.ech_config_base64)} />
                    </div>
                    <SmallBtn onClick={handleRefreshEch}>{t("proxyConfig.refreshEch")}</SmallBtn>
                  </div>
                </FieldGroup>
                <FieldGroup label={t("proxyConfig.echToken")}>
                  <InputRow value={config.ech_token} onChange={v => handleChange("ech_token", v)}
                    placeholder="23ba5c1e97d380d288b2fd6e4cc5114e" />
                </FieldGroup>
              </>
            )}

            {/* H2 / H3 / QUIC config */}
            {(activeProtocol === "h2" || activeProtocol === "h3" || activeProtocol === "quic") && (
              <>
                <FieldGroup label={t("proxyConfig.serverAddr")}>
                  <InputRow value={config.server_addr} onChange={v => handleChange("server_addr", v)}
                    placeholder="gidy.eu.cc" actionIcon="⧉" onAction={() => copyToClipboard(config.server_addr)} />
                </FieldGroup>
                <FieldGroup label={t("proxyConfig.serverPort")}>
                  <NumberInput value={config.server_port} onChange={v => handleChange("server_port", v)}
                    min={1} max={65535} placeholder={String(DEFAULT_PORTS[activeProtocol])} />
                </FieldGroup>
                <FieldGroup label={t("proxyConfig.serverName")}>
                  <InputRow value={config.server_name} onChange={v => handleChange("server_name", v)}
                    actionIcon="⧉" onAction={() => copyToClipboard(config.server_name)} />
                </FieldGroup>
              </>
            )}

            {/* SOCKS5 port (shared) */}
            <FieldGroup label={t("proxyConfig.socks5Port")}>
              <NumberInput value={config.socks5_port} onChange={v => handleChange("socks5_port", v)}
                min={1} max={65535} placeholder="5555" />
            </FieldGroup>
          </div>
        </div>
      )}

      {/* Rules tab — placeholder */}
      {activeTab === "rules" && (
        <div style={{
          display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center",
          padding: "60px 0", color: "var(--muted-fg)", gap: 12,
        }}>
          <div style={{ fontSize: 40, opacity: 0.3 }}>📋</div>
          <div style={{ fontSize: 14 }}>{t("proxyConfig.rulesPlaceholder")}</div>
        </div>
      )}

      {/* Proxy mode tab */}
      {activeTab === "mode" && (
        <div style={{ display: "flex", flexDirection: "column", gap: 16, maxWidth: 600, paddingBottom: 32 }}>
          {([
            { key: "rule" as ProxyMode, icon: "📋", title: t("proxyConfig.ruleMode"), desc: t("proxyConfig.ruleModeDesc") },
            { key: "global" as ProxyMode, icon: "🌐", title: t("proxyConfig.globalMode"), desc: t("proxyConfig.globalModeDesc") },
            { key: "direct" as ProxyMode, icon: "🔗", title: t("proxyConfig.directMode"), desc: t("proxyConfig.directModeDesc") },
          ]).map(m => (
            <button
              key={m.key}
              onClick={() => setProxyMode(m.key)}
              style={{
                display: "flex", alignItems: "flex-start", gap: 14,
                padding: "18px 20px", borderRadius: 10,
                border: proxyMode === m.key ? "1px solid rgba(46,204,113,0.5)" : "1px solid var(--border)",
                background: proxyMode === m.key
                  ? "linear-gradient(135deg, rgba(46,204,113,0.15), rgba(46,204,113,0.05))"
                  : "var(--card)",
                cursor: "pointer", textAlign: "left", transition: "all 0.15s",
                fontFamily: "var(--font-ui)",
              }}
              onMouseEnter={e => {
                if (proxyMode !== m.key) {
                  (e.currentTarget as HTMLElement).style.borderColor = "rgba(255,255,255,0.12)";
                }
              }}
              onMouseLeave={e => {
                if (proxyMode !== m.key) {
                  (e.currentTarget as HTMLElement).style.borderColor = "var(--border)";
                }
              }}
            >
              <div style={{
                width: 38, height: 38, borderRadius: 8, flexShrink: 0,
                background: proxyMode === m.key ? "rgba(46,204,113,0.18)" : "rgba(255,255,255,0.06)",
                border: proxyMode === m.key ? "1px solid rgba(46,204,113,0.3)" : "1px solid var(--border)",
                display: "flex", alignItems: "center", justifyContent: "center",
                fontSize: 16, transition: "all 0.15s",
              }}>
                {m.icon}
              </div>
              <div style={{ flex: 1 }}>
                <div style={{
                  fontSize: 14, fontWeight: 600,
                  color: proxyMode === m.key ? "var(--accent-green)" : "var(--fg)",
                  marginBottom: 4, transition: "color 0.15s",
                }}>
                  {m.title}
                </div>
                <div style={{ fontSize: 12, color: "var(--muted-fg)", lineHeight: 1.6 }}>{m.desc}</div>
              </div>
              {proxyMode === m.key && (
                <div style={{
                  width: 20, height: 20, borderRadius: "50%", flexShrink: 0,
                  background: "var(--accent-green)", display: "flex",
                  alignItems: "center", justifyContent: "center",
                  fontSize: 11, color: "#000", fontWeight: 700,
                  boxShadow: "0 0 8px rgba(46,204,113,0.4)",
                }}>✓</div>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

/* ── Tiny reusable components ── */

function FieldGroup({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 7 }}>
      <div style={{ fontSize: 12, color: "var(--muted-fg)", fontWeight: 500, letterSpacing: "0.05em" }}>{label}</div>
      {children}
    </div>
  );
}

function InputRow({ value, type = "text", onChange, actionIcon, onAction, placeholder }: {
  value: string; type?: string; onChange?: (v: string) => void;
  actionIcon?: string; onAction?: () => void; placeholder?: string;
}) {
  return (
    <div style={{
      display: "flex", alignItems: "center", background: "var(--card)", border: "1px solid var(--border)",
      borderRadius: 8, overflow: "hidden", transition: "border-color 0.15s",
    }}
    onMouseEnter={e => ((e.currentTarget as HTMLElement).style.borderColor = "rgba(255,255,255,0.12)")}
    onMouseLeave={e => ((e.currentTarget as HTMLElement).style.borderColor = "var(--border)")}
    >
      <input type={type} value={value} placeholder={placeholder} onChange={e => onChange?.(e.target.value)}
        style={{
          flex: 1, background: "transparent", border: "none", outline: "none",
          padding: "12px 14px", fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--fg)",
          letterSpacing: type === "password" ? "0.15em" : "0.04em",
        }}
      />
      {actionIcon && (
        <button onClick={onAction} style={{
          width: 44, height: 44, background: "transparent", border: "none",
          borderLeft: "1px solid var(--border)", cursor: "pointer",
          display: "flex", alignItems: "center", justifyContent: "center",
          color: "var(--text-muted, #4a5268)", fontSize: 15, transition: "all 0.15s", flexShrink: 0,
        }}
        onMouseEnter={e => { (e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.04)"; (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)"; }}
        onMouseLeave={e => { (e.currentTarget as HTMLElement).style.background = "transparent"; (e.currentTarget as HTMLElement).style.color = "var(--text-muted, #4a5268)"; }}
        >{actionIcon}</button>
      )}
    </div>
  );
}

function NumberInput({ value, onChange, min, max, placeholder }: {
  value: number; onChange: (v: number) => void; min?: number; max?: number; placeholder?: string;
}) {
  return (
    <div style={{
      display: "flex", alignItems: "center", background: "var(--card)", border: "1px solid var(--border)",
      borderRadius: 8, overflow: "hidden", transition: "border-color 0.15s",
    }}
    onMouseEnter={e => ((e.currentTarget as HTMLElement).style.borderColor = "rgba(255,255,255,0.12)")}
    onMouseLeave={e => ((e.currentTarget as HTMLElement).style.borderColor = "var(--border)")}
    >
      <input type="number" value={value} min={min} max={max} placeholder={placeholder}
        onChange={e => onChange(Number(e.target.value))}
        style={{
          flex: 1, background: "transparent", border: "none", outline: "none",
          padding: "12px 14px", fontFamily: "var(--font-mono)", fontSize: 13, color: "var(--fg)", letterSpacing: "0.04em",
        }}
      />
    </div>
  );
}

function SmallBtn({ onClick, children }: { onClick: () => void; children: React.ReactNode }) {
  return (
    <button onClick={onClick} style={{
      padding: "0 14px", background: "var(--card)", border: "1px solid var(--border)",
      borderRadius: 8, color: "var(--muted-fg)", fontSize: 12, cursor: "pointer",
      whiteSpace: "nowrap", fontFamily: "var(--font-ui)", height: 44,
    }}
    onMouseEnter={e => { (e.currentTarget as HTMLElement).style.borderColor = "rgba(46,204,113,0.4)"; (e.currentTarget as HTMLElement).style.color = "var(--accent-green)"; }}
    onMouseLeave={e => { (e.currentTarget as HTMLElement).style.borderColor = "var(--border)"; (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)"; }}
    >{children}</button>
  );
}

import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Shield, Network, Info } from "lucide-react";
import {
  getConfig,
  updateConfig,
  generatePsk,
  getStatus,
  GuiConfig,
} from "../api";

export default function SystemConfig() {
  const { t } = useTranslation();
  const [config, setConfig] = useState<GuiConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [running, setRunning] = useState(false);
  const [message, setMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  useEffect(() => {
    getConfig().then(setConfig).catch(() => {});
    const tick = async () => {
      try {
        const s = await getStatus();
        setRunning(s.running);
      } catch {}
    };
    tick();
    const id = setInterval(tick, 2000);
    return () => clearInterval(id);
  }, []);

  const handleChange = (
    key: keyof GuiConfig,
    value: string | number | boolean,
  ) => {
    if (!config) return;
    setConfig({ ...config, [key]: value });
    setMessage(null);
  };

  const handleGeneratePsk = async () => {
    try {
      const psk = await generatePsk();
      setConfig((prev) => (prev ? { ...prev, psk_hex: psk } : prev));
    } catch {}
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      await updateConfig(config);
      setMessage({ type: "success", text: t("systemConfig.saved") });
    } catch {
      setMessage({ type: "error", text: t("systemConfig.saveFailed") });
    }
    setSaving(false);
  };

  if (!config) return null;

  const inputClass =
    "w-full bg-muted border border-border rounded-lg px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:border-foreground/40 transition-colors tabular";
  const labelClass = "text-xs text-muted-foreground mb-1.5 block";

  return (
    <div className="space-y-5">
      {/* Connection status badge */}
      <div className="flex items-center justify-between">
        <h2 className="text-base font-semibold">{t("systemConfig.title")}</h2>
        <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-card border border-border">
          <span className="relative flex h-2 w-2">
            {running && (
              <span className="absolute inline-flex h-full w-full rounded-full bg-foreground opacity-40 animate-ping" />
            )}
            <span
              className={`relative inline-flex rounded-full h-2 w-2 ${
                running ? "bg-foreground" : "bg-muted-foreground/50"
              }`}
            />
          </span>
          <span className="text-xs text-muted-foreground">
            {running ? t("common.connected") : t("common.disconnected")}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        {/* Proxy Server */}
        <div className="bg-card rounded-2xl border border-border p-6">
          <div className="flex items-center gap-2 mb-5">
            <Shield size={15} strokeWidth={1.75} className="text-muted-foreground" />
            <h3 className="font-semibold text-sm">
              {t("systemConfig.proxyServer")}
            </h3>
          </div>
          <div className="space-y-4">
            <div>
              <label className={labelClass}>
                {t("systemConfig.serverAddr")}
              </label>
              <input
                className={inputClass}
                value={config.server_addr}
                onChange={(e) => handleChange("server_addr", e.target.value)}
                placeholder="127.0.0.1"
              />
            </div>
            <div>
              <label className={labelClass}>
                {t("systemConfig.serverPort")}
              </label>
              <input
                className={inputClass}
                value={config.server_port}
                onChange={(e) =>
                  handleChange("server_port", parseInt(e.target.value) || 0)
                }
                placeholder="443"
              />
            </div>
            <div>
              <div className="flex items-center justify-between mb-1.5">
                <label className="text-xs text-muted-foreground">
                  {t("systemConfig.psk")}
                </label>
                <button
                  onClick={handleGeneratePsk}
                  className="text-xs text-foreground hover:opacity-70 transition-opacity"
                >
                  {t("systemConfig.generatePsk")}
                </button>
              </div>
              <input
                className={inputClass}
                value={config.psk_hex}
                onChange={(e) => handleChange("psk_hex", e.target.value)}
                placeholder={t("systemConfig.pskHint")}
              />
            </div>
            <div>
              <label className={labelClass}>{t("systemConfig.protocol")}</label>
              <select
                className={inputClass}
                value={config.protocol}
                onChange={(e) => handleChange("protocol", e.target.value)}
              >
                <option value="gidy">gidy</option>
              </select>
            </div>
          </div>
        </div>

        {/* Local Proxy */}
        <div className="bg-card rounded-2xl border border-border p-6">
          <div className="flex items-center gap-2 mb-5">
            <Network size={15} strokeWidth={1.75} className="text-muted-foreground" />
            <h3 className="font-semibold text-sm">
              {t("systemConfig.localProxy")}
            </h3>
          </div>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className={labelClass}>
                  {t("systemConfig.socks5Addr")}
                </label>
                <input
                  className={inputClass}
                  value={config.socks5_addr}
                  onChange={(e) => handleChange("socks5_addr", e.target.value)}
                  placeholder="127.0.0.1"
                />
              </div>
              <div>
                <label className={labelClass}>
                  {t("systemConfig.socks5Port")}
                </label>
                <input
                  className={inputClass}
                  value={config.socks5_port}
                  onChange={(e) =>
                    handleChange("socks5_port", parseInt(e.target.value) || 0)
                  }
                  placeholder="1080"
                />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className={labelClass}>
                  {t("systemConfig.httpAddr")}
                </label>
                <input
                  className={inputClass}
                  value={config.http_addr}
                  onChange={(e) => handleChange("http_addr", e.target.value)}
                  placeholder="127.0.0.1"
                />
              </div>
              <div>
                <label className={labelClass}>
                  {t("systemConfig.httpPort")}
                </label>
                <input
                  className={inputClass}
                  value={config.http_port}
                  onChange={(e) =>
                    handleChange("http_port", parseInt(e.target.value) || 0)
                  }
                  placeholder="8080"
                />
              </div>
            </div>
            <div>
              <label className={labelClass}>{t("systemConfig.mode")}</label>
              <div className="grid grid-cols-2 gap-2">
                <button
                  onClick={() => handleChange("mode", "global")}
                  className={`py-2 rounded-lg text-xs font-medium border transition-colors ${
                    config.mode === "global"
                      ? "bg-primary text-primary-foreground border-primary"
                      : "bg-muted border-border text-muted-foreground hover:text-foreground"
                  }`}
                >
                  {t("systemConfig.globalMode")}
                </button>
                <button
                  onClick={() => handleChange("mode", "pac")}
                  className={`py-2 rounded-lg text-xs font-medium border transition-colors ${
                    config.mode === "pac"
                      ? "bg-primary text-primary-foreground border-primary"
                      : "bg-muted border-border text-muted-foreground hover:text-foreground"
                  }`}
                >
                  {t("systemConfig.pacMode")}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Connection Note */}
      <div className="bg-card rounded-2xl border border-border p-6">
        <div className="flex items-center gap-2 mb-2.5">
          <Info size={14} strokeWidth={1.75} className="text-muted-foreground" />
          <h3 className="font-semibold text-sm">
            {t("systemConfig.connectionNote")}
          </h3>
        </div>
        <p className="text-xs text-muted-foreground leading-relaxed">
          {t("systemConfig.connectionNoteText")}
        </p>
      </div>

      {/* Save button */}
      <div className="flex items-center justify-end gap-4">
        {message && (
          <span
            className={`text-xs ${
              message.type === "success"
                ? "text-muted-foreground"
                : "text-destructive"
            }`}
          >
            {message.text}
          </span>
        )}
        <button
          onClick={handleSave}
          disabled={saving}
          className="px-7 py-2.5 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
        >
          {saving ? t("common.loading") : t("systemConfig.saveAndConnect")}
        </button>
      </div>
    </div>
  );
}

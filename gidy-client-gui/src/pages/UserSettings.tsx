import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Sun, Moon } from "lucide-react";
import { getConfig, updateConfig, GuiConfig } from "../api";

interface UserSettingsProps {
  theme: "light" | "dark";
  themeColor: string;
  onThemeChange: (t: "light" | "dark") => void;
  onThemeColorChange: (c: string) => void;
}

const APP_VERSION = "v0.2.6";

const COLOR_OPTIONS = [
  { value: "blue", class: "bg-blue-500" },
  { value: "emerald", class: "bg-emerald-500" },
  { value: "purple", class: "bg-purple-500" },
  { value: "orange", class: "bg-orange-500" },
  { value: "rose", class: "bg-rose-500" },
];

export default function UserSettings({
  theme,
  themeColor,
  onThemeChange,
  onThemeColorChange,
}: UserSettingsProps) {
  const { t, i18n } = useTranslation();
  const [config, setConfig] = useState<GuiConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    getConfig()
      .then((c) => {
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

  const toggleClass = (v: boolean) =>
    `relative w-10 h-5 rounded-full transition-colors ${v ? "bg-foreground" : "bg-muted-foreground/30"}`;
  const knobClass = (v: boolean) =>
    `absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-card transition-transform ${v ? "translate-x-5" : "translate-x-0"}`;

  return (
    <div className="space-y-5">
      <div className="grid grid-cols-2 gap-4">
        {/* Basic Settings */}
        <div className="bg-card rounded-2xl border border-border p-6">
          <h3 className="text-sm font-semibold mb-5">
            {t("userSettings.basicSettings")}
          </h3>
          <div className="space-y-5">
            <div className="flex items-center justify-between">
              <span className="text-sm">{t("userSettings.autoStart")}</span>
              <button
                onClick={() => handleToggle("auto_start")}
                className={toggleClass(config.auto_start)}
              >
                <span className={knobClass(config.auto_start)} />
              </button>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">{t("userSettings.autoConnect")}</span>
              <button
                onClick={() => handleToggle("auto_connect")}
                className={toggleClass(config.auto_connect)}
              >
                <span className={knobClass(config.auto_connect)} />
              </button>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">
                {t("userSettings.minimizeToTray")}
              </span>
              <button
                onClick={() => handleToggle("minimize_to_tray")}
                className={toggleClass(config.minimize_to_tray)}
              >
                <span className={knobClass(config.minimize_to_tray)} />
              </button>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">{t("userSettings.logRetention")}</span>
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  className="w-16 bg-muted border border-border rounded-lg px-2 py-1 text-xs text-center tabular focus:outline-none focus:border-foreground/40"
                  value={config.log_retention_days}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      log_retention_days: parseInt(e.target.value) || 7,
                    })
                  }
                />
                <span className="text-xs text-muted-foreground">
                  {t("userSettings.days")}
                </span>
              </div>
            </div>
          </div>
        </div>

        {/* Appearance */}
        <div className="bg-card rounded-2xl border border-border p-6">
          <h3 className="text-sm font-semibold mb-5">
            {t("userSettings.themeMode")}
          </h3>
          <div className="space-y-5">
            <div className="flex items-center justify-between">
              <span className="text-sm">{t("userSettings.themeMode")}</span>
              <div className="flex gap-1 bg-muted rounded-lg p-0.5">
                <button
                  onClick={() => onThemeChange("light")}
                  className={`flex items-center gap-1 px-3 py-1 rounded-md text-xs transition-colors ${
                    theme === "light"
                      ? "bg-card text-foreground shadow-sm"
                      : "text-muted-foreground"
                  }`}
                >
                  <Sun size={12} strokeWidth={1.75} />
                  {t("userSettings.light")}
                </button>
                <button
                  onClick={() => onThemeChange("dark")}
                  className={`flex items-center gap-1 px-3 py-1 rounded-md text-xs transition-colors ${
                    theme === "dark"
                      ? "bg-card text-foreground shadow-sm"
                      : "text-muted-foreground"
                  }`}
                >
                  <Moon size={12} strokeWidth={1.75} />
                  {t("userSettings.dark")}
                </button>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">{t("userSettings.themeColor")}</span>
              <div className="flex gap-2">
                {COLOR_OPTIONS.map((c) => (
                  <button
                    key={c.value}
                    onClick={() => onThemeColorChange(c.value)}
                    className={`w-5 h-5 rounded-full ${c.class} transition-transform ${
                      themeColor === c.value
                        ? "ring-2 ring-offset-2 ring-offset-card ring-foreground scale-110"
                        : ""
                    }`}
                  />
                ))}
              </div>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm">Language</span>
              <div className="flex gap-1 bg-muted rounded-lg p-0.5">
                <button
                  onClick={() => i18n.changeLanguage("zh")}
                  className={`px-3 py-1 rounded-md text-xs transition-colors ${
                    i18n.language === "zh"
                      ? "bg-card text-foreground shadow-sm"
                      : "text-muted-foreground"
                  }`}
                >
                  中文
                </button>
                <button
                  onClick={() => i18n.changeLanguage("en")}
                  className={`px-3 py-1 rounded-md text-xs transition-colors ${
                    i18n.language === "en"
                      ? "bg-card text-foreground shadow-sm"
                      : "text-muted-foreground"
                  }`}
                >
                  English
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Version */}
      <div className="bg-card rounded-2xl border border-border p-6">
        <h3 className="text-sm font-semibold mb-4">
          {t("userSettings.updateCheck")}
        </h3>
        <div className="space-y-3">
          <div className="flex justify-between text-sm">
            <span className="text-muted-foreground">
              {t("userSettings.currentVersion")}
            </span>
            <span className="tabular">{APP_VERSION}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-muted-foreground">
              {t("userSettings.latestVersion")}
            </span>
            <span className="tabular">{APP_VERSION}</span>
          </div>
          <button className="w-full py-2 rounded-lg border border-border text-sm hover:bg-muted transition-colors">
            {t("userSettings.checkUpdate")}
          </button>
        </div>
      </div>

      {/* Save */}
      <div className="flex items-center justify-end gap-4">
        {message && (
          <span className="text-xs text-muted-foreground">{message}</span>
        )}
        <button
          onClick={handleSave}
          disabled={saving}
          className="px-7 py-2.5 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50"
        >
          {saving ? t("common.loading") : t("userSettings.saveConfig")}
        </button>
      </div>
    </div>
  );
}

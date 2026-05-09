import { useState, useEffect } from "react";
import { Routes, Route, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";
import Sidebar from "./components/Sidebar";
import Dashboard from "./pages/Dashboard";
import SystemConfig from "./pages/SystemConfig";
import TrafficMonitor from "./pages/TrafficMonitor";
import UserSettings from "./pages/UserSettings";
import About from "./pages/About";

export default function App() {
  const { t, i18n } = useTranslation();
  const [theme, setTheme] = useState<"light" | "dark">("dark");
  const [themeColor, setThemeColor] = useState("blue");
  const location = useLocation();

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
  }, [theme]);

  const pageTitle = () => {
    const path = location.pathname;
    if (path === "/" || path === "/dashboard") return t("nav.dashboard");
    if (path === "/system-config") return t("nav.systemConfig");
    if (path === "/traffic-monitor") return t("nav.trafficMonitor");
    if (path === "/user-settings") return t("nav.userSettings");
    if (path === "/about") return t("nav.about");
    return "";
  };

  const toggleLang = () => {
    const next = i18n.language === "zh" ? "en" : "zh";
    i18n.changeLanguage(next);
  };

  return (
    <div className="flex h-screen w-screen bg-background text-foreground overflow-hidden">
      <Sidebar
        theme={theme}
        onToggleTheme={() => setTheme(theme === "dark" ? "light" : "dark")}
        onToggleLang={toggleLang}
        themeColor={themeColor}
        currentLang={i18n.language as "zh" | "en"}
      />
      <div className="flex-1 flex flex-col min-w-0">
        <header className="h-12 shrink-0 flex items-center px-6 border-b border-border bg-card">
          <h1 className="text-sm font-medium text-muted-foreground">
            {pageTitle()}
          </h1>
        </header>
        <main className="flex-1 overflow-auto p-6">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route
              path="/system-config"
              element={<SystemConfig />}
            />
            <Route path="/traffic-monitor" element={<TrafficMonitor />} />
            <Route
              path="/user-settings"
              element={
                <UserSettings
                  theme={theme}
                  themeColor={themeColor}
                  onThemeChange={setTheme}
                  onThemeColorChange={setThemeColor}
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

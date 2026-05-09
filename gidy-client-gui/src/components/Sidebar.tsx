import { useNavigate, useLocation } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  LayoutDashboard,
  Settings,
  Activity,
  UserCog,
  Info,
  Moon,
  Sun,
  Languages,
} from "lucide-react";
import clsx from "clsx";

interface SidebarProps {
  theme: "light" | "dark";
  onToggleTheme: () => void;
  onToggleLang: () => void;
  themeColor: string;
  currentLang: "zh" | "en";
}

const navItems = [
  { path: "/dashboard", icon: LayoutDashboard, key: "nav.dashboard" },
  { path: "/system-config", icon: Settings, key: "nav.systemConfig" },
  { path: "/traffic-monitor", icon: Activity, key: "nav.trafficMonitor" },
  { path: "/user-settings", icon: UserCog, key: "nav.userSettings" },
  { path: "/about", icon: Info, key: "nav.about" },
];

export default function Sidebar({
  theme,
  onToggleTheme,
  onToggleLang,
  currentLang,
}: SidebarProps) {
  const navigate = useNavigate();
  const location = useLocation();
  const { t } = useTranslation();

  const isActive = (path: string) => {
    if (path === "/dashboard") {
      return location.pathname === "/" || location.pathname === "/dashboard";
    }
    return location.pathname === path;
  };

  return (
    <aside className="w-52 shrink-0 flex flex-col border-r border-border bg-card">
      {/* Logo */}
      <div className="h-14 flex items-center gap-3 px-4 border-b border-border">
        <img src="/logo.png" alt="gidy" className="w-8 h-8 rounded-lg" />
        <span className="font-semibold text-sm">gidy client</span>
      </div>

      {/* Nav items */}
      <nav className="flex-1 py-4 px-3 space-y-1">
        {navItems.map(({ path, icon: Icon, key }) => (
          <button
            key={path}
            onClick={() => navigate(path)}
            className={clsx(
              "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-all duration-200",
              isActive(path)
                ? "bg-primary text-primary-foreground shadow-sm"
                : "text-muted-foreground hover:bg-muted hover:text-foreground"
            )}
          >
            <Icon size={18} />
            <span>{t(key)}</span>
          </button>
        ))}
      </nav>

      {/* Bottom controls */}
      <div className="p-3 border-t border-border space-y-1">
        <button
          onClick={onToggleTheme}
          className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          {theme === "dark" ? <Sun size={18} /> : <Moon size={18} />}
          <span>{theme === "dark" ? t("userSettings.light") : t("userSettings.dark")}</span>
        </button>
        <button
          onClick={onToggleLang}
          className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          <Languages size={18} />
          <span>{currentLang === "zh" ? "English" : "中文"}</span>
        </button>
      </div>
    </aside>
  );
}

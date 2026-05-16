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
    <aside className="w-56 shrink-0 flex flex-col border-r border-border bg-card">
      {/* Brand */}
      <div className="h-20 flex items-center gap-3 px-5 border-b border-border">
        <img
          src="/logo.png"
          alt="gidy"
          className="w-10 h-10 rounded-xl ring-1 ring-border"
        />
        <div className="flex flex-col leading-tight">
          <span className="font-semibold text-sm tracking-wide">gidy client</span>
          <span className="text-[10px] text-muted-foreground mt-0.5">
            secure proxy tunnel
          </span>
        </div>
      </div>

      {/* Nav items */}
      <nav className="flex-1 py-4 px-3 space-y-0.5">
        {navItems.map(({ path, icon: Icon, key }) => {
          const active = isActive(path);
          return (
            <button
              key={path}
              onClick={() => navigate(path)}
              className={clsx(
                "relative w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-colors",
                active
                  ? "bg-muted text-foreground font-medium"
                  : "text-muted-foreground hover:bg-muted/60 hover:text-foreground"
              )}
            >
              <span
                className={clsx(
                  "absolute left-0 top-1/2 -translate-y-1/2 h-5 w-0.5 rounded-r-full bg-foreground transition-opacity",
                  active ? "opacity-100" : "opacity-0"
                )}
              />
              <Icon size={17} strokeWidth={1.75} />
              <span>{t(key)}</span>
            </button>
          );
        })}
      </nav>

      {/* Bottom controls */}
      <div className="p-3 border-t border-border space-y-0.5">
        <button
          onClick={onToggleTheme}
          className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          {theme === "dark" ? <Sun size={17} strokeWidth={1.75} /> : <Moon size={17} strokeWidth={1.75} />}
          <span>
            {theme === "dark" ? t("userSettings.light") : t("userSettings.dark")}
          </span>
        </button>
        <button
          onClick={onToggleLang}
          className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
        >
          <Languages size={17} strokeWidth={1.75} />
          <span>{currentLang === "zh" ? "English" : "中文"}</span>
        </button>
      </div>
    </aside>
  );
}

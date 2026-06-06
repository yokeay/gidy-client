import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { getConfig, updateConfig, GuiConfig } from "../api";

const GITHUB_REPO = "yokeay/gidy-client";

interface UserSettingsProps {
  theme: "light" | "dark";
  themeColor: string;
  onThemeChange: (t: "light" | "dark") => void;
  onThemeColorChange: (c: string) => void;
}

const APP_VERSION = "v0.3.0";

const SECTIONS = [
  { id: "general",  label: "基本设置" },
  { id: "proxy",    label: "代理设置" },
  { id: "update",   label: "更新检查" },
  { id: "language", label: "语言 / Language" },
  { id: "about",    label: "关于软件" },
];

function Toggle({ value, onChange }: { value: boolean; onChange: () => void }) {
  return (
    <button
      onClick={onChange}
      style={{
        position: "relative", width: 48, height: 24, borderRadius: 12, flexShrink: 0,
        background: value ? "var(--accent-green)" : "rgba(255,255,255,0.12)",
        border: "none", cursor: "pointer", transition: "background 0.2s",
      }}
    >
      <span style={{
        position: "absolute", top: 2, left: value ? 26 : 2,
        width: 20, height: 20, borderRadius: "50%", background: "#fff",
        boxShadow: "0 1px 4px rgba(0,0,0,0.3)", transition: "left 0.2s", display: "block",
      }} />
    </button>
  );
}

function Row({ label, sub, right }: { label: string; sub?: string; right: React.ReactNode }) {
  return (
    <div style={{
      display: "flex", alignItems: "center", justifyContent: "space-between",
      padding: "16px 0",
      borderBottom: "1px solid var(--border)",
    }}>
      <div>
        <div style={{ fontSize: 14, color: "var(--fg)" }}>{label}</div>
        {sub && <div style={{ fontSize: 12, color: "var(--muted-fg)", marginTop: 2 }}>{sub}</div>}
      </div>
      <div style={{ flexShrink: 0, marginLeft: 24 }}>{right}</div>
    </div>
  );
}

function SectionBlock({ id, title, children }: { id: string; title: string; children: React.ReactNode }) {
  return (
    <div id={id} style={{ marginBottom: 36 }}>
      <div style={{
        fontSize: 13, fontWeight: 600, color: "var(--accent-green)",
        letterSpacing: "0.06em", marginBottom: 12,
        paddingBottom: 8, borderBottom: "1px solid rgba(46,204,113,0.2)",
      }}>
        {title}
      </div>
      {children}
    </div>
  );
}

function GreenBtn({ onClick, children, disabled }: { onClick?: () => void; children: React.ReactNode; disabled?: boolean }) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        padding: "8px 18px", background: "var(--accent-green)", color: "#000",
        border: "none", borderRadius: 8, fontSize: 13, fontWeight: 600,
        cursor: "pointer", whiteSpace: "nowrap",
        boxShadow: "0 0 12px rgba(46,204,113,0.25)",
        transition: "all 0.15s", opacity: disabled ? 0.5 : 1,
        fontFamily: "var(--font-ui)",
      }}
    >
      {children}
    </button>
  );
}

type UpdateState =
  | { status: "idle" }
  | { status: "checking" }
  | { status: "latest"; version: string }
  | { status: "available"; current: string; latest: string; url: string }
  | { status: "error"; message: string };

export default function UserSettings(_props: UserSettingsProps) {
  const { i18n } = useTranslation();
  const [config, setConfig] = useState<GuiConfig | null>(null);
  const [savedMsg, setSavedMsg] = useState(false);
  const [kernelPath, setKernelPath] = useState("/usr/local/bin/gidy-core");
  const [activeSection, setActiveSection] = useState("general");
  const contentRef = useRef<HTMLDivElement>(null);
  const [updateState, setUpdateState] = useState<UpdateState>({ status: "idle" });
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    getConfig().then(c => {
      setConfig(c);
    }).catch(() => {});
  }, []);

  // Auto-save: whenever config changes, debounce 600ms then save
  useEffect(() => {
    if (!config) return;
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(async () => {
      try {
        await updateConfig(config);
        setSavedMsg(true);
        setTimeout(() => setSavedMsg(false), 1800);
      } catch {}
    }, 600);
    return () => { if (saveTimer.current) clearTimeout(saveTimer.current); };
  }, [config]);

  const updateField = useCallback((key: keyof GuiConfig, value: string | number | boolean) => {
    if (!config) return;
    setConfig({ ...config, [key]: value });
  }, [config]);

  const toggle = useCallback((key: keyof GuiConfig) => {
    if (!config) return;
    setConfig({ ...config, [key]: !config[key] });
  }, [config]);

  const handleBrowseKernel = async () => {
    try {
      const selected = await open({ directory: false, multiple: false, title: "选择内核文件" });
      if (selected) setKernelPath(selected as string);
    } catch {}
  };

  const scrollTo = (id: string) => {
    setActiveSection(id);
    const el = contentRef.current?.querySelector(`#${id}`);
    el?.scrollIntoView({ behavior: "smooth", block: "start" });
  };

  const checkUpdate = async () => {
    setUpdateState({ status: "checking" });
    try {
      const res = await fetch(
        `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`,
        { headers: { Accept: "application/vnd.github+json" } }
      );
      if (!res.ok) throw new Error(`GitHub API ${res.status}`);
      const data = await res.json() as { tag_name: string; html_url: string };
      const latest = data.tag_name;
      if (latest === APP_VERSION || latest === APP_VERSION.replace(/^v/, "").replace(/^/, "v")) {
        setUpdateState({ status: "latest", version: latest });
      } else {
        setUpdateState({ status: "available", current: APP_VERSION, latest, url: data.html_url });
      }
    } catch (e) {
      setUpdateState({ status: "error", message: String(e) });
    }
  };

  if (!config) return null;

  return (
    <div style={{ display: "flex", gap: 0, height: "100%", minHeight: 0 }}>
      {/* Left TOC */}
      <div style={{
        width: 160, flexShrink: 0, paddingRight: 20,
        borderRight: "1px solid var(--border)",
        paddingTop: 4,
      }}>
        {SECTIONS.map(s => (
          <div
            key={s.id}
            onClick={() => scrollTo(s.id)}
            style={{
              padding: "9px 12px",
              borderRadius: 7,
              fontSize: 13,
              cursor: "pointer",
              marginBottom: 2,
              color: activeSection === s.id ? "var(--accent-green)" : "var(--muted-fg)",
              background: activeSection === s.id ? "rgba(46,204,113,0.1)" : "transparent",
              fontWeight: activeSection === s.id ? 500 : 400,
              borderLeft: activeSection === s.id ? "2px solid var(--accent-green)" : "2px solid transparent",
              transition: "all 0.15s",
              userSelect: "none",
            }}
            onMouseEnter={e => {
              if (activeSection !== s.id) (e.currentTarget as HTMLElement).style.color = "var(--fg)";
            }}
            onMouseLeave={e => {
              if (activeSection !== s.id) (e.currentTarget as HTMLElement).style.color = "var(--muted-fg)";
            }}
          >
            {s.label}
          </div>
        ))}
      </div>

      {/* Right content */}
      <div
        ref={contentRef}
        className="scroll-thin"
        style={{ flex: 1, overflowY: "auto", paddingLeft: 28, paddingRight: 4, paddingTop: 4 }}
      >
        {/* 基本设置 */}
        <SectionBlock id="general" title="基本设置">
          <Row
            label="开机自动启动"
            sub="系统启动时自动运行 gidy-client"
            right={<Toggle value={config.auto_start} onChange={() => toggle("auto_start")} />}
          />
          <Row
            label="自动连接"
            sub="启动后自动建立代理连接"
            right={<Toggle value={config.auto_connect} onChange={() => toggle("auto_connect")} />}
          />
          <Row
            label="最小化到系统托盘"
            sub="关闭窗口时最小化而非退出"
            right={<Toggle value={config.minimize_to_tray} onChange={() => toggle("minimize_to_tray")} />}
          />
          <Row
            label="日志保留天数"
            right={
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <input
                  type="number"
                  value={config.log_retention_days}
                  onChange={e => updateField("log_retention_days", parseInt(e.target.value) || 7)}
                  style={{
                    width: 60, padding: "6px 8px", textAlign: "center",
                    background: "var(--muted)", border: "1px solid var(--border)",
                    borderRadius: 6, fontSize: 13, color: "var(--fg)",
                    fontFamily: "var(--font-mono)", outline: "none",
                  }}
                />
                <span style={{ fontSize: 12, color: "var(--muted-fg)" }}>天</span>
              </div>
            }
          />
        </SectionBlock>

        {/* 代理设置 */}
        <SectionBlock id="proxy" title="代理设置">
          <Row
            label="内核路径"
            sub="gidy-core 可执行文件路径"
            right={
              <div style={{ display: "flex", alignItems: "center", gap: 0, background: "var(--card)", border: "1px solid var(--border)", borderRadius: 7, overflow: "hidden" }}>
                <input
                  type="text"
                  value={kernelPath}
                  onChange={e => setKernelPath(e.target.value)}
                  style={{
                    width: 220, padding: "7px 12px",
                    fontFamily: "var(--font-mono)", fontSize: 12,
                    color: "var(--muted-fg)", background: "transparent",
                    border: "none", outline: "none",
                  }}
                />
                <button onClick={handleBrowseKernel} style={{
                  padding: "7px 12px", background: "transparent",
                  border: "none", borderLeft: "1px solid var(--border)",
                  color: "var(--muted-fg)", fontSize: 12, cursor: "pointer",
                  fontFamily: "var(--font-ui)", whiteSpace: "nowrap",
                }}>
                  浏览
                </button>
              </div>
            }
          />
          <Row
            label="代理模式"
            sub="全局模式：所有流量走代理；PAC 模式：按规则分流"
            right={
              <div style={{ display: "flex", gap: 8 }}>
                {["global", "pac"].map(m => (
                  <button
                    key={m}
                    onClick={() => updateField("mode", m)}
                    style={{
                      padding: "7px 16px", borderRadius: 7, fontSize: 12, fontWeight: 500,
                      cursor: "pointer", fontFamily: "var(--font-ui)",
                      border: config.mode === m ? "1px solid rgba(46,204,113,0.5)" : "1px solid var(--border)",
                      background: config.mode === m ? "rgba(46,204,113,0.15)" : "var(--card)",
                      color: config.mode === m ? "var(--accent-green)" : "var(--muted-fg)",
                      transition: "all 0.15s",
                    }}
                  >
                    {m === "global" ? "全局" : "PAC"}
                  </button>
                ))}
              </div>
            }
          />
        </SectionBlock>

        {/* 更新检查 */}
        <SectionBlock id="update" title="更新检查">
          <Row
            label="当前版本"
            sub={APP_VERSION}
            right={
              <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 6 }}>
                <GreenBtn onClick={checkUpdate} disabled={updateState.status === "checking"}>
                  {updateState.status === "checking" ? "检查中..." : "检查更新"}
                </GreenBtn>
                {updateState.status === "latest" && (
                  <span style={{ fontSize: 12, color: "var(--accent-green)" }}>
                    ✓ 已是最新版本 {updateState.version}
                  </span>
                )}
                {updateState.status === "available" && (
                  <span style={{ fontSize: 12, color: "#f39c12" }}>
                    发现新版本 {updateState.latest}，{" "}
                    <a
                      href={updateState.url}
                      target="_blank"
                      rel="noreferrer"
                      style={{ color: "var(--accent-blue)", textDecoration: "none" }}
                    >
                      前往下载 ↗
                    </a>
                  </span>
                )}
                {updateState.status === "error" && (
                  <span style={{ fontSize: 12, color: "#e74c3c" }}>
                    检查失败，请重试
                  </span>
                )}
              </div>
            }
          />
        </SectionBlock>

        {/* 语言 */}
        <SectionBlock id="language" title="语言 / Language">
          <Row
            label="界面语言"
            sub="Interface Language"
            right={
              <div style={{ display: "flex", gap: 8 }}>
                {[{ code: "zh", label: "中文" }, { code: "en", label: "English" }].map(l => (
                  <button
                    key={l.code}
                    onClick={() => i18n.changeLanguage(l.code)}
                    style={{
                      padding: "7px 18px", borderRadius: 7, fontSize: 13, fontWeight: 500,
                      cursor: "pointer", fontFamily: "var(--font-ui)",
                      border: i18n.language === l.code ? "1px solid rgba(46,204,113,0.5)" : "1px solid var(--border)",
                      background: i18n.language === l.code ? "rgba(46,204,113,0.15)" : "var(--card)",
                      color: i18n.language === l.code ? "var(--accent-green)" : "var(--muted-fg)",
                      transition: "all 0.15s",
                    }}
                  >
                    {l.label}
                  </button>
                ))}
              </div>
            }
          />
        </SectionBlock>

        {/* 关于软件 */}
        <SectionBlock id="about" title="关于软件">
          <div style={{
            background: "var(--card)", border: "1px solid var(--border)",
            borderRadius: 10, padding: "20px 24px",
          }}>
            <div style={{ display: "flex", alignItems: "center", gap: 14, marginBottom: 20 }}>
              <div style={{
                width: 44, height: 44, borderRadius: 10,
                background: "var(--accent-green)",
                display: "flex", alignItems: "center", justifyContent: "center",
                fontSize: 22, fontWeight: 700, color: "#000",
                boxShadow: "0 0 14px rgba(46,204,113,0.4)",
              }}>
                G
              </div>
              <div>
                <div style={{ fontSize: 16, fontWeight: 700, color: "var(--fg)" }}>Gidy-Client</div>
                <div style={{ fontSize: 12, color: "var(--muted-fg)", fontFamily: "var(--font-mono)", marginTop: 2 }}>
                  {APP_VERSION}
                </div>
              </div>
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 10, fontSize: 13, color: "var(--muted-fg)" }}>
              {[
                "基于 QUIC 协议的安全代理客户端，支持协议混淆",
                "Tauri + React + Rust 构建，跨平台桌面应用",
                "安全的端对端加密代理隧道",
              ].map((t, i) => (
                <div key={i} style={{ display: "flex", gap: 10 }}>
                  <span style={{ color: "var(--accent-green)", flexShrink: 0 }}>▸</span>
                  <span>{t}</span>
                </div>
              ))}
            </div>
            <div style={{ marginTop: 20, paddingTop: 16, borderTop: "1px solid var(--border)" }}>
              <a
                href="https://github.com/yokeay/gidy-client"
                target="_blank" rel="noreferrer"
                style={{ fontSize: 12, color: "var(--accent-blue)", fontFamily: "var(--font-mono)", textDecoration: "none" }}
              >
                github.com/yokeay/gidy-client ↗
              </a>
            </div>
          </div>
        </SectionBlock>

        {/* Auto-save indicator */}
        {savedMsg && (
          <div style={{ textAlign: "right", paddingBottom: 16, fontSize: 12, color: "var(--accent-green)" }}>
            ✓ 已自动保存
          </div>
        )}
      </div>
    </div>
  );
}

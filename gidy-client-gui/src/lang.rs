use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            let lang_id = unsafe {
                windows::Win32::Globalization::GetUserDefaultUILanguage()
            };
            if lang_id & 0x3FF == 0x04 {
                return Lang::Zh;
            }
        }
        Lang::En
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKey {
    // Tab labels
    TrafficMonitor,
    Settings,
    SystemConfig,
    About,
    // Status
    Stopping,
    ProxyStopped,
    Starting,
    ProxyStarted,
    Ready,
    Connected,
    Disconnected,
    Start,
    Stop,
    // Traffic monitor
    UploadRate,
    DownloadRate,
    TotalUpload,
    TotalDownload,
    Uptime,
    EventLog,
    Status,
    // Config form
    PskLabel,
    PskHint,
    ServerAddrLabel,
    SniLabel,
    ListenAddrLabel,
    LogLevelLabel,
    Language,
    Save,
    Cancel,
    // System config
    AutoStartBoot,
    AutoConnect,
    MinimizeToTray,
    // About
    Version,
    AboutDescription,
    // Common
    Title,
    TrayTooltip,
    TrayShowWindow,
    TrayExit,
    CloseDialogTitle,
    CloseDialogText,
    CloseDialogYes,
    CloseDialogNo,
}

impl Lang {
    pub fn text(self, key: TextKey) -> &'static str {
        match self {
            Lang::En => match key {
                TextKey::TrafficMonitor => "Traffic Monitor",
                TextKey::Settings => "Settings",
                TextKey::SystemConfig => "System Config",
                TextKey::About => "About",
                TextKey::Stopping => "Stopping proxy...",
                TextKey::ProxyStopped => "Proxy stopped",
                TextKey::Starting => "Starting proxy...",
                TextKey::ProxyStarted => "Proxy started",
                TextKey::Ready => "Ready",
                TextKey::Connected => "Connected",
                TextKey::Disconnected => "Disconnected",
                TextKey::Start => "Start",
                TextKey::Stop => "Stop",
                TextKey::UploadRate => "Upload Rate",
                TextKey::DownloadRate => "Download Rate",
                TextKey::TotalUpload => "Upload:",
                TextKey::TotalDownload => "Download:",
                TextKey::Uptime => "Uptime:",
                TextKey::EventLog => "Event Log",
                TextKey::Status => "Status",
                TextKey::PskLabel => "PSK Key",
                TextKey::PskHint => "64 hex characters",
                TextKey::ServerAddrLabel => "Server Address",
                TextKey::SniLabel => "SNI / Server Name",
                TextKey::ListenAddrLabel => "Listen Address",
                TextKey::LogLevelLabel => "Log Level",
                TextKey::Language => "Language",
                TextKey::Save => "Save",
                TextKey::Cancel => "Cancel",
                TextKey::AutoStartBoot => "Auto-start on boot",
                TextKey::AutoConnect => "Auto-connect on start",
                TextKey::MinimizeToTray => "Minimize to system tray on close",
                TextKey::Version => "Version",
                TextKey::AboutDescription => "A secure QUIC-based proxy client",
                TextKey::Title => "gidy client",
                TextKey::TrayTooltip => "gidy client",
                TextKey::TrayShowWindow => "Show Window",
                TextKey::TrayExit => "Exit",
                TextKey::CloseDialogTitle => "Close Confirmation",
                TextKey::CloseDialogText => "Minimize to system tray?",
                TextKey::CloseDialogYes => "Yes (Minimize)",
                TextKey::CloseDialogNo => "No (Exit)",
            },
            Lang::Zh => match key {
                TextKey::TrafficMonitor => "流量监测",
                TextKey::Settings => "设置",
                TextKey::SystemConfig => "系统配置",
                TextKey::About => "关于",
                TextKey::Stopping => "正在停止代理...",
                TextKey::ProxyStopped => "代理已停止",
                TextKey::Starting => "正在启动代理...",
                TextKey::ProxyStarted => "代理已启动",
                TextKey::Ready => "就绪",
                TextKey::Connected => "已连接",
                TextKey::Disconnected => "未连接",
                TextKey::Start => "启动",
                TextKey::Stop => "停止",
                TextKey::UploadRate => "上传速率",
                TextKey::DownloadRate => "下载速率",
                TextKey::TotalUpload => "总上传：",
                TextKey::TotalDownload => "总下载：",
                TextKey::Uptime => "运行时间：",
                TextKey::EventLog => "事件日志",
                TextKey::Status => "状态",
                TextKey::PskLabel => "PSK 密钥",
                TextKey::PskHint => "64位十六进制字符",
                TextKey::ServerAddrLabel => "服务器地址",
                TextKey::SniLabel => "SNI / 服务器名称",
                TextKey::ListenAddrLabel => "本地监听地址",
                TextKey::LogLevelLabel => "日志级别",
                TextKey::Language => "语言",
                TextKey::Save => "保存",
                TextKey::Cancel => "取消",
                TextKey::AutoStartBoot => "开机自启",
                TextKey::AutoConnect => "启动时自动连接",
                TextKey::MinimizeToTray => "关闭时最小化到系统托盘",
                TextKey::Version => "版本",
                TextKey::AboutDescription => "基于 QUIC 的安全代理客户端",
                TextKey::Title => "gidy client",
                TextKey::TrayTooltip => "gidy client",
                TextKey::TrayShowWindow => "显示主界面",
                TextKey::TrayExit => "退出程序",
                TextKey::CloseDialogTitle => "关闭确认",
                TextKey::CloseDialogText => "是否最小化到系统托盘？",
                TextKey::CloseDialogYes => "是（最小化）",
                TextKey::CloseDialogNo => "否（退出）",
            },
        }
    }
}

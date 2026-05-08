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
    Ready,
    Stopping,
    Stopped,
    ProxyStopped,
    Starting,
    Connecting,
    ProxyStarted,
    PskLabel,
    PskHint,
    ServerAddrLabel,
    SniLabel,
    ListenAddrLabel,
    LogLevelLabel,
    ConfigTab,
    MonitorTab,
    TotalUpload,
    TotalDownload,
    Uptime,
    EventLog,
    Title,
    Language,
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
                TextKey::Ready => "Ready",
                TextKey::Stopping => "Stopping proxy...",
                TextKey::Stopped => "Stopped",
                TextKey::ProxyStopped => "Proxy stopped",
                TextKey::Starting => "Starting proxy...",
                TextKey::Connecting => "Connecting...",
                TextKey::ProxyStarted => "Proxy started",
                TextKey::PskLabel => "PSK Key",
                TextKey::PskHint => "64 hex characters",
                TextKey::ServerAddrLabel => "Server Address",
                TextKey::SniLabel => "SNI / Server Name",
                TextKey::ListenAddrLabel => "Listen Address",
                TextKey::LogLevelLabel => "Log Level",
                TextKey::ConfigTab => "Config",
                TextKey::MonitorTab => "Monitor",
                TextKey::TotalUpload => "Upload:",
                TextKey::TotalDownload => "Download:",
                TextKey::Uptime => "Uptime:",
                TextKey::EventLog => "Event Log",
                TextKey::Title => "gidy client",
                TextKey::Language => "Language",
                TextKey::TrayTooltip => "gidy client",
                TextKey::TrayShowWindow => "Show Window",
                TextKey::TrayExit => "Exit",
                TextKey::CloseDialogTitle => "Close Confirmation",
                TextKey::CloseDialogText => "Minimize to system tray?",
                TextKey::CloseDialogYes => "Yes",
                TextKey::CloseDialogNo => "No",
            },
            Lang::Zh => match key {
                TextKey::Ready => "准备就绪",
                TextKey::Stopping => "正在停止代理...",
                TextKey::Stopped => "已停止",
                TextKey::ProxyStopped => "代理已停止",
                TextKey::Starting => "正在启动代理...",
                TextKey::Connecting => "连接中...",
                TextKey::ProxyStarted => "代理已启动",
                TextKey::PskLabel => "PSK 密钥",
                TextKey::PskHint => "64位十六进制字符",
                TextKey::ServerAddrLabel => "服务器地址",
                TextKey::SniLabel => "SNI / 服务器名称",
                TextKey::ListenAddrLabel => "本地监听地址",
                TextKey::LogLevelLabel => "日志级别",
                TextKey::ConfigTab => "系统配置",
                TextKey::MonitorTab => "流量监测",
                TextKey::TotalUpload => "总上传：",
                TextKey::TotalDownload => "总下载：",
                TextKey::Uptime => "运行时间：",
                TextKey::EventLog => "事件日志",
                TextKey::Title => "gidy client",
                TextKey::Language => "语言",
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

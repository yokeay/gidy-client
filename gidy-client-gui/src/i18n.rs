#[derive(Clone, Copy, PartialEq)]
pub enum Lang { Zh, En }

impl Lang {
    pub fn toggle(self) -> Self {
        match self { Lang::Zh => Lang::En, Lang::En => Lang::Zh }
    }
}

pub struct I18n { lang: Lang }

impl I18n {
    pub fn new() -> Self { I18n { lang: Lang::Zh } }
    pub fn lang(&self) -> Lang { self.lang }
    pub fn set_lang(&mut self, l: Lang) { self.lang = l; }

    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        match self.lang {
            Lang::Zh => Self::zh(key),
            Lang::En => Self::en(key),
        }
    }

    fn zh<'a>(key: &'a str) -> &'a str {
        match key {
            "nav.dashboard" => "仪表盘",
            "nav.systemConfig" => "系统配置",
            "nav.trafficMonitor" => "流量监测",
            "nav.userSettings" => "用户设置",
            "nav.about" => "关于我们",
            "dashboard.version" => "版本",
            "dashboard.connected" => "已连接",
            "dashboard.disconnected" => "未连接",
            "dashboard.uploadSpeed" => "上传速率",
            "dashboard.downloadSpeed" => "下载速率",
            "dashboard.totalUpload" => "总上传",
            "dashboard.totalDownload" => "总下载",
            "dashboard.dnsElapsed" => "DNS 耗时",
            "dashboard.serviceUptime" => "服务运行时间",
            "dashboard.proxyConnections" => "代理连接数",
            "dashboard.start" => "启动",
            "dashboard.stop" => "停止",
            "systemConfig.title" => "系统配置",
            "systemConfig.proxyServer" => "代理服务器",
            "systemConfig.serverAddr" => "服务器地址",
            "systemConfig.serverPort" => "端口",
            "systemConfig.psk" => "PSK 预共享密钥",
            "systemConfig.pskHint" => "64位十六进制字符",
            "systemConfig.protocol" => "协议",
            "systemConfig.saveAndConnect" => "保存并连接",
            "systemConfig.localProxy" => "本地代理",
            "systemConfig.socks5Addr" => "SOCKS5 地址",
            "systemConfig.socks5Port" => "SOCKS5 端口",
            "systemConfig.httpAddr" => "HTTP 地址",
            "systemConfig.httpPort" => "HTTP 端口",
            "systemConfig.mode" => "模式",
            "systemConfig.globalMode" => "全局模式",
            "systemConfig.pacMode" => "PAC 模式",
            "systemConfig.connectionNote" => "连接说明",
            "systemConfig.connectionNoteText" => "启动代理后，系统将自动配置代理设置。全局模式下所有流量通过代理服务器转发；PAC 模式下仅代理名单内的流量。",
            "systemConfig.generatePsk" => "生成随机 PSK",
            "systemConfig.saved" => "配置已保存",
            "systemConfig.saveFailed" => "保存失败",
            "trafficMonitor.title" => "流量监测",
            "trafficMonitor.upload" => "上传",
            "trafficMonitor.download" => "下载",
            "trafficMonitor.totalUpload" => "总上传",
            "trafficMonitor.totalDownload" => "总下载",
            "trafficMonitor.uptime" => "运行时间",
            "trafficMonitor.connectionLog" => "连接日志",
            "trafficMonitor.time" => "时间",
            "trafficMonitor.target" => "目标地址",
            "trafficMonitor.type" => "类型",
            "trafficMonitor.size" => "大小",
            "trafficMonitor.duration" => "耗时",
            "trafficMonitor.noData" => "暂无数据",
            "userSettings.title" => "用户设置",
            "userSettings.basicSettings" => "基本设置",
            "userSettings.autoStart" => "开机自动启动",
            "userSettings.autoConnect" => "自动连接",
            "userSettings.minimizeToTray" => "启动后最小化到系统托盘",
            "userSettings.logRetention" => "日志保留天数",
            "userSettings.days" => "天",
            "userSettings.themeMode" => "主题模式",
            "userSettings.light" => "浅色",
            "userSettings.dark" => "深色",
            "userSettings.themeColor" => "主题色",
            "userSettings.updateCheck" => "更新检查",
            "userSettings.currentVersion" => "当前版本",
            "userSettings.latestVersion" => "最新版本",
            "userSettings.checkUpdate" => "检查更新",
            "userSettings.saveConfig" => "保存配置",
            "userSettings.saved" => "设置已保存",
            "common.save" => "保存",
            "common.cancel" => "取消",
            "common.error" => "错误",
            "common.loading" => "加载中...",
            "common.gidyClient" => "gidy client",
            _ => key,
        }
    }

    fn en<'a>(key: &'a str) -> &'a str {
        match key {
            "nav.dashboard" => "Dashboard",
            "nav.systemConfig" => "System Config",
            "nav.trafficMonitor" => "Traffic Monitor",
            "nav.userSettings" => "User Settings",
            "nav.about" => "About",
            "dashboard.version" => "Version",
            "dashboard.connected" => "Connected",
            "dashboard.disconnected" => "Disconnected",
            "dashboard.uploadSpeed" => "Upload Speed",
            "dashboard.downloadSpeed" => "Download Speed",
            "dashboard.totalUpload" => "Total Upload",
            "dashboard.totalDownload" => "Total Download",
            "dashboard.dnsElapsed" => "DNS Elapsed",
            "dashboard.serviceUptime" => "Service Uptime",
            "dashboard.proxyConnections" => "Proxy Connections",
            "dashboard.start" => "Start",
            "dashboard.stop" => "Stop",
            "systemConfig.title" => "System Config",
            "systemConfig.proxyServer" => "Proxy Server",
            "systemConfig.serverAddr" => "Server Address",
            "systemConfig.serverPort" => "Port",
            "systemConfig.psk" => "PSK Key",
            "systemConfig.pskHint" => "64 hex characters",
            "systemConfig.protocol" => "Protocol",
            "systemConfig.saveAndConnect" => "Save & Connect",
            "systemConfig.localProxy" => "Local Proxy",
            "systemConfig.socks5Addr" => "SOCKS5 Address",
            "systemConfig.socks5Port" => "SOCKS5 Port",
            "systemConfig.httpAddr" => "HTTP Address",
            "systemConfig.httpPort" => "HTTP Port",
            "systemConfig.mode" => "Mode",
            "systemConfig.globalMode" => "Global Mode",
            "systemConfig.pacMode" => "PAC Mode",
            "systemConfig.connectionNote" => "Connection Note",
            "systemConfig.connectionNoteText" => "After starting the proxy, system proxy settings will be configured automatically. In Global mode, all traffic goes through the proxy server. In PAC mode, only listed traffic is proxied.",
            "systemConfig.generatePsk" => "Generate Random PSK",
            "systemConfig.saved" => "Configuration saved",
            "systemConfig.saveFailed" => "Save failed",
            "trafficMonitor.title" => "Traffic Monitor",
            "trafficMonitor.upload" => "Upload",
            "trafficMonitor.download" => "Download",
            "trafficMonitor.totalUpload" => "Total Upload",
            "trafficMonitor.totalDownload" => "Total Download",
            "trafficMonitor.uptime" => "Uptime",
            "trafficMonitor.connectionLog" => "Connection Log",
            "trafficMonitor.time" => "Time",
            "trafficMonitor.target" => "Target",
            "trafficMonitor.type" => "Type",
            "trafficMonitor.size" => "Size",
            "trafficMonitor.duration" => "Duration",
            "trafficMonitor.noData" => "No data",
            "userSettings.title" => "User Settings",
            "userSettings.basicSettings" => "Basic Settings",
            "userSettings.autoStart" => "Auto-start on boot",
            "userSettings.autoConnect" => "Auto-connect on launch",
            "userSettings.minimizeToTray" => "Minimize to system tray",
            "userSettings.logRetention" => "Log retention",
            "userSettings.days" => "days",
            "userSettings.themeMode" => "Theme Mode",
            "userSettings.light" => "Light",
            "userSettings.dark" => "Dark",
            "userSettings.themeColor" => "Theme Color",
            "userSettings.updateCheck" => "Update Check",
            "userSettings.currentVersion" => "Current Version",
            "userSettings.latestVersion" => "Latest Version",
            "userSettings.checkUpdate" => "Check for Updates",
            "userSettings.saveConfig" => "Save Configuration",
            "userSettings.saved" => "Settings saved",
            "common.save" => "Save",
            "common.cancel" => "Cancel",
            "common.error" => "Error",
            "common.loading" => "Loading...",
            "common.gidyClient" => "gidy client",
            _ => key,
        }
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 { return format!("{:.2} GB", bytes as f64 / 1_073_741_824.0); }
    if bytes >= 1_048_576 { return format!("{:.2} MB", bytes as f64 / 1_048_576.0); }
    if bytes >= 1024 { return format!("{:.2} KB", bytes as f64 / 1024.0); }
    format!("{} B", bytes)
}

pub fn format_speed(kbps: f64) -> String {
    if kbps >= 1000.0 { return format!("{:.1} Mbps", kbps / 1000.0); }
    format!("{:.1} Kbps", kbps)
}

pub fn format_uptime(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 { return format!("{}:{:02}:{:02}", h, m, s); }
    if m > 0 { return format!("{}:{:02}", m, s); }
    format!("{}s", s)
}

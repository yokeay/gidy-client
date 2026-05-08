## v0.2.3 - 2026-05-08

### 变更内容
- 将 gidy-client-gui 从 egui/eframe 重写为 Tauri v2 + React/TypeScript
- 新增仪表盘页面（Dashboard）：版本/状态横幅、上传/下载速率卡片、速率图表、DNS/运行时间/连接数指标
- 新增系统配置页面（SystemConfig）：代理服务器设置、本地代理设置、全局/PAC 模式切换、PSK 生成
- 新增流量监测页面（TrafficMonitor）：实时速率、流量图表、连接日志表格
- 新增用户设置页面（UserSettings）：开关设置、主题模式/颜色、更新检查
- 新增关于页面（About）：版本信息与技术栈说明
- i18n 国际化支持（中/英文）
- 侧边栏导航、明暗主题切换
- Rust 后端 Tauri 命令集成（connect/disconnect/get_stats/get_config/update_config/get_status/generate_psk）
- CLI 客户端连通性测试通过：HTTP/HTTPS 代理正常，文件下载正常

### 影响范围
- gidy-client-gui 前端（React/TypeScript）
- gidy-client-gui/src-tauri Rust 后端
- 工作空间 Cargo.toml（members 路径更新）
- gidy-client-cli 测试验证通过

### 功能列表
- 仪表盘：启停代理、实时速率显示、流量图表
- 系统配置：代理服务器/本地代理设置、模式切换
- 流量监测：实时监控、连接日志
- 用户设置：主题切换、开关设置、更新检查
- i18n：中文/英文切换

## v0.2.5 - 2026-05-09

### 变更内容
- 修复默认服务器端口配置（4433 → 4434）与服务器实际端口对齐
- 修复 gidy-core generate_psk() panic（nanos 位移后类型不匹配）
- 流量监测页面新增圆形速度表盘组件（Speedometer），匹配 UI 稿件
- 用户设置页面新增独立语言切换器
- Dashboard 新增连接错误提示
- 更新版本号引用至 v0.2.5
- 前端 build / Rust build / Tauri build 均通过

### 影响范围
- gidy-core src/lib.rs（PSK 生成修复）
- gidy-client-gui/src-tauri/src/commands.rs（默认端口）
- gidy-client-gui/src/pages/（Dashboard, TrafficMonitor, UserSettings, About）
- gidy-client-gui/src/components/Speedometer.tsx（新增）

### 功能列表
- 速度表盘实时显示代理速率
- 连接错误前端提示
- 默认端口匹配服务器实际配置

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

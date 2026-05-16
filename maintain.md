## v0.2.7 - 2026-05-16 · Android Stage 1（入库 + CI）

### 变更内容
- gidy-android/ Compose UI shell 入库（Kotlin 2.0 + Material3，UI 与 GUI 黑白灰主题一致）
  - 5 个屏幕：Dashboard / SystemConfig / TrafficMonitor / UserSettings / About
  - 8 个组件：Speedometer / SpeedChart / KpiCard / AppleCard / AppleSwitch / SegmentedToggle / SectionHeader / StatusBadge
  - DataStore Preferences 骨架（ConfigDataStore + Config + MockStats）
  - Navigation Compose 底部导航 + 中/英 i18n（values/strings.xml + values-zh/strings.xml）
  - Apple-like 主题（Theme.kt / Color.kt / Shape.kt / Type.kt），含 light/dark
- 新增 gidy-android/.gitignore（覆盖 .gradle/build/keystore/.aab/.apk/local.properties 等，强化签名密钥屏蔽）
- 新增 .github/workflows/android.yml（push 触发：lintDebug + assembleDebug + 上传 APK & lint 报告 artifacts，仅在 gidy-android/** 变更时触发）
- 本地不执行 build；CI 在 ubuntu-latest 上跑 JDK 17 + Android SDK，按需自动生成 Gradle wrapper
- 更新 plan.md：新增 Android 客户端四阶段任务，Stage 1 标记完成

### 影响范围
- gidy-android/（新建，全量）
- .github/workflows/android.yml（新建）
- plan.md
- maintain.md

### 功能列表
- Android UI shell 完整入库可被 IDE 直接打开
- CI 自动产出 unsigned debug APK 作为 artifacts（保留 14 天）
- Android 与 Rust/Tauri CI 工作流解耦，互不触发

### 待办（Stage 2 起）
- VpnService 前台服务 + tun 接管
- DataStore 真实读写替换 Mock
- 连接开关、流量统计实采
- 集成 gidy-client-core（JNI/UniFFI）


## v0.2.6 - 2026-05-16

### 变更内容
- GUI 全面 1:1 复刻重构（按 docs/ 设计稿，仅换皮，不动 API/路由/Tauri 逻辑）
- index.css 主题变量重写：纯黑白灰（zinc/neutral 系），强化 light/dark；新增 .tabular 等宽数字工具类与 scroll-thin 滚动条
- Sidebar：w-56，h-20 Logo 区+slogan，菜单项左侧高亮指示条
- Speedometer 重写：270° 表盘 + 主次刻度 + tabular 中央数字
- SpeedChart：双 Area 图，download 实线 / upload 虚线，纯黑白配色
- TrafficMonitor：顶部 4 列 KPI（实时/累计 上/下行）+ 表盘(col-span-1)与图表(col-span-2)并排 + 连接日志表
- SystemConfig：新增连接状态徽章（脉冲点 + connected/disconnected），卡片统一 rounded-2xl
- UserSettings：卡片 rounded-2xl，设置项视觉更克制
- Dashboard：Banner 去翠绿，改 bg-foreground/text-background 反白卡，按钮统一
- About：rounded-2xl 卡片 + Repository 链接
- App.tsx：header h-14 px-8 + main px-8 py-6 scroll-thin
- 窗口默认 1100x720，最小 980x640（容纳 4 列 KPI）
- 版本号同步 0.2.3 → 0.2.6（package.json / tauri.conf.json / About 常量）
- .gitignore 新增 .claude/ 与 clod.sh

### 影响范围
- gidy-client-gui/src/index.css
- gidy-client-gui/src/App.tsx
- gidy-client-gui/src/components/（Sidebar, Speedometer, SpeedChart）
- gidy-client-gui/src/pages/（Dashboard, TrafficMonitor, SystemConfig, UserSettings, About）
- gidy-client-gui/package.json（version）
- gidy-client-gui/src-tauri/tauri.conf.json（version + 窗口尺寸）
- docs/（设计稿入仓）
- .gitignore

### 功能列表
- 完整黑白灰主题（light/dark）一致性
- 等宽数字渲染（流量数字 / 时间 / 端口等）
- 连接状态可视化徽章（脉冲点）
- 4 列流量 KPI + 仪表盘/趋势图并排

### 已知告警
- 项目根目录 clod.sh 存在硬编码 OPENAI_API_KEY / ANTHROPIC_AUTH_TOKEN（违反 SECTION 7 / 18）；已加入 .gitignore 防止误提交，但**仍建议旋转 Token 并迁移至 .env**

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

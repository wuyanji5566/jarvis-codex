# Windows 10 跨平台改造计划

> 本文基于当前 main 分支代码审计编写。目标是新增 Windows 10 22H2 x64 支持，同时保留 macOS 构建、现有 Jarvis UI、Codex app-server、Voice、线程续接、权限模式和 STOP 主链路。

## 1. 当前目录结构

~~~text
.
├─ .github/workflows/ci.yml
├─ docs/ARCHITECTURE.md
├─ public/assets/
├─ scripts/build-wake-helper.sh
├─ src/main.ts
├─ src/style.css
├─ src-tauri/
│  ├─ capabilities/default.json
│  ├─ src/lib.rs
│  ├─ src/main.rs
│  ├─ wake-helper/JarvisWakeListener.swift
│  ├─ Cargo.toml
│  ├─ Entitlements.plist
│  ├─ Info.plist
│  └─ tauri.conf.json
├─ tests/wav.test.mjs
├─ package.json
├─ tsconfig.json
└─ vite.config.ts
~~~

前端只有一个主要入口 src/main.ts；Rust 后端目前把 Codex RPC、线程生命周期、唤醒助手、平台路径、窗口激活和权限预检集中在 src-tauri/src/lib.rs。

## 2. macOS 专用代码清单

- Cargo.toml 使用 macos-private-api，并通过 macOS 条件依赖引入 Objective-C、AVFoundation。
- lib.rs 使用 /Applications、Homebrew、open、pkill、kill、HOME 和 .app 资源路径。
- lib.rs 的窗口激活包含 NSApplication；麦克风预检使用 AVCaptureDevice。
- lib.rs 的唤醒监督器假定 JarvisWakeListener.app、LaunchServices 和 JSONL 事件文件。
- lib.rs 的 autostart 初始化使用 MacosLauncher。
- tauri.conf.json 强制 macOSPrivateApi，资源固定为 .app，bundle 只配置 .app 和 .dmg。
- build-wake-helper.sh 是无条件调用的 zsh + Swift 构建脚本。
- wake-helper 下的 Swift、Entitlements 和 Info.plist 只能在 macOS 构建。

## 3. 可直接复用的跨平台代码

- main.ts 的 Jarvis 视觉层、Canvas 粒子、状态机、字幕、文字任务、线程 ID 的 localStorage 映射和 STOP UI 可保持。
- WebRTC 的 RTCPeerConnection、麦克风音轨、远端音频、SDP offer 和音频电平分析不依赖 macOS。
- Rust 的 JSON-RPC framing、pending request 映射、Codex thread/start、thread/resume、turn/start、turn/interrupt、后台终端清理、权限 profile 和事件转发可保持。
- CodexRuntime 的 stdin/stdout/stderr 异步通信可保持，平台差异只放在命令构造、路径、权限、唤醒助手和窗口兜底。
- Tauri 的窗口 show、unminimize、set_focus、set_always_on_top、close-requested 隐藏逻辑优先复用。

## 4. Windows 兼容风险

1. Codex 可能安装为 .exe、.cmd 或 .bat，必须分别构造 Command，不能把参数拼成 shell 字符串。
2. Windows 用户目录、中文路径、空格路径和 \\?\ canonicalize 前缀不能被误转成 WSL 路径。
3. getUserMedia 权限由 WebView2 管理，不能继续把非 macOS 平台硬编码为 authorized。
4. 第一阶段不实现持续唤醒，arm_wake_listener 必须返回 manual-only，不能运行 macOS 命令。
5. macOSPrivateApi、.app、.dmg、Swift helper 和 entitlements 不能进入 Windows bundle。
6. Windows WebView2 运行时可能缺失或版本过低，NSIS 应使用合理的 WebView2 安装策略。
7. Windows helper 需要限制重启次数，避免语音识别不可用时无限拉起进程。
8. Codex realtime 是实验性协议，Windows 必须继续使用同一 V3 请求形状，并保留真实 Voice smoke test。

## 5. 文件变更计划

### 新增

- docs/WINDOWS_PORT_PLAN.md：本审计和实施计划。
- src-tauri/src/platform/mod.rs：统一平台接口和跨平台路径/命令类型。
- src-tauri/src/platform/macos.rs：现有 macOS Codex 查找、唤醒、窗口、权限和进程实现。
- src-tauri/src/platform/windows.rs：Windows Codex 查找、.cmd/.bat 启动、窗口前置、权限状态和手动唤醒降级。
- scripts/build-wake-helper.mjs：按 process.platform 分派 helper 构建。
- src-tauri/tauri.windows.conf.json：Windows NSIS、资源、WebView2 和快捷方式配置。
- src-tauri/wake-helper/windows/JarvisWakeListener.csproj：Windows self-contained x64 helper 工程。
- src-tauri/wake-helper/windows/Program.cs：阶段二 Windows JSONL 唤醒 helper。
- .github/workflows/windows-build.yml：Windows CI 和 NSIS artifact。
- Rust 单元测试或 tests/platform.test.mjs：Codex 路径优先级、命令构造和错误信息测试。

### 修改

- src-tauri/src/lib.rs：删除散落的平台命令，改用 platform 模块；保留 Codex RPC 和任务逻辑。
- src-tauri/src/main.rs：保持入口最小化并支持 Windows 启动参数。
- src-tauri/Cargo.toml：把 macOS-only feature 和依赖限定在 macOS，增加 Windows 编译所需配置。
- src-tauri/tauri.conf.json：拆出通用配置与平台配置。
- src/main.ts：处理 browser-managed/manual-only/denied/unavailable 状态，并把 Windows 唤醒降级展示为手动启动。
- package.json：使用 Node 构建入口，新增 build:windows，移除无条件 zsh。
- tests/wav.test.mjs：保留现有 WAV 测试并增加纯函数行为测试。

### 删除

- 不删除现有 macOS Swift helper、macOS plist 或 macOS UI。
- 仅删除 package.json 对无条件 zsh scripts/build-wake-helper.sh 的直接依赖；原 shell 脚本继续由 Node 分派器在 macOS 调用。

## 6. 分阶段实现

### 阶段一：Windows 核心可运行版

1. 先为平台模块和 Codex command builder 写失败测试。
2. 拆分 Rust 平台代码，Windows 实现 Codex 查找、路径清理、工作目录和 .exe/.cmd/.bat 命令构造。
3. Windows 麦克风权限改为 browser-managed，前端用真实 getUserMedia 结果展示错误。
4. Windows arm_wake_listener 返回 manual-only，点击麦克风、文字任务、Voice 和 STOP 正常工作。
5. 加入 Windows 后台启动和关闭隐藏逻辑。
6. 每个小步完成后执行 Rust fmt、clippy、Node 测试和 web build。

### 阶段二：Windows 唤醒助手

1. 先测试 JSONL 事件解析和有限重启策略。
2. 使用 .NET 8 Windows 语音识别能力实现 self-contained x64 helper。
3. Rust 监督器改为平台模块启动/停止 helper，禁止使用 open/pkill/kill。
4. helper 仅发送唤醒事件，不读取登录凭据、不上传持续原始音频。
5. 在无识别器、拒绝权限和 helper 异常时保留手动麦克风入口。

### 阶段三：跨平台构建与发布

1. Node 构建脚本按 darwin/win32/other 分派。
2. 补齐 Windows Tauri 配置、ICO 资源、NSIS、WebView2 策略和 autostart。
3. 增加 Windows GitHub Actions，构建并上传 NSIS artifact。
4. 在 Windows 10 22H2 x64 验证安装、升级、后台启动和卸载。

## 7. 测试方案

### 自动检查

~~~powershell
npm ci
npm test
npm run web:build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run build:windows
git diff --check
~~~

### Windows 手工矩阵

- 英文用户名、中文用户名。
- 项目路径含空格、中文。
- JARVIS_CODEX_BIN、官方 .exe、npm .cmd、未安装 Codex。
- 麦克风允许、拒绝、WebView2 不可用。
- 点击麦克风启动 Voice、文本任务、STOP Voice、STOP Codex turn。
- 工作目录切换、新线程、断网错误。
- 关闭窗口隐藏、--background、开机自启。
- 阶段二再测试无识别器、helper 崩溃恢复、连续唤醒不重复进程。

## 8. 回滚方案

- 每个阶段单独提交，阶段验证失败时只回滚该阶段提交。
- 不修改用户全局 Codex 配置，不提交 token、证书或登录信息。
- 保留 macOS 文件和调用路径；Windows 功能通过 cfg 和平台配置隔离。
- 如果 Windows Voice 或 WebView2 验证失败，保留可运行的文字任务和手动麦克风版本，并将持续唤醒关闭为 manual-only。

## 9. 当前审计结论

项目可以增量改造成 Windows 版本，但必须先把 lib.rs 中的进程、路径、窗口和唤醒逻辑从 Codex 核心 RPC 中拆开。阶段一不应急于实现 Windows 持续唤醒；先确保 Codex CLI 查找、app-server、WebRTC、文字任务、STOP、后台隐藏和 NSIS 基础链路稳定。

## 10. 本轮实施状态

- 已完成：平台模块骨架、Windows Codex 查找优先级、.exe/.cmd/.bat 启动、Windows 路径处理、浏览器管理麦克风权限、手动唤醒降级、窗口隐藏/显示兼容入口。
- 已完成：Node 跨平台 helper 构建入口、Windows .NET helper 工程、JSONL 唤醒事件、有限次数 helper 重启、Windows NSIS 配置和 GitHub Actions。
- 已完成：Windows 路径测试、原有 WAV/行为测试回归、TypeScript/Vite 构建和 Rust 格式检查。
- 未完成验证：当前开发机没有 .NET SDK，无法本地 publish Windows helper；Cargo 缓存缺少依赖且 crates.io 下载超时，无法在本机给出 Cargo check/clippy 结论。
- Windows 真实验收仍需在 Windows 10 22H2 x64、安装 WebView2、安装 Codex CLI 的机器上执行，尤其是 Voice、麦克风权限、NSIS 安装和自启动。

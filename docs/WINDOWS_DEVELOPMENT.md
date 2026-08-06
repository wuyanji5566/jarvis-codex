# Windows 开发与打包

目标环境：Windows 10 22H2 x64、Node.js 20+、Rust stable MSVC、WebView2。

## 安装依赖

在 PowerShell 中运行：

~~~powershell
npm ci
~~~

如果需要构建 Windows 唤醒助手，还需要安装 .NET 8 SDK。没有 .NET 时，项目会安全降级为手动点击麦克风模式，不影响主体运行。

## 开发

~~~powershell
$env:JARVIS_WORKSPACE = "C:\Users\你的用户名\项目 文件夹"
npm run dev
~~~

首次使用时，点击 Jarvis 头像上的麦克风按钮，由 WebView2 请求麦克风权限。Windows 持续唤醒助手未安装时，页面会显示 manual-only 提示。

Codex CLI 查找顺序：

1. JARVIS_CODEX_BIN
2. Tauri resource 中的 codex.exe
3. where.exe codex.exe
4. where.exe codex
5. %LOCALAPPDATA%\OpenAI\Codex\bin\codex.exe
6. %USERPROFILE%\.codex\bin\codex.exe
7. %APPDATA%\npm\codex.cmd
8. %APPDATA%\npm\codex.exe

.cmd 和 .bat 通过 cmd.exe /D /S /C 启动，参数不会拼接成未经转义的 shell 字符串。

## 检查

~~~powershell
npm run wake:build
npm test
npm run web:build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
~~~

## 打包

~~~powershell
npm run build:windows
~~~

NSIS 安装包输出在：

~~~text
src-tauri\target\release\bundle\nsis\
~~~

Windows 安装使用当前用户模式，不要求管理员权限。安装器不携带 Codex 登录凭据，也不修改用户全局 Codex 配置。

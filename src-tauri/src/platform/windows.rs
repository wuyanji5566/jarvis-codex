use std::{
    env,
    path::{Path, PathBuf},
    process::Command as StdCommand,
};

use tauri::{AppHandle, Manager};
use tokio::process::Command;

fn normalize_path(path: PathBuf) -> PathBuf {
    let value = path.to_string_lossy();
    if let Some(rest) = value.strip_prefix("\\\\?\\UNC\\") {
        return PathBuf::from(format!("\\\\{rest}"));
    }
    if let Some(rest) = value.strip_prefix("\\\\?\\") {
        return PathBuf::from(rest);
    }
    path
}

fn is_windows_apps_path(path: &Path) -> bool {
    path.to_string_lossy()
        .to_ascii_lowercase()
        .contains("\\windowsapps\\")
}

fn existing(path: PathBuf) -> Option<PathBuf> {
    if path.is_file() && !is_windows_apps_path(&path) {
        Some(normalize_path(path))
    } else {
        None
    }
}

fn where_candidates(command: &str) -> Vec<PathBuf> {
    let output = StdCommand::new("where.exe").arg(command).output();
    output
        .ok()
        .filter(|value| value.status.success())
        .map(|value| {
            String::from_utf8_lossy(&value.stdout)
                .lines()
                .map(|line| PathBuf::from(line.trim()))
                .collect()
        })
        .unwrap_or_default()
}

fn configured_windows_proxy() -> Option<String> {
    let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings";
    let enabled = StdCommand::new("reg")
        .args(["query", key, "/v", "ProxyEnable"])
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).contains("0x1"))
        .unwrap_or(false);
    if !enabled {
        return None;
    }
    let value = StdCommand::new("reg")
        .args(["query", key, "/v", "ProxyServer"])
        .output()
        .ok()
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .find(|line| line.contains("REG_SZ"))
                .and_then(|line| line.split("REG_SZ").nth(1))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })?;
    let proxy = value
        .split(';')
        .find_map(|entry| entry.strip_prefix("https="))
        .or_else(|| {
            value
                .split(';')
                .find_map(|entry| entry.strip_prefix("http="))
        })
        .unwrap_or(&value);
    Some(
        if proxy.starts_with("http://") || proxy.starts_with("https://") {
            proxy.to_owned()
        } else {
            format!("http://{proxy}")
        },
    )
}

fn apply_system_proxy(command: &mut Command) {
    if env::var_os("HTTPS_PROXY").is_some() || env::var_os("https_proxy").is_some() {
        return;
    }
    if let Some(proxy) = configured_windows_proxy() {
        command.env("HTTP_PROXY", &proxy);
        command.env("HTTPS_PROXY", &proxy);
    }
}

fn codex_binary_path(app: &AppHandle) -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Ok(value) = env::var("JARVIS_CODEX_BIN") {
        candidates.push(PathBuf::from(value));
    }
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("codex.exe"));
    }
    candidates.extend(where_candidates("codex.exe"));
    candidates.extend(where_candidates("codex"));
    if let Ok(value) = env::var("USERPROFILE") {
        candidates.push(PathBuf::from(&value).join("bin/codex.exe"));
        candidates.push(PathBuf::from(value).join(".codex/bin/codex.exe"));
    }
    if let Ok(value) = env::var("LOCALAPPDATA") {
        candidates.push(PathBuf::from(value).join("OpenAI/Codex/bin/codex.exe"));
    }
    if let Ok(value) = env::var("APPDATA") {
        candidates.push(PathBuf::from(value.clone()).join("npm/codex.cmd"));
        candidates.push(PathBuf::from(value).join("npm/codex.exe"));
    }
    candidates.into_iter().find_map(existing).ok_or_else(|| {
        "未找到 Codex CLI。请安装官方 Codex，或设置 JARVIS_CODEX_BIN 指向 codex.exe/codex.cmd。"
            .to_owned()
    })
}

pub fn codex_command(app: &AppHandle) -> Result<Command, String> {
    let path = codex_binary_path(app)?;
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let mut command = if matches!(extension.to_ascii_lowercase().as_str(), "cmd" | "bat") {
        let mut command = Command::new("cmd.exe");
        command.args(["/D", "/S", "/C"]).arg(path);
        command
    } else {
        Command::new(path)
    };
    apply_system_proxy(&mut command);
    // Keep the tool hosts explicit for the embedded app-server. The official
    // desktop app passes these feature overrides when it starts its child;
    // Jarvis must do the same or browser/desktop calls may be treated as text.
    command.args([
        "-c",
        "features.code_mode_host=true",
        "-c",
        "features.browser_use=true",
        "-c",
        "features.computer_use=true",
        "app-server",
        "--enable",
        "realtime_conversation",
    ]);
    Ok(command)
}

pub async fn open_browser(url: &str) -> Result<(), String> {
    Command::new("cmd.exe")
        .args(["/D", "/S", "/C", "start", "", url])
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("无法打开 Windows 浏览器：{error}"))
}

pub fn default_workspace() -> Result<String, String> {
    if let Ok(value) = env::var("JARVIS_WORKSPACE") {
        return validated_workspace(&value);
    }
    if let Ok(value) = env::var("USERPROFILE") {
        if let Ok(path) = validated_workspace(&value) {
            return Ok(path);
        }
    }
    env::current_dir()
        .map(normalize_path)
        .map(|value| value.to_string_lossy().into_owned())
        .map_err(|error| format!("无法确定默认工作目录：{error}"))
}

pub fn validated_workspace(value: &str) -> Result<String, String> {
    let path = PathBuf::from(value.trim());
    if !path.is_dir() {
        return Err(format!("工作目录不存在或不是文件夹：{value}"));
    }
    path.canonicalize()
        .map(normalize_path)
        .map(|value| value.to_string_lossy().into_owned())
        .map_err(|error| format!("无法读取工作目录：{error}"))
}

pub async fn request_microphone_permission() -> Result<String, String> {
    Ok("browser-managed".to_owned())
}

pub fn raise_window(app: &AppHandle) {
    let app_handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_always_on_top(true);
            let _ = window.set_focus();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(700)).await;
                let _ = window.set_always_on_top(false);
            });
        }
    });
}

pub fn wake_helper_path(app: &AppHandle) -> Result<PathBuf, String> {
    let relative = PathBuf::from("wake-helper/windows/publish/JarvisWakeListener.exe");
    if let Ok(resource_dir) = app.path().resource_dir() {
        let bundled = resource_dir.join(&relative);
        if bundled.is_file() {
            return Ok(bundled);
        }
    }
    let development = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&relative);
    if development.is_file() {
        return Ok(development);
    }
    Err("Windows 唤醒助手尚未安装；可点击麦克风按钮启动 Voice。".to_owned())
}

pub async fn spawn_wake_helper(
    app: &AppHandle,
    event_file: &Path,
) -> Result<tokio::process::Child, String> {
    let helper = wake_helper_path(app)?;
    Command::new(helper)
        .arg("--event-file")
        .arg(event_file)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| format!("无法启动 Windows 唤醒助手：{error}"))
}

pub async fn terminate_process(process_id: u32) {
    if process_id == 0 {
        return;
    }
    let _ = Command::new("taskkill")
        .args(["/PID", &process_id.to_string(), "/T", "/F"])
        .status()
        .await;
}

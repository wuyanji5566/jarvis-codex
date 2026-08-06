#![allow(dead_code)]

use std::{env, path::PathBuf};

use tauri::{AppHandle, Manager};
use tokio::process::Command;

pub fn codex_command(app: &AppHandle) -> Result<Command, String> {
    let mut candidates = Vec::new();
    if let Ok(value) = env::var("JARVIS_CODEX_BIN") {
        candidates.push(PathBuf::from(value));
    }
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("codex"));
    }
    candidates.extend([
        PathBuf::from("/Applications/ChatGPT.app/Contents/Resources/codex"),
        PathBuf::from("/Applications/Codex.app/Contents/Resources/codex"),
        PathBuf::from("/opt/homebrew/bin/codex"),
        PathBuf::from("/usr/local/bin/codex"),
    ]);
    if let Ok(home) = env::var("HOME") {
        candidates.push(PathBuf::from(&home).join(".local/bin/codex"));
        candidates.push(PathBuf::from(home).join(".cargo/bin/codex"));
    }
    let path = candidates
        .into_iter()
        .find(|value| value.is_file())
        .ok_or_else(|| {
            "未找到 Codex 可执行文件；请安装 Codex，或设置 JARVIS_CODEX_BIN。".to_owned()
        })?;
    Ok({
        let mut command = Command::new(path);
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
        command
    })
}

pub async fn open_browser(url: &str) -> Result<(), String> {
    Command::new("open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("无法打开 macOS 浏览器：{error}"))
}

pub fn default_workspace() -> Result<String, String> {
    if let Ok(value) = env::var("JARVIS_WORKSPACE") {
        if PathBuf::from(&value).is_dir() {
            return validated_workspace(&value);
        }
    }
    if let Ok(home) = env::var("HOME") {
        if PathBuf::from(&home).is_dir() {
            return Ok(home);
        }
    }
    env::current_dir()
        .map(|value| value.to_string_lossy().into_owned())
        .map_err(|error| format!("无法确定默认工作目录：{error}"))
}

pub fn validated_workspace(value: &str) -> Result<String, String> {
    let path = PathBuf::from(value);
    if !path.is_dir() {
        return Err(format!("工作目录不存在或不是文件夹：{value}"));
    }
    path.canonicalize()
        .map(|value| value.to_string_lossy().into_owned())
        .map_err(|error| format!("无法读取工作目录：{error}"))
}

pub async fn request_microphone_permission() -> Result<String, String> {
    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_av_foundation::{AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio};
    use std::sync::{Arc, Mutex};
    use tokio::sync::oneshot;

    let media_type =
        unsafe { AVMediaTypeAudio }.ok_or_else(|| "macOS 未提供音频授权类型".to_owned())?;
    let status = unsafe { AVCaptureDevice::authorizationStatusForMediaType(media_type) };
    match status {
        AVAuthorizationStatus::Authorized => return Ok("authorized".to_owned()),
        AVAuthorizationStatus::Denied => return Ok("denied".to_owned()),
        AVAuthorizationStatus::Restricted => return Ok("restricted".to_owned()),
        _ => {}
    }
    let (sender, receiver) = oneshot::channel::<bool>();
    let sender = Arc::new(Mutex::new(Some(sender)));
    let completion_sender = sender.clone();
    let completion = RcBlock::new(move |granted: Bool| {
        if let Ok(mut guard) = completion_sender.lock() {
            if let Some(sender) = guard.take() {
                let _ = sender.send(granted.as_bool());
            }
        }
    });
    unsafe {
        AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &completion)
    };
    receiver
        .await
        .map(|granted| if granted { "authorized" } else { "denied" }.to_owned())
        .map_err(|_| "macOS 麦克风授权回调中断".to_owned())
}

pub fn raise_window(app: &AppHandle) {
    let app_handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSApplication;
        if let Some(mtm) = MainThreadMarker::new() {
            let application = NSApplication::sharedApplication(mtm);
            #[allow(deprecated)]
            application.activateIgnoringOtherApps(true);
        }
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

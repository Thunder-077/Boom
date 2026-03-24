use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use chrono::Utc;
use tauri::{AppHandle, Manager};

use crate::score::AppError;

pub fn log_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let mut dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::new(format!("获取日志目录失败: {e}")))?;
    dir.push("logs");
    fs::create_dir_all(&dir).map_err(|e| AppError::new(format!("创建日志目录失败: {e}")))?;
    Ok(dir)
}

pub fn log_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let mut path = log_dir(app)?;
    path.push("app.log");
    Ok(path)
}

pub fn append_log(app: &AppHandle, level: &str, scope: &str, message: &str) -> Result<(), AppError> {
    let path = log_path(app)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| AppError::new(format!("打开日志文件失败: {e}")))?;
    let now = Utc::now().to_rfc3339();
    writeln!(
        file,
        "[{now}] [{}] [{}] {}",
        level.trim().to_uppercase(),
        scope.trim(),
        message.trim().replace('\r', " ").replace('\n', " | ")
    )
    .map_err(|e| AppError::new(format!("写入日志文件失败: {e}")))?;
    Ok(())
}

pub fn log_error(app: &AppHandle, scope: &str, error: &str) {
    let _ = append_log(app, "error", scope, error);
}

#[tauri::command]
pub fn append_app_log(app: AppHandle, level: String, scope: String, message: String) -> Result<(), String> {
    append_log(&app, &level, &scope, &message).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_app_log_path(app: AppHandle) -> Result<String, String> {
    log_path(&app)
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reveal_in_explorer(path: String) -> Result<(), String> {
    let target = path.trim();
    if target.is_empty() {
        return Err("路径不能为空".to_string());
    }
    Command::new("explorer.exe")
        .arg("/select,")
        .arg(target)
        .spawn()
        .map_err(|e| format!("打开资源管理器失败: {e}"))?;
    Ok(())
}

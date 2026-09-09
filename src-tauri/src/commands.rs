use crate::db;
use crate::model::{NewTask, Project, Settings, Task, TaskPatch};
use crate::settings as settings_store;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub settings: Mutex<Settings>,
    pub data_dir: std::path::PathBuf,
}

fn busy() -> String {
    "数据库繁忙，请稍后重试".into()
}

/// 显示并聚焦主窗口（托盘左键 / 菜单共用）
pub fn focus_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn toggle_tile_inner(app: &tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("tile") {
        window.close().map_err(|err| err.to_string())?;
        return Ok(false);
    }
    WebviewWindowBuilder::new(app, "tile", WebviewUrl::App("index.html".into()))
        .title("Todo 磁贴")
        .inner_size(320.0, 480.0)
        .min_inner_size(260.0, 320.0)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .background_color(tauri::utils::config::Color(239, 232, 216, 255))
        .build()
        .map_err(|err| err.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn list_tasks(state: State<AppState>) -> Result<Vec<Task>, String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    db::list(&conn)
}

#[tauri::command]
pub fn create_task(state: State<AppState>, input: NewTask) -> Result<Task, String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    db::insert(&conn, input)
}

#[tauri::command]
pub fn update_task(state: State<AppState>, id: String, patch: TaskPatch) -> Result<Task, String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    db::update(&conn, &id, patch)
}

#[tauri::command]
pub fn complete_task(state: State<AppState>, id: String) -> Result<Task, String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    db::complete(&conn, &id)
}

#[tauri::command]
pub fn delete_task(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    db::delete(&conn, &id)
}

#[tauri::command]
pub fn mark_notified(state: State<AppState>, id: String) -> Result<Task, String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    db::mark_notified(&conn, &id)
}

#[tauri::command]
pub fn reorder_tasks(state: State<AppState>, ordered_ids: Vec<String>) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(|_| busy())?;
    db::reorder(&mut conn, &ordered_ids)
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    db::list_projects(&conn)
}

#[tauri::command]
pub fn create_project(
    state: State<AppState>,
    name: String,
    color: String,
) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    db::insert_project(&conn, &name, &color)
}

#[tauri::command]
pub fn update_project(
    state: State<AppState>,
    id: String,
    name: String,
    color: String,
) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    db::update_project(&conn, &id, &name, &color)
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    db::delete_project(&conn, &id)
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    let current = state.settings.lock().map_err(|_| busy())?;
    Ok(current.clone())
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: Settings) -> Result<Settings, String> {
    let mut sanitized = settings;
    sanitized.notification_lead_minutes = sanitized.notification_lead_minutes.min(720);
    settings_store::save(&state.data_dir, &sanitized)?;
    let mut current = state.settings.lock().map_err(|_| busy())?;
    *current = sanitized.clone();
    Ok(sanitized)
}

/// 必须是 async：Windows 上同步 command 运行在主线程，原地创建 webview
/// 会得到一个永远不绘制的空白窗口（白屏，拉伸露黑底）
#[tauri::command]
pub async fn toggle_tile_window(app: tauri::AppHandle) -> Result<bool, String> {
    toggle_tile_inner(&app)
}

#[tauri::command]
pub fn tile_state(app: tauri::AppHandle) -> bool {
    app.get_webview_window("tile").is_some()
}

#[tauri::command]
pub fn data_dir(state: State<AppState>) -> Result<String, String> {
    Ok(state.data_dir.display().to_string())
}

#[tauri::command]
pub fn export_backup(app: tauri::AppHandle, state: State<AppState>) -> Result<String, String> {
    let conn = state.db.lock().map_err(|_| busy())?;
    let tasks = db::list(&conn)?;
    let projects = db::list_projects(&conn)?;
    let snapshot = state.settings.lock().map_err(|_| busy())?.clone();
    drop(conn);

    let target_dir = match app.path().document_dir() {
        Ok(dir) => dir.join("TodoBackup"),
        Err(_) => state.data_dir.join("backups"),
    };
    std::fs::create_dir_all(&target_dir).map_err(|err| err.to_string())?;

    let now = chrono::Local::now();
    let path = target_dir.join(format!("todo-backup-{}.json", now.format("%Y%m%d-%H%M%S")));
    let payload = serde_json::json!({
        "app": "todo",
        "version": 1,
        "exportedAt": now.format("%Y-%m-%dT%H:%M:%S").to_string(),
        "tasks": tasks,
        "projects": projects,
        "settings": snapshot,
    });
    let text = serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?;
    std::fs::write(&path, text).map_err(|err| err.to_string())?;
    Ok(path.display().to_string())
}

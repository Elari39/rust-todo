use crate::db;
use crate::model::{NewTask, Project, Settings, Task, TaskPatch};
use crate::settings as settings_store;
use rusqlite::Connection;
use std::sync::{Mutex, MutexGuard};
use tauri::{Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub settings: Mutex<Settings>,
    pub tile_lock: Mutex<()>,
    pub data_dir: std::path::PathBuf,
}

/// 锁中毒时直接接管数据继续服务，避免一次 panic 让所有命令永久失败
fn lock_db(state: &AppState) -> MutexGuard<'_, Connection> {
    match state.db.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn lock_settings(state: &AppState) -> MutexGuard<'_, Settings> {
    match state.settings.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// 任务数据变更后广播给所有窗口（payload 为来源窗口 label，
/// 前端据此跳过自己发起的变更，避免重复刷新）
fn emit_tasks_changed(window: &tauri::WebviewWindow) {
    let _ = window.emit("tasks-changed", window.label().to_string());
}

/// 显示并聚焦主窗口（托盘左键 / 菜单共用）
pub fn focus_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn close_to_tray_pref(state: State<'_, AppState>) -> bool {
    lock_settings(&state).close_to_tray
}

pub fn toggle_tile_inner(app: &tauri::AppHandle) -> Result<bool, String> {
    let state = app
        .try_state::<AppState>()
        .ok_or_else(|| "应用尚未就绪".to_string())?;
    // 全程持锁：防止快速连点时两次都判定「不存在」而重复建窗
    let _tile_guard = state.tile_lock.lock();
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
    let conn = lock_db(&state);
    db::list(&conn)
}

#[tauri::command]
pub fn create_task(
    window: tauri::WebviewWindow,
    state: State<AppState>,
    input: NewTask,
) -> Result<Task, String> {
    let conn = lock_db(&state);
    let task = db::insert(&conn, input)?;
    emit_tasks_changed(&window);
    Ok(task)
}

#[tauri::command]
pub fn update_task(
    window: tauri::WebviewWindow,
    state: State<AppState>,
    id: String,
    patch: TaskPatch,
) -> Result<Task, String> {
    let conn = lock_db(&state);
    let task = db::update(&conn, &id, patch)?;
    emit_tasks_changed(&window);
    Ok(task)
}

#[tauri::command]
pub fn complete_task(
    window: tauri::WebviewWindow,
    state: State<AppState>,
    id: String,
) -> Result<Task, String> {
    let conn = lock_db(&state);
    let task = db::complete(&conn, &id)?;
    emit_tasks_changed(&window);
    Ok(task)
}

#[tauri::command]
pub fn delete_task(
    window: tauri::WebviewWindow,
    state: State<AppState>,
    id: String,
) -> Result<(), String> {
    let conn = lock_db(&state);
    db::delete(&conn, &id)?;
    emit_tasks_changed(&window);
    Ok(())
}

#[tauri::command]
pub fn mark_notified(
    window: tauri::WebviewWindow,
    state: State<AppState>,
    id: String,
) -> Result<Task, String> {
    let conn = lock_db(&state);
    let task = db::mark_notified(&conn, &id)?;
    emit_tasks_changed(&window);
    Ok(task)
}

#[tauri::command]
pub fn reorder_tasks(
    window: tauri::WebviewWindow,
    state: State<AppState>,
    ordered_ids: Vec<String>,
) -> Result<(), String> {
    let mut conn = lock_db(&state);
    db::reorder(&mut conn, &ordered_ids)?;
    emit_tasks_changed(&window);
    Ok(())
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    let conn = lock_db(&state);
    db::list_projects(&conn)
}

#[tauri::command]
pub fn create_project(
    state: State<AppState>,
    name: String,
    color: String,
) -> Result<Project, String> {
    let conn = lock_db(&state);
    db::insert_project(&conn, &name, &color)
}

#[tauri::command]
pub fn update_project(
    state: State<AppState>,
    id: String,
    name: String,
    color: String,
) -> Result<Project, String> {
    let conn = lock_db(&state);
    db::update_project(&conn, &id, &name, &color)
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = lock_db(&state);
    db::delete_project(&conn, &id)
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    Ok(lock_settings(&state).clone())
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: Settings) -> Result<Settings, String> {
    let mut sanitized = settings;
    sanitized.notification_lead_minutes = sanitized.notification_lead_minutes.min(720);
    settings_store::save(&state.data_dir, &sanitized)?;
    *lock_settings(&state) = sanitized.clone();
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

/// async + spawn_blocking：Windows 上同步 command 运行在主线程，阻塞文件 IO 会冻结 UI
#[tauri::command]
pub async fn export_backup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (tasks, projects, snapshot, fallback_dir) = {
        let conn = lock_db(&state);
        let tasks = db::list(&conn)?;
        let projects = db::list_projects(&conn)?;
        let snapshot = lock_settings(&state).clone();
        (tasks, projects, snapshot, state.data_dir.clone())
    };

    let target_dir = match app.path().document_dir() {
        Ok(dir) => dir.join("TodoBackup"),
        Err(_) => fallback_dir.join("backups"),
    };
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
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
    })
    .await
    .map_err(|err| err.to_string())?
}

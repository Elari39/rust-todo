use crate::db;
use crate::model::{BackupPayload, NewTask, Project, Settings, Task, TaskPatch};
use crate::settings as settings_store;
use rusqlite::Connection;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub settings: Arc<Mutex<Settings>>,
    pub tile_lock: Arc<Mutex<()>>,
    pub data_dir: std::path::PathBuf,
}

/// 锁中毒时直接接管数据继续服务，避免一次 panic 让所有命令永久失败
fn lock_db(db: &Mutex<Connection>) -> MutexGuard<'_, Connection> {
    db.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn lock_settings(settings: &Mutex<Settings>) -> MutexGuard<'_, Settings> {
    settings
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 在后台线程执行阻塞的 SQLite 操作：Windows 上同步 command 运行在主线程，
/// 阻塞 IO 会冻结 UI，所有数据库命令统一经由 spawn_blocking。
async fn with_db<T, F>(db: &Arc<Mutex<Connection>>, run: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut Connection) -> Result<T, String> + Send + 'static,
{
    let db = Arc::clone(db);
    tauri::async_runtime::spawn_blocking(move || {
        let mut conn = lock_db(&db);
        run(&mut conn)
    })
    .await
    .map_err(|err| err.to_string())?
}

/// 任务数据变更后广播给所有窗口（payload 为来源窗口 label，
/// 前端据此跳过自己发起的变更，避免重复刷新）
fn emit_tasks_changed(window: &tauri::WebviewWindow) {
    let _ = window.emit("tasks-changed", window.label().to_string());
}

/// 显示并聚焦主窗口（托盘左键 / 菜单 / 二次启动共用）
pub fn focus_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn close_to_tray_pref(state: State<'_, AppState>) -> bool {
    lock_settings(&state.settings).close_to_tray
}

pub fn toggle_tile_inner(app: &tauri::AppHandle) -> Result<bool, String> {
    let state = app
        .try_state::<AppState>()
        .ok_or_else(|| "应用尚未就绪".to_string())?;
    // 全程持锁：防止快速连点时两次都判定「不存在」而重复建窗。
    // 本函数只允许在 async runtime 线程上执行（async command / 托盘 spawn），
    // 主线程不参与争锁——否则 build() 派发到主线程的窗口创建会与锁互等死锁。
    let _tile_guard = state
        .tile_lock
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
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
        .background_color(tauri::utils::config::Color(237, 242, 251, 255))
        .build()
        .map_err(|err| err.to_string())?;
    // 广播给所有窗口，让设置页/侧栏的磁贴开关同步（关闭路径由
    // lib.rs 的 Destroyed 事件兜底）
    let _ = app.emit("tile-changed", true);
    Ok(true)
}

/// 托盘菜单入口：派发到 async runtime 执行，主线程立即返回
pub fn spawn_toggle_tile(app: &tauri::AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(err) = toggle_tile_inner(&handle) {
            eprintln!("切换磁贴窗口失败: {err}");
        }
    });
}

#[tauri::command]
pub async fn list_tasks(state: State<'_, AppState>) -> Result<Vec<Task>, String> {
    with_db(&state.db, |conn| db::list(conn)).await
}

#[tauri::command]
pub async fn create_task(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    input: NewTask,
) -> Result<Task, String> {
    let task = with_db(&state.db, move |conn| db::insert(conn, input)).await?;
    emit_tasks_changed(&window);
    Ok(task)
}

#[tauri::command]
pub async fn update_task(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    id: String,
    patch: TaskPatch,
) -> Result<Task, String> {
    let task = with_db(&state.db, move |conn| db::update(conn, &id, patch)).await?;
    emit_tasks_changed(&window);
    Ok(task)
}

#[tauri::command]
pub async fn complete_task(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    id: String,
) -> Result<Task, String> {
    let task = with_db(&state.db, move |conn| db::complete(conn, &id)).await?;
    emit_tasks_changed(&window);
    Ok(task)
}

#[tauri::command]
pub async fn delete_task(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    with_db(&state.db, move |conn| db::delete(conn, &id)).await?;
    emit_tasks_changed(&window);
    Ok(())
}

#[tauri::command]
pub async fn mark_notified(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    id: String,
) -> Result<Task, String> {
    let task = with_db(&state.db, move |conn| db::mark_notified(conn, &id)).await?;
    emit_tasks_changed(&window);
    Ok(task)
}

#[tauri::command]
pub async fn reorder_tasks(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    ordered_ids: Vec<String>,
) -> Result<(), String> {
    with_db(&state.db, move |conn| db::reorder(conn, &ordered_ids)).await?;
    emit_tasks_changed(&window);
    Ok(())
}

#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    with_db(&state.db, |conn| db::list_projects(conn)).await
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<Project, String> {
    with_db(&state.db, move |conn| {
        db::insert_project(conn, &name, &color)
    })
    .await
}

#[tauri::command]
pub async fn update_project(
    state: State<'_, AppState>,
    id: String,
    name: String,
    color: String,
) -> Result<Project, String> {
    with_db(&state.db, move |conn| {
        db::update_project(conn, &id, &name, &color)
    })
    .await
}

#[tauri::command]
pub async fn delete_project(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    with_db(&state.db, move |conn| db::delete_project(conn, &id)).await?;
    // 删除项目会同时解绑任务，任务列表也需要跨窗口刷新
    emit_tasks_changed(&window);
    Ok(())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(lock_settings(&state.settings).clone())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    let mut sanitized = settings;
    sanitized.notification_lead_minutes = sanitized.notification_lead_minutes.min(720);
    let settings_lock = Arc::clone(&state.settings);
    let data_dir = state.data_dir.clone();
    let to_save = sanitized.clone();
    // 持锁覆盖「写文件 + 更新内存」全过程：并发保存时文件与内存不会交错出不一致状态
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_settings(&settings_lock);
        settings_store::save(&data_dir, &to_save)?;
        *guard = to_save;
        Ok::<(), String>(())
    })
    .await
    .map_err(|err| err.to_string())??;
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
pub fn data_dir(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.data_dir.display().to_string())
}

/// async + spawn_blocking：Windows 上同步 command 运行在主线程，阻塞文件 IO 会冻结 UI
#[tauri::command]
pub async fn export_backup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {    let db = Arc::clone(&state.db);
    let settings = Arc::clone(&state.settings);
    let (tasks, projects, snapshot) = tauri::async_runtime::spawn_blocking(
        move || -> Result<(Vec<Task>, Vec<Project>, Settings), String> {
            let conn = lock_db(&db);
            let tasks = db::list(&conn)?;
            let projects = db::list_projects(&conn)?;
            let snapshot = lock_settings(&settings).clone();
            Ok((tasks, projects, snapshot))
        },
    )
    .await
    .map_err(|err| err.to_string())??;

    let fallback_dir = state.data_dir.clone();
    let target_dir = match app.path().document_dir() {
        Ok(dir) => dir.join("TodoBackup"),
        Err(_) => fallback_dir.join("backups"),
    };
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        std::fs::create_dir_all(&target_dir).map_err(|err| err.to_string())?;

        let now = chrono::Local::now();
        // 精确到毫秒：同秒内连续导出不再互相覆盖；exportedAt 带时区偏移
        let path = target_dir.join(format!(
            "todo-backup-{}.json",
            now.format("%Y%m%d-%H%M%S%.3f")
        ));
        let payload = serde_json::json!({
            "app": "todo",
            "version": 1,
            "exportedAt": now.format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
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

/// 导入备份：文件读取与解析先在阻塞线程完成（不持 DB 锁），避免磁盘慢时
/// 卡住其他窗口的数据库命令；通过后再进入事务整体替换。
/// 返回导入的任务/项目数量描述，供前端提示。
#[tauri::command]
pub async fn import_backup(
    window: tauri::WebviewWindow,
    state: State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    let payload =
        tauri::async_runtime::spawn_blocking(move || -> Result<BackupPayload, String> {
            let text = std::fs::read_to_string(&path)
                .map_err(|err| format!("读取备份文件失败: {err}"))?;
            serde_json::from_str(&text).map_err(|err| format!("备份文件格式无效: {err}"))
        })
        .await
        .map_err(|err| err.to_string())??;

    let db = Arc::clone(&state.db);
    let (task_count, project_count) =
        with_db(&db, move |conn| db::import_replace(conn, payload)).await?;
    emit_tasks_changed(&window);
    Ok(format!("{task_count} 个任务、{project_count} 个项目"))
}

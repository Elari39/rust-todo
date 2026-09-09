mod commands;
mod db;
mod model;
mod platform;
mod settings;

use commands::AppState;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 单实例：开机自启 + 手动双开等场景直接聚焦已有主窗口，
        // 也杜绝两个进程同时写 SQLite。必须是第一个注册的插件。
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            commands::focus_main(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() != "main" {
                    return;
                }
                let app = window.app_handle();
                let close_to_tray = app
                    .try_state::<AppState>()
                    .map(commands::close_to_tray_pref)
                    .unwrap_or(true);
                // 系统关机时不拦截，避免托盘化阻塞 Windows 关机流程
                if close_to_tray && !platform::shutting_down() {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn =
                db::open(&dir.join("todo.db")).map_err(Box::<dyn std::error::Error>::from)?;
            let app_settings = settings::load(&dir);
            app.manage(AppState {
                db: Arc::new(Mutex::new(conn)),
                settings: Arc::new(Mutex::new(app_settings)),
                tile_lock: Arc::new(Mutex::new(())),
                data_dir: dir,
            });

            let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let tile = MenuItem::with_id(app, "tile", "开关置顶磁贴", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &tile, &quit])?;
            TrayIconBuilder::with_id("todo-tray")
                .icon(
                    app.default_window_icon()
                        .expect("missing bundle icon")
                        .clone(),
                )
                .tooltip("Todo")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => commands::focus_main(app),
                    // 在 async runtime 上执行：主线程不参与 tile_lock 竞争，
                    // 否则与 async 命令持锁等待 build() 派发会互相等待死锁
                    "tile" => commands::spawn_toggle_tile(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        commands::focus_main(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_tasks,
            commands::create_task,
            commands::update_task,
            commands::complete_task,
            commands::delete_task,
            commands::mark_notified,
            commands::reorder_tasks,
            commands::list_projects,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::get_settings,
            commands::save_settings,
            commands::toggle_tile_window,
            commands::tile_state,
            commands::data_dir,
            commands::export_backup,
            commands::import_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

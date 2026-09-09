mod commands;
mod db;
mod model;
mod platform;
mod settings;

use commands::AppState;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
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
            let conn = db::open(&dir.join("todo.db"))
                .map_err(|err| Box::<dyn std::error::Error>::from(err))?;
            let app_settings = settings::load(&dir);
            app.manage(AppState {
                db: Mutex::new(conn),
                settings: Mutex::new(app_settings),
                tile_lock: Mutex::new(()),
                data_dir: dir,
            });

            let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let tile = MenuItem::with_id(app, "tile", "开关置顶磁贴", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &tile, &quit])?;
            TrayIconBuilder::with_id("todo-tray")
                .icon(app.default_window_icon().expect("missing bundle icon").clone())
                .tooltip("Todo")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => commands::focus_main(app),
                    "tile" => {
                        if let Err(err) = commands::toggle_tile_inner(app) {
                            eprintln!("切换磁贴窗口失败: {err}");
                        }
                    }
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

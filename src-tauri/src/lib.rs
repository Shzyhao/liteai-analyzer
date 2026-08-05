//! Tauri 应用壳：唯一能接触 tauri API 的 crate。核心逻辑全部在 crates/liteai-*。

mod commands;
mod shell_integration;

use liteai_config::AppConfig;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

/// 全局状态：配置、密钥存储、取消标志、待分析队列。
pub struct AppState {
    pub config_dir: PathBuf,
    pub config: Mutex<AppConfig>,
    pub secrets: Mutex<Box<dyn liteai_core::SecretStore>>,
    pub cancel: Arc<AtomicBool>,
    pub pending: Mutex<Vec<String>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 单实例插件必须第一个注册
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let paths: Vec<String> = argv
                .iter()
                .skip(1)
                .filter(|a| PathBuf::from(a).is_file())
                .cloned()
                .collect();
            if !paths.is_empty() {
                // 第二个实例的 main.rs 会把路径写入 pending.json，但 setup 不会再执行，
                // 这里直接入队并清掉 pending.json，避免下次启动重复入队。
                let _ = std::fs::remove_file(liteai_config::default_config_dir().join("pending.json"));
                app.state::<AppState>().pending.lock().unwrap().extend(paths);
                let _ = app.emit("queue-updated", ());
                if let Some(w) = app.webview_windows().values().next() {
                    let _ = w.set_focus();
                }
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_dir = liteai_config::default_config_dir();
            let config = liteai_config::load_config(&config_dir).unwrap_or_default();
            let secrets = liteai_config::default_secret_store(&config_dir);
            app.manage(AppState {
                config_dir: config_dir.clone(),
                config: Mutex::new(config),
                secrets: Mutex::new(secrets),
                cancel: Arc::new(AtomicBool::new(false)),
                pending: Mutex::new(Vec::new()),
            });

            // 读取右键入队（pending.json），避免窗口未就绪时的竞态
            let pending_path = config_dir.join("pending.json");
            if let Ok(raw) = std::fs::read_to_string(&pending_path) {
                if let Ok(paths) = serde_json::from_str::<Vec<String>>(&raw) {
                    app.state::<AppState>().pending.lock().unwrap().extend(paths);
                    let _ = std::fs::remove_file(&pending_path);
                    let _ = app.emit("queue-updated", ());
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::analyze_files,
            commands::cancel_all,
            commands::get_pending,
            commands::get_config,
            commands::save_config,
            commands::set_api_key,
            commands::delete_api_key,
            commands::has_api_key,
            commands::test_connection,
            commands::get_templates,
            commands::save_templates,
            commands::import_templates,
            commands::register_shell_menu,
            commands::unregister_shell_menu,
            commands::shell_menu_registered,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

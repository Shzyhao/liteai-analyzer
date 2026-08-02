// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::Path;

fn main() {
    // 右键触发时，Windows 将选中文件路径作为参数传入。
    // 首个实例由这里收集并写入 pending.json，setup 阶段再入队（避免窗口未就绪竞态）。
    let paths: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| Path::new(a).is_file())
        .collect();
    if !paths.is_empty() {
        let dir = liteai_config::default_config_dir();
        if std::fs::create_dir_all(&dir).is_ok() {
            let pending = dir.join("pending.json");
            let existing: Vec<String> = std::fs::read_to_string(&pending)
                .ok()
                .and_then(|raw| serde_json::from_str(&raw).ok())
                .unwrap_or_default();
            let mut all = existing;
            all.extend(paths);
            let _ = std::fs::write(&pending, serde_json::to_string(&all).unwrap_or_default());
        }
    }
    liteai_app_lib::run();
}

//! Tauri 命令层：分析（Channel 流式）、配置、模板、右键菜单。

use crate::shell_integration;
use crate::AppState;
use liteai_config::Template;
use liteai_core::*;
use liteai_model::OpenAiClient;
use std::path::Path;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};

/// 白名单过滤 + 构建 FileMeta 列表。
fn resolve_files(state: &AppState, paths: &[String]) -> Vec<FileMeta> {
    let cfg = state.config.lock().unwrap().clone();
    let mut out = Vec::new();
    for p in paths {
        let path = Path::new(p);
        let ext = path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase());
        // 白名单为空表示全允许
        let allowed = cfg.prefs.whitelist.is_empty()
            || ext.as_deref().map(|e| cfg.prefs.whitelist.contains(&e.to_string())).unwrap_or(false);
        if !allowed {
            continue;
        }
        if let Ok(m) = FileMeta::from_path(path) {
            out.push(m);
        }
    }
    out
}

/// 流式分析：增量 token 通过 Channel 推送前端。
#[tauri::command]
pub async fn analyze_files(
    app: AppHandle,
    paths: Vec<String>,
    on_event: tauri::ipc::Channel<PipelineEvent>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    state.cancel.store(false, Ordering::Relaxed);

    let cfg = state.config.lock().unwrap().clone();
    let base_url = cfg.api.base_url.clone();
    let model = cfg.api.model.clone();
    let out_cfg = OutputConfig {
        mode: cfg.prefs.output_mode,
        out_dir: cfg.prefs.output_dir.clone(),
        export_docx: cfg.prefs.export_docx,
    };
    let key = state
        .secrets
        .lock()
        .unwrap()
        .get("api_key")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "尚未配置 API Key，请到「设置」中填写".to_string())?;

    let template = liteai_config::all_templates(&state.config_dir)
        .into_iter()
        .find(|t| t.id == cfg.active_template_id)
        .unwrap_or_else(|| liteai_config::builtin_templates().remove(0));

    let files = resolve_files(&state, &paths);
    if files.is_empty() {
        return Err("没有符合白名单的可分析文件".into());
    }

    let pipeline = AnalysisPipeline {
        parsers: liteai_parsers::default_registry(),
        model: Box::new(OpenAiClient::new(key, base_url.clone())),
        md_serializer: Box::new(liteai_output::MarkdownSerializer),
        docx_serializer: Some(Box::new(liteai_output::DocxSerializer)),
        prompt: PromptBuilder::new(template.system, template.prompt),
    };

    // 从 pending 中移除本次已入队的文件
    {
        let mut pending = state.pending.lock().unwrap();
        for p in &paths {
            pending.retain(|x| x != p);
        }
    }
    let _ = app.emit("queue-updated", ());

    let model_cfg = ModelConfig { base_url, model };
    let cancel = state.cancel.clone();
    let events = on_event;

    let _ = tauri::async_runtime::spawn(async move {
        let _ = pipeline
            .analyze_batch(files, &model_cfg, &out_cfg, &mut |ev| {
                events.send(ev).map_err(|_| ())
            }, Some(&cancel))
            .await;
    });
    Ok(())
}

/// 取消当前分析。
#[tauri::command]
pub fn cancel_all(app: AppHandle) {
    app.state::<AppState>().cancel.store(true, Ordering::Relaxed);
}

/// 待分析队列（右键触发后暂存的文件）。
#[tauri::command]
pub fn get_pending(app: AppHandle) -> Vec<String> {
    app.state::<AppState>().pending.lock().unwrap().clone()
}

// ---------- 配置 ----------

#[tauri::command]
pub fn get_config(app: AppHandle) -> liteai_config::AppConfig {
    app.state::<AppState>().config.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: liteai_config::AppConfig) -> Result<(), String> {
    let state = app.state::<AppState>();
    *state.config.lock().unwrap() = config.clone();
    liteai_config::save_config(&state.config_dir, &config)
}

#[tauri::command]
pub fn set_api_key(app: AppHandle, key: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let result = state.secrets.lock().unwrap().set("api_key", &key);
    result.map_err(|e| e.to_string())
}

/// 清除已保存的 API Key（从 Windows 凭据管理器/回退文件中删除）。
#[tauri::command]
pub fn delete_api_key(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let result = state.secrets.lock().unwrap().delete("api_key");
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn has_api_key(app: AppHandle) -> bool {
    app.state::<AppState>()
        .secrets
        .lock()
        .unwrap()
        .get("api_key")
        .ok()
        .flatten()
        .map(|k| !k.is_empty())
        .unwrap_or(false)
}

/// 连通性测试 + 余额查询（一次完成）。
#[tauri::command]
pub async fn test_connection(app: AppHandle, base_url: String, model: String) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();
    let key = state.secrets.lock().unwrap().get("api_key").ok().flatten().unwrap_or_default();
    let client = OpenAiClient::new(key, base_url.clone());
    client.ping(&base_url, &model).await.map_err(|e| e.to_string())?;
    let balance = client.check_balance().await.map_err(|e| e.to_string())?;
    serde_json::to_value(balance).map_err(|e| e.to_string())
}

// ---------- 模板 ----------

#[tauri::command]
pub fn get_templates(app: AppHandle) -> Vec<Template> {
    let state = app.state::<AppState>();
    liteai_config::all_templates(&state.config_dir)
}

#[tauri::command]
pub fn save_templates(app: AppHandle, templates: Vec<Template>) -> Result<(), String> {
    let state = app.state::<AppState>();
    liteai_config::save_custom_templates(&state.config_dir, &templates)
}

#[tauri::command]
pub fn import_templates(app: AppHandle, json: String) -> Result<Vec<Template>, String> {
    let _ = app;
    liteai_config::import_templates(&json)
}

// ---------- 右键菜单 ----------

#[tauri::command]
pub fn register_shell_menu() -> Result<(), String> {
    shell_integration::register()
}

#[tauri::command]
pub fn unregister_shell_menu() -> Result<(), String> {
    shell_integration::unregister()
}

#[tauri::command]
pub fn shell_menu_registered() -> bool {
    shell_integration::is_registered()
}


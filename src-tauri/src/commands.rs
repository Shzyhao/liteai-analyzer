//! Tauri 命令层：分析（Channel 流式）、配置、模板、右键菜单。

use crate::shell_integration;
use crate::AppState;
use liteai_config::Template;
use liteai_core::*;
use liteai_model::OpenAiClient;
use std::path::Path;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};

/// 当前使用的 API 配置。
fn active_profile_cfg(state: &AppState) -> Result<liteai_config::ApiProfile, String> {
    let cfg = state.config.lock().unwrap().clone();
    liteai_config::active_profile(&cfg)
        .cloned()
        .ok_or_else(|| "未配置有效的 API 配置".to_string())
}

/// 取某配置的密钥（键名 `api_key:<id>`；default 兼容旧版 "api_key"）。
fn get_profile_key(state: &AppState, profile_id: &str) -> Result<String, String> {
    let store = state.secrets.lock().unwrap();
    let key = store
        .get(&format!("api_key:{profile_id}"))
        .map_err(|e| e.to_string())?
        .or_else(|| {
            if profile_id == "default" {
                store.get("api_key").ok().flatten()
            } else {
                None
            }
        });
    key.filter(|k| !k.is_empty())
        .ok_or_else(|| "该 API 配置尚未填写 Key".to_string())
}

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

    let profile = active_profile_cfg(&state)?;
    let base_url = profile.base_url.clone();
    let model = profile.model.clone();
    let key = get_profile_key(&state, &profile.id)?;

    let cfg = state.config.lock().unwrap().clone();
    let out_cfg = OutputConfig {
        mode: cfg.prefs.output_mode,
        out_dir: cfg.prefs.output_dir.clone(),
        export_docx: cfg.prefs.export_docx,
        export_xlsx: cfg.prefs.export_xlsx,
    };

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
        xlsx_serializer: Some(Box::new(liteai_output::XlsxSerializer)),
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
    let config_dir = state.config_dir.clone();
    let template_name = template.name.clone();

    let _ = tauri::async_runtime::spawn(async move {
        let outcome = pipeline
            .analyze_batch(files, &model_cfg, &out_cfg, &mut |ev| {
                events.send(ev).map_err(|_| ())
            }, Some(&cancel))
            .await;
        // 分析完成后写入历史
        if let Ok(outcome) = outcome {
            for r in &outcome.results {
                let id = format!(
                    "h{}-{}",
                    r.file.file_name.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0)
                );
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                let entry = liteai_config::HistoryEntry {
                    id,
                    timestamp_ms: ts,
                    source_file: r.file.path.display().to_string(),
                    template: template_name.clone(),
                    analysis: r.analysis.clone(),
                    output_files: r.output_path.clone().into_iter().map(|p| p.display().to_string()).collect(),
                    prompt_tokens: r.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0),
                    completion_tokens: r.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0),
                };
                let _ = liteai_config::append_history(&config_dir, entry);
            }
        }
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
pub fn set_api_key(app: AppHandle, profile_id: String, key: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let result = state.secrets.lock().unwrap().set(&format!("api_key:{profile_id}"), &key);
    result.map_err(|e| e.to_string())
}

/// 清除指定配置的 API Key。
#[tauri::command]
pub fn delete_api_key(app: AppHandle, profile_id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    let result = state.secrets.lock().unwrap().delete(&format!("api_key:{profile_id}"));
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn has_api_key(app: AppHandle, profile_id: String) -> bool {
    let state = app.state::<AppState>();
    get_profile_key(&state, &profile_id).map(|k| !k.is_empty()).unwrap_or(false)
}

/// 连通性测试 + 余额查询（一次完成）。
#[tauri::command]
pub async fn test_connection(
    app: AppHandle,
    profile_id: String,
    base_url: String,
    model: String,
) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();
    let key = get_profile_key(&state, &profile_id)?;
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

// ---------- 导出技能 ----------

#[tauri::command]
pub fn get_skills(app: AppHandle) -> Vec<liteai_config::ExportSkill> {
    let state = app.state::<AppState>();
    liteai_config::load_skills(&state.config_dir)
}

/// 当前技能存放目录（配置值或默认桌面\liteai-skills，仅展示不校验）。
#[tauri::command]
pub fn get_skills_dir(app: AppHandle) -> String {
    let state = app.state::<AppState>();
    let cfg = state.config.lock().unwrap().clone();
    let dir = match cfg.prefs.skills_dir {
        Some(d) => d,
        None => liteai_config::default_skills_dir(&state.config_dir),
    };
    dir.display().to_string()
}

/// 检查一个目录路径是否存在（设置技能目录时用）。
#[tauri::command]
pub fn check_path_exists(path: String) -> bool {
    std::path::Path::new(&path).is_dir()
}

#[tauri::command]
pub fn save_skills(app: AppHandle, skills: Vec<liteai_config::ExportSkill>) -> Result<(), String> {
    let state = app.state::<AppState>();
    liteai_config::save_skills(&state.config_dir, &skills)
}

/// 运行导出技能：把分析结果写入临时文件，作为最后一个参数传给脚本，
/// 并通过环境变量提供 LITEAI_SOURCE_FILE / LITEAI_ANALYSIS_FILE / LITEAI_OUTPUT_DIR。
#[tauri::command]
pub async fn run_skill(
    app: AppHandle,
    skill_id: String,
    source_file: String,
    analysis_text: String,
) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();
    let skills = liteai_config::load_skills(&state.config_dir);
    let skill = skills
        .iter()
        .find(|s| s.id == skill_id)
        .ok_or_else(|| "导出技能不存在".to_string())?
        .clone();

    // 写分析结果到临时文件
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let analysis_file = std::env::temp_dir().join(format!("liteai-analysis-{ts}.md"));
    std::fs::write(&analysis_file, &analysis_text).map_err(|e| format!("写入临时文件失败: {e}"))?;

    let out_dir = std::path::Path::new(&source_file)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(std::env::temp_dir);

    let mut cmd = tokio::process::Command::new(&skill.command);
    for a in split_args(&skill.args) {
        cmd.arg(a);
    }
    cmd.arg(&analysis_file); // 最后一个位置参数 = 分析结果文件
    cmd.env("LITEAI_SOURCE_FILE", &source_file);
    cmd.env("LITEAI_ANALYSIS_FILE", &analysis_file);
    cmd.env("LITEAI_OUTPUT_DIR", &out_dir);
    if let Some(cwd) = &skill.cwd {
        cmd.current_dir(cwd);
    }
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let run = async {
        let output = cmd
            .output()
            .await
            .map_err(|e| format!("启动脚本失败: {e}"))?;
        Ok::<_, String>(output)
    };

    match tokio::time::timeout(std::time::Duration::from_secs(120), run).await {
        Ok(Ok(output)) => Ok(serde_json::json!({
            "success": output.status.success(),
            "exit_code": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
            "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
            "analysis_file": analysis_file.display().to_string(),
            "output_dir": out_dir.display().to_string(),
        })),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("脚本执行超时（120 秒）".into()),
    }
}

/// AI 生成的严格安全提示词：约束模型只生成「读入结果 + 写一个输出文件」的安全脚本。
const STRICT_SKILL_PROMPT: &str = r#"你是 Windows 本地脚本生成助手。根据用户需求描述，生成一个【数据导出】脚本。

【硬性安全约束 - 违反任一即视为失败】
1. 脚本只做两件事：读取分析结果输入文件 + 生成一个输出文件。禁止任何其他行为。
2. 禁止一切网络操作（禁止 requests/urllib/socket/http 等；禁止访问任何 URL）。
3. 禁止删除、修改、移动任何现有文件或目录（只允许创建新的输出文件）。
4. 禁止修改注册表、系统设置、环境变量；禁止启动/执行其它程序或命令（禁止 os.system、subprocess、Invoke-Expression 等）。
5. 只读给定的输入文件，不扫描、不读取用户主目录或其他敏感路径。
6. 脚本必须自包含、可独立运行、无副作用、无恶意行为。

【输入输出约定】
- 输入：分析结果文件路径 = 脚本最后一个参数（Python: sys.argv[-1]；PowerShell: $args[-1]），也可用环境变量 LITEAI_ANALYSIS_FILE。
- 输出：必须写入环境变量 LITEAI_OUTPUT_DIR 指向的目录，输出文件名由脚本自拟（如 report.html）。

【语言要求】
- 默认生成 PowerShell 脚本（Windows 自带，无需额外安装）；仅当用户明确要求时才用 Python 等其他语言。
- 只输出一个完整、可直接运行的脚本，用 ``` 代码块包裹并标注语言（如 ```powershell 或 ```python）。不要输出任何解释文字。"#;

/// 从模型输出中提取代码块：(语言, 代码)。
fn extract_code(output: &str) -> (String, String) {
    let mut lang = "powershell".to_string();
    let mut code = String::new();
    let mut in_code = false;
    for line in output.lines() {
        let t = line.trim_start();
        if t.starts_with("```") {
            let rest = t.trim_start_matches('`').trim();
            if !rest.is_empty() {
                lang = rest.to_lowercase();
            }
            in_code = !in_code;
            continue;
        }
        if in_code {
            code.push_str(line);
            code.push('\n');
        }
    }
    (lang, code.trim().to_string())
}

/// 用 AI 根据一句话描述生成导出技能：调模型 → 提取脚本 → 存到技能目录 → 注册技能。
#[tauri::command]
pub async fn generate_skill(app: AppHandle, description: String) -> Result<serde_json::Value, String> {
    let state = app.state::<AppState>();
    let cfg = state.config.lock().unwrap().clone();
    let profile = active_profile_cfg(&state)?;
    let key = get_profile_key(&state, &profile.id)?;
    if description.trim().is_empty() {
        return Err("请描述你想要的导出效果".into());
    }

    // 调用模型生成
    let client = OpenAiClient::new(key, profile.base_url.clone());
    let req = ChatRequest {
        base_url: profile.base_url.clone(),
        model: profile.model.clone(),
        messages: vec![
            ChatMessage { role: "system".into(), content: STRICT_SKILL_PROMPT.into() },
            ChatMessage { role: "user".into(), content: format!("需求描述：{}\n\n请生成脚本。", description.trim()) },
        ],
        temperature: 0.2,
    };
    let mut output = String::new();
    client
        .stream_chat(&req, &mut |tok: String| {
            output.push_str(&tok);
            Ok(())
        })
        .await
        .map_err(|e| format!("AI 生成失败: {e}"))?;

    let (lang, code) = extract_code(&output);
    if code.is_empty() {
        return Err("AI 未能生成有效脚本，请换个描述重试".into());
    }

    // 解析语言 → 扩展名 / 命令
    let (ext, command, args) = match lang.as_str() {
        "python" | "py" => ("py", "python".to_string(), format!("\"{}\"", "{path}")),
        "bat" | "batch" | "cmd" => ("bat", "".to_string(), "{path}".to_string()),
        _ => ("ps1", "powershell".to_string(), "-NoProfile -ExecutionPolicy Bypass -File \"{path}\"".to_string()),
    };

    // 技能存放目录（自定义路径须已存在，否则报错；默认桌面\liteai-skills 自动创建）
    let skills_dir = liteai_config::resolve_skills_dir(cfg.prefs.skills_dir.as_deref(), &state.config_dir)?;

    // 每个技能独立子文件夹：<skills_dir>/<slug>-<ts>/
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let slug = slugify(&description);
    let skill_dir = skills_dir.join(format!("{slug}-{ts}"));
    std::fs::create_dir_all(&skill_dir).map_err(|e| format!("创建技能文件夹失败: {e}"))?;

    let script_path = skill_dir.join(format!("ai_skill.{ext}"));
    std::fs::write(&script_path, &code).map_err(|e| format!("保存脚本失败: {e}"))?;

    // 注册技能并持久化
    let name = format!("AI生成·{}", truncate_zh(&description, 10));
    let args_str = args.replace("{path}", &format!("\"{}\"", script_path.display()));
    let skill = liteai_config::ExportSkill::new(name, command, args_str);
    let mut skills = liteai_config::load_skills(&state.config_dir);
    skills.push(skill.clone());
    liteai_config::save_skills(&state.config_dir, &skills)?;

    Ok(serde_json::json!({
        "skill": skill,
        "script_path": script_path.display().to_string(),
        "skills_dir": skills_dir.display().to_string(),
    }))
}

/// 把描述转成安全的目录名：仅保留字母数字和连字符，其余替换为 `-`，限长。
fn slugify(s: &str) -> String {
    let mut slug: String = s
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    slug = slug.trim_matches('-').to_string();
    slug = slug.chars().take(20).collect();
    if slug.is_empty() {
        "skill".to_string()
    } else {
        slug
    }
}

fn truncate_zh(s: &str, max_chars: usize) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    if chars.len() > max_chars {
        chars.truncate(max_chars);
        format!("{}…", chars.into_iter().collect::<String>())
    } else {
        s.to_string()
    }
}

/// 按引号感知方式拆分命令行参数（支持 `"C:\my path\a.py"`）。
fn split_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    for c in s.chars() {
        match c {
            '"' => in_quote = !in_quote,
            c if c.is_whitespace() && !in_quote => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

// ---------- 历史记录 ----------

#[tauri::command]
pub fn get_history(app: AppHandle) -> Vec<liteai_config::HistoryEntry> {
    let state = app.state::<AppState>();
    liteai_config::load_history(&state.config_dir)
}

#[tauri::command]
pub fn delete_history_entry(app: AppHandle, id: String) -> Result<(), String> {
    let state = app.state::<AppState>();
    liteai_config::delete_history_entry(&state.config_dir, &id)
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    liteai_config::clear_history(&state.config_dir)
}

// ---------- 打开文件/文件夹 ----------

/// 在资源管理器中打开文件所在文件夹（或直接打开文件/目录）。
#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("路径不存在：{path}"));
    }
    if p.is_dir() {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开失败: {e}"))?;
    } else {
        // 选中文件并打开所在文件夹
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| format!("打开失败: {e}"))?;
    }
    Ok(())
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


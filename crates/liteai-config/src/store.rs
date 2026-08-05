//! 应用配置：JSON 持久化于用户数据目录。API Key 不落此文件（存 SecretStore）。

use liteai_core::OutputMode;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const SERVICE_NAME: &str = "com.liteai.analyzer";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 多套 API 配置，同一时刻只使用 active_profile_id 指向的一套
    pub api_profiles: Vec<ApiProfile>,
    pub active_profile_id: String,
    pub prefs: Prefs,
    pub active_template_id: String,
}

/// 一套 API 配置。密钥单独存凭据管理器，键名 `api_key:<id>`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProfile {
    pub id: String,
    pub name: String,
    /// OpenAI 兼容 Base URL，如 https://api.deepseek.com
    pub base_url: String,
    pub model: String,
}

impl ApiProfile {
    pub fn new(name: impl Into<String>, base_url: impl Into<String>, model: impl Into<String>) -> Self {
        let name: String = name.into();
        let base_url: String = base_url.into();
        let id = format!("p{}", crate::skills::id_seed(&format!("{name}{base_url}")));
        Self {
            id,
            name,
            base_url,
            model: model.into(),
        }
    }
}

/// 返回当前使用的配置。
pub fn active_profile(cfg: &AppConfig) -> Option<&ApiProfile> {
    cfg.api_profiles.iter().find(|p| p.id == cfg.active_profile_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefs {
    pub output_mode: OutputMode,
    /// 除 Markdown 外是否额外导出 Word
    pub export_docx: bool,
    /// 除 Markdown 外是否额外导出 Excel（旧配置兼容）
    #[serde(default)]
    pub export_xlsx: bool,
    /// None = 保存到源文件旁；Some = 指定输出目录
    pub output_dir: Option<PathBuf>,
    /// 后缀白名单（小写，不含点）
    pub whitelist: Vec<String>,
    /// 无痕模式：不保留历史
    pub incognito: bool,
    /// AI 生成技能的存放目录（None = 默认桌面）
    #[serde(default)]
    pub skills_dir: Option<PathBuf>,
    /// 主题：light / dark / system
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_theme() -> String {
    "system".into()
}

/// 默认技能目录：桌面\liteai-skills。
pub fn default_skills_dir(fallback: &Path) -> PathBuf {
    dirs::desktop_dir()
        .unwrap_or_else(|| fallback.to_path_buf())
        .join("liteai-skills")
}

/// 解析技能存放目录：
/// - 用户配置了路径：必须已存在（否则 Err），不自动创建。
/// - 未配置：默认桌面\liteai-skills，不存在则自动创建。
pub fn resolve_skills_dir(prefs_skills_dir: Option<&Path>, fallback: &Path) -> Result<PathBuf, String> {
    if let Some(d) = prefs_skills_dir {
        if !d.is_dir() {
            return Err(format!("技能目录不存在：{}", d.display()));
        }
        return Ok(d.to_path_buf());
    }
    let dir = default_skills_dir(fallback);
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建技能目录失败: {e}"))?;
    Ok(dir)
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_profiles: vec![ApiProfile {
                id: "default".into(),
                name: "DeepSeek".into(),
                base_url: "https://api.deepseek.com".into(),
                model: "deepseek-chat".into(),
            }],
            active_profile_id: "default".into(),
            prefs: Prefs {
                output_mode: OutputMode::Both,
                export_docx: false,
                export_xlsx: false,
                output_dir: None,
                whitelist: default_whitelist(),
                incognito: false,
                skills_dir: None,
                theme: "system".into(),
            },
            active_template_id: "summary".into(),
        }
    }
}

/// 默认支持解析的后缀。
pub fn default_whitelist() -> Vec<String> {
    [
        "txt", "md", "pdf", "xlsx", "xlsm", "docx", "csv", "log", "json",
        "xml", "yaml", "yml", "html", "sql", "py", "js", "ts", "java", "rs", "cpp",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// 应用数据目录：`%APPDATA%/liteai`。
pub fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("liteai")
}

pub fn config_file(base: &Path) -> PathBuf {
    base.join("config.json")
}

/// 读取配置；文件不存在则返回默认配置（不写入）。旧版单 API 配置自动迁移为第一个配置。
pub fn load_config(base: &Path) -> Result<AppConfig, String> {
    let path = config_file(base);
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("读取配置失败: {e}"))?;
    let mut v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| format!("配置格式错误: {e}"))?;

    // 迁移：旧版 `api` 字段 → `api_profiles`
    if v.get("api_profiles").is_none() {
        if let Some(api) = v.get("api").cloned() {
            let base_url = api
                .get("base_url")
                .and_then(|x| x.as_str())
                .unwrap_or("https://api.deepseek.com")
                .to_string();
            let model = api
                .get("model")
                .and_then(|x| x.as_str())
                .unwrap_or("deepseek-chat")
                .to_string();
            v["api_profiles"] = serde_json::json!([{
                "id": "default",
                "name": "DeepSeek",
                "base_url": base_url,
                "model": model,
            }]);
            v["active_profile_id"] = serde_json::json!("default");
        }
    }
    serde_json::from_value(v).map_err(|e| format!("配置格式错误: {e}"))
}

/// 保存配置（自动创建目录）。
pub fn save_config(base: &Path, cfg: &AppConfig) -> Result<(), String> {
    std::fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let raw = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(config_file(base), raw).map_err(|e| format!("写入配置失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut cfg = AppConfig::default();
        cfg.active_profile_id = "default".into();
        cfg.api_profiles[0].model = "deepseek-reasoner".into();
        cfg.prefs.output_mode = OutputMode::UiOnly;
        save_config(dir.path(), &cfg).unwrap();

        let loaded = load_config(dir.path()).unwrap();
        assert_eq!(loaded.api_profiles[0].model, "deepseek-reasoner");
        assert_eq!(loaded.prefs.output_mode, OutputMode::UiOnly);
        assert!(loaded.prefs.whitelist.contains(&"pdf".to_string()));
        assert!(active_profile(&loaded).is_some());
    }

    #[test]
    fn missing_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = load_config(dir.path()).unwrap();
        assert_eq!(cfg.api_profiles[0].base_url, "https://api.deepseek.com");
    }

    #[test]
    fn migrates_old_single_api() {
        let dir = tempfile::tempdir().unwrap();
        // 模拟真实旧版配置：prefs 不含 export_xlsx / skills_dir / theme
        let old = r#"{"api":{"base_url":"https://api.openai.com","model":"gpt-4o"},"prefs":{"output_mode":"both","export_docx":false,"output_dir":null,"whitelist":["txt","pdf"],"incognito":false},"active_template_id":"summary"}"#;
        std::fs::write(dir.path().join("config.json"), old).unwrap();
        let cfg = load_config(dir.path()).unwrap();
        assert_eq!(cfg.api_profiles.len(), 1);
        assert_eq!(cfg.api_profiles[0].base_url, "https://api.openai.com");
        assert_eq!(cfg.active_profile_id, "default");
    }
}

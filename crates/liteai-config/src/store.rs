//! 应用配置：JSON 持久化于用户数据目录。API Key 不落此文件（存 SecretStore）。

use liteai_core::OutputMode;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const SERVICE_NAME: &str = "com.liteai.analyzer";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub prefs: Prefs,
    pub active_template_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// OpenAI 兼容 Base URL，如 https://api.deepseek.com
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefs {
    pub output_mode: OutputMode,
    /// 除 Markdown 外是否额外导出 Word
    pub export_docx: bool,
    /// None = 保存到源文件旁；Some = 指定输出目录
    pub output_dir: Option<PathBuf>,
    /// 后缀白名单（小写，不含点）
    pub whitelist: Vec<String>,
    /// 无痕模式：不保留历史
    pub incognito: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig {
                base_url: "https://api.deepseek.com".into(),
                model: "deepseek-chat".into(),
            },
            prefs: Prefs {
                output_mode: OutputMode::Both,
                export_docx: false,
                output_dir: None,
                whitelist: default_whitelist(),
                incognito: false,
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

/// 读取配置；文件不存在则返回默认配置（不写入）。
pub fn load_config(base: &Path) -> Result<AppConfig, String> {
    let path = config_file(base);
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("读取配置失败: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("配置格式错误: {e}"))
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
        cfg.api.model = "deepseek-reasoner".into();
        cfg.prefs.output_mode = OutputMode::UiOnly;
        save_config(dir.path(), &cfg).unwrap();

        let loaded = load_config(dir.path()).unwrap();
        assert_eq!(loaded.api.model, "deepseek-reasoner");
        assert_eq!(loaded.prefs.output_mode, OutputMode::UiOnly);
        assert!(loaded.prefs.whitelist.contains(&"pdf".to_string()));
    }

    #[test]
    fn missing_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = load_config(dir.path()).unwrap();
        assert_eq!(cfg.api.base_url, "https://api.deepseek.com");
    }
}

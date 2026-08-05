//! 领域模型：跨 crate 与 IPC 边界共享的类型定义。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 单文件正文注入 prompt 的最大字符数，超出截断。
pub const MAX_CONTENT_CHARS: usize = 60_000;

/// 待分析文件元信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub path: PathBuf,
    pub file_name: String,
    pub size_bytes: u64,
}

impl FileMeta {
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let md = std::fs::metadata(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            file_name: path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
            size_bytes: md.len(),
        })
    }
}

/// 解析器产出的文档文本。
#[derive(Debug, Clone)]
pub struct ExtractedDocument {
    pub meta: FileMeta,
    pub text: String,
    pub char_count: usize,
    pub truncated: bool,
    pub mime: String,
}

impl ExtractedDocument {
    /// 构造并统一做超长截断。
    pub fn new(meta: FileMeta, text: String, mime: impl Into<String>) -> Self {
        let char_count = text.chars().count();
        let truncated = char_count > MAX_CONTENT_CHARS;
        let text: String = if truncated {
            text.chars().take(MAX_CONTENT_CHARS).collect()
        } else {
            text
        };
        Self {
            meta,
            char_count,
            truncated,
            text,
            mime: mime.into(),
        }
    }
}

/// OpenAI 兼容的对话消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// 一次模型请求。
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub base_url: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
}

/// Token 消耗统计（取自流式响应末尾的 usage 字段）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

/// 账号余额查询结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub is_available: bool,
    pub balance_infos: Vec<CurrencyBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyBalance {
    pub currency: String,
    pub total_balance: String,
}

/// 结果输出方式（配置记忆、可切换）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    #[default]
    UiOnly,
    FileOnly,
    Both,
}

/// 分析流程事件：驱动前端 IPC Channel 与 CLI 打印的同一枚举。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum PipelineEvent {
    Started { total: usize },
    Parsing { index: usize, file: String },
    Tokens { text: String },
    FileDone { index: usize, output_path: Option<String>, usage: Option<ChatUsage> },
    Done { summary: String },
    Error { file: Option<String>, message: String },
    Cancelled,
}

/// 序列化器接收的成品文档。
#[derive(Debug, Clone)]
pub struct OutputDocument {
    pub source_name: String,
    pub source_text: String,
    pub truncated: bool,
    pub analysis: String,
}

/// 单文件分析结果。
#[derive(Debug, Clone)]
pub struct FileResult {
    pub index: usize,
    pub file: FileMeta,
    pub analysis: String,
    pub usage: Option<ChatUsage>,
    pub output_path: Option<PathBuf>,
}

/// 整批分析结果。
#[derive(Debug, Clone, Default)]
pub struct BatchOutcome {
    pub results: Vec<FileResult>,
    pub cancelled: bool,
}

impl BatchOutcome {
    pub fn succeeded(&self) -> usize {
        self.results.len()
    }
}

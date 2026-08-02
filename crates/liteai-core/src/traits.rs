//! 核心抽象接口：解析器 / 模型客户端 / 序列化器 / 密钥存储 / 企业扩展点。
//!
//! 全部使用 `Send + Sync` trait object，便于在各层（GUI、CLI、Tauri state）自由组合。

use crate::domain::*;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("不支持的文件类型")]
    Unsupported,
    #[error("读取失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("解析失败: {0}")]
    Other(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("网络错误: {0}")]
    Network(String),
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("鉴权失败，请检查 API Key")]
    Auth,
    #[error("请求已取消")]
    Cancelled,
    #[error("流式解析失败: {0}")]
    Stream(String),
}

#[derive(Debug, thiserror::Error)]
pub enum OutputError {
    #[error("写入失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("序列化失败: {0}")]
    Other(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("凭据系统不可用: {0}")]
    Unavailable(String),
    #[error("密钥操作失败: {0}")]
    Other(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("分析已取消")]
    Cancelled,
    #[error("没有可分析的文件")]
    NoFiles,
    #[error("处理失败: {0}")]
    Other(String),
}

/// 文件解析器。同步实现（pdf-extract 为阻塞调用）。
pub trait Parser: Send + Sync {
    /// 支持的扩展名（不含点，小写）。
    fn extensions(&self) -> &'static [&'static str];
    fn parse(&self, path: &Path) -> Result<ExtractedDocument, ParseError>;
}

/// OpenAI 兼容模型客户端。回调式流式，避免 `impl Stream` 装箱烦恼。
#[async_trait]
pub trait ModelClient: Send + Sync {
    /// 流式对话：每产出一个 token 片段就调用 `on_token`。
    /// `on_token` 返回 `Err` 表示取消。要求 `Send` 以便跨线程传递。
    async fn stream_chat(
        &self,
        req: &ChatRequest,
        on_token: &mut (dyn FnMut(String) -> Result<(), ModelError> + Send),
    ) -> Result<ChatUsage, ModelError>;

    async fn check_balance(&self) -> Result<BalanceInfo, ModelError>;

    /// 连通性测试。
    async fn ping(&self, base_url: &str, model: &str) -> Result<(), ModelError>;
}

/// 结果序列化器（落盘文件）。
pub trait Serializer: Send + Sync {
    /// 生成文件的扩展名，如 "md" / "docx"。
    fn extension(&self) -> &'static str;
    /// 将成品文档写入 `out_dir`，返回生成文件的完整路径。
    fn serialize(&self, doc: &OutputDocument, out_dir: &Path) -> Result<PathBuf, OutputError>;
}

/// 密钥安全存储。两个实现：Windows 凭据管理器（keyring）、AES 文件回退。
pub trait SecretStore: Send + Sync {
    fn set(&self, key: &str, value: &str) -> Result<(), SecretError>;
    fn get(&self, key: &str) -> Result<Option<String>, SecretError>;
    fn delete(&self, key: &str) -> Result<(), SecretError>;
}

/// 企业版扩展点。MVP 阶段不实现任何 Hook，仅预留接口。
pub trait EnterpriseHook: Send + Sync {
    fn before_send(&mut self, _file: &FileMeta) -> Result<(), String> {
        Ok(())
    }
    fn after_finish(&mut self, _file: &FileMeta, _usage: &ChatUsage) -> Result<(), String> {
        Ok(())
    }
}

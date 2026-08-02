//! liteai-output：分析结果的落盘序列化器。
//!
//! `MarkdownSerializer` 输出 `<源名>.ai.md`；`DocxSerializer` 输出 `<源名>.ai.docx`（Markdown → Word 迷你转换）。

pub mod markdown;
pub mod docx;

pub use docx::DocxSerializer;
pub use markdown::MarkdownSerializer;

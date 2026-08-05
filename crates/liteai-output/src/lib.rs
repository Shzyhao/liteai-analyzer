//! liteai-output：分析结果的落盘序列化器。
//!
//! `MarkdownSerializer` 输出 `<源名>.ai.md`；`DocxSerializer` 输出 `<源名>.ai.docx`；`XlsxSerializer` 输出 `<源名>.ai.xlsx`。

pub mod docx;
pub mod markdown;
pub mod xlsx;

pub use docx::DocxSerializer;
pub use markdown::MarkdownSerializer;
pub use xlsx::XlsxSerializer;

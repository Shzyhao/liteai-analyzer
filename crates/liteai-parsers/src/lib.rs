//! liteai-parsers：文件内容提取器。
//!
//! 四个实现：txt（UTF-8/GBK 容错）、xlsx（calamine）、docx（zip+quick-xml）、pdf（pdf-extract）。
//! 通过 `default_registry()` 注册到 `liteai-core::ParserRegistry`。

pub mod docx;
pub mod pdf;
pub mod txt;
pub mod xlsx;

use liteai_core::ParserRegistry;
use std::sync::Arc;

/// 返回注册好全部内置解析器的注册表。
pub fn default_registry() -> ParserRegistry {
    let mut r = ParserRegistry::new();
    r.register(Arc::new(txt::TxtParser));
    r.register(Arc::new(xlsx::XlsxParser));
    r.register(Arc::new(docx::DocxParser));
    r.register(Arc::new(pdf::PdfParser));
    r
}

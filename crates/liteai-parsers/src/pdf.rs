//! PDF 解析器（pdf-extract）。
//!
//! 注意：pdf-extract 内嵌 xpdf C++，部分畸形 PDF 会 panic，必须用 catch_unwind 包裹并兜底。

use liteai_core::{ExtractedDocument, FileMeta, ParseError, Parser};
use std::path::Path;
use std::panic::{catch_unwind, AssertUnwindSafe};

pub struct PdfParser;

impl Parser for PdfParser {
    fn extensions(&self) -> &'static [&'static str] {
        &["pdf"]
    }

    fn parse(&self, path: &Path) -> Result<ExtractedDocument, ParseError> {
        let meta = FileMeta::from_path(path)?;
        let text = match catch_unwind(AssertUnwindSafe(|| pdf_extract::extract_text(path))) {
            Ok(Ok(t)) => t,
            Ok(Err(e)) => return Err(ParseError::Other(format!("PDF 解析失败: {e}"))),
            Err(_) => return Err(ParseError::Other("PDF 解析器内部错误（可能为加密或畸形 PDF）".into())),
        };
        if text.trim().is_empty() {
            return Err(ParseError::Other("PDF 中没有可提取的文本（可能是扫描件或加密文档）".into()));
        }
        Ok(ExtractedDocument::new(meta, text, "application/pdf"))
    }
}

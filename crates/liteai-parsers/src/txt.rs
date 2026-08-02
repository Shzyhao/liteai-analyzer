//! 纯文本解析器：优先 UTF-8，失败则按 GBK 解码（兼容国内常见编码）。

use liteai_core::{ExtractedDocument, FileMeta, ParseError, Parser};
use std::path::Path;

pub struct TxtParser;

impl Parser for TxtParser {
    fn extensions(&self) -> &'static [&'static str] {
        &["txt", "md", "log", "csv", "json", "xml", "yaml", "yml", "ini", "conf", "bat", "sh", "py", "js", "ts", "rs", "java", "c", "cpp", "h", "html", "css", "sql"]
    }

    fn parse(&self, path: &Path) -> Result<ExtractedDocument, ParseError> {
        let meta = FileMeta::from_path(path)?;
        let bytes = std::fs::read(path)?;

        let text = match String::from_utf8(bytes.clone()) {
            Ok(s) => s,
            Err(_) => {
                let (cow, _, _) = encoding_rs::GBK.decode(&bytes);
                cow.into_owned()
            }
        };
        Ok(ExtractedDocument::new(meta, text, "text/plain"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_utf8() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("a.txt");
        std::fs::write(&f, "你好，世界\nhello").unwrap();
        let doc = TxtParser.parse(&f).unwrap();
        assert_eq!(doc.text, "你好，世界\nhello");
        assert!(!doc.truncated);
    }

    #[test]
    fn parses_gbk_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("b.txt");
        // "中文测试" 的 GBK 字节
        let gbk = encoding_rs::GBK.encode("中文测试").0;
        std::fs::write(&f, gbk).unwrap();
        let doc = TxtParser.parse(&f).unwrap();
        assert!(doc.text.contains("中文测试"), "got: {}", doc.text);
    }
}

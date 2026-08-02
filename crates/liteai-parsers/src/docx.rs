//! Word(.docx) 解析器：解压后读取 word/document.xml，抽取 w:t 文本，按 w:p 换行。

use liteai_core::{ExtractedDocument, FileMeta, ParseError, Parser};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use std::path::Path;

pub struct DocxParser;

impl Parser for DocxParser {
    fn extensions(&self) -> &'static [&'static str] {
        &["docx"]
    }

    fn parse(&self, path: &Path) -> Result<ExtractedDocument, ParseError> {
        let meta = FileMeta::from_path(path)?;
        let file = std::fs::File::open(path)?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| ParseError::Other(format!("不是有效的 docx: {e}")))?;
        let mut doc = zip
            .by_name("word/document.xml")
            .map_err(|e| ParseError::Other(format!("读取 document.xml 失败: {e}")))?;
        let mut xml = String::new();
        doc.read_to_string(&mut xml)?;

        let text = extract_text(&xml);
        if text.trim().is_empty() {
            return Err(ParseError::Other("Word 文档中没有可提取的文本".into()));
        }
        Ok(ExtractedDocument::new(meta, text, "application/vnd.openxmlformats-officedocument.wordprocessingml.document"))
    }
}

fn extract_text(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut out = String::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(t)) => {
                if let Ok(s) = t.unescape() {
                    out.push_str(&s);
                }
            }
            Ok(Event::End(e)) if e.local_name().as_ref() == b"p" => out.push('\n'),
            Ok(Event::Empty(e)) if e.local_name().as_ref() == b"p" => out.push('\n'),
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_minimal_docx() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("sample.docx");
        let file = std::fs::File::create(&f).unwrap();
        let mut zw = zip::ZipWriter::new(file);
        // 只需 word/document.xml 即可驱动本解析器
        let xml = r#"<?xml version="1.0"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>第一段内容</w:t></w:r></w:p>
    <w:p><w:r><w:t>第二段：Hello</w:t></w:r></w:p>
  </w:body>
</w:document>"#;
        zw.start_file("word/document.xml", zip::write::SimpleFileOptions::default()).unwrap();
        zw.write_all(xml.as_bytes()).unwrap();
        zw.finish().unwrap();

        let doc = DocxParser.parse(&f).unwrap();
        assert!(doc.text.contains("第一段内容"));
        assert!(doc.text.contains("Hello"));
        assert_eq!(doc.text.lines().count(), 2);
    }
}

//! Markdown 序列化器：`<源名>.ai.md`。

use liteai_core::{OutputDocument, OutputError, Serializer};
use std::path::{Path, PathBuf};

pub struct MarkdownSerializer;

impl Serializer for MarkdownSerializer {
    fn extension(&self) -> &'static str {
        "md"
    }

    fn serialize(&self, doc: &OutputDocument, out_dir: &Path) -> Result<PathBuf, OutputError> {
        let stem = output_stem(&doc.source_name);
        let name = format!("{stem}.ai.md");
        let path = out_dir.join(&name);

        let mut md = String::new();
        md.push_str(&format!("# {} · AI 分析报告\n\n", doc.source_name));
        if doc.truncated {
            md.push_str("> 原文较长，已截断至前 60000 字符用于分析。\n\n");
        }
        md.push_str("## 原文内容\n\n```text\n");
        md.push_str(&doc.source_text);
        if !doc.source_text.ends_with('\n') {
            md.push('\n');
        }
        md.push_str("```\n\n---\n\n## AI 分析结果\n\n");
        md.push_str(&doc.analysis);
        if !doc.analysis.ends_with('\n') {
            md.push('\n');
        }

        std::fs::write(&path, md)?;
        Ok(path)
    }
}

pub(crate) fn output_stem(source_name: &str) -> String {
    Path::new(source_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("分析结果")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use liteai_core::OutputDocument;

    fn doc() -> OutputDocument {
        OutputDocument {
            source_name: "周报.xlsx".into(),
            source_text: "A1\tB1\n2\t3\n".into(),
            truncated: false,
            analysis: "# 总结\n- 要点一\n- 要点二\n".into(),
        }
    }

    #[test]
    fn writes_md_file() {
        let dir = tempfile::tempdir().unwrap();
        let ser = MarkdownSerializer;
        let path = ser.serialize(&doc(), dir.path()).unwrap();
        assert!(path.ends_with("周报.ai.md"));
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("AI 分析报告"));
        assert!(content.contains("A1\tB1"));
        assert!(content.contains("要点一"));
    }

    #[test]
    fn trims_extension_in_output_name() {
        let d = OutputDocument {
            source_name: "data.csv".into(),
            source_text: "a\n".into(),
            truncated: false,
            analysis: "ok".into(),
        };
        let dir = tempfile::tempdir().unwrap();
        let path = MarkdownSerializer.serialize(&d, dir.path()).unwrap();
        assert!(path.ends_with("data.ai.md"));
    }
}

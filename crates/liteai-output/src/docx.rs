//! Word(.docx) 序列化器：`<源名>.ai.docx`。
//!
//! 内置 Markdown → Word 迷你转换：标题(#/##/###)、列表(-)、代码块(```)、普通段落、加粗(**x**)。
//! 使用 docx-rs 0.4.22 的 API（Docx::build() -> XMLDocx::pack()）。

use crate::markdown::output_stem;
use docx_rs::{Docx, Paragraph, Run, RunFonts};
use liteai_core::{OutputDocument, OutputError, Serializer};
use std::path::{Path, PathBuf};

pub struct DocxSerializer;

impl Serializer for DocxSerializer {
    fn extension(&self) -> &'static str {
        "docx"
    }

    fn serialize(&self, doc: &OutputDocument, out_dir: &Path) -> Result<PathBuf, OutputError> {
        let stem = output_stem(&doc.source_name);
        let name = format!("{stem}.ai.docx");
        let path = out_dir.join(&name);

        let mut paragraphs: Vec<Paragraph> = Vec::new();
        // 标题 + 原文区
        paragraphs.push(heading(&doc.source_name, 1));
        if doc.truncated {
            paragraphs.push(
                Paragraph::new()
                    .add_run(Run::new().add_text("（原文较长，已截断至前 60000 字符）").italic().size(18)),
            );
        }
        paragraphs.push(heading("原文内容", 2));
        for line in doc.source_text.lines().take(5000) {
            paragraphs.push(Paragraph::new().add_run(code_run(line)));
        }
        paragraphs.push(Paragraph::new());
        paragraphs.push(heading("AI 分析结果", 2));
        // 分析结果 markdown 转换
        paragraphs.extend(markdown_paragraphs(&doc.analysis));

        let mut docx = Docx::new();
        for p in paragraphs {
            docx = docx.add_paragraph(p);
        }
        let file = std::fs::File::create(&path)?;
        docx.build().pack(file).map_err(|e| OutputError::Other(format!("生成 Word 失败: {e}")))?;
        Ok(path)
    }
}

fn heading(text: &str, level: u32) -> Paragraph {
    let size = match level {
        1 => 36,
        2 => 30,
        _ => 24,
    };
    Paragraph::new().add_run(Run::new().add_text(text).bold().size(size))
}

fn code_run(text: &str) -> Run {
    Run::new()
        .add_text(text)
        .fonts(RunFonts::new().ascii("Consolas").east_asia("Consolas"))
        .size(20)
}

/// 将 Markdown 文本转为 Word 段落集合。
fn markdown_paragraphs(md: &str) -> Vec<Paragraph> {
    let mut out = Vec::new();
    let mut in_code = false;
    for raw in md.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            out.push(Paragraph::new().add_run(code_run(line)));
            continue;
        }
        if trimmed.is_empty() {
            out.push(Paragraph::new());
            continue;
        }
        if let Some(h) = trimmed.strip_prefix("### ") {
            out.push(heading(h.trim(), 3));
        } else if let Some(h) = trimmed.strip_prefix("## ") {
            out.push(heading(h.trim(), 2));
        } else if let Some(h) = trimmed.strip_prefix("# ") {
            out.push(heading(h.trim(), 1));
        } else if let Some(b) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            out.push(
                Paragraph::new()
                    .add_run(Run::new().add_text(format!("• {}", strip_inline(b))))
                    .indent(Some(400), None, None, None),
            );
        } else {
            let text = strip_inline(trimmed);
            out.push(Paragraph::new().add_run(Run::new().add_text(text).size(22)));
        }
    }
    out
}

/// 去掉 `**加粗**`、`` `code` `` 等行内标记，保留可读文本。
fn strip_inline(s: &str) -> String {
    let mut out = s.to_string();
    for marker in ["**", "`", "*", "~~"] {
        out = out.replace(marker, "");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use liteai_core::OutputDocument;

    #[test]
    fn writes_docx_file() {
        let dir = tempfile::tempdir().unwrap();
        let doc = OutputDocument {
            source_name: "合同.docx".into(),
            source_text: "第一条 甲乙双方…\n".into(),
            truncated: false,
            analysis: "# 结论\n- 条款一\n- 条款二\n".into(),
        };
        let path = DocxSerializer.serialize(&doc, dir.path()).unwrap();
        assert!(path.ends_with("合同.ai.docx"));
        let bytes = std::fs::read(&path).unwrap();
        // docx 是 zip 格式，文件头应为 PK
        assert_eq!(&bytes[..2], b"PK");
        assert!(bytes.len() > 0);
    }
}

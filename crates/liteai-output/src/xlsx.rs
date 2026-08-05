//! Excel(.xlsx) 序列化器：`<源名>.ai.xlsx`。
//!
//! 用 rust_xlsxwriter 原生生成，不依赖 Python/Excel。
//! 结构：Sheet「分析结果」把 Markdown 逐行转成行（标题加粗、表格拆列、列表/正文换行），
//!       Sheet「原文」放源文件内容。

use crate::markdown::output_stem;
use liteai_core::{OutputDocument, OutputError, Serializer};
use rust_xlsxwriter::{Format, FormatAlign, Workbook};
use std::path::{Path, PathBuf};

pub struct XlsxSerializer;

impl Serializer for XlsxSerializer {
    fn extension(&self) -> &'static str {
        "xlsx"
    }

    fn serialize(&self, doc: &OutputDocument, out_dir: &Path) -> Result<PathBuf, OutputError> {
        let stem = output_stem(&doc.source_name);
        let name = format!("{stem}.ai.xlsx");
        let path = out_dir.join(&name);

        let title = Format::new().set_bold().set_font_size(14);
        let heading = Format::new().set_bold().set_font_size(11).set_background_color("D9E1F2");
        let header_cell = Format::new().set_bold().set_background_color("F0F1F4").set_align(FormatAlign::Center);
        let wrap = Format::new().set_text_wrap();
        let code = Format::new().set_text_wrap().set_font_name("Consolas");

        let mut workbook = Workbook::new();
        // Sheet 1：分析结果
        let ws = workbook.add_worksheet().set_name("分析结果").map_err(oe)?;
        ws.set_column_width(0, 110).map_err(oe)?;
        ws.set_column_width(1, 60).map_err(oe)?;
        ws.write_with_format(0, 0, format!("{} · AI 分析报告", doc.source_name), &title).map_err(oe)?;
        ws.set_row_height(0, 22).map_err(oe)?;

        write_markdown(ws, &doc.analysis, &heading, &header_cell, &wrap, &code)?;

        // Sheet 2：原文
        let ws2 = workbook.add_worksheet().set_name("原文").map_err(oe)?;
        ws2.set_column_width(0, 110).map_err(oe)?;
        for (i, line) in doc.source_text.lines().enumerate() {
            ws2.write_string_with_format(i as u32 + 1, 0, line, &wrap).map_err(oe)?;
        }

        workbook.save(&path).map_err(|e| OutputError::Other(format!("生成 Excel 失败: {e}")))?;
        Ok(path)
    }
}

fn oe<E: std::fmt::Display>(e: E) -> OutputError {
    OutputError::Other(e.to_string())
}

/// 把 Markdown 逐行写入工作表。
fn write_markdown(
    ws: &mut rust_xlsxwriter::Worksheet,
    md: &str,
    heading: &Format,
    header_cell: &Format,
    wrap: &Format,
    code: &Format,
) -> Result<(), OutputError> {
    let mut row: u32 = 2; // 标题占 0 行，空一行
    let mut table_header = false;
    let mut in_code = false;

    for raw in md.lines() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();

        if trimmed.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            ws.write_string_with_format(row, 0, line, code).map_err(oe)?;
            row += 1;
            continue;
        }

        // 表格行（| 开头）
        if trimmed.starts_with('|') {
            let cells: Vec<&str> = trimmed
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim())
                .collect();
            // 分隔行 |---|---| 跳过
            if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')) {
                continue;
            }
            if !table_header {
                for (col, cell) in cells.iter().copied().enumerate() {
                    ws.write_string_with_format(row, col as u16, cell, header_cell).map_err(oe)?;
                }
                table_header = true;
            } else {
                for (col, cell) in cells.iter().copied().enumerate() {
                    ws.write_string_with_format(row, col as u16, cell, wrap).map_err(oe)?;
                }
            }
            row += 1;
            continue;
        }
        table_header = false;

        // 标题
        if let Some(h) = trimmed.strip_prefix("### ") {
            ws.write_string_with_format(row, 0, h.trim(), heading).map_err(oe)?;
            row += 1;
            continue;
        }
        if let Some(h) = trimmed.strip_prefix("## ") {
            ws.write_string_with_format(row, 0, h.trim(), heading).map_err(oe)?;
            row += 1;
            continue;
        }
        if let Some(h) = trimmed.strip_prefix("# ") {
            ws.write_string_with_format(row, 0, h.trim(), heading).map_err(oe)?;
            row += 1;
            continue;
        }

        if trimmed.is_empty() {
            row += 1;
            continue;
        }

        // 列表与正文
        let text = if let Some(b) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            format!("• {b}")
        } else {
            trimmed.to_string()
        };
        ws.write_string_with_format(row, 0, text, wrap).map_err(oe)?;
        row += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use liteai_core::OutputDocument;

    #[test]
    fn writes_xlsx_file() {
        let dir = tempfile::tempdir().unwrap();
        let doc = OutputDocument {
            source_name: "周报.md".into(),
            source_text: "周报正文\n".into(),
            truncated: false,
            analysis: "# 总结\n\n| 指标 | 数值 |\n|---|---|\n| 用户数 | 12450 |\n| 新增 | 1030 |\n\n- 要点一\n- 要点二\n".into(),
        };
        let path = XlsxSerializer.serialize(&doc, dir.path()).unwrap();
        assert!(path.ends_with("周报.ai.xlsx"));
        let bytes = std::fs::read(&path).unwrap();
        // xlsx 是 zip 格式
        assert_eq!(&bytes[..2], b"PK");
        assert!(bytes.len() > 500);
    }
}

//! Excel 解析器（calamine）：遍历所有 sheet，逐行拼接为制表符分隔文本。

use calamine::{open_workbook_auto, Reader};
use liteai_core::{ExtractedDocument, FileMeta, ParseError, Parser};
use std::path::Path;

pub struct XlsxParser;

impl Parser for XlsxParser {
    fn extensions(&self) -> &'static [&'static str] {
        &["xlsx", "xlsm"]
    }

    fn parse(&self, path: &Path) -> Result<ExtractedDocument, ParseError> {
        let meta = FileMeta::from_path(path)?;
        let mut wb = open_workbook_auto(path).map_err(|e| ParseError::Other(format!("打开 Excel 失败: {e}")))?;

        let mut out = String::new();
        for (name, range) in wb.worksheets() {
            out.push_str(&format!("===== Sheet: {name} =====\n"));
            for row in range.rows().take(2000) {
                let cells: Vec<String> = row.iter().map(|c| cell_text(c)).collect();
                if cells.iter().any(|c| !c.is_empty()) {
                    out.push_str(&cells.join("\t"));
                    out.push('\n');
                }
            }
        }
        if out.trim().is_empty() {
            return Err(ParseError::Other("Excel 中没有可提取的内容".into()));
        }
        Ok(ExtractedDocument::new(meta, out, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"))
    }
}

/// 单元格文本：1.0/2.0 之类整数值去掉小数点尾巴。
fn cell_text(c: &calamine::Data) -> String {
    match c {
        calamine::Data::Float(v) => {
            let s = format!("{v}");
            if s.ends_with(".0") {
                s[..s.len() - 2].to_string()
            } else {
                s
            }
        }
        _ => c.to_string(),
    }
}

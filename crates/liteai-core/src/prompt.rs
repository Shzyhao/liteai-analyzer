//! Prompt 构建与模板变量渲染。

use crate::domain::*;
use std::collections::HashMap;

/// 用变量表渲染模板中的 `{name}` 占位符。
pub fn render_template(template: &str, vars: &HashMap<String, String>) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

/// 将模型请求组装为 ChatRequest。
#[derive(Debug, Clone)]
pub struct PromptBuilder {
    pub system: String,
    pub user_template: String,
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self {
            system: "你是一个专业的文件分析助手。请使用简洁、结构化的中文输出分析结果，优先用 Markdown 排版。".into(),
            user_template: "请分析以下文件。\n\n文件名：{filename}\n\n文件内容：\n{content}\n\n请按模板要求完成分析。".into(),
        }
    }
}

impl PromptBuilder {
    pub fn new(system: impl Into<String>, user_template: impl Into<String>) -> Self {
        Self {
            system: system.into(),
            user_template: user_template.into(),
        }
    }

    /// 根据文档构造请求（注入 `{filename}` / `{path}` / `{content}` 变量）。
    pub fn build_request(&self, base_url: &str, model: &str, doc: &ExtractedDocument) -> ChatRequest {
        let mut vars = HashMap::new();
        vars.insert("filename".to_string(), doc.meta.file_name.clone());
        vars.insert("path".to_string(), doc.meta.path.display().to_string());
        vars.insert("content".to_string(), doc.text.clone());

        let user = render_template(&self.user_template, &vars);
        ChatRequest {
            base_url: base_url.to_string(),
            model: model.to_string(),
            messages: vec![
                ChatMessage { role: "system".into(), content: self.system.clone() },
                ChatMessage { role: "user".into(), content: user },
            ],
            temperature: 0.7,
        }
    }
}

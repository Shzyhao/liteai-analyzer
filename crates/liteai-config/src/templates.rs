//! Prompt 模板系统：内置 5 个常用模板 + 自定义模板 + JSON 导入导出。

use liteai_core::render_template;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    /// system 指令（可空）
    pub system: String,
    /// 用户模板，支持 {filename} / {path} / {content} 变量
    pub prompt: String,
    pub builtin: bool,
}

impl Template {
    /// 渲染完整用户消息。
    pub fn render_user(&self, filename: &str, path: &str, content: &str) -> String {
        let mut vars = HashMap::new();
        vars.insert("filename".to_string(), filename.to_string());
        vars.insert("path".to_string(), path.to_string());
        vars.insert("content".to_string(), content.to_string());
        render_template(&self.prompt, &vars)
    }
}

/// 内置 5 个模板。
pub fn builtin_templates() -> Vec<Template> {
    let mut v = Vec::new();
    v.push(Template {
        id: "summary".into(),
        name: "内容摘要".into(),
        system: "你是一个专业的文件分析助手。".into(),
        prompt: "请对以下文件内容进行结构化摘要，输出：1）核心主题；2）要点列表；3）关键数据/结论。使用 Markdown 排版。\n\n文件名：{filename}\n\n文件内容：\n{content}".into(),
        builtin: true,
    });
    v.push(Template {
        id: "translate".into(),
        name: "翻译".into(),
        system: "你是一个专业翻译。".into(),
        prompt: "请将以下文件内容翻译成中文，保留原文格式。若原文已是中文则翻译成英文。\n\n文件名：{filename}\n\n文件内容：\n{content}".into(),
        builtin: true,
    });
    v.push(Template {
        id: "code_explain".into(),
        name: "代码解释".into(),
        system: "你是资深软件工程师。".into(),
        prompt: "请解释以下代码：1）功能概述；2）关键函数/逻辑说明；3）潜在问题与改进建议。\n\n文件名：{filename}\n\n代码内容：\n{content}".into(),
        builtin: true,
    });
    v.push(Template {
        id: "data_check".into(),
        name: "数据纠错".into(),
        system: "你是数据分析专家。".into(),
        prompt: "请检查以下数据内容：1）异常值、缺失值、格式问题；2）统计特征；3）纠错建议。\n\n文件名：{filename}\n\n数据内容：\n{content}".into(),
        builtin: true,
    });
    v.push(Template {
        id: "sentiment".into(),
        name: "情感分析".into(),
        system: "你是文本分析专家。".into(),
        prompt: "请对以下内容进行情感分析：1）整体情感倾向（积极/中性/消极）；2）情绪强度；3）主要观点。\n\n文件名：{filename}\n\n文本内容：\n{content}".into(),
        builtin: true,
    });
    v
}

/// 用户自定义模板持久化文件。
pub fn templates_file(base: &std::path::Path) -> std::path::PathBuf {
    base.join("templates.json")
}

pub fn load_custom_templates(base: &std::path::Path) -> Vec<Template> {
    let path = templates_file(base);
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_custom_templates(base: &std::path::Path, templates: &[Template]) -> Result<(), String> {
    let custom: Vec<&Template> = templates.iter().filter(|t| !t.builtin).collect();
    std::fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let raw = serde_json::to_string_pretty(&custom).map_err(|e| e.to_string())?;
    std::fs::write(templates_file(base), raw).map_err(|e| e.to_string())
}

/// 全量模板（内置 + 自定义）。
pub fn all_templates(base: &std::path::Path) -> Vec<Template> {
    let mut all = builtin_templates();
    all.extend(load_custom_templates(base));
    all
}

/// 导入 JSON（单个或数组）。返回导入的模板列表（去掉 builtin 标记）。
pub fn import_templates(json: &str) -> Result<Vec<Template>, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("JSON 格式错误: {e}"))?;
    let templates: Vec<Template> = if v.is_array() {
        serde_json::from_value(v).map_err(|e| e.to_string())?
    } else {
        let t: Template = serde_json::from_value(v).map_err(|e| e.to_string())?;
        vec![t]
    };
    let mut out = templates;
    for t in out.iter_mut() {
        t.builtin = false;
    }
    Ok(out)
}

pub fn export_templates(templates: &[Template]) -> String {
    serde_json::to_string_pretty(templates).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_render_variables() {
        let ts = builtin_templates();
        assert_eq!(ts.len(), 5);
        let t = &ts[0];
        let msg = t.render_user("报告.txt", "C:/a.txt", "正文内容");
        assert!(msg.contains("报告.txt"));
        assert!(msg.contains("正文内容"));
    }

    #[test]
    fn import_export_roundtrip() {
        let json = r#"[{"id":"c1","name":"合同审查","system":"","prompt":"请审查合同 {filename}","builtin":false}]"#;
        let imported = import_templates(json).unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].name, "合同审查");
        let exported = export_templates(&imported);
        assert!(exported.contains("合同审查"));
    }
}

//! 导出技能：用户自定义的外部脚本，接收分析结果并产出自定义文件。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 一个导出技能 = 一条外部脚本命令。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSkill {
    pub id: String,
    /// 显示名称，如「生成 HTML 报告」
    pub name: String,
    /// 可执行程序或解释器：python / node / powershell / 绝对路径的 .exe .bat .cmd .ps1
    pub command: String,
    /// 追加参数（支持引号），如 `-File "C:\scripts\report.py"`
    pub args: String,
    /// 可选工作目录
    pub cwd: Option<String>,
}

impl ExportSkill {
    /// 新建技能（生成稳定 id）。
    pub fn new(name: impl Into<String>, command: impl Into<String>, args: impl Into<String>) -> Self {
        let name: String = name.into();
        let id = format!("skill_{:x}", id_seed(&name));
        Self {
            id,
            name,
            command: command.into(),
            args: args.into(),
            cwd: None,
        }
    }
}

pub(crate) fn id_seed(name: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    name.hash(&mut h);
    h.finish()
}

pub fn skills_file(base: &Path) -> PathBuf {
    base.join("skills.json")
}

pub fn load_skills(base: &Path) -> Vec<ExportSkill> {
    let path = skills_file(base);
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_skills(base: &Path, skills: &[ExportSkill]) -> Result<(), String> {
    std::fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let raw = serde_json::to_string_pretty(skills).map_err(|e| e.to_string())?;
    std::fs::write(skills_file(base), raw).map_err(|e| format!("写入技能配置失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skills_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let skills = vec![ExportSkill::new("HTML 报告", "python", r#"-File "C:\scripts\report.py""#)];
        save_skills(dir.path(), &skills).unwrap();
        let loaded = load_skills(dir.path());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "HTML 报告");
        assert_eq!(loaded[0].args, r#"-File "C:\scripts\report.py""#);
        // id 稳定
        assert_eq!(loaded[0].id, skills[0].id);
    }
}

//! 分析历史记录：轻量 JSON 持久化，上限 50 条。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const HISTORY_LIMIT: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp_ms: u64,
    pub source_file: String,
    pub template: String,
    pub analysis: String,
    pub output_files: Vec<String>,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
}

pub fn history_file(base: &Path) -> PathBuf {
    base.join("history.json")
}

pub fn load_history(base: &Path) -> Vec<HistoryEntry> {
    let path = history_file(base);
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

/// 追加一条历史（新的在前，超上限丢弃最旧），并持久化。
pub fn append_history(base: &Path, entry: HistoryEntry) -> Result<(), String> {
    let mut all = load_history(base);
    all.insert(0, entry);
    all.truncate(HISTORY_LIMIT);
    save_history(base, &all)
}

pub fn save_history(base: &Path, entries: &[HistoryEntry]) -> Result<(), String> {
    std::fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let raw = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    std::fs::write(history_file(base), raw).map_err(|e| format!("写入历史失败: {e}"))
}

pub fn delete_history_entry(base: &Path, id: &str) -> Result<(), String> {
    let all = load_history(base);
    let filtered: Vec<HistoryEntry> = all.into_iter().filter(|e| e.id != id).collect();
    save_history(base, &filtered)
}

pub fn clear_history(base: &Path) -> Result<(), String> {
    std::fs::create_dir_all(base).map_err(|e| e.to_string())?;
    std::fs::write(history_file(base), "[]").map_err(|e| format!("清空历史失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(ts: u64) -> HistoryEntry {
        HistoryEntry {
            id: format!("h{ts}"),
            timestamp_ms: ts,
            source_file: format!("file{ts}.txt"),
            template: "summary".into(),
            analysis: "结果内容".into(),
            output_files: vec![],
            prompt_tokens: 10,
            completion_tokens: 5,
        }
    }

    #[test]
    fn history_caps_at_limit() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..60 {
            append_history(dir.path(), entry(i)).unwrap();
        }
        let all = load_history(dir.path());
        assert_eq!(all.len(), HISTORY_LIMIT);
        // 最新的在最前
        assert_eq!(all[0].id, "h59");
    }

    #[test]
    fn delete_and_clear() {
        let dir = tempfile::tempdir().unwrap();
        append_history(dir.path(), entry(1)).unwrap();
        append_history(dir.path(), entry(2)).unwrap();
        delete_history_entry(dir.path(), "h1").unwrap();
        assert_eq!(load_history(dir.path()).len(), 1);
        clear_history(dir.path()).unwrap();
        assert!(load_history(dir.path()).is_empty());
    }
}

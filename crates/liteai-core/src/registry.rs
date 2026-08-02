//! 解析器注册表：按扩展名分发到对应解析器。
//!
//! 使用 `Arc<dyn Parser>` 使同一解析器可注册多个扩展名。

use crate::traits::Parser;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Default)]
pub struct ParserRegistry {
    by_ext: HashMap<String, Arc<dyn Parser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个解析器（按它的 `extensions()` 全部分发）。
    pub fn register(&mut self, parser: Arc<dyn Parser>) {
        for ext in parser.extensions() {
            self.by_ext.insert(ext.to_string(), parser.clone());
        }
    }

    /// 根据文件扩展名取解析器。
    pub fn get(&self, path: &Path) -> Option<&dyn Parser> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        self.by_ext.get(&ext).map(|arc| arc.as_ref())
    }
}

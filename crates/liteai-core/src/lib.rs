//! liteai-core：AI 文件分析助手核心库。
//!
//! 零依赖 `tauri`，保证可直接 `cargo test` 验证全部核心逻辑。
//! GUI（src-tauri）与 CLI（liteai-cli）共用这里的领域模型、trait 与编排管线。

pub mod domain;
pub mod pipeline;
pub mod prompt;
pub mod registry;
pub mod traits;

pub use domain::*;
pub use pipeline::{AnalysisPipeline, ModelConfig, OutputConfig};
pub use prompt::{render_template, PromptBuilder};
pub use registry::ParserRegistry;
pub use traits::*;

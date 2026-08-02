//! liteai-model：OpenAI 兼容模型客户端。

pub mod openai;
pub mod sse;

pub use openai::OpenAiClient;

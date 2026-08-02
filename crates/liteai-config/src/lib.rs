//! liteai-config：配置存储 / 密钥安全存储 / 模板系统。

pub mod secret;
pub mod store;
pub mod templates;

pub use secret::{default_secret_store, FileStore, KeyringStore};
pub use store::{default_config_dir, load_config, save_config, AppConfig, ApiConfig, Prefs, SERVICE_NAME};
pub use templates::{
    all_templates, builtin_templates, export_templates, import_templates, load_custom_templates,
    save_custom_templates, Template,
};

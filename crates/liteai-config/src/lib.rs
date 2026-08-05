//! liteai-config：配置存储 / 密钥安全存储 / 模板系统 / 导出技能。

pub mod history;
pub mod secret;
pub mod skills;
pub mod store;
pub mod templates;

pub use secret::{default_secret_store, FileStore, KeyringStore};
pub use store::{
    active_profile, default_config_dir, default_skills_dir, load_config, resolve_skills_dir,
    save_config, AppConfig, ApiProfile, Prefs, SERVICE_NAME,
};
pub use history::{
    append_history, clear_history, delete_history_entry, load_history, HistoryEntry,
};
pub use skills::{load_skills, save_skills, ExportSkill};
pub use templates::{
    all_templates, builtin_templates, export_templates, import_templates, load_custom_templates,
    save_custom_templates, Template,
};

//! CC Switch 核心业务逻辑层
//!
//! 本 crate 包含所有不依赖 Tauri 的业务逻辑：
//! - database: SQLite DAO
//! - services: 业务服务（ProviderService / ProxyService / McpService 等）
//! - proxy: 本地代理服务器
//! - core: bootstrap / provider_manager / decl_config
//! - error / provider / app_config / config / settings: 基础类型
//!
//! 当 `feature = "tauri"` 未启用时（CLI 模式），Tauri 类型退化为空桩。

#![allow(clippy::module_inception)]

// 跨 feature 的 Tauri 类型别名
#[cfg(feature = "tauri")]
pub use tauri::AppHandle as TauriAppHandle;
#[cfg(not(feature = "tauri"))]
/// CLI 模式：空 AppHandle 桩
#[derive(Clone, Debug, Default)]
pub struct TauriAppHandle;

pub mod app_config;
pub mod claude_desktop_config;
pub mod claude_mcp;
pub mod claude_plugin;
pub mod codex_config;
pub mod codex_history_migration;
pub mod codex_oauth;
pub mod config;
pub mod copilot_auth;
pub mod core;
pub mod database;
pub mod deeplink;
pub mod error;
pub mod event_callback;
pub mod gemini_config;
pub mod gemini_mcp;
pub mod hermes_config;
pub mod init_status;
pub mod mcp;
pub mod openclaw_config;
pub mod opencode_config;
pub mod prompt;
pub mod prompt_files;
pub mod provider;
pub mod provider_defaults;
pub mod proxy;
pub mod services;
pub mod session_manager;
pub mod settings;
pub mod store;
pub mod usage_events;
pub mod usage_script;

// 常用类型 re-export
pub use app_config::{AppType, InstalledSkill, McpApps, McpServer, MultiAppConfig, SkillApps};
pub use database::Database;
pub use error::AppError;
pub use provider::{Provider, ProviderMeta};
pub use settings::{get_settings, reload_settings, update_settings, AppSettings};
pub use store::AppState;

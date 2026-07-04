//! CC Switch 核心业务逻辑层
//!
//! 本 crate 包含所有不依赖 Tauri 的业务逻辑：
//! - database: SQLite DAO
//! - services: 业务服务（ProviderService / ProxyService / McpService 等）
//! - proxy: 本地代理服务器
//! - core: bootstrap / provider_manager / decl_config
//! - error / provider / app_config / config / settings: 基础类型
//!
//! 严禁依赖 tauri / tauri-plugin-* / webkit2gtk。
//! Tauri 命令包装层在 cc-switch-tauri-commands crate 中。

#![allow(clippy::module_inception)]

// 模块在后续逐步迁移到此 crate。

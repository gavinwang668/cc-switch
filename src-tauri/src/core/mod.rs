//! 核心逻辑模块
//!
//! 提取不依赖 Tauri 的核心业务逻辑，供 CLI 和 GUI 共享复用。

pub mod database;
pub mod provider_manager;
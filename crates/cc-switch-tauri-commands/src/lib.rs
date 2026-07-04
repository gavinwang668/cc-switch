//! CC Switch Tauri 命令包装层
//!
//! 本 crate 是 `#[tauri::command]` 函数的包装层，负责：
//! - 从 Tauri managed state 取出 service 实例
//! - 调用 cc-switch-core 的业务逻辑
//! - 将结果返回给前端
//!
//! 所有业务逻辑在 cc-switch-core 中，本 crate 只做 Tauri 适配。

pub mod commands;

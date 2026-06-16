//! 数据库初始化封装
//!
//! 提供 CLI 和 GUI 共享的数据库初始化逻辑。

use crate::Database;

/// 初始化数据库连接
pub fn init_database() -> Result<Database, String> {
    Database::init().map_err(|e| format!("数据库初始化失败: {e}"))
}
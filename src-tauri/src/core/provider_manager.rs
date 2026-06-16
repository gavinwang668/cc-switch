//! 供应商管理逻辑
//!
//! 封装供应商的增删改查等操作，供 CLI 和 GUI 复用。

use indexmap::IndexMap;

use crate::provider::Provider;
use crate::Database;

/// 获取指定应用类型的所有供应商
pub fn get_all_providers(
    db: &Database,
    app_type: &str,
) -> Result<IndexMap<String, Provider>, String> {
    db.get_all_providers(app_type)
        .map_err(|e| format!("获取供应商列表失败: {e}"))
}

/// 获取当前选中的供应商 ID
pub fn get_current_provider_id(db: &Database, app_type: &str) -> Result<Option<String>, String> {
    db.get_current_provider(app_type)
        .map_err(|e| format!("获取当前供应商失败: {e}"))
}

/// 切换当前供应商
pub fn switch_provider(
    db: &Database,
    app_type: &str,
    provider_id: &str,
) -> Result<(), String> {
    db.set_current_provider(app_type, provider_id)
        .map_err(|e| format!("切换供应商失败: {e}"))
}

/// 删除指定供应商
pub fn remove_provider(db: &Database, app_type: &str, provider_id: &str) -> Result<(), String> {
    db.delete_provider(app_type, provider_id)
        .map_err(|e| format!("删除供应商失败: {e}"))
}

/// 获取供应商详细信息
pub fn get_provider_by_id(
    db: &Database,
    app_type: &str,
    provider_id: &str,
) -> Result<Option<Provider>, String> {
    db.get_provider_by_id(app_type, provider_id)
        .map_err(|e| format!("获取供应商信息失败: {e}"))
}
use crate::database::Database;
use crate::services::{ProxyService, UsageCache};
use std::sync::Arc;

/// 全局应用状态
pub struct AppState {
    pub db: Arc<Database>,
    pub proxy_service: ProxyService,
    pub usage_cache: Arc<UsageCache>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(db: Arc<Database>) -> Self {
        let proxy_service = ProxyService::new(db.clone());

        Self {
            db,
            proxy_service,
            usage_cache: Arc::new(UsageCache::new()),
        }
    }
}

// ============================================================================
// API Key 安全管理
// ============================================================================

#[cfg(not(target_os = "linux"))]
const KEYCHAIN_SERVICE: &str = "cc-switch";

/// 将 API Key 安全存储到系统 Keychain
///
/// 使用 provider_id + app_type 作为唯一标识，通过操作系统级凭证管理器存储。
/// - Windows: Windows Credential Manager
/// - macOS: Keychain
/// - Linux: 不可用（CLI 模式无需 Keychain）
#[cfg(not(target_os = "linux"))]
pub fn set_api_key(provider_id: &str, app_type: &str, api_key: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &keychain_account(provider_id, app_type))
        .map_err(|e| format!("Failed to create keychain entry: {}", e))?;
    entry
        .set_password(api_key)
        .map_err(|e| format!("Failed to store API key: {}", e))
}

/// 从系统 Keychain 读取 API Key
#[cfg(not(target_os = "linux"))]
pub fn get_api_key(provider_id: &str, app_type: &str) -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &keychain_account(provider_id, app_type))
        .map_err(|e| format!("Failed to create keychain entry: {}", e))?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to read API key: {}", e)),
    }
}

/// 从系统 Keychain 删除 API Key
#[cfg(not(target_os = "linux"))]
pub fn delete_api_key(provider_id: &str, app_type: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(KEYCHAIN_SERVICE, &keychain_account(provider_id, app_type))
        .map_err(|e| format!("Failed to create keychain entry: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete API key: {}", e))
}

#[cfg(not(target_os = "linux"))]
fn keychain_account(provider_id: &str, app_type: &str) -> String {
    format!("{}:{}", app_type, provider_id)
}

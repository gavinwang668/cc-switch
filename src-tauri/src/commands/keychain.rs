use crate::error::AppError;
use crate::store::{self, AppState};
use tauri::State;

/// 将 API Key 安全存储到系统 Keychain
#[tauri::command]
pub fn set_api_key(provider_id: String, app_type: String, api_key: String) -> Result<(), String> {
    store::set_api_key(&provider_id, &app_type, &api_key)
}

/// 从系统 Keychain 读取 API Key
#[tauri::command]
pub fn get_api_key(provider_id: String, app_type: String) -> Result<Option<String>, String> {
    store::get_api_key(&provider_id, &app_type)
}

/// 从系统 Keychain 删除 API Key
#[tauri::command]
pub fn delete_api_key(provider_id: String, app_type: String) -> Result<(), String> {
    store::delete_api_key(&provider_id, &app_type)
}

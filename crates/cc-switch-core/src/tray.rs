//! 托盘菜单桩模块
//!
//! 完整实现在 `src-tauri/src/tray.rs`（依赖 tauri::menu / tauri::Emitter 等）。
//! core crate 中仅提供最小桩，使 `proxy::failover_switch` 在 GUI 模式下能编译通过；
//! 实际菜单创建由 src-tauri 中的实现负责。
//!
//! 此桩在 CLI 模式（无 `tauri` feature）下不存在。

#[cfg(feature = "tauri")]
pub const TRAY_ID: &str = "main";

#[cfg(feature = "tauri")]
pub async fn create_tray_menu<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
    _state: &crate::store::AppState,
) -> Result<tauri::menu::Menu<R>, String> {
    Err("tray menu not available in core".to_string())
}

/// 桩函数：在 src-tauri 中有真实实现（合并+节流的托盘刷新）。
///
/// cc-switch-tauri-commands 中的命令调用此函数以触发托盘刷新；在 core crate 中
/// 仅做日志记录，真正的刷新由 src-tauri 中的实现接管（命令最终通过
/// `tauri::AppHandle` 触发事件，src-tauri 监听并刷新）。
#[cfg(feature = "tauri")]
pub fn schedule_tray_refresh(_app: &tauri::AppHandle) {
    // 桩实现：实际刷新由 src-tauri 中托盘刷新逻辑负责。
    // 这里不做任何操作，避免 core crate 依赖菜单/Emitter 实现。
}

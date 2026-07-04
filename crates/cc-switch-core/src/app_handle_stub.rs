//! Tauri 兼容类型别名（cc-switch-core 去 Tauri 化）
//!
//! 当某些模块必须接受 AppHandle 参数但本身不需要 Tauri 功能时，
//! 使用此模块的空类型替代。实际 AppHandle 由 tauri-commands 层注入。

/// 空 AppHandle 桩类型。
/// 当 CLI 调用这些函数时传 None；GUI 通过 tauri-commands 传真实 AppHandle。
#[derive(Clone, Debug)]
pub struct AppHandleStub;

impl AppHandleStub {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AppHandleStub {
    fn default() -> Self {
        Self::new()
    }
}

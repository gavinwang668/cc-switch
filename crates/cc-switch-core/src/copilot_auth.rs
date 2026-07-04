//! Copilot 认证状态类型别名
//!
//! 原始类型在 `cc_switch_core::proxy::providers::copilot_auth::CopilotAuthManager`。
//! 此处提供类型别名，供 service 层和 commands 层共用。
//!
//! Tauri 命令使用 `State<'_, CopilotAuthState>` 注入；
//! Service 层直接使用 `&CopilotAuthState`。

use crate::proxy::providers::copilot_auth::CopilotAuthManager;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Copilot 认证管理器的共享状态
///
/// 满足 `Send + Sync + 'static`，可用于 Tauri managed state。
pub type CopilotAuthState = Arc<RwLock<CopilotAuthManager>>;

/// 创建新的 CopilotAuthState 实例
pub fn new_copilot_auth_state(manager: CopilotAuthManager) -> CopilotAuthState {
    Arc::new(RwLock::new(manager))
}

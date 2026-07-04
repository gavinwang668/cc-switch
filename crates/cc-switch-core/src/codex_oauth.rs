//! Codex OAuth 认证状态类型别名
//!
//! 原始类型在 `cc_switch_core::proxy::providers::codex_oauth_auth::CodexOAuthManager`。
//! 此处提供类型别名，供 service 层和 commands 层共用。

use crate::proxy::providers::codex_oauth_auth::CodexOAuthManager;
use std::sync::{Arc, RwLock};

/// Codex OAuth 认证管理器的共享状态
///
/// 满足 `Send + Sync + 'static`，可用于 Tauri managed state。
pub type CodexOAuthState = Arc<RwLock<CodexOAuthManager>>;

/// 创建新的 CodexOAuthState 实例
pub fn new_codex_oauth_state(manager: CodexOAuthManager) -> CodexOAuthState {
    Arc::new(RwLock::new(manager))
}

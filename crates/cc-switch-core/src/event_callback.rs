//! 事件回调 trait，用于解耦 service 层与 Tauri 运行时。
//!
//! GUI 实现走 Tauri emit；CLI 实现为空 callback（不向前端推送）。
//! service 层只依赖此 trait，不直接依赖 tauri::AppHandle。

use serde::Serialize;

/// 事件回调接口。
///
/// 实现者负责把事件转发到前端（GUI）或丢弃（CLI）。
pub trait EventCallback: Send + Sync {
    /// 发射事件到前端，payload 必须可序列化。
    fn emit<T: Serialize + Clone>(&self, event: &str, payload: T);
}

/// 空实现，CLI / 无头模式使用。
pub struct NoopEventCallback;

impl EventCallback for NoopEventCallback {
    fn emit<T: Serialize + Clone>(&self, _event: &str, _payload: T) {
        // CLI 不向前端推送事件
    }
}

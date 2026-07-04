//! 使用统计实时刷新事件模块
//!
//! 当 `proxy_request_logs` 表写入新数据时（代理日志、会话同步、归档等），
//! 通过本模块向前端 emit `usage-log-recorded` 事件，让 UsageDashboard
//! 立刻 invalidate 查询缓存而无需等待轮询周期。
//!
//! 设计要点：
//! - 全局单例 AppHandle：写日志路径上不持有 AppHandle，用 OnceCell 共享。
//! - 200ms 防抖合并：流式响应等场景在短时间内可能写入多条日志，
//!   合并成一次事件可避免前端连续 invalidate。
//! - 不阻塞写入：通知失败仅记录 warn 日志，不向上传播错误。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

#[cfg(feature = "tauri")]
use tauri::{AppHandle, Emitter};

/// 前端监听的事件名
pub const EVENT_USAGE_LOG_RECORDED: &str = "usage-log-recorded";

/// 防抖窗口：合并 200ms 内的多次通知。
const DEBOUNCE_WINDOW: Duration = Duration::from_millis(200);

#[cfg(feature = "tauri")]
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// 防抖标记：true 表示已有调度任务在等待 emit，后续通知合并到该任务。
static EMIT_SCHEDULED: AtomicBool = AtomicBool::new(false);

/// 在应用 setup 阶段调用一次，注入 AppHandle（仅 GUI 模式）。
///
/// CLI 模式下此函数为 no-op。
#[cfg(feature = "tauri")]
pub fn init(handle: AppHandle) {
    if APP_HANDLE.set(handle).is_err() {
        log::debug!("usage_events::init 重复调用，已忽略");
    } else {
        log::info!("[usage-event] AppHandle 已注入，事件推送启用");
    }
}

#[cfg(not(feature = "tauri"))]
pub fn init(_handle: crate::TauriAppHandle) {
    log::info!("[usage-event] CLI 模式，事件推送已禁用");
}

/// 通知前端有新的使用日志写入。
///
/// CLI 模式下为 no-op。
pub fn notify_log_recorded() {
    #[cfg(feature = "tauri")]
    {
        let Some(handle) = APP_HANDLE.get() else {
            return;
        };
        if EMIT_SCHEDULED.swap(true, Ordering::AcqRel) {
            return;
        }
        let handle = handle.clone();
        std::thread::spawn(move || {
            std::thread::sleep(DEBOUNCE_WINDOW);
            EMIT_SCHEDULED.store(false, Ordering::Release);
            if let Err(e) = handle.emit(EVENT_USAGE_LOG_RECORDED, ()) {
                log::warn!("emit {EVENT_USAGE_LOG_RECORDED} 失败: {e}");
            }
        });
    }
    #[cfg(not(feature = "tauri"))]
    {
        // CLI 模式：无前端，不推送事件
        let _ = EMIT_SCHEDULED.load(Ordering::Relaxed);
    }
}

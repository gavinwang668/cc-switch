//! CC Switch Tauri 命令包装层
//!
//! 本 crate 是 `#[tauri::command]` 函数的包装层，负责：
//! - 从 Tauri managed state 取出 service 实例
//! - 调用 cc-switch-core 的业务逻辑
//! - 将结果返回给前端
//!
//! 所有业务逻辑在 cc-switch-core 中，本 crate 只做 Tauri 适配。
//!
//! 注意：部分 GUI 专属功能（轻量模式、自启动、窗口状态保存、托盘、退出清理）
//! 的完整实现在 `src-tauri/src/` 中。本 crate 仅提供桩实现，使命令包装层
//! 在不依赖 src-tauri 的情况下也能编译；运行时由 src-tauri 中的真实实现接管。

pub mod commands;

// ── GUI 专属功能桩实现 ─────────────────────────────────────
//
// 这些函数/模块在 `src-tauri/src/` 中有真实实现。cc-switch-tauri-commands
// 中的命令在调用它们时，实际由 src-tauri 中的实现接管（src-tauri 既链接
// 本 crate，也链接自己的实现；本 crate 的桩仅用于独立编译时的占位）。
//
// 桩实现策略：
// - 能用 Tauri 公共 API 直接实现的功能（如保存窗口状态、重启进程）→ 真实实现
// - 涉及 src-tauri 私有状态/插件的功能（如代理清理、托盘图标移除、
//   single-instance 锁释放）→ no-op，由 src-tauri 在 ExitRequested 路径中
//   自行处理

/// 轻量模式桩模块
///
/// 完整实现在 `src-tauri/src/lightweight.rs`，依赖窗口隐藏/显示等 Tauri API。
/// 此处仅提供 no-op 桩，运行时由 src-tauri 中的实现接管。
pub mod lightweight {
    /// 进入轻量模式（隐藏窗口等）。
    pub fn enter_lightweight_mode(_app: &tauri::AppHandle) -> Result<(), String> {
        // 桩实现：真实逻辑在 src-tauri/src/lightweight.rs
        Ok(())
    }

    /// 退出轻量模式（恢复窗口等）。
    pub fn exit_lightweight_mode(_app: &tauri::AppHandle) -> Result<(), String> {
        // 桩实现：真实逻辑在 src-tauri/src/lightweight.rs
        Ok(())
    }

    /// 当前是否处于轻量模式。
    pub fn is_lightweight_mode() -> bool {
        // 桩实现：始终返回 false，真实状态由 src-tauri 维护
        false
    }
}

/// 自启动桩模块
///
/// 完整实现在 `src-tauri/src/auto_launch.rs`，依赖 `auto-launch` crate。
pub mod auto_launch {
    /// 启用开机自启。
    pub fn enable_auto_launch() -> Result<(), String> {
        // 桩实现：真实逻辑在 src-tauri/src/auto_launch.rs
        Ok(())
    }

    /// 禁用开机自启。
    pub fn disable_auto_launch() -> Result<(), String> {
        // 桩实现：真实逻辑在 src-tauri/src/auto_launch.rs
        Ok(())
    }

    /// 查询开机自启是否已启用。
    pub fn is_auto_launch_enabled() -> Result<bool, String> {
        // 桩实现：始终返回 false，真实状态由 src-tauri 查询
        Ok(false)
    }
}

/// 托盘桩模块
///
/// 完整实现在 `src-tauri/src/tray.rs`，依赖 tauri::menu / Emitter 等。
pub mod tray {
    /// 触发托盘刷新。
    ///
    /// 桩实现：真实逻辑在 src-tauri/src/tray.rs::schedule_tray_refresh，
    /// 包含合并与节流逻辑。
    pub fn schedule_tray_refresh(_app: &tauri::AppHandle) {
        // 桩实现：no-op，真实刷新由 src-tauri 中的实现接管
    }
}

/// app_config_dir Store 桩模块
///
/// 完整实现在 `src-tauri/src/app_store.rs`，依赖 tauri-plugin-store。
/// cc-switch-core 中也有同名桩模块；此处的桩使 tauri-commands 内部调用
/// `crate::app_store::xxx` 时能解析到符号。
pub mod app_store {
    use std::path::PathBuf;

    /// 刷新 app_config_dir 覆盖值（从 Store 读取并更新缓存）。
    ///
    /// 桩实现返回 None；真实逻辑在 src-tauri/src/app_store.rs。
    pub fn refresh_app_config_dir_override(_app: &tauri::AppHandle) -> Option<PathBuf> {
        // 桩实现：始终返回 None，真实状态由 src-tauri 维护
        None
    }

    /// 将 app_config_dir 写入 Tauri Store。
    ///
    /// 桩实现直接返回 Ok；真实逻辑在 src-tauri/src/app_store.rs。
    pub fn set_app_config_dir_to_store(
        _app: &tauri::AppHandle,
        _path: Option<&str>,
    ) -> Result<(), cc_switch_core::error::AppError> {
        // 桩实现：no-op
        Ok(())
    }
}

// ── 退出路径清理函数桩 ─────────────────────────────────────
//
// 这些函数在 src-tauri/src/lib.rs 中有真实实现，依赖 src-tauri 私有状态
// （代理 service、托盘图标、single-instance 锁等）。cc-switch-tauri-commands
// 中的命令在退出/重启路径上调用它们，运行时由 src-tauri 中的实现接管；
// 此处的桩仅用于独立编译时的占位。
//
// 注意：src-tauri 的 lib.rs 中同名函数会遮蔽此处桩实现——src-tauri 内部
// 调用 `crate::xxx` 解析到自己的实现，而 tauri-commands 内部调用
// `crate::xxx` 解析到这里的桩。

/// 在退出前保存窗口状态。
///
/// 桩实现：no-op。真实逻辑在 src-tauri/src/lib.rs::save_window_state_before_exit，
/// 调用 `tauri-plugin-window-state` 的 `save_window_state` API。
pub fn save_window_state_before_exit(_app: &tauri::AppHandle) {
    // 桩实现：no-op
}

/// 应用退出前的清理（停止代理、恢复 Live 配置等）。
///
/// 桩实现：no-op。真实逻辑在 src-tauri/src/lib.rs::cleanup_before_exit，
/// 依赖 src-tauri 私有的 proxy_service 状态。
pub async fn cleanup_before_exit(_app: &tauri::AppHandle) {
    // 桩实现：no-op
}

/// 从系统托盘移除托盘图标。
///
/// 桩实现：no-op。真实逻辑在 src-tauri/src/lib.rs::remove_tray_icon_before_exit，
/// 通过 `tray.set_visible(false)` 触发 NIM_DELETE。
pub fn remove_tray_icon_before_exit(_app: &tauri::AppHandle) {
    // 桩实现：no-op
}

/// 主动释放 single-instance 锁。
///
/// 桩实现：no-op。真实逻辑在 src-tauri/src/lib.rs::destroy_single_instance_lock，
/// 调用 `tauri_plugin_single_instance::destroy`。
pub fn destroy_single_instance_lock(_app: &tauri::AppHandle) {
    // 桩实现：no-op
}

/// 清理托盘图标、释放 single-instance 锁后重启当前应用。
///
/// 此处提供真实实现：直接调用 `tauri::process::restart`。
/// 托盘图标 / single-instance 锁的清理由 src-tauri 中的实现接管
/// （src-tauri 中的 `restart_process` 会先调用 `remove_tray_icon_before_exit`
/// 和 `destroy_single_instance_lock`，再调用 `tauri::process::restart`）。
pub fn restart_process(app: &tauri::AppHandle) -> ! {
    use tauri::Manager;
    tauri::process::restart(&app.env());
}

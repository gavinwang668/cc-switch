//! app_config_dir 覆盖路径桩模块
//!
//! 完整实现在 `src-tauri/src/app_store.rs`（依赖 tauri-plugin-store）。
//! core crate 中仅提供最小桩，使 `config::get_app_config_dir()` 在 GUI 模式下
//! 能编译通过；实际覆盖路径由 src-tauri 启动时缓存到全局 OnceLock 中。
//!
//! 此桩在 CLI 模式（无 `tauri` feature）下不存在。

#[cfg(feature = "tauri")]
pub fn get_app_config_dir_override() -> Option<std::path::PathBuf> {
    None
}

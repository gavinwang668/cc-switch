//! Tauri 兼容层
//!
//! 当 `feature = "tauri"` 未启用时，提供空桩类型。
//! CLI 构建走桩路径，GUI 构建走真实 tauri。

#[cfg(feature = "tauri")]
pub use tauri::{AppHandle, Emitter, Manager, Runtime, async_runtime};

#[cfg(not(feature = "tauri"))]
pub mod compat {
    /// 空 AppHandle 桩（CLI 模式不使用）
    #[derive(Clone)]
    pub struct AppHandle;

    impl AppHandle {
        pub fn state<T: Send + Sync + 'static>(&self) -> tauri::State<T> {
            unimplemented!("CLI 模式不支持 Tauri managed state")
        }

        pub fn emit_to<S: AsRef<str>, P: serde::Serialize + Clone>(
            &self,
            _target: &str,
            _event: S,
            _payload: P,
        ) -> Result<(), tauri::Error> {
            Ok(())
        }
    }

    /// CLI 模式的 Emitter trait 桩
    pub trait Emitter {
        fn emit<S: AsRef<str>, P: serde::Serialize + Clone>(
            &self,
            _event: S,
            _payload: P,
        ) -> Result<(), crate::tauri_compat::compat::Error> {
            Ok(())
        }
    }

    /// CLI 模式的 Manager trait 桩
    pub trait Manager: Emitter {
        fn state<T: Send + Sync + 'static>(&self) -> tauri::State<T>;
    }

    impl Emitter for AppHandle {}
    impl Manager for AppHandle {
        fn state<T: Send + Sync + 'static>(&self) -> tauri::State<T> {
            unimplemented!()
        }
    }

    #[derive(Debug)]
    pub struct Error(String);

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for Error {}

    /// CLI 模式：无法异步运行时（不使用）
    pub mod async_runtime {
        pub fn spawn<F: std::future::Future + Send + 'static>(_f: F) {
            // CLI 不使用 Tauri 异步运行时
        }
    }
}

#[cfg(not(feature = "tauri"))]
pub use compat::*;

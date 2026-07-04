//! 故障转移切换模块
//!
//! 处理故障转移成功后的供应商切换逻辑，包括：
//! - 去重控制（避免多个请求同时触发）
//! - 托盘菜单更新
//! - 前端事件发射

use crate::database::Database;
use crate::error::AppError;
use std::collections::HashSet;
use std::sync::Arc;
#[cfg(feature = "tauri")]
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

/// RAII guard for pending switch cleanup.
///
/// 确保在作用域结束时（无论是正常返回还是 panic）自动清理 pending 标记。
struct PendingSwitchGuard {
    pending_switches: Arc<Mutex<HashSet<String>>>,
    switch_key: String,
    done: bool,
}

impl PendingSwitchGuard {
    /// 尝试创建 guard，如果相同的切换已在进行中则返回错误。
    async fn new(
        pending_switches: &Arc<Mutex<HashSet<String>>>,
        switch_key: String,
    ) -> Result<Self, AppError> {
        let mut pending = pending_switches.lock().await;
        if pending.contains(&switch_key) {
            return Err(AppError::Message(format!("切换已在进行中: {}", switch_key)));
        }
        pending.insert(switch_key.clone());
        Ok(Self {
            pending_switches: pending_switches.clone(),
            switch_key,
            done: false,
        })
    }

    /// 标记切换已完成，后续 drop 时不再清理。
    fn mark_done(&mut self) {
        self.done = true;
    }
}

impl Drop for PendingSwitchGuard {
    fn drop(&mut self) {
        if !self.done {
            let pending_switches = self.pending_switches.clone();
            let switch_key = self.switch_key.clone();
            // 使用 tokio runtime 执行清理
            if let Ok(rt) = tokio::runtime::Handle::try_current() {
                rt.spawn(async move {
                    let mut pending = pending_switches.lock().await;
                    pending.remove(&switch_key);
                });
            }
        }
    }
}

/// 故障转移切换管理器
///
/// 负责处理故障转移成功后的供应商切换，确保 UI 能够直观反映当前使用的供应商。
#[derive(Clone)]
pub struct FailoverSwitchManager {
    /// 正在处理中的切换（key = "app_type:provider_id"）
    pending_switches: Arc<Mutex<HashSet<String>>>,
    db: Arc<Database>,
}

impl FailoverSwitchManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            pending_switches: Arc::new(Mutex::new(HashSet::new())),
            db,
        }
    }

    /// 尝试执行故障转移切换
    ///
    /// 如果相同的切换已在进行中，则跳过；否则执行切换逻辑。
    ///
    /// # Returns
    /// - `Ok(true)` - 切换成功执行
    /// - `Ok(false)` - 切换已在进行中，跳过
    /// - `Err(e)` - 切换过程中发生错误
    pub async fn try_switch(
        &self,
        app_handle: Option<&TauriAppHandle>,
        app_type: &str,
        provider_id: &str,
        provider_name: &str,
    ) -> Result<bool, AppError> {
        let switch_key = format!("{app_type}:{provider_id}");

        // 使用 RAII guard 确保 panic 时也能正确清理
        let mut guard = match PendingSwitchGuard::new(&self.pending_switches, switch_key).await {
            Ok(g) => g,
            Err(_) => {
                log::debug!("[Failover] 切换已在进行中，跳过: {app_type} -> {provider_id}");
                return Ok(false);
            }
        };

        // 执行切换
        let result = self
            .do_switch(app_handle, app_type, provider_id, provider_name)
            .await;

        // 标记完成，guard.drop() 时不会再清理
        guard.mark_done();

        result
    }

    async fn do_switch(
        &self,
        app_handle: Option<&TauriAppHandle>,
        app_type: &str,
        provider_id: &str,
        provider_name: &str,
    ) -> Result<bool, AppError> {
        // 检查该应用是否已被代理接管（enabled=true）
        // 只有被接管的应用才允许执行故障转移切换
        let app_enabled = match self.db.get_proxy_config_for_app(app_type).await {
            Ok(config) => config.enabled,
            Err(e) => {
                log::warn!("[FO-002] 无法读取 {app_type} 配置: {e}，跳过切换");
                return Ok(false);
            }
        };

        if !app_enabled {
            log::debug!("[Failover] {app_type} 未启用代理，跳过切换");
            return Ok(false);
        }

        log::info!("[FO-001] 切换: {app_type} → {provider_name}");

        let mut switched = false;

        if let Some(app) = app_handle {
            if let Some(app_state) = app.try_state::<crate::store::AppState>() {
                switched = app_state
                    .proxy_service
                    .hot_switch_provider(app_type, provider_id)
                    .await
                    .map_err(AppError::Message)?
                    .logical_target_changed;

                if !switched {
                    return Ok(false);
                }

                #[cfg(feature = "tauri")]
                {
                if let Ok(new_menu) = crate::tray::create_tray_menu(app, app_state.inner()).await {
                    if let Some(tray) = app.tray_by_id(crate::tray::TRAY_ID) {
                        if let Err(e) = tray.set_menu(Some(new_menu)) {
                            log::error!("[Failover] 更新托盘菜单失败: {e}");
                        }
                    }
                }}
            }

            // 发射事件到前端
            let event_data = serde_json::json!({
                "appType": app_type,
                "providerId": provider_id,
                "source": "failover"  // 标识来源是故障转移
            });
            if let Err(e) = app.emit("provider-switched", event_data) {
                log::error!("[Failover] 发射事件失败: {e}");
            }
        }

        Ok(switched)
    }
}

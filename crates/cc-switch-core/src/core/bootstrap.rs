//! 无头模式初始化引导
//!
//! 抽取 `lib.rs` setup 闭包中的初始化逻辑，供 CLI / daemon 模式复用。
//! GUI 模式仍使用原有的 Tauri setup 闭包（含对话框交互），
//! 此模块仅包含不依赖 `tauri::AppHandle` 的核心初始化步骤。

use std::sync::Arc;

use crate::database::Database;
use crate::services::{McpService, PromptService, ProviderService};
use crate::store::AppState;

/// 引导无头模式：执行完整的初始化序列，返回就绪的 AppState。
///
/// 包含：数据库初始化、schema 迁移、config.json 迁移、
/// Live 配置导入、官方预设 seed、MCP/Prompts/Skills 导入、
/// 全局出站代理初始化、异常退出恢复。
///
/// **不包含**（需要 AppHandle 或 Tauri 运行时）：
/// - WebDAV/S3 自动同步 worker（需 AppHandle emit 事件）
/// - CopilotAuthManager / CodexOAuthManager 初始化（需 `app.manage()`）
/// - `usage_events::init`（需 AppHandle 推送日志到前端）
/// - 代理状态恢复（需异步执行，由调用方在 runtime 中 spawn）
pub fn bootstrap_headless() -> Result<AppState, String> {
    // ============================================================
    // 1. 初始化数据库（含 schema 迁移）
    // ============================================================
    let db = match Database::init() {
        Ok(db) => Arc::new(db),
        Err(e) => {
            return Err(format!("数据库初始化失败: {e}"));
        }
    };

    // ============================================================
    // 2. config.json → SQLite 迁移（如有旧配置文件）
    // ============================================================
    let app_config_dir = crate::config::get_app_config_dir();
    let json_path = app_config_dir.join("config.json");
    let db_path = app_config_dir.join("cc-switch.db");

    let has_json = json_path.exists();
    let has_db = db_path.exists();

    if !has_db && has_json {
        log::info!("检测到旧版配置文件，开始迁移...");
        match crate::app_config::MultiAppConfig::load() {
            Ok(config) => match db.migrate_from_json(&config) {
                Ok(_) => {
                    log::info!("✓ 配置迁移成功");
                    let archive_path = json_path.with_extension("json.migrated");
                    if let Err(e) = std::fs::rename(&json_path, &archive_path) {
                        log::warn!("归档旧配置文件失败: {e}");
                    } else {
                        log::info!("✓ 旧配置已归档为 config.json.migrated");
                    }
                }
                Err(e) => {
                    log::error!("配置迁移失败: {e}，将从现有配置导入");
                }
            },
            Err(e) => {
                return Err(format!("加载旧配置文件失败: {e}"));
            }
        }
    }

    // ============================================================
    // 3. 默认 Skills 仓库初始化
    // ============================================================
    match db.init_default_skill_repos() {
        Ok(count) if count > 0 => log::info!("✓ Initialized {count} default skill repositories"),
        Ok(_) => {}
        Err(e) => log::warn!("✗ Failed to initialize default skill repos: {e}"),
    }

    // ============================================================
    // 4. Skills SSOT 迁移
    // ============================================================
    match db.get_setting("skills_ssot_migration_pending") {
        Ok(Some(flag)) if flag == "true" || flag == "1" => {
            let has_existing = db
                .get_all_installed_skills()
                .map(|skills| !skills.is_empty())
                .unwrap_or(false);

            if has_existing {
                log::info!(
                    "Detected skills_ssot_migration_pending but skills table not empty; skipping."
                );
                let _ = db.set_setting("skills_ssot_migration_pending", "false");
            } else {
                match crate::services::skill::migrate_skills_to_ssot(&db) {
                    Ok(count) => {
                        log::info!("✓ Auto imported {count} skill(s) into SSOT");
                        let _ = db.set_setting("skills_ssot_migration_pending", "false");
                    }
                    Err(e) => {
                        log::warn!("✗ Failed to auto import legacy skills to SSOT: {e}");
                    }
                }
            }
        }
        Ok(_) => {}
        Err(e) => log::warn!("✗ Failed to read skills migration flag: {e}"),
    }

    // ============================================================
    // 5. 创建 AppState
    // ============================================================
    let app_state = AppState::new(db.clone());

    // ============================================================
    // 6. Live 配置导入 + 官方预设供应商 seed
    // ============================================================
    for app_type in crate::app_config::AppType::all().filter(|t| !t.is_additive_mode()) {
        if !crate::services::provider::should_import_default_config_on_startup(
            &app_state, &app_type,
        )
        .unwrap_or(false)
        {
            continue;
        }

        match crate::services::provider::import_default_config(&app_state, app_type.clone()) {
            Ok(true) => log::info!(
                "✓ Imported live config for {} as default provider",
                app_type.as_str()
            ),
            Ok(false) => {}
            Err(e) => log::debug!("○ No live config to import for {}: {e}", app_type.as_str()),
        }
    }

    match db.init_default_official_providers() {
        Ok(count) if count > 0 => log::info!("✓ Seeded {count} official provider(s)"),
        Ok(_) => {}
        Err(e) => log::warn!("✗ Failed to seed official providers: {e}"),
    }

    // ============================================================
    // 7. OpenCode / OpenClaw / Hermes Live 导入
    // ============================================================
    match crate::services::provider::import_opencode_providers_from_live(&app_state) {
        Ok(count) if count > 0 => {
            log::info!("✓ Imported {count} OpenCode provider(s) from live config")
        }
        Ok(_) => {}
        Err(e) => log::warn!("✗ Failed to import OpenCode providers: {e}"),
    }
    match crate::services::provider::import_openclaw_providers_from_live(&app_state) {
        Ok(count) if count > 0 => {
            log::info!("✓ Imported {count} OpenClaw provider(s) from live config")
        }
        Ok(_) => {}
        Err(e) => log::warn!("✗ Failed to import OpenClaw providers: {e}"),
    }
    match crate::services::provider::import_hermes_providers_from_live(&app_state) {
        Ok(count) if count > 0 => {
            log::info!("✓ Imported {count} Hermes provider(s) from live config")
        }
        Ok(_) => {}
        Err(e) => log::warn!("✗ Failed to import Hermes providers: {e}"),
    }

    // ============================================================
    // 8. OMO 配置导入
    // ============================================================
    {
        let has_omo = db
            .get_all_providers("opencode")
            .map(|providers| {
                providers
                    .values()
                    .any(|p| p.category.as_deref() == Some("omo"))
            })
            .unwrap_or(false);
        if !has_omo {
            match crate::services::OmoService::import_from_local(
                &app_state,
                &crate::services::omo::STANDARD,
            ) {
                Ok(provider) => log::info!(
                    "✓ Imported OMO config from local as provider '{}'",
                    provider.name
                ),
                Err(crate::error::AppError::OmoConfigNotFound) => {}
                Err(e) => log::warn!("✗ Failed to import OMO config from local: {e}"),
            }
        }
    }
    {
        let has_omo_slim = db
            .get_all_providers("opencode")
            .map(|providers| {
                providers
                    .values()
                    .any(|p| p.category.as_deref() == Some("omo-slim"))
            })
            .unwrap_or(false);
        if !has_omo_slim {
            match crate::services::OmoService::import_from_local(
                &app_state,
                &crate::services::omo::SLIM,
            ) {
                Ok(provider) => log::info!(
                    "✓ Imported OMO Slim config from local as provider '{}'",
                    provider.name
                ),
                Err(crate::error::AppError::OmoConfigNotFound) => {}
                Err(e) => log::warn!("✗ Failed to import OMO Slim config from local: {e}"),
            }
        }
    }

    // ============================================================
    // 9. MCP 导入（表空时）
    // ============================================================
    if db.is_mcp_table_empty().unwrap_or(false) {
        log::info!("MCP table empty, importing from live configurations...");
        let _ = McpService::import_from_claude(&app_state);
        let _ = McpService::import_from_codex(&app_state);
        let _ = McpService::import_from_gemini(&app_state);
        let _ = McpService::import_from_opencode(&app_state);
        let _ = McpService::import_from_hermes(&app_state);
    }

    // ============================================================
    // 10. Prompts 导入（表空时）
    // ============================================================
    if db.is_prompts_table_empty().unwrap_or(false) {
        log::info!("Prompts table empty, importing from live configurations...");
        for app in [
            crate::app_config::AppType::Claude,
            crate::app_config::AppType::Codex,
            crate::app_config::AppType::Gemini,
            crate::app_config::AppType::OpenCode,
            crate::app_config::AppType::OpenClaw,
            crate::app_config::AppType::Hermes,
        ] {
            let _ = PromptService::import_from_file_on_first_launch(&app_state, app.clone());
        }
    }

    // ============================================================
    // 11. 通用配置片段提取（必须在代理接管恢复之前）
    // ============================================================
    extract_common_config_snippets(&app_state);

    // ============================================================
    // 12. 全局出站代理 HTTP 客户端初始化
    // ============================================================
    {
        let proxy_url = db.get_global_proxy_url().ok().flatten();
        if let Err(e) = crate::proxy::http_client::init(proxy_url.as_deref()) {
            log::error!("[GlobalProxy] Failed to initialize with saved config: {e}");
            if proxy_url.is_some() {
                log::warn!("[GlobalProxy] Clearing invalid proxy config from database");
                let _ = db.set_global_proxy_url(None);
            }
            if let Err(fallback_err) = crate::proxy::http_client::init(None) {
                log::error!("[GlobalProxy] Failed to initialize direct connection: {fallback_err}");
            }
        }
    }

    // ============================================================
    // 13. 周期性备份检查（启动时执行一次）
    // ============================================================
    if let Err(e) = db.periodic_backup_if_needed() {
        log::warn!("Periodic backup failed on startup: {e}");
    }

    Ok(app_state)
}

/// 从干净的 Live 配置中自动提取通用配置片段。
///
/// 必须在代理接管恢复之前执行，否则会读到代理占位符配置而非用户实际设置。
fn extract_common_config_snippets(state: &AppState) {
    for app_type in crate::app_config::AppType::all() {
        if !state
            .db
            .should_auto_extract_config_snippet(app_type.as_str())
            .unwrap_or(false)
        {
            continue;
        }

        let settings = match ProviderService::read_live_settings(app_type.clone()) {
            Ok(s) => s,
            Err(_) => continue,
        };

        match ProviderService::extract_common_config_snippet_from_settings(
            app_type.clone(),
            &settings,
        ) {
            Ok(snippet) if !snippet.is_empty() && snippet != "{}" => {
                match state
                    .db
                    .set_config_snippet(app_type.as_str(), Some(snippet))
                {
                    Ok(()) => {
                        let _ = state
                            .db
                            .set_config_snippet_cleared(app_type.as_str(), false);
                        log::info!(
                            "✓ Auto-extracted common config snippet for {}",
                            app_type.as_str()
                        );
                    }
                    Err(e) => log::warn!(
                        "✗ Failed to save config snippet for {}: {e}",
                        app_type.as_str()
                    ),
                }
            }
            Ok(_) => {}
            Err(e) => log::warn!(
                "✗ Failed to extract config snippet for {}: {e}",
                app_type.as_str()
            ),
        }
    }

    let should_run_legacy_migration = state
        .db
        .is_legacy_common_config_migrated()
        .map(|done| !done)
        .unwrap_or(true);

    if should_run_legacy_migration {
        for app_type in [
            crate::app_config::AppType::Claude,
            crate::app_config::AppType::Codex,
            crate::app_config::AppType::Gemini,
        ] {
            if let Err(e) = ProviderService::migrate_legacy_common_config_usage_if_needed(
                state,
                app_type.clone(),
            ) {
                log::warn!(
                    "✗ Failed to migrate legacy common-config usage for {}: {e}",
                    app_type.as_str()
                );
            }
        }
        if let Err(e) = state.db.set_legacy_common_config_migrated(true) {
            log::warn!("✗ Failed to persist legacy common-config migration flag: {e}");
        }
    }
}

/// 异步恢复任务：异常退出恢复 + 代理状态自动恢复。
///
/// 应在 tokio runtime 中 spawn 执行。
pub async fn restore_on_startup(state: &AppState) {
    // 检查是否有 Live 备份（表示上次异常退出时可能处于接管状态）
    let has_backups = match state.db.has_any_live_backup().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("检查 Live 备份失败: {e}");
            false
        }
    };
    let live_taken_over = state.proxy_service.detect_takeover_in_live_configs();

    if has_backups || live_taken_over {
        log::warn!("检测到上次异常退出（存在接管残留），正在恢复 Live 配置...");
        if let Err(e) = state.proxy_service.recover_from_crash().await {
            log::error!("恢复 Live 配置失败: {e}");
        } else {
            log::info!("Live 配置已恢复");
        }
    }

    // 检查 settings 表中的代理状态，自动恢复代理服务
    restore_proxy_state_on_startup(state).await;
}

/// 启动时根据 proxy_config 表中的代理状态自动恢复代理接管。
async fn restore_proxy_state_on_startup(state: &AppState) {
    let mut apps_to_restore = Vec::new();
    for app_type in ["claude", "codex", "gemini"] {
        if let Ok(config) = state.db.get_proxy_config_for_app(app_type).await {
            if config.enabled {
                apps_to_restore.push(app_type);
            }
        }
    }

    if apps_to_restore.is_empty() {
        log::debug!("启动时无需恢复代理状态");
        return;
    }

    log::info!("检测到上次代理状态需要恢复，应用列表: {apps_to_restore:?}");

    for app_type in apps_to_restore {
        match state
            .proxy_service
            .set_takeover_for_app(app_type, true)
            .await
        {
            Ok(()) => log::info!("✓ 已恢复 {app_type} 的代理接管状态"),
            Err(e) => {
                log::error!("✗ 恢复 {app_type} 的代理接管状态失败: {e}");
                if let Err(clear_err) = state
                    .proxy_service
                    .set_takeover_for_app(app_type, false)
                    .await
                {
                    log::error!("清除 {app_type} 代理状态失败: {clear_err}");
                }
            }
        }
    }
}

/// 启动会话用量同步 worker（首次同步 + 每 60 秒定期同步）。
///
/// 在 daemon 模式下调用，持续同步各 CLI 工具的会话用量到数据库。
pub fn start_usage_sync_worker(db: Arc<Database>) {
    tokio::spawn(async move {
        const SESSION_SYNC_INTERVAL_SECS: u64 = 60;

        fn run_step<T>(name: &str, result: Result<T, crate::error::AppError>) {
            if let Err(e) = result {
                log::warn!("{name} failed: {e}");
            }
        }

        // 首次同步
        run_step(
            "Usage cost startup backfill",
            db.backfill_missing_usage_costs(),
        );
        run_step(
            "Session usage initial sync",
            crate::services::session_usage::sync_claude_session_logs(&db),
        );
        run_step(
            "Codex usage initial sync",
            crate::services::session_usage_codex::sync_codex_usage(&db),
        );
        run_step(
            "Gemini usage initial sync",
            crate::services::session_usage_gemini::sync_gemini_usage(&db),
        );
        run_step(
            "OpenCode usage initial sync",
            crate::services::session_usage_opencode::sync_opencode_usage(&db),
        );

        // 定期同步
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(SESSION_SYNC_INTERVAL_SECS));
        interval.tick().await; // skip immediate first tick
        loop {
            interval.tick().await;
            run_step(
                "Session usage periodic sync",
                crate::services::session_usage::sync_claude_session_logs(&db),
            );
            run_step(
                "Codex usage periodic sync",
                crate::services::session_usage_codex::sync_codex_usage(&db),
            );
            run_step(
                "Gemini usage periodic sync",
                crate::services::session_usage_gemini::sync_gemini_usage(&db),
            );
            run_step(
                "OpenCode usage periodic sync",
                crate::services::session_usage_opencode::sync_opencode_usage(&db),
            );
        }
    });
}

/// 启动 WebDAV/S3 自动同步 worker（headless 模式，AppHandle 为 None）。
pub fn start_sync_workers(db: Arc<Database>) {
    crate::services::webdav_auto_sync::start_worker(db.clone(), None);
    crate::services::s3_auto_sync::start_worker(db, None);
}

/// 启动周期性备份 timer（每 24 小时执行一次）。
pub fn start_periodic_backup_timer(db: Arc<Database>) {
    tokio::spawn(async move {
        const PERIODIC_MAINTENANCE_INTERVAL_SECS: u64 = 24 * 60 * 60;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            PERIODIC_MAINTENANCE_INTERVAL_SECS,
        ));
        interval.tick().await; // skip immediate first tick
        loop {
            interval.tick().await;
            if let Err(e) = db.periodic_backup_if_needed() {
                log::warn!("Periodic maintenance timer failed: {e}");
            }
        }
    });
}

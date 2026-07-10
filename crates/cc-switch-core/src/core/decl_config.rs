//! 声明式配置文件支持
//!
//! 允许通过 YAML 文件批量配置供应商、代理、故障转移等设置，
//! 适合无头服务器部署和自动化配置管理。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// 声明式配置应用上下文。
///
/// CLI 传 `proxy_service: None`，apply 时对代理字段写日志"需手动设置"；
/// GUI 传完整 ctx，apply 时真正应用。
pub struct ApplyContext<'a> {
    pub db: &'a crate::database::Database,
    pub proxy_service: Option<&'a crate::services::ProxyService>,
}

impl<'a> ApplyContext<'a> {
    pub fn new(db: &'a crate::database::Database) -> Self {
        Self {
            db,
            proxy_service: None,
        }
    }
}

/// 声明式配置文件根结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct DeclConfig {
    /// 代理服务器配置
    #[serde(default)]
    pub proxy: ProxySection,
    /// 供应商列表
    #[serde(default)]
    pub providers: Vec<ProviderEntry>,
    /// 故障转移配置
    #[serde(default)]
    pub failover: FailoverSection,
    /// 全局出站代理
    #[serde(default)]
    pub global_proxy: Option<GlobalProxySection>,
    /// 设备级设置
    #[serde(default)]
    pub settings: SettingsSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct ProxySection {
    /// 仅保留 takeover（可应用）。listen/port 已移除：使用 CC_SWITCH_LISTEN / CC_SWITCH_PORT
    /// 环境变量或 proxy-config 命令设置监听地址与端口。
    #[serde(default)]
    pub takeover: HashMap<String, bool>,
}

impl Default for ProxySection {
    fn default() -> Self {
        Self {
            takeover: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderEntry {
    pub app: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct FailoverSection {
    #[serde(default)]
    pub auto: bool,
    #[serde(default)]
    pub queue: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GlobalProxySection {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct SettingsSection {
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub backup_interval_hours: Option<u32>,
    #[serde(default)]
    pub backup_retain_count: Option<u32>,
    #[serde(default)]
    pub claude_config_dir: Option<String>,
    #[serde(default)]
    pub codex_config_dir: Option<String>,
    #[serde(default)]
    pub gemini_config_dir: Option<String>,
}

impl DeclConfig {
    /// 从 YAML 文件加载配置
    pub fn from_yaml_file(path: &str) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("读取配置文件失败: {e}"))?;
        Self::from_yaml_str(&content)
    }

    /// 从 YAML 字符串加载配置
    pub fn from_yaml_str(yaml: &str) -> Result<Self, String> {
        serde_yaml::from_str(yaml).map_err(|e| format!("解析 YAML 失败: {e}"))
    }

    /// 校验配置合法性
    pub fn validate(&self) -> Result<(), String> {
        const VALID_APPS: &[&str] = &[
            "claude",
            "claude-desktop",
            "codex",
            "gemini",
            "opencode",
            "openclaw",
            "hermes",
        ];
        const VALID_TAKEOVER_APPS: &[&str] = &["claude", "codex", "gemini"];

        for p in &self.providers {
            if !VALID_APPS.contains(&p.app.as_str()) {
                return Err(format!("无效的应用类型: {} (供应商 {})", p.app, p.id));
            }
            if p.id.is_empty() {
                return Err("供应商 ID 不能为空".to_string());
            }
            if p.name.is_empty() {
                return Err(format!("供应商 {} 的名称不能为空", p.id));
            }
        }

        for (app, _) in &self.proxy.takeover {
            if !VALID_TAKEOVER_APPS.contains(&app.as_str()) {
                return Err(format!("代理接管仅支持 claude/codex/gemini，不支持: {app}"));
            }
        }

        for (app, _) in &self.failover.queue {
            if !VALID_TAKEOVER_APPS.contains(&app.as_str()) {
                return Err(format!(
                    "故障转移队列仅支持 claude/codex/gemini，不支持: {app}"
                ));
            }
        }

        Ok(())
    }

    /// 将声明式配置应用到数据库和代理服务。
    ///
    /// - `ctx.proxy_service = None`（CLI 模式）：跳过代理字段应用，记录日志
    /// - `ctx.proxy_service = Some(_)`（GUI 模式）：完整应用代理字段
    pub async fn apply(&self, ctx: &ApplyContext<'_>) -> Result<String, String> {
        let db = ctx.db;
        let mut actions = Vec::new();

        // 0. 应用前自动创建备份（供 rollback 命令恢复）
        match db.backup_database_file() {
            Ok(Some(backup_path)) => {
                if let Some(fname) = backup_path.file_name().and_then(|n| n.to_str()) {
                    let _ = db.set_setting("last_apply_backup", fname);
                    log::info!("apply-config 前已创建备份: {fname}");
                }
            }
            Ok(None) => {
                log::warn!("数据库文件不存在，跳过 apply 前备份");
            }
            Err(e) => {
                log::warn!("apply 前备份失败（继续应用）: {e}");
            }
        }

        // 1. 应用供应商配置
        for p in &self.providers {
            let mut env = serde_json::Map::new();
            for (k, v) in &p.env {
                env.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            let settings_config = serde_json::json!({ "env": env });
            let provider =
                crate::Provider::with_id(p.id.clone(), p.name.clone(), settings_config, None);
            db.save_provider(&p.app, &provider)
                .map_err(|e| format!("保存供应商 {} 失败: {e}", p.id))?;

            if p.current {
                crate::core::provider_manager::switch_provider(db, &p.app, &p.id)
                    .map_err(|e| format!("切换供应商 {} 失败: {e}", p.id))?;
            }
            actions.push(format!("供应商 {}/{} ({}) 已配置", p.app, p.id, p.name));
        }

        // 2. 应用全局出站代理
        if let Some(gp) = &self.global_proxy {
            db.set_global_proxy_url(Some(&gp.url))
                .map_err(|e| format!("设置全局代理失败: {e}"))?;
            crate::proxy::http_client::init(Some(&gp.url))
                .map_err(|e| format!("初始化 HTTP 客户端失败: {e}"))?;
            actions.push(format!("全局出站代理已设置为: {}", gp.url));
        }

        // 3. 应用故障转移队列
        for (app, ids) in &self.failover.queue {
            if let Ok(existing) = db.get_failover_queue(app) {
                for item in existing {
                    let _ = db.remove_from_failover_queue(app, &item.provider_id);
                }
            }
            for id in ids {
                db.add_to_failover_queue(app, id)
                    .map_err(|e| format!("添加故障转移队列 {app}/{id} 失败: {e}"))?;
            }
            actions.push(format!("故障转移队列 {app} 已配置 ({} 项)", ids.len()));
        }

        // 4. 应用自动故障转移
        if self.failover.auto {
            for app in ["claude", "codex", "gemini"] {
                let (enabled, _) = db.get_proxy_flags_sync(app);
                db.set_proxy_flags_sync(app, enabled, true)
                    .map_err(|e| format!("设置自动故障转移 {app} 失败: {e}"))?;
            }
            actions.push("自动故障转移已全局开启".to_string());
        }

        // 5. 应用代理接管状态（依赖 proxy_service）
        for (app, enabled) in &self.proxy.takeover {
            match ctx.proxy_service {
                Some(svc) => {
                    let takeover_app = crate::app_config::AppType::from_str(app)
                        .map_err(|_| format!("无效的应用类型: {app}"))?;
                    svc.set_takeover_for_app(takeover_app.as_str(), *enabled)
                        .await
                        .map_err(|e| format!("设置接管 {app}={} 失败: {e}", enabled))?;
                    actions.push(format!("代理接管 {app}={}", enabled));
                }
                None => {
                    log::warn!(
                        "代理接管 {app}={enabled} 需 proxy_service，当前 CLI 模式未提供，请手动执行 takeover 命令"
                    );
                    actions.push(format!(
                        "（跳过）代理接管 {app}={enabled} —— CLI 模式需手动执行 `cc-switch-cli takeover {app} on`"
                    ));
                }
            }
        }

        // 6. 应用设备级设置
        let mut settings = crate::settings::get_settings();
        if let Some(lang) = &self.settings.language {
            settings.language = Some(lang.clone());
        }
        if let Some(hours) = self.settings.backup_interval_hours {
            settings.backup_interval_hours = Some(hours);
        }
        if let Some(count) = self.settings.backup_retain_count {
            settings.backup_retain_count = Some(count);
        }
        if let Some(dir) = &self.settings.claude_config_dir {
            settings.claude_config_dir = Some(dir.clone());
        }
        if let Some(dir) = &self.settings.codex_config_dir {
            settings.codex_config_dir = Some(dir.clone());
        }
        if let Some(dir) = &self.settings.gemini_config_dir {
            settings.gemini_config_dir = Some(dir.clone());
        }
        match crate::settings::update_settings(settings) {
            Ok(_) => actions.push("设备级设置已更新".to_string()),
            Err(e) => log::warn!("更新设备级设置失败: {e}"),
        }

        Ok(actions.join("\n"))
    }

    /// 从数据库反向构造 DeclConfig（用于 export-yaml / diff）
    pub fn from_database(db: &crate::database::Database) -> Result<Self, String> {
        let mut config = DeclConfig::default();

        // 读取所有应用的供应商
        let app_types = [
            "claude",
            "claude-desktop",
            "codex",
            "gemini",
            "opencode",
            "openclaw",
            "hermes",
        ];
        for app_type in &app_types {
            let providers = db
                .get_all_providers(app_type)
                .map_err(|e| format!("读取 {app_type} 供应商失败: {e}"))?;
            let current_id = db
                .get_current_provider(app_type)
                .unwrap_or(None)
                .unwrap_or_default();
            for (_id, p) in &providers {
                let mut env_map = HashMap::new();
                if let Some(env_val) = p.settings_config.get("env") {
                    if let Some(obj) = env_val.as_object() {
                        for (k, v) in obj {
                            if let Some(s) = v.as_str() {
                                env_map.insert(k.clone(), s.to_string());
                            }
                        }
                    }
                }
                config.providers.push(ProviderEntry {
                    app: app_type.to_string(),
                    id: p.id.clone(),
                    name: p.name.clone(),
                    env: env_map,
                    current: p.id == current_id,
                });
            }
        }

        // 读取故障转移队列
        for app in &["claude", "codex", "gemini"] {
            if let Ok(queue) = db.get_failover_queue(app) {
                if !queue.is_empty() {
                    config.failover.queue.insert(
                        app.to_string(),
                        queue.iter().map(|q| q.provider_id.clone()).collect(),
                    );
                }
            }
            let (_, af) = db.get_proxy_flags_sync(app);
            if af {
                config.failover.auto = true;
            }
        }

        // 读取接管状态（通过 proxy flags 推断）
        for app in &["claude", "codex", "gemini"] {
            let (enabled, _) = db.get_proxy_flags_sync(app);
            if enabled {
                config.proxy.takeover.insert(app.to_string(), true);
            }
        }

        Ok(config)
    }
}

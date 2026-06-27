//! 声明式配置文件支持
//!
//! 允许通过 YAML 文件批量配置供应商、代理、故障转移等设置，
//! 适合无头服务器部署和自动化配置管理。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
#[serde(rename_all = "snake_case")]
pub struct ProxySection {
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub takeover: HashMap<String, bool>,
}

impl Default for ProxySection {
    fn default() -> Self {
        Self {
            listen: default_listen(),
            port: default_port(),
            takeover: HashMap::new(),
        }
    }
}

fn default_listen() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    9090
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

    /// 将声明式配置应用到数据库和设置
    pub fn apply(&self, db: &crate::database::Database) -> Result<String, String> {
        let mut actions = Vec::new();

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
            // 清空现有队列（通过移除再添加实现）
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

        // 5. 应用设备级设置
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
}

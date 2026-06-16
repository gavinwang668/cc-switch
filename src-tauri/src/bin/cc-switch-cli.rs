//! cc-switch CLI — 无头模式管理工具
//!
//! 提供命令行界面管理代理服务器和供应商配置，
//! 适用于 Linux 无头环境或需要脚本化管理的场景。

use clap::{Parser, Subcommand};

use cc_switch_lib::core::provider_manager;
use cc_switch_lib::Database;
use cc_switch_lib::ProxyConfig;
use cc_switch_lib::ProxyServer;

// ============================================================================
// CLI 定义
// ============================================================================

#[derive(Parser)]
#[command(name = "cc-switch-cli", about = "cc-switch 命令行管理工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动代理服务器（前台运行）
    Start,
    /// 停止代理服务器（通过发送停止信号）
    Stop,
    /// 查看代理服务器状态
    Status,
    /// 查看或修改配置
    Config {
        /// 配置键名（不指定则列出全部）
        #[arg(long)]
        key: Option<String>,
        /// 配置值（不指定则查看当前值）
        #[arg(long)]
        value: Option<String>,
    },
    /// 列出所有供应商
    ListProviders {
        /// 应用类型 (claude, codex, gemini)，不指定则列出全部
        app: Option<String>,
    },
    /// 添加供应商
    AddProvider {
        /// 应用类型 (claude, codex, gemini)
        app: String,
        /// 供应商 ID
        id: String,
        /// 供应商名称
        name: String,
        /// API Key
        #[arg(long)]
        api_key: Option<String>,
        /// Base URL
        #[arg(long)]
        base_url: Option<String>,
    },
    /// 删除供应商
    RemoveProvider {
        /// 应用类型 (claude, codex, gemini)
        app: String,
        /// 供应商 ID
        id: String,
    },
    /// 切换当前供应商
    SwitchProvider {
        /// 应用类型 (claude, codex, gemini)
        app: String,
        /// 目标供应商 ID
        id: String,
    },
}

// ============================================================================
// 入口
// ============================================================================

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => cmd_start(),
        Commands::Stop => cmd_stop(),
        Commands::Status => cmd_status(),
        Commands::Config { key, value } => cmd_config(key.as_deref(), value.as_deref()),
        Commands::ListProviders { app } => cmd_list_providers(app.as_deref()),
        Commands::AddProvider { app, id, name, api_key, base_url } => {
            cmd_add_provider(app, id, name, api_key.as_deref(), base_url.as_deref());
        }
        Commands::RemoveProvider { app, id } => cmd_remove_provider(app, id),
        Commands::SwitchProvider { app, id } => cmd_switch_provider(app, id),
    }
}

// ============================================================================
// 命令实现
// ============================================================================

/// 初始化数据库连接
fn init_db() -> Result<Database, String> {
    Database::init().map_err(|e| format!("数据库初始化失败: {e}"))
}

/// validated_app: 检查应用类型是否合法
fn validated_app(app: &str) -> Result<(), String> {
    const SUPPORTED_APPS: &[&str] = &[
        "claude",
        "claude-desktop",
        "codex",
        "gemini",
        "opencode",
        "openclaw",
        "hermes",
    ];
    if !SUPPORTED_APPS.contains(&app) {
        return Err(format!(
            "不支持的应用类型: {app}，支持: {}",
            SUPPORTED_APPS.join(", ")
        ));
    }
    Ok(())
}

/// start: 启动代理服务器（前台运行，阻塞）
fn cmd_start() {
    let db = match init_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let db = std::sync::Arc::new(db);

    let listen_address = std::env::var("CC_SWITCH_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let listen_port: u16 = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090);

    let config = ProxyConfig {
        listen_address,
        listen_port,
        ..Default::default()
    };

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    let server = ProxyServer::new(config, db, None);

    rt.block_on(async {
        println!("正在启动代理服务器...");
        match server.start().await {
            Ok(info) => {
                println!("代理服务器已启动: {}:{}", info.address, info.port);
                println!("按 Ctrl+C 停止服务器");
                tokio::signal::ctrl_c().await.unwrap_or_default();
                println!("\n正在停止代理服务器...");
                if let Err(e) = server.stop().await {
                    eprintln!("停止代理服务器时出错: {e}");
                }
                println!("代理服务器已停止");
            }
            Err(e) => {
                eprintln!("启动代理服务器失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

/// status: 查看代理服务状态
fn cmd_status() {
    let db = match init_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let listen_address = std::env::var("CC_SWITCH_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let listen_port: u16 = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090);

    let config = ProxyConfig {
        listen_address,
        listen_port,
        ..Default::default()
    };

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    let db = std::sync::Arc::new(db);
    let server = ProxyServer::new(config, db.clone(), None);

    rt.block_on(async {
        let status = server.get_status().await;
        println!("代理服务器状态:");
        println!("  运行中: {}", if status.running { "是" } else { "否" });
        if status.running {
            println!("  地址: {}:{}", status.address, status.port);
        }

        println!("\n当前供应商:");
        for app_type in &["claude", "codex", "gemini"] {
            let current = provider_manager::get_current_provider_id(&db, app_type)
                .ok()
                .flatten()
                .unwrap_or_else(|| "(无)".to_string());
            println!("  {app_type:<10}: {current}");
        }
    });
}

/// list-providers: 列出供应商
fn cmd_list_providers(app: Option<&str>) {
    let db = match init_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let app_types: Vec<&str> = match app {
        Some(a) => {
            if let Err(e) = validated_app(a) {
                eprintln!("错误: {e}");
                std::process::exit(1);
            }
            vec![a]
        }
        None => vec!["claude", "codex", "gemini"],
    };

    for app_type in &app_types {
        let providers = match provider_manager::get_all_providers(&db, app_type) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("错误: 获取 {app_type} 供应商列表失败: {e}");
                continue;
            }
        };

        println!("\n── {app_type} ─────────────────────────────────");
        if providers.is_empty() {
            println!("  (无供应商)");
            continue;
        }

        let current_id = provider_manager::get_current_provider_id(&db, app_type)
            .ok()
            .flatten();

        println!("  {:<3} {:<20} {:<30} {}",
            "ID", "名称", "Base URL", "当前");
        for (id, provider) in &providers {
            let marker = if Some(id.as_str()) == current_id.as_deref() {
                "*"
            } else {
                " "
            };
            let base_url = provider
                .settings_config
                .pointer("/env/ANTHROPIC_BASE_URL")
                .or_else(|| provider.settings_config.pointer("/env/BASE_URL"))
                .or_else(|| provider.settings_config.pointer("/baseUrl"))
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            println!("  {marker:<3} {id:<20} {:<30} {}", provider.name, base_url);
        }
    }
}

/// add-provider: 添加供应商
fn cmd_add_provider(app: &str, id: &str, name: &str, api_key: Option<&str>, base_url: Option<&str>) {
    if let Err(e) = validated_app(app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }

    let db = match init_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let mut env = serde_json::Map::new();
    if let Some(key) = api_key {
        env.insert(
            "ANTHROPIC_API_KEY".to_string(),
            serde_json::Value::String(key.to_string()),
        );
    }
    if let Some(url) = base_url {
        env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            serde_json::Value::String(url.to_string()),
        );
    }

    let settings_config = serde_json::json!({
        "env": env,
    });

    let provider = cc_switch_lib::Provider::with_id(
        id.to_string(),
        name.to_string(),
        settings_config,
        None,
    );

    match db.save_provider(app, &provider) {
        Ok(_) => println!("供应商 '{id}' ({name}) 已添加到 {app}"),
        Err(e) => {
            eprintln!("添加供应商失败: {e}");
            std::process::exit(1);
        }
    }
}

/// remove-provider: 删除供应商
fn cmd_remove_provider(app: &str, id: &str) {
    if let Err(e) = validated_app(app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }

    let db = match init_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    match provider_manager::remove_provider(&db, app, id) {
        Ok(_) => println!("供应商 '{id}' 已从 {app} 删除"),
        Err(e) => {
            eprintln!("删除供应商失败: {e}");
            std::process::exit(1);
        }
    }
}

/// switch-provider: 切换当前供应商
fn cmd_switch_provider(app: &str, id: &str) {
    if let Err(e) = validated_app(app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }

    let db = match init_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    match provider_manager::switch_provider(&db, app, id) {
        Ok(_) => println!("已切换到供应商 '{id}' ({app})"),
        Err(e) => {
            eprintln!("切换供应商失败: {e}");
            std::process::exit(1);
        }
    }
}

/// stop: 停止代理服务器（通过 HTTP 请求通知后台进程停止）
fn cmd_stop() {
    let listen_address = std::env::var("CC_SWITCH_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1".to_string());
    let listen_port: u16 = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090);

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    
    rt.block_on(async {
        let url = format!("http://{}:{}/stop", listen_address, listen_port);
        
        match reqwest::get(&url).await {
            Ok(_) => {
                println!("已发送停止信号到代理服务器 {}:{}", listen_address, listen_port);
            }
            Err(e) => {
                eprintln!("停止代理服务器失败: {e}");
                eprintln!("请确认代理服务器正在运行");
                std::process::exit(1);
            }
        }
    });
}

/// config: 查看或修改配置
fn cmd_config(key: Option<&str>, value: Option<&str>) {
    let db = match init_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    match (key, value) {
        // 列出所有配置
        (None, None) => {
            let settings = match db.get_settings() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("获取配置失败: {e}");
                    std::process::exit(1);
                }
            };
            
            println!("当前配置:");
            println!("{}", serde_json::to_string_pretty(&settings).unwrap_or_else(|_| "{}".to_string()));
        }
        // 查看指定配置
        (Some(k), None) => {
            let settings = match db.get_settings() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("获取配置失败: {e}");
                    std::process::exit(1);
                }
            };
            
            if let Some(val) = settings.get(k) {
                println!("{} = {}", k, serde_json::to_string_pretty(val).unwrap_or_else(|_| val.to_string()));
            } else {
                eprintln!("配置项 '{}' 不存在", k);
                std::process::exit(1);
            }
        }
        // 修改配置
        (Some(k), Some(v)) => {
            let mut settings = match db.get_settings() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("获取配置失败: {e}");
                    std::process::exit(1);
                }
            };
            
            // 尝试解析 JSON 值，如果失败则作为字符串处理
            let json_value: serde_json::Value = serde_json::from_str(v)
                .unwrap_or_else(|_| serde_json::Value::String(v.to_string()));
            
            settings.insert(k.to_string(), json_value);
            
            match db.save_settings(&settings) {
                Ok(_) => println!("配置 '{}' 已更新", k),
                Err(e) => {
                    eprintln!("保存配置失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        // 无效参数组合
        (None, Some(_)) => {
            eprintln!("错误: 不能只指定 --value 而不指定 --key");
            std::process::exit(1);
        }
    }
}
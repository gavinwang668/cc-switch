//! cc-switch CLI — 无头模式管理工具
//!
//! 提供命令行界面管理代理服务器和供应商配置，
//! 适用于 Linux 无头环境或需要脚本化管理的场景。

use std::sync::Arc;

use clap::{Parser, Subcommand};

use cc_switch_lib::core::{bootstrap, provider_manager};
use cc_switch_lib::Database;

// ============================================================================
// CLI 定义
// ============================================================================

#[derive(Parser)]
#[command(name = "cc-switch-cli", about = "cc-switch 命令行管理工具")]
struct Cli {
    /// 日志级别 (error/warn/info/debug/trace)
    #[arg(long, global = true, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动代理服务器（前台运行）
    Start {
        /// 内部参数：由 daemon 命令启动的后台模式
        #[arg(long, hide = true)]
        internal_daemon: bool,
    },
    /// 以守护进程方式在后台启动代理服务器
    Daemon,
    /// 停止代理服务器（通过发送停止信号）
    Stop,
    /// 查看代理服务器状态
    Status,
    /// 查看或修改设备级设置（settings.json）
    Settings {
        /// 设置键名（不指定则列出全部）
        key: Option<String>,
        /// 设置值（不指定则查看当前值）
        value: Option<String>,
    },
    /// 查看或修改数据库配置（settings 表）
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
        /// 应用类型 (claude, claude-desktop, codex, gemini, opencode, openclaw, hermes)，不指定则列出全部
        app: Option<String>,
    },
    /// 添加供应商
    AddProvider {
        /// 应用类型 (claude, claude-desktop, codex, gemini, opencode, openclaw, hermes)
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
        /// API 格式（仅 claude/codex/gemini/claude-desktop）
        /// claude: anthropic / openai_chat / openai_responses
        /// codex: openai_responses / openai_chat
        /// gemini: gemini_native / openai_chat / openai_responses / anthropic
        /// claude-desktop: anthropic / openai_chat / openai_responses / gemini_native / bedrock
        #[arg(long)]
        api_format: Option<String>,
    },
    /// 删除供应商
    RemoveProvider {
        /// 应用类型 (claude, claude-desktop, codex, gemini, opencode, openclaw, hermes)
        app: String,
        /// 供应商 ID
        id: String,
    },
    /// 切换当前供应商
    SwitchProvider {
        /// 应用类型 (claude, claude-desktop, codex, gemini, opencode, openclaw, hermes)
        app: String,
        /// 目标供应商 ID
        id: String,
    },
    /// 更新供应商配置
    UpdateProvider {
        app: String,
        id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        base_url: Option<String>,
        /// API 格式（仅 claude/codex/gemini/claude-desktop）
        /// claude: anthropic / openai_chat / openai_responses
        /// codex: openai_responses / openai_chat
        /// gemini: gemini_native / openai_chat / openai_responses / anthropic
        /// claude-desktop: anthropic / openai_chat / openai_responses / gemini_native / bedrock
        #[arg(long)]
        api_format: Option<String>,
        /// 清除 API 格式设置
        #[arg(long)]
        clear_api_format: bool,
    },
    /// 设置/查看代理接管状态
    Takeover {
        /// 应用类型 (claude, codex, gemini)
        app: String,
        /// on 或 off，不指定则查看当前状态
        enabled: Option<String>,
    },
    /// 代理模式下热切换供应商
    SwitchProxy {
        /// 应用类型 (claude, codex, gemini)
        app: String,
        /// 目标供应商 ID
        id: String,
    },
    /// 查看/设置故障转移队列
    FailoverQueue {
        /// list / add / remove
        action: String,
        /// 应用类型
        app: Option<String>,
        /// 供应商 ID（add/remove 时需要）
        id: Option<String>,
    },
    /// 查看/设置自动故障转移
    AutoFailover {
        /// 应用类型，不指定则查看全部
        app: Option<String>,
        /// on 或 off，不指定则查看当前状态
        enabled: Option<String>,
    },
    /// 查看/设置熔断器配置
    CircuitBreaker {
        /// get / set / reset
        action: String,
        /// 应用类型
        app: Option<String>,
        /// 供应商 ID（reset 时需要）
        id: Option<String>,
        /// 配置 JSON（set 时需要）
        #[arg(long)]
        config: Option<String>,
    },
    /// 查看/设置请求修正器配置
    Rectifier {
        /// get 或 set（set 需要 --config）
        action: String,
        #[arg(long)]
        config: Option<String>,
    },
    /// 查看/设置优化器配置
    Optimizer {
        /// get 或 set（set 需要 --config）
        action: String,
        #[arg(long)]
        config: Option<String>,
    },
    /// 查看/设置全局出站代理
    GlobalProxy {
        /// get / set / clear / test
        action: String,
        /// 代理 URL（set 时需要）
        url: Option<String>,
    },
    /// 列出 MCP 服务器
    ListMcp,
    /// 列出 Prompts
    ListPrompts { app: Option<String> },
    /// 导出配置到文件
    ExportConfig {
        /// 输出文件路径
        path: String,
    },
    /// 从文件导入配置
    ImportConfig {
        /// 输入文件路径
        path: String,
    },
    /// 创建数据库备份
    BackupCreate,
    /// 列出数据库备份
    BackupList,
    /// 从备份恢复
    BackupRestore {
        /// 备份文件名
        name: String,
    },
    /// 查看用量统计摘要
    UsageSummary {
        /// 天数（默认7天）
        #[arg(long, default_value = "7")]
        days: u32,
    },
    /// 测试 API 端点延迟
    Speedtest {
        /// API URL
        url: String,
        #[arg(long, default_value = "10")]
        timeout: u64,
    },
    /// 验证 API Key
    VerifyKey {
        /// Base URL
        #[arg(long)]
        base_url: String,
        /// API Key
        #[arg(long)]
        api_key: String,
    },
    /// 验证声明式配置文件
    Validate {
        /// YAML 配置文件路径
        path: String,
    },
    /// 应用声明式配置文件到数据库
    ApplyConfig {
        /// YAML 配置文件路径
        path: String,
    },
    /// 列出所有可用命令
    Help,
}

// ============================================================================
// 入口
// ============================================================================

fn main() {
    let cli = Cli::parse();

    // 初始化日志
    init_logging(&cli.log_level);

    match &cli.command {
        Commands::Start { internal_daemon } => cmd_start(*internal_daemon),
        Commands::Daemon => cmd_daemon(),
        Commands::Stop => cmd_stop(),
        Commands::Status => cmd_status(),
        Commands::Settings { key, value } => cmd_settings(key.as_deref(), value.as_deref()),
        Commands::Config { key, value } => cmd_config(key.as_deref(), value.as_deref()),
        Commands::ListProviders { app } => cmd_list_providers(app.as_deref()),
        Commands::AddProvider {
            app,
            id,
            name,
            api_key,
            base_url,
            api_format,
        } => {
            cmd_add_provider(app, id, name, api_key.as_deref(), base_url.as_deref(), api_format.as_deref());
        }
        Commands::RemoveProvider { app, id } => cmd_remove_provider(app, id),
        Commands::SwitchProvider { app, id } => cmd_switch_provider(app, id),
        Commands::UpdateProvider {
            app,
            id,
            name,
            api_key,
            base_url,
            api_format,
            clear_api_format,
        } => {
            cmd_update_provider(
                app,
                id,
                name.as_deref(),
                api_key.as_deref(),
                base_url.as_deref(),
                api_format.as_deref(),
                clear_api_format,
            );
        }
        Commands::Takeover { app, enabled } => cmd_takeover(app, enabled.as_deref()),
        Commands::SwitchProxy { app, id } => cmd_switch_proxy(app, id),
        Commands::FailoverQueue { action, app, id } => {
            cmd_failover_queue(&action, app.as_deref(), id.as_deref())
        }
        Commands::AutoFailover { app, enabled } => {
            cmd_auto_failover(app.as_deref(), enabled.as_deref())
        }
        Commands::CircuitBreaker {
            action,
            app,
            id,
            config,
        } => {
            cmd_circuit_breaker(&action, app.as_deref(), id.as_deref(), config.as_deref());
        }
        Commands::Rectifier { action, config } => cmd_rectifier(&action, config.as_deref()),
        Commands::Optimizer { action, config } => cmd_optimizer(&action, config.as_deref()),
        Commands::GlobalProxy { action, url } => cmd_global_proxy(&action, url.as_deref()),
        Commands::ListMcp => cmd_list_mcp(),
        Commands::ListPrompts { app } => cmd_list_prompts(app.as_deref()),
        Commands::ExportConfig { path } => cmd_export_config(path),
        Commands::ImportConfig { path } => cmd_import_config(path),
        Commands::BackupCreate => cmd_backup_create(),
        Commands::BackupList => cmd_backup_list(),
        Commands::BackupRestore { name } => cmd_backup_restore(name),
        Commands::UsageSummary { days } => cmd_usage_summary(*days),
        Commands::Speedtest { url, timeout } => cmd_speedtest(url, *timeout),
        Commands::VerifyKey { base_url, api_key } => cmd_verify_key(&base_url, &api_key),
        Commands::Validate { path } => cmd_validate(&path),
        Commands::ApplyConfig { path } => cmd_apply_config(&path),
        Commands::Help => cmd_help(),
    }
}

// ============================================================================
// 日志与 PID 文件
// ============================================================================

/// 初始化 env_logger
fn init_logging(level: &str) {
    let level = match level.to_lowercase().as_str() {
        "error" => "error",
        "warn" => "warn",
        "info" => "info",
        "debug" => "debug",
        "trace" => "trace",
        _ => "info",
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level))
        .format_timestamp_secs()
        .init();
}

/// 获取 PID 文件路径
fn pid_file_path() -> std::path::PathBuf {
    crate_config_dir().join("cc-switch-daemon.pid")
}

/// 获取配置目录 (~/.cc-switch)
fn crate_config_dir() -> std::path::PathBuf {
    cc_switch_lib::get_app_config_dir()
}

/// 写入 PID 文件
fn write_pid_file(pid: u32) -> Result<(), String> {
    let path = pid_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::write(&path, pid.to_string()).map_err(|e| format!("写入 PID 文件失败: {e}"))
}

/// 读取 PID 文件
fn read_pid_file() -> Option<u32> {
    let path = pid_file_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

/// 删除 PID 文件
fn remove_pid_file() {
    let _ = std::fs::remove_file(pid_file_path());
}

/// 检查 PID 对应的进程是否存活
#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    // kill(pid, 0) 返回 0 表示进程存在
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(not(unix))]
fn is_process_alive(pid: u32) -> bool {
    // Windows: 尝试 OpenProcess
    use std::process::Command;
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false)
}

// ============================================================================
// 命令实现
// ============================================================================

/// start: 启动代理服务器（前台运行，阻塞）
fn cmd_start(internal_daemon: bool) {
    // 后台模式：写 PID 文件
    if internal_daemon {
        let pid = std::process::id();
        if let Err(e) = write_pid_file(pid) {
            log::warn!("写入 PID 文件失败: {e}");
        }
    }

    // 引导初始化
    let app_state = match bootstrap::bootstrap_headless() {
        Ok(state) => state,
        Err(e) => {
            eprintln!("错误: {e}");
            if internal_daemon {
                remove_pid_file();
            }
            std::process::exit(1);
        }
    };

    let listen_address =
        std::env::var("CC_SWITCH_LISTEN").unwrap_or_else(|_| "127.0.0.1".to_string());
    let listen_port: u16 = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090);

    // 更新数据库中的代理配置（让 ProxyService.start 使用正确的监听地址和端口）
    let db = app_state.db.clone();
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    let proxy_service = app_state.proxy_service.clone();

    rt.block_on(async move {
        // 更新代理监听配置到数据库
        {
            let mut config = db.get_proxy_config().await.unwrap_or_default();
            config.listen_address = listen_address.clone();
            config.listen_port = listen_port;
            let _ = db.update_proxy_config(config).await;
        }

        // 异步恢复任务（异常退出恢复 + 代理状态恢复）
        bootstrap::restore_on_startup(&app_state).await;

        // 启动会话用量同步 worker
        bootstrap::start_usage_sync_worker(app_state.db.clone());

        // 启动周期性备份 timer
        bootstrap::start_periodic_backup_timer(app_state.db.clone());

        // 启动 WebDAV/S3 自动同步 worker（headless 模式，无 AppHandle）
        bootstrap::start_sync_workers(app_state.db.clone());

        println!("正在启动代理服务器...");
        match proxy_service.start().await {
            Ok(info) => {
                println!("代理服务器已启动: {}:{}", info.address, info.port);
                println!("按 Ctrl+C 停止服务器");

                // 信号处理：SIGTERM / SIGINT → 优雅停止
                setup_signal_handlers(&proxy_service, internal_daemon).await;
            }
            Err(e) => {
                eprintln!("启动代理服务器失败: {e}");
                if internal_daemon {
                    remove_pid_file();
                }
                std::process::exit(1);
            }
        }
    });
}

/// daemon: 以守护进程方式在后台启动代理服务器
fn cmd_daemon() {
    // 检查是否已有实例运行
    if let Some(existing_pid) = read_pid_file() {
        if is_process_alive(existing_pid) {
            eprintln!("错误: 代理服务器已在运行 (PID: {existing_pid})");
            std::process::exit(1);
        } else {
            log::warn!("发现残留 PID 文件 (PID: {existing_pid} 已退出)，正在清理");
            remove_pid_file();
        }
    }

    // 启动后台进程
    let exe = std::env::current_exe().unwrap_or_else(|e| {
        eprintln!("无法获取可执行文件路径: {e}");
        std::process::exit(1);
    });

    // 日志文件：daemon 子进程的 stdout/stderr 重定向到此文件
    let log_path = crate_config_dir().join("cc-switch-daemon.log");
    let log_file = match std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("无法创建日志文件 {}: {e}", log_path.display());
            std::process::exit(1);
        }
    };

    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("start")
        .arg("--internal-daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(
            log_file
                .try_clone()
                .unwrap_or_else(|_| log_file.try_clone().unwrap()),
        ))
        .stderr(std::process::Stdio::from(log_file));

    // Windows: 使用 DETACHED_PROCESS 标志使子进程脱离父进程控制台
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }

    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("启动后台进程失败: {e}");
            std::process::exit(1);
        }
    };

    let pid = child.id();
    println!("代理服务器已在后台启动 (PID: {pid})");
    println!("日志文件: {}", log_path.display());

    // 等待子进程初始化（短暂等待确保 PID 文件已写入）
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 父进程退出，子进程继续运行
    // Windows: DETACHED_PROCESS 已使子进程独立；Unix: 子进程成为孤儿由 init 收养
    std::mem::forget(child);
}

/// 设置信号处理器并等待停止信号
async fn setup_signal_handlers(proxy_service: &cc_switch_lib::ProxyService, is_daemon: bool) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = signal(SignalKind::terminate()).expect("无法安装 SIGTERM 处理器");
        let mut sigint = signal(SignalKind::interrupt()).expect("无法安装 SIGINT 处理器");
        let mut sighup = signal(SignalKind::hangup()).expect("无法安装 SIGHUP 处理器");

        loop {
            tokio::select! {
                _ = sigterm.recv() => {
                    log::info!("收到 SIGTERM 信号，正在优雅停止...");
                    break;
                }
                _ = sigint.recv() => {
                    log::info!("收到 SIGINT 信号，正在优雅停止...");
                    break;
                }
                _ = sighup.recv() => {
                    log::info!("收到 SIGHUP 信号，正在重载配置...");
                    match cc_switch_lib::reload_settings() {
                        Ok(()) => log::info!("✓ 配置已重载"),
                        Err(e) => log::error!("✗ 配置重载失败: {e}"),
                    }
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        // Windows: 仅支持 Ctrl+C (SIGINT)
        tokio::signal::ctrl_c().await.unwrap_or_default();
        log::info!("收到 Ctrl+C 信号，正在优雅停止...");
    }

    // 优雅停止：stop_with_restore_keep_state 会停止 HTTP 服务器并恢复 Live 配置
    println!("\n正在停止代理服务器...");

    if let Err(e) = proxy_service.stop_with_restore_keep_state().await {
        log::error!("优雅停止失败: {e}，尝试强制停止...");
        if let Err(e) = proxy_service.stop().await {
            eprintln!("停止代理服务器时出错: {e}");
        }
    }

    if is_daemon {
        remove_pid_file();
    }

    println!("代理服务器已停止");
}

/// status: 查看代理服务状态
fn cmd_status() {
    let listen_address =
        std::env::var("CC_SWITCH_LISTEN").unwrap_or_else(|_| "127.0.0.1".to_string());
    let listen_port: u16 = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090);

    // 先检查 PID 文件判断是否在运行
    let daemon_pid = read_pid_file();
    let daemon_running = daemon_pid.map(|pid| is_process_alive(pid)).unwrap_or(false);

    println!("代理服务器状态:");
    println!(
        "  守护进程: {}",
        if daemon_running {
            format!("运行中 (PID: {})", daemon_pid.unwrap())
        } else {
            "未运行".to_string()
        }
    );

    // 通过 HTTP 查询代理服务器实际状态（带超时，避免卡住）
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async {
        let url = format!("http://{}:{}/status", listen_address, listen_port);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_default();
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    let running = json
                        .get("running")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    println!("  代理服务: {}", if running { "运行中" } else { "已停止" });
                    if running {
                        let addr = json.get("address").and_then(|v| v.as_str()).unwrap_or("?");
                        let port = json.get("port").and_then(|v| v.as_u64()).unwrap_or(0);
                        let uptime = json
                            .get("uptimeSeconds")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        println!("  监听地址: {addr}:{port}");
                        println!("  运行时间: {}秒", uptime);
                    }
                }
            }
            _ => {
                println!("  代理服务: 未运行（无法连接到 {listen_address}:{listen_port}）");
            }
        }

        // 查看当前供应商
        println!("\n当前供应商:");
        let db = match Database::init() {
            Ok(db) => Arc::new(db),
            Err(e) => {
                eprintln!("错误: 数据库初始化失败: {e}");
                return;
            }
        };
        for app_type in &[
            "claude",
            "claude-desktop",
            "codex",
            "gemini",
            "opencode",
            "openclaw",
            "hermes",
        ] {
            let current = provider_manager::get_current_provider_id(&db, app_type)
                .ok()
                .flatten()
                .unwrap_or_else(|| "(无)".to_string());
            println!("  {app_type:<16}: {current}");
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

    let all_apps = [
        "claude",
        "claude-desktop",
        "codex",
        "gemini",
        "opencode",
        "openclaw",
        "hermes",
    ];
    let app_types: Vec<&str> = match app {
        Some(a) => {
            if let Err(e) = validated_app(a) {
                eprintln!("错误: {e}");
                std::process::exit(1);
            }
            vec![a]
        }
        None => all_apps.to_vec(),
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

        println!(
            "  {:<2} {:<20} {:<22} {:<18} {}",
            "", "ID", "名称", "API 格式", "Base URL"
        );
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
            let api_fmt = provider
                .meta
                .as_ref()
                .and_then(|m| m.api_format.as_deref())
                .unwrap_or("-");
            println!(
                "  {marker:<2} {id:<20} {:<22} {api_fmt:<18} {base_url}",
                provider.name,
            );
        }
    }
}

/// add-provider: 添加供应商
fn cmd_add_provider(
    app: &str,
    id: &str,
    name: &str,
    api_key: Option<&str>,
    base_url: Option<&str>,
    api_format: Option<&str>,
) {
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

    let meta = api_format.map(|fmt| {
        let mut meta = cc_switch_lib::ProviderMeta::default();
        meta.api_format = Some(fmt.to_string());
        meta
    });

    let provider = cc_switch_lib::Provider {
        id: id.to_string(),
        name: name.to_string(),
        settings_config,
        website_url: None,
        category: None,
        created_at: None,
        sort_index: None,
        notes: None,
        meta,
    };

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

/// stop: 停止代理服务器（通过 HTTP POST /stop 通知后台进程停止）
fn cmd_stop() {
    let listen_address =
        std::env::var("CC_SWITCH_LISTEN").unwrap_or_else(|_| "127.0.0.1".to_string());
    let listen_port: u16 = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090);

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");

    rt.block_on(async {
        let url = format!("http://{}:{}/stop", listen_address, listen_port);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        match client.post(&url).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    println!(
                        "已发送停止信号到代理服务器 {}:{}",
                        listen_address, listen_port
                    );
                    // 等待进程退出（最多5秒）
                    for _ in 0..10 {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        match read_pid_file() {
                            Some(pid) => {
                                if !is_process_alive(pid) {
                                    remove_pid_file();
                                    println!("代理服务器已停止");
                                    return;
                                }
                            }
                            None => {
                                // PID 文件已被删除，说明进程已清理退出
                                println!("代理服务器已停止");
                                return;
                            }
                        }
                    }
                    println!("停止信号已发送，进程可能仍在清理中，请稍后检查状态");
                } else {
                    eprintln!("停止代理服务器失败: HTTP {status}");
                    eprintln!("请确认代理服务器正在运行");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("停止代理服务器失败: {e}");
                eprintln!("请确认代理服务器正在运行");
                std::process::exit(1);
            }
        }
    });
}

/// settings: 查看或修改设备级设置（~/.cc-switch/settings.json）
fn cmd_settings(key: Option<&str>, value: Option<&str>) {
    match (key, value) {
        (None, None) => {
            // 列出所有设置
            let settings = cc_switch_lib::AppSettings::default();
            let current = cc_switch_lib::get_settings();
            let merged = merge_settings_for_display(&current, &settings);
            println!("当前设备级设置 (~/.cc-switch/settings.json):");
            println!(
                "{}",
                serde_json::to_string_pretty(&merged).unwrap_or_else(|_| "{}".to_string())
            );
        }
        (Some(k), None) => {
            // 查看指定设置
            let settings = cc_switch_lib::get_settings();
            let json = serde_json::to_value(&settings).unwrap_or_default();
            match json.get(&k) {
                Some(v) => println!(
                    "{} = {}",
                    k,
                    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
                ),
                None => {
                    eprintln!("设置项 '{}' 不存在", k);
                    std::process::exit(1);
                }
            }
        }
        (Some(k), Some(v)) => {
            // 修改设置
            let mut settings = cc_switch_lib::get_settings();
            let mut json = serde_json::to_value(&settings).unwrap_or_default();
            let parsed_value: serde_json::Value = serde_json::from_str(v)
                .unwrap_or_else(|_| serde_json::Value::String(v.to_string()));
            if let Some(obj) = json.as_object_mut() {
                obj.insert(k.to_string(), parsed_value);
            }
            if let Ok(updated) = serde_json::from_value::<cc_switch_lib::AppSettings>(json) {
                settings = updated;
            }
            match cc_switch_lib::update_settings(settings) {
                Ok(_) => println!("设置 '{}' 已更新", k),
                Err(e) => {
                    eprintln!("保存设置失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        (None, Some(_)) => {
            eprintln!("错误: 不能只指定 value 而不指定 key");
            std::process::exit(1);
        }
    }
}

/// config: 查看或修改数据库配置（settings 表）
fn cmd_config(key: Option<&str>, value: Option<&str>) {
    let db = match init_db() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    match (key, value) {
        (None, None) => {
            let settings = match db.get_all_settings() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("获取配置失败: {e}");
                    std::process::exit(1);
                }
            };
            println!("当前数据库配置 (settings 表):");
            println!(
                "{}",
                serde_json::to_string_pretty(&settings).unwrap_or_else(|_| "{}".to_string())
            );
        }
        (Some(k), None) => match db.get_setting(k) {
            Ok(Some(val)) => {
                let display = serde_json::from_str::<serde_json::Value>(&val)
                    .ok()
                    .and_then(|v| serde_json::to_string_pretty(&v).ok())
                    .unwrap_or_else(|| val.clone());
                println!("{} = {}", k, display);
            }
            Ok(None) => {
                eprintln!("配置项 '{}' 不存在", k);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("获取配置失败: {e}");
                std::process::exit(1);
            }
        },
        (Some(k), Some(v)) => match db.set_setting(k, v) {
            Ok(_) => println!("配置 '{}' 已更新", k),
            Err(e) => {
                eprintln!("保存配置失败: {e}");
                std::process::exit(1);
            }
        },
        (None, Some(_)) => {
            eprintln!("错误: 不能只指定 --value 而不指定 --key");
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 2: 代理管理命令
// ============================================================================

/// update-provider: 更新供应商配置
fn cmd_update_provider(
    app: &str,
    id: &str,
    name: Option<&str>,
    api_key: Option<&str>,
    base_url: Option<&str>,
    api_format: Option<&str>,
    clear_api_format: bool,
) {
    if let Err(e) = validated_app(app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let providers = match db.get_all_providers(app) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("获取供应商失败: {e}");
            std::process::exit(1);
        }
    };
    let provider = match providers.get(id) {
        Some(p) => p.clone(),
        None => {
            eprintln!("供应商 '{id}' 不存在");
            std::process::exit(1);
        }
    };

    let mut settings_config = provider.settings_config.clone();
    if let Some(env) = settings_config
        .get_mut("env")
        .and_then(|v| v.as_object_mut())
    {
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
    } else if api_key.is_some() || base_url.is_some() {
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
        settings_config["env"] = serde_json::Value::Object(env);
    }

    // 保留已有 meta，按需更新 api_format
    let mut meta = provider.meta.unwrap_or_default();
    if clear_api_format {
        meta.api_format = None;
    } else if let Some(fmt) = api_format {
        meta.api_format = Some(fmt.to_string());
    }

    let new_name = name.map(|s| s.to_string()).unwrap_or(provider.name);
    let updated = cc_switch_lib::Provider {
        id: id.to_string(),
        name: new_name,
        settings_config,
        website_url: provider.website_url,
        category: provider.category,
        created_at: provider.created_at,
        sort_index: provider.sort_index,
        notes: provider.notes,
        meta: Some(meta),
    };
    match db.save_provider(app, &updated) {
        Ok(_) => println!("供应商 '{id}' 已更新"),
        Err(e) => {
            eprintln!("更新供应商失败: {e}");
            std::process::exit(1);
        }
    }
}

/// takeover: 设置/查看代理接管状态
fn cmd_takeover(app: &str, enabled: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let proxy_service = cc_switch_lib::ProxyService::new(db);

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match enabled {
            None => {
                // 查看当前状态
                match proxy_service.get_takeover_status().await {
                    Ok(status) => {
                        println!("代理接管状态:");
                        println!(
                            "  claude: {}",
                            if status.claude {
                                "已接管"
                            } else {
                                "未接管"
                            }
                        );
                        println!(
                            "  codex:  {}",
                            if status.codex {
                                "已接管"
                            } else {
                                "未接管"
                            }
                        );
                        println!(
                            "  gemini: {}",
                            if status.gemini {
                                "已接管"
                            } else {
                                "未接管"
                            }
                        );
                    }
                    Err(e) => {
                        eprintln!("获取接管状态失败: {e}");
                        std::process::exit(1);
                    }
                }
            }
            Some(val) => {
                let enable = match val.to_lowercase().as_str() {
                    "on" | "true" | "1" => true,
                    "off" | "false" | "0" => false,
                    _ => {
                        eprintln!("无效的值: {val}，请使用 on/off");
                        std::process::exit(1);
                    }
                };
                match proxy_service.set_takeover_for_app(app, enable).await {
                    Ok(_) => println!("{} 接管已{}", app, if enable { "开启" } else { "关闭" }),
                    Err(e) => {
                        eprintln!("设置接管失败: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
    });
}

/// switch-proxy: 代理模式下热切换供应商
fn cmd_switch_proxy(app: &str, id: &str) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let proxy_service = cc_switch_lib::ProxyService::new(db);

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match proxy_service.hot_switch_provider(app, id).await {
            Ok(result) => println!(
                "已热切换 {} 到供应商 '{id}' (逻辑目标变更: {})",
                app, result.logical_target_changed
            ),
            Err(e) => {
                eprintln!("热切换失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

/// failover-queue: 查看/管理故障转移队列
fn cmd_failover_queue(action: &str, app: Option<&str>, id: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let app = match app {
        Some(a) => a,
        None => {
            eprintln!("请指定应用类型");
            std::process::exit(1);
        }
    };

    match action {
        "list" => match db.get_failover_queue(app) {
            Ok(queue) => {
                println!("故障转移队列 ({app}):");
                if queue.is_empty() {
                    println!("  (空)");
                } else {
                    for (i, item) in queue.iter().enumerate() {
                        println!(
                            "  {}. {} (sort_index: {:?})",
                            i + 1,
                            item.provider_id,
                            item.sort_index
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("获取队列失败: {e}");
                std::process::exit(1);
            }
        },
        "add" => {
            let id = match id {
                Some(i) => i,
                None => {
                    eprintln!("请指定供应商 ID");
                    std::process::exit(1);
                }
            };
            match db.add_to_failover_queue(app, id) {
                Ok(_) => println!("已添加 '{id}' 到 {app} 故障转移队列"),
                Err(e) => {
                    eprintln!("添加失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        "remove" => {
            let id = match id {
                Some(i) => i,
                None => {
                    eprintln!("请指定供应商 ID");
                    std::process::exit(1);
                }
            };
            match db.remove_from_failover_queue(app, id) {
                Ok(_) => println!("已从 {app} 队列移除 '{id}'"),
                Err(e) => {
                    eprintln!("移除失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("未知操作: {action}，支持: list, add, remove");
            std::process::exit(1);
        }
    }
}

/// auto-failover: 查看/设置自动故障转移
fn cmd_auto_failover(app: Option<&str>, enabled: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match enabled {
        None => {
            for app_type in ["claude", "codex", "gemini"] {
                let (_, auto_enabled) = db.get_proxy_flags_sync(app_type);
                println!(
                    "  {app_type}: {}",
                    if auto_enabled { "开启" } else { "关闭" }
                );
            }
        }
        Some(val) => {
            let app = match app {
                Some(a) => a,
                None => {
                    eprintln!("设置时请指定应用类型");
                    std::process::exit(1);
                }
            };
            let enable = match val.to_lowercase().as_str() {
                "on" | "true" | "1" => true,
                "off" | "false" | "0" => false,
                _ => {
                    eprintln!("无效的值: {val}");
                    std::process::exit(1);
                }
            };
            let (current_enabled, _) = db.get_proxy_flags_sync(app);
            match db.set_proxy_flags_sync(app, current_enabled, enable) {
                Ok(_) => println!(
                    "{} 自动故障转移已{}",
                    app,
                    if enable { "开启" } else { "关闭" }
                ),
                Err(e) => {
                    eprintln!("设置失败: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}

/// circuit-breaker: 查看/设置/重置熔断器
fn cmd_circuit_breaker(action: &str, app: Option<&str>, id: Option<&str>, config: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let proxy_service = cc_switch_lib::ProxyService::new(db.clone());

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match action {
            "get" => {
                let app = app.unwrap_or("claude");
                match db.get_circuit_breaker_config().await {
                    Ok(cfg) => println!(
                        "{} 熔断器配置:\n{}",
                        app,
                        serde_json::to_string_pretty(&cfg).unwrap_or_default()
                    ),
                    Err(e) => {
                        eprintln!("获取失败: {e}");
                        std::process::exit(1);
                    }
                }
            }
            "set" => {
                let app = match app {
                    Some(a) => a,
                    None => {
                        eprintln!("请指定应用类型");
                        std::process::exit(1);
                    }
                };
                let config_json = match config {
                    Some(c) => c,
                    None => {
                        eprintln!("请用 --config 指定配置 JSON");
                        std::process::exit(1);
                    }
                };
                let cfg: cc_switch_lib::CircuitBreakerConfig =
                    match serde_json::from_str(config_json) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("解析配置失败: {e}");
                            std::process::exit(1);
                        }
                    };
                match proxy_service
                    .update_circuit_breaker_config_for_app(app, cfg)
                    .await
                {
                    Ok(_) => println!("{} 熔断器配置已更新", app),
                    Err(e) => {
                        eprintln!("设置失败: {e}");
                        std::process::exit(1);
                    }
                }
            }
            "reset" => {
                let app = match app {
                    Some(a) => a,
                    None => {
                        eprintln!("请指定应用类型");
                        std::process::exit(1);
                    }
                };
                let id = match id {
                    Some(i) => i,
                    None => {
                        eprintln!("请指定供应商 ID");
                        std::process::exit(1);
                    }
                };
                let _ = proxy_service.reset_provider_circuit_breaker(id, app).await;
                println!("已重置 {app}/{id} 的熔断器");
            }
            _ => {
                eprintln!("未知操作: {action}，支持: get, set, reset");
                std::process::exit(1);
            }
        }
    });
}

/// rectifier: 查看/设置请求修正器配置
fn cmd_rectifier(action: &str, config: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match action {
        "get" => match db.get_rectifier_config() {
            Ok(cfg) => println!("{}", serde_json::to_string_pretty(&cfg).unwrap_or_default()),
            Err(e) => {
                eprintln!("获取失败: {e}");
                std::process::exit(1);
            }
        },
        "set" => {
            let config_json = match config {
                Some(c) => c,
                None => {
                    eprintln!("请用 --config 指定配置 JSON");
                    std::process::exit(1);
                }
            };
            match db.set_setting("rectifier_config", config_json) {
                Ok(_) => println!("修正器配置已更新"),
                Err(e) => {
                    eprintln!("设置失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("未知操作: {action}，支持: get, set");
            std::process::exit(1);
        }
    }
}

/// optimizer: 查看/设置优化器配置
fn cmd_optimizer(action: &str, config: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match action {
        "get" => match db.get_optimizer_config() {
            Ok(cfg) => println!("{}", serde_json::to_string_pretty(&cfg).unwrap_or_default()),
            Err(e) => {
                eprintln!("获取失败: {e}");
                std::process::exit(1);
            }
        },
        "set" => {
            let config_json = match config {
                Some(c) => c,
                None => {
                    eprintln!("请用 --config 指定配置 JSON");
                    std::process::exit(1);
                }
            };
            match db.set_setting("optimizer_config", config_json) {
                Ok(_) => println!("优化器配置已更新"),
                Err(e) => {
                    eprintln!("设置失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("未知操作: {action}，支持: get, set");
            std::process::exit(1);
        }
    }
}

/// global-proxy: 查看/设置全局出站代理
fn cmd_global_proxy(action: &str, url: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match action {
        "get" => match db.get_global_proxy_url() {
            Ok(Some(url)) => println!("全局出站代理: {url}"),
            Ok(None) => println!("全局出站代理: 未设置"),
            Err(e) => {
                eprintln!("获取失败: {e}");
                std::process::exit(1);
            }
        },
        "set" => {
            let url = match url {
                Some(u) => u,
                None => {
                    eprintln!("请指定代理 URL");
                    std::process::exit(1);
                }
            };
            match db.set_global_proxy_url(Some(url)) {
                Ok(_) => {
                    if let Err(e) = cc_switch_lib::http_client::init(Some(url)) {
                        eprintln!("HTTP 客户端初始化失败: {e}");
                    }
                    println!("全局出站代理已设置为: {url}");
                }
                Err(e) => {
                    eprintln!("设置失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        "clear" => match db.set_global_proxy_url(None) {
            Ok(_) => {
                let _ = cc_switch_lib::http_client::init(None);
                println!("全局出站代理已清除");
            }
            Err(e) => {
                eprintln!("清除失败: {e}");
                std::process::exit(1);
            }
        },
        "test" => {
            let url = match url
                .map(|s| s.to_string())
                .or_else(|| db.get_global_proxy_url().ok().flatten())
            {
                Some(u) => u,
                None => {
                    eprintln!("未设置代理 URL");
                    std::process::exit(1);
                }
            };
            let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
            rt.block_on(async {
                let start = std::time::Instant::now();
                let client = reqwest::Client::builder()
                    .proxy(
                        reqwest::Proxy::all(&url)
                            .unwrap_or_else(|_| reqwest::Proxy::all("http://127.0.0.1").unwrap()),
                    )
                    .timeout(std::time::Duration::from_secs(10))
                    .build();
                match client {
                    Ok(c) => match c.get("https://httpbin.org/get").send().await {
                        Ok(resp) => {
                            let elapsed = start.elapsed();
                            println!(
                                "代理连接测试成功: {url} (HTTP {}, {}ms)",
                                resp.status(),
                                elapsed.as_millis()
                            );
                        }
                        Err(e) => {
                            eprintln!("代理连接测试失败: {e}");
                            std::process::exit(1);
                        }
                    },
                    Err(e) => {
                        eprintln!("构建代理客户端失败: {e}");
                        std::process::exit(1);
                    }
                }
            });
        }
        _ => {
            eprintln!("未知操作: {action}，支持: get, set, clear, test");
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Phase 3: 高级功能命令
// ============================================================================

/// list-mcp: 列出 MCP 服务器
fn cmd_list_mcp() {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match db.get_all_mcp_servers() {
        Ok(servers) => {
            if servers.is_empty() {
                println!("(无 MCP 服务器)");
                return;
            }
            println!("{:<3} {:<25} {:<20}", "#", "名称", "ID");
            for (i, (id, server)) in servers.iter().enumerate() {
                println!("{:<3} {:<25} {:<20}", i + 1, server.name, id);
            }
        }
        Err(e) => {
            eprintln!("获取 MCP 列表失败: {e}");
            std::process::exit(1);
        }
    }
}

/// list-prompts: 列出 Prompts
fn cmd_list_prompts(app: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let apps: Vec<&str> = match app {
        Some(a) => vec![a],
        None => vec![
            "claude", "codex", "gemini", "opencode", "openclaw", "hermes",
        ],
    };
    for app_type in &apps {
        match db.get_prompts(app_type) {
            Ok(prompts) => {
                if prompts.is_empty() {
                    continue;
                }
                println!("\n── {app_type} ──");
                for (id, prompt) in &prompts {
                    println!(
                        "  {} {} {}",
                        if prompt.enabled { "*" } else { " " },
                        id,
                        prompt.name
                    );
                }
            }
            Err(e) => log::debug!("获取 {app_type} prompts 失败: {e}"),
        }
    }
}

/// export-config: 导出配置
fn cmd_export_config(path: &str) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match db.export_sql(std::path::Path::new(path)) {
        Ok(_) => println!("配置已导出到 {path}"),
        Err(e) => {
            eprintln!("导出失败: {e}");
            std::process::exit(1);
        }
    }
}

/// import-config: 导入配置
fn cmd_import_config(path: &str) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match db.import_sql(std::path::Path::new(path)) {
        Ok(msg) => println!("配置导入成功: {msg}"),
        Err(e) => {
            eprintln!("导入失败: {e}");
            std::process::exit(1);
        }
    }
}

/// backup-create: 创建数据库备份（导出 SQL 到备份目录）
fn cmd_backup_create() {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let backup_dir = cc_switch_lib::get_app_config_dir().join("backups");
    let _ = std::fs::create_dir_all(&backup_dir);
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("cc-switch-backup-{timestamp}.sql"));
    match db.export_sql(&backup_path) {
        Ok(_) => println!("备份已创建: {}", backup_path.display()),
        Err(e) => {
            eprintln!("创建备份失败: {e}");
            std::process::exit(1);
        }
    }
}

/// backup-list: 列出备份
fn cmd_backup_list() {
    match cc_switch_lib::Database::list_backups() {
        Ok(backups) => {
            if backups.is_empty() {
                println!("(无备份)");
                return;
            }
            println!("{:<40} {:<12} {}", "文件名", "大小", "创建时间");
            for b in &backups {
                println!(
                    "{:<40} {:<12} {}",
                    b.filename,
                    format!("{} KB", b.size_bytes / 1024),
                    b.created_at
                );
            }
        }
        Err(e) => {
            eprintln!("列出备份失败: {e}");
            std::process::exit(1);
        }
    }
}

/// backup-restore: 从备份恢复
fn cmd_backup_restore(name: &str) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match db.restore_from_backup(name) {
        Ok(msg) => println!("已从备份 '{name}' 恢复: {msg}"),
        Err(e) => {
            eprintln!("恢复失败: {e}");
            std::process::exit(1);
        }
    }
}

/// usage-summary: 查看用量统计摘要（基于实际请求日志）
fn cmd_usage_summary(days: u32) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let end_date = chrono::Utc::now().timestamp();
    let start_date = end_date - (days as i64) * 24 * 3600;

    match db.get_usage_summary(Some(start_date), Some(end_date), None, None, None) {
        Ok(summary) => {
            println!("用量统计 (最近 {days} 天):");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  总请求数:       {}", summary.total_requests);
            println!(
                "  总成本:         ${}",
                summary.total_cost
            );
            println!(
                "  输入 Token:     {}",
                format_tokens(summary.total_input_tokens)
            );
            println!(
                "  输出 Token:     {}",
                format_tokens(summary.total_output_tokens)
            );
            println!(
                "  缓存写入 Token: {}",
                format_tokens(summary.total_cache_creation_tokens)
            );
            println!(
                "  缓存读取 Token: {}",
                format_tokens(summary.total_cache_read_tokens)
            );
            println!(
                "  实际总 Token:   {}",
                format_tokens(summary.real_total_tokens)
            );
            println!(
                "  成功率:         {:.1}%",
                summary.success_rate * 100.0
            );
            println!(
                "  缓存命中率:     {:.1}%",
                summary.cache_hit_rate * 100.0
            );

            if summary.total_requests == 0 {
                println!("\n提示：暂无请求记录。请确保代理服务器正在运行以记录用量数据。");
            }
        }
        Err(e) => {
            eprintln!("查询用量统计失败: {e}");
            std::process::exit(1);
        }
    }
}

/// speedtest: 测试 API 端点延迟
fn cmd_speedtest(url: &str, timeout: u64) {
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async {
        match cc_switch_lib::SpeedtestService::test_endpoints(vec![url.to_string()], Some(timeout))
            .await
        {
            Ok(results) => {
                for r in &results {
                    match &r.latency {
                        Some(ms) => {
                            println!("{} 延迟: {}ms (HTTP {})", r.url, ms, r.status.unwrap_or(0))
                        }
                        None => println!(
                            "{} 失败: {}",
                            r.url,
                            r.error.as_deref().unwrap_or("未知错误")
                        ),
                    }
                }
            }
            Err(e) => {
                eprintln!("测速失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

/// verify-key: 验证 API Key
fn cmd_verify_key(base_url: &str, api_key: &str) {
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async {
        let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
        let start = std::time::Instant::now();
        match reqwest::Client::new()
            .get(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("x-api-key", api_key)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
        {
            Ok(resp) => {
                let elapsed = start.elapsed();
                let status = resp.status();
                if status.is_success() {
                    println!(
                        "API Key 验证成功: {base_url} (HTTP {}, {}ms)",
                        status,
                        elapsed.as_millis()
                    );
                } else {
                    println!("API Key 验证失败: {base_url} (HTTP {})", status);
                }
            }
            Err(e) => {
                eprintln!("验证失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

// ============================================================================
// Phase 4: 声明式配置文件
// ============================================================================

/// validate: 校验声明式配置文件
fn cmd_validate(path: &str) {
    match cc_switch_lib::core::decl_config::DeclConfig::from_yaml_file(path) {
        Ok(config) => {
            if let Err(e) = config.validate() {
                eprintln!("配置校验失败: {e}");
                std::process::exit(1);
            }
            println!("✓ 配置文件校验通过");
            println!("  供应商数量: {}", config.providers.len());
            println!("  故障转移队列: {} 个应用", config.failover.queue.len());
            println!(
                "  全局代理: {}",
                if config.global_proxy.is_some() {
                    "已配置"
                } else {
                    "未配置"
                }
            );
            println!("  代理接管: {} 个应用", config.proxy.takeover.len());
        }
        Err(e) => {
            eprintln!("加载配置文件失败: {e}");
            std::process::exit(1);
        }
    }
}

/// apply-config: 应用声明式配置文件到数据库
fn cmd_apply_config(path: &str) {
    let config = match cc_switch_lib::core::decl_config::DeclConfig::from_yaml_file(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("加载配置文件失败: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = config.validate() {
        eprintln!("配置校验失败: {e}");
        std::process::exit(1);
    }
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match config.apply(&db) {
        Ok(summary) => {
            println!("✓ 配置已应用:");
            println!("{summary}");
        }
        Err(e) => {
            eprintln!("应用配置失败: {e}");
            std::process::exit(1);
        }
    }
}

/// help: 列出所有命令
fn cmd_help() {
    println!("cc-switch-cli — 无头模式管理工具");
    println!();
    println!("用法: cc-switch-cli [OPTIONS] <COMMAND>");
    println!();
    println!("选项:");
    println!("  --log-level <LEVEL>  日志级别 (error/warn/info/debug/trace，默认 info)");
    println!();
    println!("命令:");
    println!();
    println!("  代理管理:");
    println!("    start               启动代理服务器（前台运行）");
    println!("    daemon              以守护进程方式后台启动代理");
    println!("    stop                停止后台代理服务器");
    println!("    status              查看代理服务器状态");
    println!();
    println!("  供应商管理:");
    println!("    list-providers [APP]        列出供应商");
    println!("    add-provider <APP> <ID> <NAME> [--api-key K] [--base-url U] [--api-format F]");
    println!("    update-provider <APP> <ID> [--name N] [--api-key K] [--base-url U] [--api-format F] [--clear-api-format]");
    println!("    remove-provider <APP> <ID>  删除供应商");
    println!("    switch-provider <APP> <ID>  切换当前供应商");
    println!();
    println!("  代理配置:");
    println!("    takeover <APP> [on|off]     查看/设置代理接管");
    println!("    switch-proxy <APP> <ID>     代理模式下热切换");
    println!("    failover-queue <list|add|remove> <APP> [ID]");
    println!("    auto-failover [APP] [on|off]");
    println!("    circuit-breaker <get|set|reset> [APP] [--config JSON] [ID]");
    println!("    rectifier <get|set> [--config JSON]");
    println!("    optimizer <get|set> [--config JSON]");
    println!("    global-proxy <get|set|clear|test> [URL]");
    println!();
    println!("  配置与设置:");
    println!("    settings [KEY] [VALUE]      设备级设置 (settings.json)");
    println!("    config [--key K] [--value V]  数据库配置 (settings 表)");
    println!("    export-config <PATH>        导出配置到 SQL 文件");
    println!("    import-config <PATH>        从 SQL 文件导入配置");
    println!("    validate <PATH>             校验声明式 YAML 配置");
    println!("    apply-config <PATH>         应用声明式 YAML 配置");
    println!();
    println!("  备份与恢复:");
    println!("    backup-create               创建数据库备份");
    println!("    backup-list                 列出备份");
    println!("    backup-restore <NAME>       从备份恢复");
    println!();
    println!("  其他:");
    println!("    list-mcp                    列出 MCP 服务器");
    println!("    list-prompts [APP]          列出 Prompts");
    println!("    usage-summary [--days N]    查看用量统计");
    println!("    speedtest <URL> [--timeout S]");
    println!("    verify-key --base-url U --api-key K");
    println!("    help                        显示此帮助信息");
    println!();
    println!("环境变量:");
    println!("  CC_SWITCH_LISTEN    代理监听地址 (默认 127.0.0.1)");
    println!("  CC_SWITCH_PORT      代理监听端口 (默认 9090)");
    println!();
    println!("更多信息: https://github.com/farion1231/cc-switch");
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 初始化数据库连接
fn init_db() -> Result<Arc<Database>, String> {
    Database::init()
        .map(Arc::new)
        .map_err(|e| format!("数据库初始化失败: {e}"))
}

/// 检查应用类型是否合法
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

/// 合并设置用于显示（将当前值与默认值合并）
fn merge_settings_for_display(
    current: &cc_switch_lib::AppSettings,
    _default: &cc_switch_lib::AppSettings,
) -> cc_switch_lib::AppSettings {
    current.clone()
}

/// 格式化 Token 数量为可读字符串（K/M/B）
fn format_tokens(count: u64) -> String {
    if count >= 1_000_000_000 {
        format!("{:.2}B", count as f64 / 1_000_000_000.0)
    } else if count >= 1_000_000 {
        format!("{:.2}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1f}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

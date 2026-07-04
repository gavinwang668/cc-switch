//! cc-switch CLI — 无头模式管理工具
//!
//! 提供命令行界面管理代理服务器和供应商配置，
//! 适用于 Linux 无头环境或需要脚本化管理的场景。

use std::str::FromStr;
use std::sync::Arc;

use clap::{Parser, Subcommand};

use cc_switch_core::core::{bootstrap, provider_manager};
use cc_switch_core::Database;

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
        /// API 格式（按应用支持情况选择）
        /// - claude: anthropic / openai_chat / openai_responses
        /// - claude-desktop: anthropic / openai_chat / openai_responses / gemini_native / bedrock
        /// - codex: openai_responses / openai_chat
        /// - gemini: gemini_native / openai_chat / openai_responses / anthropic
        /// - opencode: openai_chat / openai_responses
        /// - openclaw: openai_chat / openai_responses / anthropic
        /// - hermes: openai_chat / openai_responses / anthropic
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
        /// API 格式（按应用支持情况选择）
        /// - claude: anthropic / openai_chat / openai_responses
        /// - claude-desktop: anthropic / openai_chat / openai_responses / gemini_native / bedrock
        /// - codex: openai_responses / openai_chat
        /// - gemini: gemini_native / openai_chat / openai_responses / anthropic
        /// - opencode: openai_chat / openai_responses
        /// - openclaw: openai_chat / openai_responses / anthropic
        /// - hermes: openai_chat / openai_responses / anthropic
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
    /// 调整供应商排序
    SortProviders {
        /// 应用类型
        app: String,
        /// 排序项 JSON 数组，格式: [{"id":"provider_id","sortIndex":0},...]
        #[arg(long)]
        order: String,
    },
    /// 从 Live 配置文件导入供应商
    ImportLive {
        /// 应用类型 (claude, codex, gemini, opencode, openclaw, hermes)
        app: String,
    },
    /// 读取 Live 配置文件内容
    ReadLive {
        /// 应用类型
        app: String,
    },
    /// 获取供应商支持的模型列表
    FetchModels {
        /// Base URL
        #[arg(long)]
        base_url: String,
        /// API Key
        #[arg(long)]
        api_key: String,
        /// 是否为完整 URL（默认 false，自动拼接 /v1/models）
        #[arg(long)]
        full_url: bool,
        /// 自定义 models 路径
        #[arg(long)]
        models_path: Option<String>,
    },
    /// 将数据库供应商同步到 Live 配置文件
    SyncLive,
    /// 查看/设置代理配置
    ProxyConfig {
        /// get / set
        action: String,
        /// 配置 JSON（set 时需要）
        #[arg(long)]
        config: Option<String>,
    },
    /// 查看/设置全局代理配置
    GlobalProxyConfig {
        /// get / set
        action: String,
        /// 配置 JSON（set 时需要）
        #[arg(long)]
        config: Option<String>,
    },
    /// 查看/设置应用级代理配置
    AppProxyConfig {
        /// get / set
        action: String,
        /// 应用类型
        app: String,
        /// 配置 JSON（set 时需要）
        #[arg(long)]
        config: Option<String>,
    },
    /// 查看/设置默认成本倍率
    CostMultiplier {
        /// get / set
        action: String,
        /// 应用类型
        app: String,
        /// 倍率值（set 时需要，如 "1.0"、"0.5"）
        #[arg(long)]
        value: Option<String>,
    },
    /// 查看/设置计费模型来源
    PricingSource {
        /// get / set
        action: String,
        /// 应用类型
        app: String,
        /// 来源值（set 时需要，如 "official"、"custom"）
        #[arg(long)]
        value: Option<String>,
    },
    /// 检测 Live 配置是否被代理接管
    TakeoverStatus,
    /// 查看熔断器运行统计
    CircuitBreakerStats {
        /// 应用类型
        app: String,
        /// 供应商 ID
        id: String,
    },
    /// 查看供应商健康状态
    ProviderHealth {
        /// 应用类型
        app: String,
        /// 供应商 ID
        id: String,
    },
    /// 列出可加入故障转移队列的供应商
    FailoverAvailable {
        /// 应用类型
        app: String,
    },
    /// 查看/设置通用配置片段
    ConfigSnippet {
        /// get / set / extract
        action: String,
        /// 应用类型
        app: String,
        /// 配置片段 JSON（set 时需要）
        #[arg(long)]
        snippet: Option<String>,
    },
    /// 按应用查看用量统计
    UsageByApp {
        /// 天数（默认7天）
        #[arg(long, default_value = "7")]
        days: u32,
    },
    /// 查看请求日志
    RequestLogs {
        /// 页码（从1开始）
        #[arg(long, default_value = "1")]
        page: u32,
        /// 每页条数
        #[arg(long, default_value = "20")]
        page_size: u32,
        /// 按应用过滤
        #[arg(long)]
        app: Option<String>,
        /// 按供应商名称过滤
        #[arg(long)]
        provider: Option<String>,
        /// 按模型过滤
        #[arg(long)]
        model: Option<String>,
        /// 按状态码过滤
        #[arg(long)]
        status: Option<u16>,
    },
    /// 查看请求详情
    RequestDetail {
        /// 请求 ID
        request_id: String,
    },
    /// 检查供应商用量限额
    CheckLimits {
        /// 应用类型
        app: String,
        /// 供应商 ID
        id: String,
    },
    /// 删除数据库备份
    BackupDelete {
        /// 备份文件名
        name: String,
    },
    /// 重命名数据库备份
    BackupRename {
        /// 原文件名
        old_name: String,
        /// 新名称
        new_name: String,
    },
    /// 管理自定义测速端点
    Endpoint {
        /// list / add / remove
        action: String,
        /// 应用类型
        app: Option<String>,
        /// 供应商 ID
        id: Option<String>,
        /// 端点 URL（add 时需要）
        #[arg(long)]
        url: Option<String>,
    },
    /// 添加/更新 MCP 服务器
    AddMcp {
        /// MCP 服务器 ID
        id: String,
        /// 显示名称
        name: String,
        /// 命令（如 npx、node）
        #[arg(long)]
        command: String,
        /// 参数（JSON 数组）
        #[arg(long)]
        args: Option<String>,
        /// 环境变量（JSON 对象）
        #[arg(long)]
        env: Option<String>,
    },
    /// 删除 MCP 服务器
    RemoveMcp {
        /// MCP 服务器 ID
        id: String,
    },
    /// 启用/禁用 MCP 服务器
    ToggleMcp {
        /// MCP 服务器 ID
        id: String,
        /// 应用类型
        app: String,
        /// on / off
        enabled: String,
    },
    /// 测试 MCP 连接
    TestMcp {
        /// MCP 服务器 ID
        id: String,
    },
    /// 添加/更新 Prompt
    AddPrompt {
        /// 应用类型
        app: String,
        /// Prompt ID
        id: String,
        /// 显示名称
        name: String,
        /// 内容（或使用 --file 从文件读取）
        #[arg(long)]
        content: Option<String>,
        /// 从文件读取内容
        #[arg(long)]
        file: Option<String>,
    },
    /// 删除 Prompt
    RemovePrompt {
        /// 应用类型
        app: String,
        /// Prompt ID
        id: String,
    },
    /// 启用/禁用 Prompt
    EnablePrompt {
        /// 应用类型
        app: String,
        /// Prompt ID
        id: String,
        /// on / off
        enabled: String,
    },
    /// 列出已安装 Skills
    ListSkills {
        /// 应用类型（不指定则列出全部）
        app: Option<String>,
    },
    /// 卸载 Skill
    RemoveSkill {
        /// Skill ID
        id: String,
        /// 应用类型
        app: Option<String>,
    },
    /// 启用/禁用 Skill
    ToggleSkill {
        /// Skill ID
        id: String,
        /// 应用类型
        app: String,
        /// on / off
        enabled: String,
    },
    /// 检查环境变量冲突
    CheckEnv,
    /// 列出会话
    ListSessions {
        /// 应用类型
        app: Option<String>,
        /// 限制数量
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// 查看用量趋势
    UsageTrends {
        /// 天数
        #[arg(long, default_value = "7")]
        days: u32,
    },
    /// 查看供应商统计
    ProviderStats {
        /// 天数
        #[arg(long, default_value = "7")]
        days: u32,
    },
    /// 查看模型统计
    ModelStats {
        /// 天数
        #[arg(long, default_value = "7")]
        days: u32,
    },
    /// 代理热重载
    Reload,
    /// 设置/清除代理访问令牌
    AuthToken {
        /// set 或 clear
        action: Option<String>,
        /// 令牌值（set 时需要）
        token: Option<String>,
    },
    /// 管理 IP 白名单
    Acl {
        /// list / add / remove
        action: Option<String>,
        /// CIDR 地址
        #[arg(long)]
        cidr: Option<String>,
    },
    /// 协议转换烟雾测试
    SmokeTest {
        /// 应用类型（不指定则全部测试）
        app: Option<String>,
    },
    /// 将配置导出为 YAML
    ExportYaml {
        /// 输出文件路径
        path: String,
    },
    /// 对比 YAML 与当前配置
    Diff {
        /// YAML 文件路径
        path: String,
    },
    /// 回滚到上一个 apply 前的备份
    Rollback,
    /// 启用/禁用供应商
    ToggleProvider {
        /// 应用类型
        app: String,
        /// 供应商 ID
        id: String,
        /// on 或 off
        enabled: String,
    },
    /// 预览协议转换
    PreviewConversion {
        /// 源格式
        #[arg(long)]
        from: String,
        /// 目标格式
        #[arg(long)]
        to: String,
        /// 请求体 JSON
        #[arg(long)]
        payload: String,
        /// Base URL
        #[arg(long)]
        base_url: Option<String>,
    },
    /// 请求代理链路跟踪
    ProxyTrace {
        /// 应用类型
        app: String,
        /// 模型名
        #[arg(long)]
        model: String,
    },
    /// 重放历史请求
    ReplayRequest {
        /// 请求 ID
        request_id: String,
    },
    /// 列出所有可用命令
    Help,
}

// ============================================================================
// 入口
// ============================================================================

fn main() {
    // 安装 rustls 默认 CryptoProvider（与 GUI lib.rs setup 一致，选 ring）
    let _ = rustls::crypto::ring::default_provider().install_default();

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
            cmd_add_provider(
                app,
                id,
                name,
                api_key.as_deref(),
                base_url.as_deref(),
                api_format.as_deref(),
            );
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
                *clear_api_format,
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
        Commands::SortProviders { app, order } => cmd_sort_providers(app.clone(), order.clone()),
        Commands::ImportLive { app } => cmd_import_live(app.clone()),
        Commands::ReadLive { app } => cmd_read_live(app.clone()),
        Commands::FetchModels {
            base_url,
            api_key,
            full_url,
            models_path,
        } => cmd_fetch_models(base_url, api_key, *full_url, models_path.as_deref()),
        Commands::SyncLive => cmd_sync_live(),
        Commands::ProxyConfig { action, config } => cmd_proxy_config(&action, config.as_deref()),
        Commands::GlobalProxyConfig { action, config } => {
            cmd_global_proxy_config(&action, config.as_deref())
        }
        Commands::AppProxyConfig {
            action,
            app,
            config,
        } => cmd_app_proxy_config(&action, app, config.as_deref()),
        Commands::CostMultiplier { action, app, value } => {
            cmd_cost_multiplier(&action, app, value.as_deref())
        }
        Commands::PricingSource { action, app, value } => {
            cmd_pricing_source(&action, app, value.as_deref())
        }
        Commands::TakeoverStatus => cmd_takeover_status(),
        Commands::CircuitBreakerStats { app, id } => {
            cmd_circuit_breaker_stats(app.clone(), id.clone())
        }
        Commands::ProviderHealth { app, id } => cmd_provider_health(app.clone(), id.clone()),
        Commands::FailoverAvailable { app } => cmd_failover_available(app.clone()),
        Commands::ConfigSnippet {
            action,
            app,
            snippet,
        } => cmd_config_snippet(&action, app.clone(), snippet.as_deref()),
        Commands::UsageByApp { days } => cmd_usage_by_app(*days),
        Commands::RequestLogs {
            page,
            page_size,
            app,
            provider,
            model,
            status,
        } => cmd_request_logs(
            *page,
            *page_size,
            app.as_deref(),
            provider.as_deref(),
            model.as_deref(),
            *status,
        ),
        Commands::RequestDetail { request_id } => cmd_request_detail(request_id.clone()),
        Commands::CheckLimits { app, id } => cmd_check_limits(app.clone(), id.clone()),
        Commands::BackupDelete { name } => cmd_backup_delete(name.clone()),
        Commands::BackupRename { old_name, new_name } => {
            cmd_backup_rename(old_name.clone(), new_name.clone())
        }
        Commands::Endpoint {
            action,
            app,
            id,
            url,
        } => cmd_endpoint(&action, app.as_deref(), id.as_deref(), url.as_deref()),
        Commands::AddMcp {
            id,
            name,
            command,
            args,
            env,
        } => cmd_add_mcp(
            id.clone(),
            name.clone(),
            command.clone(),
            args.as_deref(),
            env.as_deref(),
        ),
        Commands::RemoveMcp { id } => cmd_remove_mcp(id.clone()),
        Commands::ToggleMcp { id, app, enabled } => {
            cmd_toggle_mcp(id.clone(), app.clone(), enabled.clone())
        }
        Commands::TestMcp { id } => cmd_test_mcp(id.clone()),
        Commands::AddPrompt {
            app,
            id,
            name,
            content,
            file,
        } => cmd_add_prompt(
            app.clone(),
            id.clone(),
            name.clone(),
            content.as_deref(),
            file.as_deref(),
        ),
        Commands::RemovePrompt { app, id } => cmd_remove_prompt(app.clone(), id.clone()),
        Commands::EnablePrompt { app, id, enabled } => {
            cmd_enable_prompt(app.clone(), id.clone(), enabled.clone())
        }
        Commands::ListSkills { app } => cmd_list_skills(app.as_deref()),
        Commands::RemoveSkill { id, app } => cmd_remove_skill(id.clone(), app.as_deref()),
        Commands::ToggleSkill { id, app, enabled } => {
            cmd_toggle_skill(id.clone(), app.clone(), enabled.clone())
        }
        Commands::CheckEnv => cmd_check_env(),
        Commands::ListSessions { app, limit } => cmd_list_sessions(app.as_deref(), *limit),
        Commands::UsageTrends { days } => cmd_usage_trends(*days),
        Commands::ProviderStats { days } => cmd_provider_stats(*days),
        Commands::ModelStats { days } => cmd_model_stats(*days),
        Commands::Reload => cmd_reload(),
        Commands::AuthToken { action, token } => cmd_auth_token(action.as_deref(), token.as_deref()),
        Commands::Acl { action, cidr } => cmd_acl(action.as_deref(), cidr.as_deref()),
        Commands::SmokeTest { app } => cmd_smoke_test(app.as_deref()),
        Commands::ExportYaml { path } => cmd_export_yaml(path),
        Commands::Diff { path } => cmd_diff(path),
        Commands::Rollback => cmd_rollback(),
        Commands::ToggleProvider { app, id, enabled } => cmd_toggle_provider(app, id, enabled),
        Commands::PreviewConversion { from, to, payload, base_url } => {
            cmd_preview_conversion(from, to, payload, base_url.as_deref())
        }
        Commands::ProxyTrace { app, model } => cmd_proxy_trace(app, model),
        Commands::ReplayRequest { request_id } => cmd_replay_request(request_id),
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
    cc_switch_core::get_app_config_dir()
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
async fn setup_signal_handlers(proxy_service: &cc_switch_core::ProxyService, is_daemon: bool) {
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
                    match cc_switch_core::reload_settings() {
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
                .or_else(|| provider.settings_config.pointer("/env/OPENAI_BASE_URL"))
                .or_else(|| provider.settings_config.pointer("/env/GEMINI_BASE_URL"))
                .or_else(|| provider.settings_config.pointer("/env/OPENCLAW_BASE_URL"))
                .or_else(|| provider.settings_config.pointer("/env/HERMES_BASE_URL"))
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

    // 按 app 类型选择正确的 env 字段名
    let (key_field, url_field) = match app {
        "claude" | "claude-desktop" => ("ANTHROPIC_API_KEY", "ANTHROPIC_BASE_URL"),
        "codex" | "opencode" => ("OPENAI_API_KEY", "OPENAI_BASE_URL"),
        "gemini" => ("GEMINI_API_KEY", "GEMINI_BASE_URL"),
        "openclaw" => ("OPENCLAW_API_KEY", "OPENCLAW_BASE_URL"),
        "hermes" => ("HERMES_API_KEY", "HERMES_BASE_URL"),
        _ => {
            eprintln!("错误: 不支持的应用类型: {app}");
            std::process::exit(1);
        }
    };

    let mut env = serde_json::Map::new();
    if let Some(key) = api_key {
        env.insert(
            key_field.to_string(),
            serde_json::Value::String(key.to_string()),
        );
    }
    if let Some(url) = base_url {
        env.insert(
            url_field.to_string(),
            serde_json::Value::String(url.to_string()),
        );
    }

    let settings_config = serde_json::json!({
        "env": env,
    });

    let meta = api_format.map(|fmt| {
        let mut meta = cc_switch_core::ProviderMeta::default();
        meta.api_format = Some(fmt.to_string());
        meta
    });

    let provider = cc_switch_core::Provider {
        id: id.to_string(),
        name: name.to_string(),
        settings_config,
        website_url: None,
        category: None,
        created_at: None,
        sort_index: None,
        notes: None,
        meta,
        icon: None,
        icon_color: None,
        in_failover_queue: false,
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
            let settings = cc_switch_core::AppSettings::default();
            let current = cc_switch_core::get_settings();
            let merged = merge_settings_for_display(&current, &settings);
            println!("当前设备级设置 (~/.cc-switch/settings.json):");
            println!(
                "{}",
                serde_json::to_string_pretty(&merged).unwrap_or_else(|_| "{}".to_string())
            );
        }
        (Some(k), None) => {
            // 查看指定设置
            let settings = cc_switch_core::get_settings();
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
            let mut settings = cc_switch_core::get_settings();
            let mut json = serde_json::to_value(&settings).unwrap_or_default();
            let parsed_value: serde_json::Value = serde_json::from_str(v)
                .unwrap_or_else(|_| serde_json::Value::String(v.to_string()));
            if let Some(obj) = json.as_object_mut() {
                obj.insert(k.to_string(), parsed_value);
            }
            if let Ok(updated) = serde_json::from_value::<cc_switch_core::AppSettings>(json) {
                settings = updated;
            }
            match cc_switch_core::update_settings(settings) {
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

    // 按 app 类型选择正确的 env 字段名
    let (key_field, url_field) = match app {
        "claude" | "claude-desktop" => ("ANTHROPIC_API_KEY", "ANTHROPIC_BASE_URL"),
        "codex" | "opencode" => ("OPENAI_API_KEY", "OPENAI_BASE_URL"),
        "gemini" => ("GEMINI_API_KEY", "GEMINI_BASE_URL"),
        "openclaw" => ("OPENCLAW_API_KEY", "OPENCLAW_BASE_URL"),
        "hermes" => ("HERMES_API_KEY", "HERMES_BASE_URL"),
        _ => {
            eprintln!("错误: 不支持的应用类型: {app}");
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
                key_field.to_string(),
                serde_json::Value::String(key.to_string()),
            );
        }
        if let Some(url) = base_url {
            env.insert(
                url_field.to_string(),
                serde_json::Value::String(url.to_string()),
            );
        }
    } else if api_key.is_some() || base_url.is_some() {
        let mut env = serde_json::Map::new();
        if let Some(key) = api_key {
            env.insert(
                key_field.to_string(),
                serde_json::Value::String(key.to_string()),
            );
        }
        if let Some(url) = base_url {
            env.insert(
                url_field.to_string(),
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
    let updated = cc_switch_core::Provider {
        id: id.to_string(),
        name: new_name,
        settings_config,
        website_url: provider.website_url,
        category: provider.category,
        created_at: provider.created_at,
        sort_index: provider.sort_index,
        notes: provider.notes,
        meta: Some(meta),
        icon: provider.icon.clone(),
        icon_color: provider.icon_color.clone(),
        in_failover_queue: provider.in_failover_queue,
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
    let proxy_service = cc_switch_core::ProxyService::new(db);

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
    let proxy_service = cc_switch_core::ProxyService::new(db);

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
    let proxy_service = cc_switch_core::ProxyService::new(db.clone());

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
                let cfg: cc_switch_core::CircuitBreakerConfig =
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
                    if let Err(e) = cc_switch_core::http_client::init(Some(url)) {
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
                let _ = cc_switch_core::http_client::init(None);
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
    let backup_dir = cc_switch_core::get_app_config_dir().join("backups");
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
    match cc_switch_core::Database::list_backups() {
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
            println!("  总成本:         ${}", summary.total_cost);
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
            println!("  成功率:         {:.1}%", summary.success_rate * 100.0);
            println!("  缓存命中率:     {:.1}%", summary.cache_hit_rate * 100.0);

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
        match cc_switch_core::SpeedtestService::test_endpoints(vec![url.to_string()], Some(timeout))
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
    match cc_switch_core::core::decl_config::DeclConfig::from_yaml_file(path) {
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
    let config = match cc_switch_core::core::decl_config::DeclConfig::from_yaml_file(path) {
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
    // CLI 模式：proxy_service = None，代理字段会被跳过并提示
    let ctx = cc_switch_core::core::decl_config::ApplyContext::new(&db);
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(config.apply(&ctx)) {
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

// ============================================================================
// Phase 1: 代理核心能力命令
// ============================================================================

/// sort-providers: 调整供应商排序
fn cmd_sort_providers(app: String, order: String) {
    if let Err(e) = validated_app(&app) {
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
    let app_state = cc_switch_core::AppState::new(db);
    let app_type = match cc_switch_core::AppType::from_str(&app) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let updates: Vec<cc_switch_core::ProviderSortUpdate> = match serde_json::from_str(&order) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("解析排序 JSON 失败: {e}");
            eprintln!("期望格式: [{{\"id\":\"provider_id\",\"sortIndex\":0}},...]");
            std::process::exit(1);
        }
    };

    match cc_switch_core::ProviderService::update_sort_order(&app_state, app_type, updates) {
        Ok(_) => println!("供应商排序已更新"),
        Err(e) => {
            eprintln!("更新排序失败: {e}");
            std::process::exit(1);
        }
    }
}

/// import-live: 从 Live 配置文件导入供应商
fn cmd_import_live(app: String) {
    if let Err(e) = validated_app(&app) {
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
    let app_state = cc_switch_core::AppState::new(db);
    let app_type = match cc_switch_core::AppType::from_str(&app) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    match cc_switch_core::ProviderService::import_default_config(&app_state, app_type) {
        Ok(true) => println!("已从 Live 配置导入供应商到 {app}"),
        Ok(false) => println!("{app} 已有供应商，跳过导入"),
        Err(e) => {
            eprintln!("导入失败: {e}");
            std::process::exit(1);
        }
    }
}

/// read-live: 读取 Live 配置文件内容
fn cmd_read_live(app: String) {
    if let Err(e) = validated_app(&app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
    let app_type = match cc_switch_core::AppType::from_str(&app) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match cc_switch_core::ProviderService::read_live_settings(app_type) {
        Ok(settings) => println!(
            "{}",
            serde_json::to_string_pretty(&settings).unwrap_or_else(|_| "{}".to_string())
        ),
        Err(e) => {
            eprintln!("读取 Live 配置失败: {e}");
            std::process::exit(1);
        }
    }
}

/// fetch-models: 获取供应商支持的模型列表
fn cmd_fetch_models(base_url: &str, api_key: &str, full_url: bool, models_path: Option<&str>) {
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async {
        match cc_switch_core::fetch_models_for_config(
            base_url.to_string(),
            api_key.to_string(),
            Some(full_url),
            models_path.map(|s| s.to_string()),
            None,
        )
        .await
        {
            Ok(models) => {
                println!("可用模型 ({}):", models.len());
                for m in &models {
                    println!("  {:?}", m);
                }
            }
            Err(e) => {
                eprintln!("获取模型列表失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

/// sync-live: 将数据库供应商同步到 Live 配置文件
fn cmd_sync_live() {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let app_state = cc_switch_core::AppState::new(db);
    match cc_switch_core::ProviderService::sync_current_to_live(&app_state) {
        Ok(_) => println!("已将数据库供应商同步到 Live 配置"),
        Err(e) => {
            eprintln!("同步失败: {e}");
            std::process::exit(1);
        }
    }
}

/// proxy-config: 查看/设置代理配置
fn cmd_proxy_config(action: &str, config: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let proxy_service = cc_switch_core::ProxyService::new(db);
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match action {
            "get" => match proxy_service.get_config().await {
                Ok(cfg) => println!(
                    "{}",
                    serde_json::to_string_pretty(&cfg).unwrap_or_else(|_| "{}".to_string())
                ),
                Err(e) => {
                    eprintln!("获取代理配置失败: {e}");
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
                let cfg: cc_switch_core::ProxyConfig = match serde_json::from_str(config_json) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("解析配置失败: {e}");
                        std::process::exit(1);
                    }
                };
                match proxy_service.update_config(&cfg).await {
                    Ok(_) => println!("代理配置已更新"),
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
    });
}

/// global-proxy-config: 查看/设置全局代理配置
fn cmd_global_proxy_config(action: &str, config: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match action {
            "get" => match db.get_global_proxy_config().await {
                Ok(cfg) => println!(
                    "{}",
                    serde_json::to_string_pretty(&cfg).unwrap_or_else(|_| "{}".to_string())
                ),
                Err(e) => {
                    eprintln!("获取全局代理配置失败: {e}");
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
                let cfg: cc_switch_core::GlobalProxyConfig = match serde_json::from_str(config_json)
                {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("解析配置失败: {e}");
                        std::process::exit(1);
                    }
                };
                match db.update_global_proxy_config(cfg).await {
                    Ok(_) => println!("全局代理配置已更新"),
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
    });
}

/// app-proxy-config: 查看/设置应用级代理配置
fn cmd_app_proxy_config(action: &str, app: &str, config: Option<&str>) {
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
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match action {
            "get" => match db.get_proxy_config_for_app(app).await {
                Ok(cfg) => println!(
                    "{}",
                    serde_json::to_string_pretty(&cfg).unwrap_or_else(|_| "{}".to_string())
                ),
                Err(e) => {
                    eprintln!("获取应用代理配置失败: {e}");
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
                let cfg: cc_switch_core::AppProxyConfig = match serde_json::from_str(config_json) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("解析配置失败: {e}");
                        std::process::exit(1);
                    }
                };
                let app_type = cfg.app_type.clone();
                match db.update_proxy_config_for_app(cfg).await {
                    Ok(_) => println!("{app_type} 代理配置已更新"),
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
    });
}

/// cost-multiplier: 查看/设置默认成本倍率
fn cmd_cost_multiplier(action: &str, app: &str, value: Option<&str>) {
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
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match action {
            "get" => {
                let key = format!("cost_multiplier_{app}");
                match db.get_setting(&key) {
                    Ok(Some(v)) => println!("{app} 成本倍率: {v}"),
                    Ok(None) => println!("{app} 成本倍率: 1.0 (默认)"),
                    Err(e) => {
                        eprintln!("获取失败: {e}");
                        std::process::exit(1);
                    }
                }
            }
            "set" => {
                let val = match value {
                    Some(v) => v,
                    None => {
                        eprintln!("请用 --value 指定倍率值");
                        std::process::exit(1);
                    }
                };
                // 验证是否为有效数字
                if val.parse::<f64>().is_err() {
                    eprintln!("无效的倍率值: {val}，请输入数字（如 1.0、0.5）");
                    std::process::exit(1);
                }
                let key = format!("cost_multiplier_{app}");
                match db.set_setting(&key, val) {
                    Ok(_) => println!("{app} 成本倍率已设置为 {val}"),
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
    });
}

/// pricing-source: 查看/设置计费模型来源
fn cmd_pricing_source(action: &str, app: &str, value: Option<&str>) {
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
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match action {
            "get" => {
                let key = format!("pricing_model_source_{app}");
                match db.get_setting(&key) {
                    Ok(Some(v)) => println!("{app} 计费模型来源: {v}"),
                    Ok(None) => println!("{app} 计费模型来源: official (默认)"),
                    Err(e) => {
                        eprintln!("获取失败: {e}");
                        std::process::exit(1);
                    }
                }
            }
            "set" => {
                let val = match value {
                    Some(v) => v,
                    None => {
                        eprintln!("请用 --value 指定来源值 (official / custom)");
                        std::process::exit(1);
                    }
                };
                let key = format!("pricing_model_source_{app}");
                match db.set_setting(&key, val) {
                    Ok(_) => println!("{app} 计费模型来源已设置为 {val}"),
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
    });
}

/// takeover-status: 检测 Live 配置是否被代理接管
fn cmd_takeover_status() {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let proxy_service = cc_switch_core::ProxyService::new(db);
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match proxy_service.is_takeover_active().await {
            Ok(active) => {
                if active {
                    println!("Live 配置已被代理接管");
                } else {
                    println!("Live 配置未被代理接管");
                }
            }
            Err(e) => {
                eprintln!("检测失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

// ============================================================================
// Phase 2: 代理运维与监控命令
// ============================================================================

/// circuit-breaker-stats: 查看熔断器运行统计
fn cmd_circuit_breaker_stats(app: String, id: String) {
    if let Err(e) = validated_app(&app) {
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
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        // 先获取供应商健康状态作为熔断器状态参考
        match db.get_provider_health(&id, &app).await {
            Ok(health) => {
                println!("熔断器/供应商状态 ({app}/{id}):");
                println!(
                    "  健康状态:     {}",
                    if health.is_healthy {
                        "健康"
                    } else {
                        "不健康（可能已熔断）"
                    }
                );
                println!("  连续失败次数: {}", health.consecutive_failures);
                println!(
                    "  最后成功:     {}",
                    health.last_success_at.as_deref().unwrap_or("(无)")
                );
                println!(
                    "  最后失败:     {}",
                    health.last_failure_at.as_deref().unwrap_or("(无)")
                );
                println!(
                    "  最后错误:     {}",
                    health.last_error.as_deref().unwrap_or("(无)")
                );
                println!("  更新时间:     {}", health.updated_at);
            }
            Err(e) => {
                eprintln!("获取状态失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

/// provider-health: 查看供应商健康状态
fn cmd_provider_health(app: String, id: String) {
    if let Err(e) = validated_app(&app) {
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
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match db.get_provider_health(&id, &app).await {
            Ok(health) => {
                println!("供应商健康状态:");
                println!("  供应商 ID:    {}", health.provider_id);
                println!("  应用类型:     {}", health.app_type);
                println!(
                    "  健康状态:     {}",
                    if health.is_healthy {
                        "健康"
                    } else {
                        "不健康"
                    }
                );
                println!("  连续失败次数: {}", health.consecutive_failures);
                println!(
                    "  最后成功:     {}",
                    health.last_success_at.as_deref().unwrap_or("(无)")
                );
                println!(
                    "  最后失败:     {}",
                    health.last_failure_at.as_deref().unwrap_or("(无)")
                );
                println!(
                    "  最后错误:     {}",
                    health.last_error.as_deref().unwrap_or("(无)")
                );
                println!("  更新时间:     {}", health.updated_at);
            }
            Err(e) => {
                eprintln!("获取供应商健康状态失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

/// failover-available: 列出可加入故障转移队列的供应商
fn cmd_failover_available(app: String) {
    if let Err(e) = validated_app(&app) {
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
    match db.get_available_providers_for_failover(&app) {
        Ok(providers) => {
            if providers.is_empty() {
                println!("无可加入故障转移队列的供应商");
                return;
            }
            println!("可加入 {app} 故障转移队列的供应商:");
            for p in &providers {
                println!("  {} ({})", p.id, p.name);
            }
        }
        Err(e) => {
            eprintln!("获取列表失败: {e}");
            std::process::exit(1);
        }
    }
}

/// config-snippet: 查看/设置/提取通用配置片段
fn cmd_config_snippet(action: &str, app: String, snippet: Option<&str>) {
    if let Err(e) = validated_app(&app) {
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
    match action {
        "get" => match db.get_config_snippet(&app) {
            Ok(Some(s)) => println!("{s}"),
            Ok(None) => println!("(无通用配置片段)"),
            Err(e) => {
                eprintln!("获取失败: {e}");
                std::process::exit(1);
            }
        },
        "set" => {
            let snippet_str = match snippet {
                Some(s) => s.to_string(),
                None => {
                    eprintln!("请用 --snippet 指定配置片段 JSON");
                    std::process::exit(1);
                }
            };
            match db.set_config_snippet(&app, Some(snippet_str)) {
                Ok(_) => println!("{app} 通用配置片段已更新"),
                Err(e) => {
                    eprintln!("设置失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        "extract" => {
            let app_type = match cc_switch_core::AppType::from_str(&app) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("错误: {e}");
                    std::process::exit(1);
                }
            };
            match cc_switch_core::ProviderService::read_live_settings(app_type.clone()) {
                Ok(settings) => {
                    match cc_switch_core::ProviderService::extract_common_config_snippet_from_settings(
                        app_type,
                        &settings,
                    ) {
                        Ok(extracted) => println!("{extracted}"),
                        Err(e) => {
                            eprintln!("提取失败: {e}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("读取 Live 配置失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("未知操作: {action}，支持: get, set, extract");
            std::process::exit(1);
        }
    }
}

/// usage-by-app: 按应用查看用量统计
fn cmd_usage_by_app(days: u32) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let end_date = chrono::Utc::now().timestamp();
    let start_date = end_date - (days as i64) * 24 * 3600;

    match db.get_usage_summary_by_app(Some(start_date), Some(end_date), None, None) {
        Ok(summaries) => {
            if summaries.is_empty() {
                println!("暂无用量数据");
                return;
            }
            println!("按应用用量统计 (最近 {days} 天):");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            for s in &summaries {
                println!("\n── {} ──", s.app_type);
                println!("  总请求数:       {}", s.summary.total_requests);
                println!("  总成本:         ${}", s.summary.total_cost);
                println!(
                    "  输入 Token:     {}",
                    format_tokens(s.summary.total_input_tokens)
                );
                println!(
                    "  输出 Token:     {}",
                    format_tokens(s.summary.total_output_tokens)
                );
                println!(
                    "  缓存读取 Token: {}",
                    format_tokens(s.summary.total_cache_read_tokens)
                );
                println!("  成功率:         {:.1}%", s.summary.success_rate * 100.0);
                println!("  缓存命中率:     {:.1}%", s.summary.cache_hit_rate * 100.0);
            }
        }
        Err(e) => {
            eprintln!("查询用量统计失败: {e}");
            std::process::exit(1);
        }
    }
}

/// request-logs: 查看请求日志
fn cmd_request_logs(
    page: u32,
    page_size: u32,
    app: Option<&str>,
    provider: Option<&str>,
    model: Option<&str>,
    status: Option<u16>,
) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let filters = cc_switch_core::LogFilters {
        app_type: app.map(|s| s.to_string()),
        provider_name: provider.map(|s| s.to_string()),
        model: model.map(|s| s.to_string()),
        status_code: status,
        start_date: None,
        end_date: None,
    };

    match db.get_request_logs(&filters, page, page_size) {
        Ok(result) => {
            println!(
                "请求日志 (第 {page} 页, 每页 {page_size} 条, 共 {} 条):",
                result.total
            );
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            if result.data.is_empty() {
                println!("  (无记录)");
                return;
            }
            for log in &result.data {
                println!(
                    "  {} | {} | {} | {} | {} | {} tokens | ${}",
                    log.request_id,
                    log.app_type,
                    log.provider_name.as_deref().unwrap_or("?"),
                    log.model,
                    log.created_at,
                    log.input_tokens + log.output_tokens,
                    log.total_cost_usd
                );
            }
        }
        Err(e) => {
            eprintln!("查询请求日志失败: {e}");
            std::process::exit(1);
        }
    }
}

/// request-detail: 查看请求详情
fn cmd_request_detail(request_id: String) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match db.get_request_detail(&request_id) {
        Ok(Some(detail)) => println!(
            "{}",
            serde_json::to_string_pretty(&detail).unwrap_or_else(|_| "{}".to_string())
        ),
        Ok(None) => println!("请求 {} 不存在", request_id),
        Err(e) => {
            eprintln!("查询请求详情失败: {e}");
            std::process::exit(1);
        }
    }
}

/// check-limits: 检查供应商用量限额
fn cmd_check_limits(app: String, id: String) {
    if let Err(e) = validated_app(&app) {
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
    match db.check_provider_limits(&id, &app) {
        Ok(status) => {
            println!("供应商限额状态:");
            println!("  供应商 ID:  {}", status.provider_id);
            println!("  日用量:     ${}", status.daily_usage);
            match &status.daily_limit {
                Some(limit) => {
                    println!("  日限额:     ${limit}");
                    println!(
                        "  日超限:     {}",
                        if status.daily_exceeded { "是" } else { "否" }
                    );
                }
                None => println!("  日限额:     (未设置)"),
            }
            println!("  月用量:     ${}", status.monthly_usage);
            match &status.monthly_limit {
                Some(limit) => {
                    println!("  月限额:     ${limit}");
                    println!(
                        "  月超限:     {}",
                        if status.monthly_exceeded {
                            "是"
                        } else {
                            "否"
                        }
                    );
                }
                None => println!("  月限额:     (未设置)"),
            }
        }
        Err(e) => {
            eprintln!("检查限额失败: {e}");
            std::process::exit(1);
        }
    }
}

/// backup-delete: 删除数据库备份
fn cmd_backup_delete(name: String) {
    match cc_switch_core::Database::delete_backup(&name) {
        Ok(_) => println!("备份 '{name}' 已删除"),
        Err(e) => {
            eprintln!("删除备份失败: {e}");
            std::process::exit(1);
        }
    }
}

/// backup-rename: 重命名数据库备份
fn cmd_backup_rename(old_name: String, new_name: String) {
    match cc_switch_core::Database::rename_backup(&old_name, &new_name) {
        Ok(new_filename) => println!("备份已重命名: {old_name} → {new_filename}"),
        Err(e) => {
            eprintln!("重命名备份失败: {e}");
            std::process::exit(1);
        }
    }
}

/// endpoint: 管理自定义测速端点
fn cmd_endpoint(action: &str, app: Option<&str>, id: Option<&str>, url: Option<&str>) {
    let app = match app {
        Some(a) => a,
        None => {
            eprintln!("请指定应用类型");
            std::process::exit(1);
        }
    };
    if let Err(e) = validated_app(app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
    let id = match id {
        Some(i) => i,
        None => {
            eprintln!("请指定供应商 ID");
            std::process::exit(1);
        }
    };

    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let app_state = cc_switch_core::AppState::new(db);
    let app_type = match cc_switch_core::AppType::from_str(app) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    match action {
        "list" => {
            match cc_switch_core::ProviderService::get_custom_endpoints(&app_state, app_type, id) {
                Ok(endpoints) => {
                    if endpoints.is_empty() {
                        println!("(无自定义端点)");
                        return;
                    }
                    println!("{app}/{id} 自定义测速端点:");
                    for ep in &endpoints {
                        println!("  {}", ep.url);
                    }
                }
                Err(e) => {
                    eprintln!("获取端点列表失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        "add" => {
            let url = match url {
                Some(u) => u,
                None => {
                    eprintln!("请用 --url 指定端点 URL");
                    std::process::exit(1);
                }
            };
            match cc_switch_core::ProviderService::add_custom_endpoint(
                &app_state,
                app_type,
                id,
                url.to_string(),
            ) {
                Ok(_) => println!("端点已添加: {url}"),
                Err(e) => {
                    eprintln!("添加端点失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        "remove" => {
            let url = match url {
                Some(u) => u,
                None => {
                    eprintln!("请用 --url 指定端点 URL");
                    std::process::exit(1);
                }
            };
            match cc_switch_core::ProviderService::remove_custom_endpoint(
                &app_state,
                app_type,
                id,
                url.to_string(),
            ) {
                Ok(_) => println!("端点已移除: {url}"),
                Err(e) => {
                    eprintln!("移除端点失败: {e}");
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

// ============================================================================
// Phase 3: 附带功能命令
// ============================================================================

/// add-mcp: 添加/更新 MCP 服务器
fn cmd_add_mcp(id: String, name: String, command: String, args: Option<&str>, env: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let app_state = cc_switch_core::AppState::new(db);

    let args_vec: Vec<String> = match args {
        Some(a) => serde_json::from_str(a).unwrap_or_else(|_| vec![]),
        None => vec![],
    };
    let env_map: serde_json::Value = match env {
        Some(e) => serde_json::from_str(e).unwrap_or(serde_json::json!({})),
        None => serde_json::json!({}),
    };

    let server_config = serde_json::json!({
        "command": command,
        "args": args_vec,
        "env": env_map,
    });

    let mcp_server = cc_switch_core::McpServer {
        id: id.clone(),
        name: name.clone(),
        server: server_config,
        apps: cc_switch_core::McpApps {
            claude: true,
            codex: false,
            gemini: false,
            opencode: false,
            hermes: false,
        },
        description: None,
        homepage: None,
        docs: None,
        tags: vec![],
    };

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match cc_switch_core::McpService::upsert_server(&app_state, mcp_server) {
            Ok(_) => println!("MCP 服务器 '{id}' ({name}) 已添加/更新"),
            Err(e) => {
                eprintln!("添加 MCP 服务器失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

/// remove-mcp: 删除 MCP 服务器
fn cmd_remove_mcp(id: String) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let app_state = cc_switch_core::AppState::new(db);
    match cc_switch_core::McpService::delete_server(&app_state, &id) {
        Ok(_) => println!("MCP 服务器 '{id}' 已删除"),
        Err(e) => {
            eprintln!("删除 MCP 服务器失败: {e}");
            std::process::exit(1);
        }
    }
}

/// toggle-mcp: 启用/禁用 MCP 服务器
fn cmd_toggle_mcp(id: String, app: String, enabled: String) {
    if let Err(e) = validated_app(&app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
    let enable = match enabled.to_lowercase().as_str() {
        "on" | "true" | "1" => true,
        "off" | "false" | "0" => false,
        _ => {
            eprintln!("无效的值: {enabled}，请使用 on/off");
            std::process::exit(1);
        }
    };
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let app_state = cc_switch_core::AppState::new(db);
    let app_type = match cc_switch_core::AppType::from_str(&app) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match cc_switch_core::McpService::toggle_app(&app_state, &id, app_type, enable) {
            Ok(_) => println!(
                "MCP '{id}' 在 {app} 中已{}",
                if enable { "启用" } else { "禁用" }
            ),
            Err(e) => {
                eprintln!("切换失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

/// test-mcp: 测试 MCP 连接
fn cmd_test_mcp(id: String) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let server_opt = match db.get_all_mcp_servers() {
        Ok(servers) => servers.get(&id).cloned(),
        Err(e) => {
            eprintln!("获取 MCP 服务器失败: {e}");
            std::process::exit(1);
        }
    };
    let server = match server_opt {
        Some(s) => s,
        None => {
            eprintln!("未找到 ID 为 '{id}' 的 MCP 服务器");
            std::process::exit(1);
        }
    };
    let spec = &server.server;
    let command = spec
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("(无命令)");
    let args: Vec<String> = spec
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    println!("MCP 服务器: {} ({})", server.name, server.id);
    println!("命令: {} {}", command, args.join(" "));

    // 基本连通性测试：尝试执行命令检查是否可运行
    let started = std::time::Instant::now();
    let result = std::process::Command::new(command)
        .args(&args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    match result {
        Ok(mut child) => {
            // 立即终止，仅测试可启动性
            let _ = child.kill();
            let _ = child.wait();
            let elapsed = started.elapsed().as_millis();
            println!("启动测试: 成功 ({}ms)", elapsed);
            println!("注意: 此测试仅验证命令可执行性，完整连接测试请在 GUI 中进行");
        }
        Err(e) => {
            println!("启动测试: 失败");
            println!("错误: {e}");
            println!("请检查命令路径和参数是否正确");
        }
    }
}

/// add-prompt: 添加/更新 Prompt
fn cmd_add_prompt(
    app: String,
    id: String,
    name: String,
    content: Option<&str>,
    file: Option<&str>,
) {
    if let Err(e) = validated_app(&app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
    let prompt_content = match (content, file) {
        (Some(c), _) => c.to_string(),
        (None, Some(f)) => match std::fs::read_to_string(f) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("读取文件失败: {e}");
                std::process::exit(1);
            }
        },
        (None, None) => {
            eprintln!("请用 --content 指定内容或 --file 指定文件路径");
            std::process::exit(1);
        }
    };

    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let app_state = cc_switch_core::AppState::new(db);
    let app_type = match cc_switch_core::AppType::from_str(&app) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    let prompt = cc_switch_core::Prompt {
        id: id.clone(),
        name: name.clone(),
        content: prompt_content,
        description: None,
        enabled: true,
        created_at: None,
        updated_at: None,
    };

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        match cc_switch_core::PromptService::upsert_prompt(&app_state, app_type, &id, prompt) {
            Ok(_) => println!("Prompt '{id}' ({name}) 已添加/更新到 {app}"),
            Err(e) => {
                eprintln!("添加 Prompt 失败: {e}");
                std::process::exit(1);
            }
        }
    });
}

/// remove-prompt: 删除 Prompt
fn cmd_remove_prompt(app: String, id: String) {
    if let Err(e) = validated_app(&app) {
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
    let app_state = cc_switch_core::AppState::new(db);
    let app_type = match cc_switch_core::AppType::from_str(&app) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match cc_switch_core::PromptService::delete_prompt(&app_state, app_type, &id) {
        Ok(_) => println!("Prompt '{id}' 已从 {app} 删除"),
        Err(e) => {
            eprintln!("删除 Prompt 失败: {e}");
            std::process::exit(1);
        }
    }
}

/// enable-prompt: 启用/禁用 Prompt
fn cmd_enable_prompt(app: String, id: String, enabled: String) {
    if let Err(e) = validated_app(&app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
    let enable = match enabled.to_lowercase().as_str() {
        "on" | "true" | "1" => true,
        "off" | "false" | "0" => false,
        _ => {
            eprintln!("无效的值: {enabled}，请使用 on/off");
            std::process::exit(1);
        }
    };
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let app_state = cc_switch_core::AppState::new(db);
    let app_type = match cc_switch_core::AppType::from_str(&app) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    if enable {
        match cc_switch_core::PromptService::enable_prompt(&app_state, app_type, &id) {
            Ok(_) => println!("Prompt '{id}' 已启用 ({app})"),
            Err(e) => {
                eprintln!("启用失败: {e}");
                std::process::exit(1);
            }
        }
    } else {
        // 禁用：先获取现有 prompt，修改 enabled 后 upsert
        match cc_switch_core::PromptService::get_prompts(&app_state, app_type.clone()) {
            Ok(prompts) => {
                let mut prompt = match prompts.get(&id) {
                    Some(p) => p.clone(),
                    None => {
                        eprintln!("Prompt '{id}' 不存在于 {app}");
                        std::process::exit(1);
                    }
                };
                prompt.enabled = false;
                match cc_switch_core::PromptService::upsert_prompt(&app_state, app_type, &id, prompt)
                {
                    Ok(_) => println!("Prompt '{id}' 已禁用 ({app})"),
                    Err(e) => {
                        eprintln!("禁用失败: {e}");
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("获取 Prompt 列表失败: {e}");
                std::process::exit(1);
            }
        }
    }
}

/// list-skills: 列出已安装 Skills
fn cmd_list_skills(app: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match cc_switch_core::SkillService::get_all_installed(&db) {
        Ok(skills) => {
            if skills.is_empty() {
                println!("(无已安装 Skills)");
                return;
            }
            let filtered: Vec<_> = match app {
                Some(a) => skills
                    .into_iter()
                    .filter(|s| {
                        s.apps.claude == (a == "claude")
                            || s.apps.codex == (a == "codex")
                            || s.apps.gemini == (a == "gemini")
                    })
                    .collect(),
                None => skills,
            };
            println!("{:<3} {:<25} {:<20} {}", "#", "名称", "ID", "应用");
            for (i, s) in filtered.iter().enumerate() {
                let mut apps = Vec::new();
                if s.apps.claude {
                    apps.push("claude");
                }
                if s.apps.codex {
                    apps.push("codex");
                }
                if s.apps.gemini {
                    apps.push("gemini");
                }
                println!(
                    "{:<3} {:<25} {:<20} {}",
                    i + 1,
                    s.name,
                    s.id,
                    apps.join(",")
                );
            }
        }
        Err(e) => {
            eprintln!("获取 Skills 失败: {e}");
            std::process::exit(1);
        }
    }
}

/// remove-skill: 卸载 Skill
fn cmd_remove_skill(id: String, app: Option<&str>) {
    let _ = app; // app 参数保留用于未来按应用卸载，当前统一卸载
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match cc_switch_core::SkillService::uninstall(&db, &id) {
        Ok(result) => {
            println!("Skill '{id}' 已卸载");
            if let Some(backup) = &result.backup_path {
                println!("  备份路径: {backup}");
            }
        }
        Err(e) => {
            eprintln!("卸载 Skill 失败: {e}");
            std::process::exit(1);
        }
    }
}

/// toggle-skill: 启用/禁用 Skill
fn cmd_toggle_skill(id: String, app: String, enabled: String) {
    if let Err(e) = validated_app(&app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
    let enable = match enabled.to_lowercase().as_str() {
        "on" | "true" | "1" => true,
        "off" | "false" | "0" => false,
        _ => {
            eprintln!("无效的值: {enabled}，请使用 on/off");
            std::process::exit(1);
        }
    };
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let app_type = match cc_switch_core::AppType::from_str(&app) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match cc_switch_core::SkillService::toggle_app(&db, &id, &app_type, enable) {
        Ok(_) => println!(
            "Skill '{id}' 在 {app} 中已{}",
            if enable { "启用" } else { "禁用" }
        ),
        Err(e) => {
            eprintln!("切换失败: {e}");
            std::process::exit(1);
        }
    }
}

/// check-env: 检查环境变量冲突
fn cmd_check_env() {
    let all_apps = [
        "claude", "codex", "gemini", "opencode", "openclaw", "hermes",
    ];
    let mut found_any = false;
    for app in &all_apps {
        match cc_switch_core::check_env_conflicts(app.to_string()) {
            Ok(conflicts) => {
                if !conflicts.is_empty() {
                    found_any = true;
                    println!("\n── {app} ──");
                    for c in &conflicts {
                        println!(
                            "  {} = {} (来源: {} - {})",
                            c.var_name, c.var_value, c.source_type, c.source_path
                        );
                    }
                }
            }
            Err(e) => log::debug!("检查 {app} 环境变量失败: {e}"),
        }
    }
    if !found_any {
        println!("未检测到环境变量冲突");
    }
}

/// list-sessions: 列出会话
fn cmd_list_sessions(app: Option<&str>, limit: u32) {
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async {
        let sessions =
            match tokio::task::spawn_blocking(|| cc_switch_core::session_manager::scan_sessions())
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("扫描会话失败: {e}");
                    std::process::exit(1);
                }
            };

        let filtered: Vec<_> = match app {
            Some(a) => sessions
                .into_iter()
                .filter(|s| s.provider_id.contains(a))
                .collect(),
            None => sessions,
        };
        let limited: Vec<_> = filtered.into_iter().take(limit as usize).collect();
        if limited.is_empty() {
            println!("(无会话)");
            return;
        }
        println!(
            "{:<3} {:<30} {:<25} {:<20}",
            "#", "会话ID", "供应商", "最后活跃"
        );
        for (i, s) in limited.iter().enumerate() {
            let sid = &s.session_id[..s.session_id.len().min(30)];
            let pid = &s.provider_id[..s.provider_id.len().min(25)];
            let active = s
                .last_active_at
                .map(|t| {
                    chrono::DateTime::from_timestamp(t, 0)
                        .map(|d| d.to_rfc3339())
                        .unwrap_or_else(|| t.to_string())
                })
                .unwrap_or_else(|| "?".to_string());
            println!("{:<3} {:<30} {:<25} {:<20}", i + 1, sid, pid, active);
        }
    });
}

/// usage-trends: 查看用量趋势
fn cmd_usage_trends(days: u32) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let end_date = chrono::Utc::now().timestamp();
    let start_date = end_date - (days as i64) * 24 * 3600;

    match db.get_daily_trends(Some(start_date), Some(end_date), None, None, None) {
        Ok(trends) => {
            if trends.is_empty() {
                println!("暂无用量趋势数据");
                return;
            }
            println!("用量趋势 (最近 {days} 天):");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!(
                "{:<12} {:<10} {:<12} {:<12} {:<10}",
                "日期", "请求数", "成本($)", "输入Token", "输出Token"
            );
            for t in &trends {
                println!(
                    "{:<12} {:<10} {:<12} {:<12} {:<10}",
                    t.date,
                    t.request_count,
                    t.total_cost,
                    format_tokens(t.total_input_tokens),
                    format_tokens(t.total_output_tokens)
                );
            }
        }
        Err(e) => {
            eprintln!("查询用量趋势失败: {e}");
            std::process::exit(1);
        }
    }
}

/// provider-stats: 查看供应商统计
fn cmd_provider_stats(days: u32) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let end_date = chrono::Utc::now().timestamp();
    let start_date = end_date - (days as i64) * 24 * 3600;

    match db.get_provider_stats(Some(start_date), Some(end_date), None, None, None) {
        Ok(stats) => {
            if stats.is_empty() {
                println!("暂无供应商统计数据");
                return;
            }
            println!("供应商统计 (最近 {days} 天):");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!(
                "{:<25} {:<10} {:<12} {:<10}",
                "供应商", "请求数", "成本($)", "成功率"
            );
            for s in &stats {
                println!(
                    "{:<25} {:<10} {:<12} {:<10.1}%",
                    s.provider_name,
                    s.request_count,
                    s.total_cost,
                    s.success_rate * 100.0
                );
            }
        }
        Err(e) => {
            eprintln!("查询供应商统计失败: {e}");
            std::process::exit(1);
        }
    }
}

/// model-stats: 查看模型统计
fn cmd_model_stats(days: u32) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let end_date = chrono::Utc::now().timestamp();
    let start_date = end_date - (days as i64) * 24 * 3600;

    match db.get_model_stats(Some(start_date), Some(end_date), None, None, None) {
        Ok(stats) => {
            if stats.is_empty() {
                println!("暂无模型统计数据");
                return;
            }
            println!("模型统计 (最近 {days} 天):");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!(
                "{:<30} {:<10} {:<12} {:<15}",
                "模型", "请求数", "成本($)", "平均成本/请求"
            );
            for s in &stats {
                println!(
                    "{:<30} {:<10} {:<12} {:<15}",
                    s.model, s.request_count, s.total_cost, s.avg_cost_per_request
                );
            }
        }
        Err(e) => {
            eprintln!("查询模型统计失败: {e}");
            std::process::exit(1);
        }
    }
}

// ============================================================================
// Plan C: 新功能（热重载 / 访问控制 / 烟雾测试）
// ============================================================================

/// reload: 热重载代理配置
fn cmd_reload() {
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async {
        let status = match cc_switch_core::services::ProxyService::get_status().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("错误: 获取代理状态失败: {e}");
                std::process::exit(1);
            }
        };
        if !status.running {
            eprintln!("错误: 代理服务器未运行，无法热重载。请先执行 start 或 daemon");
            std::process::exit(1);
        }
        // 向运行中代理发送 reload 信号
        let client = match reqwest::Client::new()
            .post(format!("http://{}:{}/__internal/reload", status.address, status.port))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("错误: 无法连接到代理服务器: {e}");
                std::process::exit(1);
            }
        };
        if client.status().is_success() {
            println!("✓ 代理配置已热重载");
        } else {
            eprintln!("代理返回错误: HTTP {}", client.status());
            std::process::exit(1);
        }
    });
}

/// auth-token: 设置/清除代理访问令牌
fn cmd_auth_token(action: Option<&str>, token: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match action {
        Some("set") => {
            let t = token.unwrap_or("");
            if t.trim().is_empty() {
                eprintln!("错误: --token 不能为空");
                std::process::exit(1);
            }
            if let Err(e) = db.set_proxy_auth_token(Some(t)) {
                eprintln!("设置令牌失败: {e}");
                std::process::exit(1);
            }
            println!("✓ 代理访问令牌已设置");
        }
        Some("clear") => {
            if let Err(e) = db.set_proxy_auth_token(None) {
                eprintln!("清除令牌失败: {e}");
                std::process::exit(1);
            }
            println!("✓ 代理访问令牌已清除（代理回到开放状态）");
        }
        _ => {
            match db.get_proxy_auth_token() {
                Ok(Some(t)) => println!("令牌已设置 ({}...)", &t[..t.len().min(8)]),
                Ok(None) => println!("令牌未设置（代理完全开放）"),
                Err(e) => {
                    eprintln!("读取令牌失败: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
}

/// acl: 管理 IP 白名单
fn cmd_acl(action: Option<&str>, cidr: Option<&str>) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match action {
        Some("list") => {
            match db.get_proxy_acl_cidrs() {
                Ok(cidrs) => {
                    if cidrs.is_empty() {
                        println!("IP 白名单未设置（不限制来源 IP）");
                    } else {
                        println!("IP 白名单 ({} 条):", cidrs.len());
                        for c in &cidrs {
                            println!("  {c}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("读取 ACL 失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some("add") => {
            let c = match cidr {
                Some(c) => c.to_string(),
                None => {
                    eprintln!("错误: --cidr 参数必填");
                    std::process::exit(1);
                }
            };
            let mut existing = db.get_proxy_acl_cidrs().unwrap_or_default();
            if existing.contains(&c) {
                println!("CIDR {c} 已在白名单中");
                return;
            }
            existing.push(c);
            if let Err(e) = db.set_proxy_acl_cidrs(&existing) {
                eprintln!("设置 ACL 失败: {e}");
                std::process::exit(1);
            }
            println!("✓ CIDR 已添加到白名单");
        }
        Some("remove") => {
            let c = match cidr {
                Some(c) => c.to_string(),
                None => {
                    eprintln!("错误: --cidr 参数必填");
                    std::process::exit(1);
                }
            };
            let mut existing = db.get_proxy_acl_cidrs().unwrap_or_default();
            if let Some(pos) = existing.iter().position(|x| x == &c) {
                existing.remove(pos);
                if let Err(e) = db.set_proxy_acl_cidrs(&existing) {
                    eprintln!("设置 ACL 失败: {e}");
                    std::process::exit(1);
                }
                println!("✓ CIDR 已从白名单移除");
            } else {
                eprintln!("CIDR {c} 不在白名单中");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("用法: cc-switch-cli acl <list|add|remove> [--cidr CIDR]");
            std::process::exit(1);
        }
    }
}

/// smoke-test: 协议转换烟雾测试
fn cmd_smoke_test(app: Option<&str>) {
    let results = if let Some(app_str) = app {
        // 单应用模式：转换为相应的测试路径
        let (from, to, model) = match app_str {
            "claude" | "claude-desktop" => ("anthropic", "openai_chat", "claude-sonnet-5"),
            "codex" => ("openai_chat", "anthropic", "gpt-5.5"),
            "gemini" => ("openai_chat", "openai_chat", "gemini-3.1-pro"),
            _ => {
                eprintln!("错误: 无效的应用类型: {app_str}");
                std::process::exit(1);
            }
        };
        vec![cc_switch_core::proxy::smoke_test::run_smoke_test(
            from, to, model,
        )]
    } else {
        cc_switch_core::proxy::smoke_test::run_all_smoke_tests()
    };

    println!("协议转换烟雾测试:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    for r in &results {
        let icon = if r.passed { "✓" } else { "✗" };
        println!("  {icon} {:<20} {}", r.app_type, r.message);
    }
    let passed = results.iter().filter(|r| r.passed).count();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("结果: {}/{} 通过", passed, results.len());
    if passed < results.len() {
        std::process::exit(1);
    }
}

// ============================================================================
// Plan D: 体验改进（export-yaml / diff / rollback / toggle / preview / trace / replay）
// ============================================================================

/// export-yaml: 将数据库配置导出为声明式 YAML
fn cmd_export_yaml(path: &str) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let config = match cc_switch_core::core::decl_config::DeclConfig::from_database(&db) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("从数据库构造配置失败: {e}");
            std::process::exit(1);
        }
    };
    let yaml = match serde_yaml::to_string(&config) {
        Ok(y) => y,
        Err(e) => {
            eprintln!("序列化 YAML 失败: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = std::fs::write(path, &yaml) {
        eprintln!("写入文件失败: {e}");
        std::process::exit(1);
    }
    println!("✓ 配置已导出到: {path}");
}

/// diff: 对比 YAML 与当前数据库配置
fn cmd_diff(path: &str) {
    let yaml_config = match cc_switch_core::core::decl_config::DeclConfig::from_yaml_file(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("加载 YAML 文件失败: {e}");
            std::process::exit(1);
        }
    };
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let db_config = match cc_switch_core::core::decl_config::DeclConfig::from_database(&db) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("从数据库读取配置失败: {e}");
            std::process::exit(1);
        }
    };

    println!("YAML 与数据库配置对比:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  YAML 供应商: {} 条", yaml_config.providers.len());
    println!("  数据库供应商: {} 条", db_config.providers.len());
    let new_count = yaml_config
        .providers
        .iter()
        .filter(|yp| !db_config.providers.iter().any(|dp| dp.app == yp.app && dp.id == yp.id))
        .count();
    let changed_count = yaml_config
        .providers
        .iter()
        .filter(|yp| {
            db_config
                .providers
                .iter()
                .any(|dp| dp.app == yp.app && dp.id == yp.id && dp.env != yp.env)
        })
        .count();
    if new_count > 0 {
        println!("  新增供应商: {new_count} 条");
    }
    if changed_count > 0 {
        println!("  变更供应商: {changed_count} 条");
    }
    if new_count == 0 && changed_count == 0 && yaml_config.providers.len() == db_config.providers.len() {
        println!("  ✓ 无差异 — YAML 与数据库配置一致");
    }
}

/// rollback: 回滚到上一个 apply 前的备份
fn cmd_rollback() {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let backups = match db.list_backups() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("列出备份失败: {e}");
            std::process::exit(1);
        }
    };
    let apply_backups: Vec<_> = backups
        .iter()
        .filter(|b| b.contains("apply-rollback"))
        .collect();
    if apply_backups.is_empty() {
        eprintln!("没有找到 apply 回滚备份。每次执行 apply-config 会自动创建备份。");
        std::process::exit(1);
    }
    let latest = apply_backups.last().unwrap();
    match db.restore_backup(latest) {
        Ok(_) => println!("✓ 已回滚到备份: {latest}"),
        Err(e) => {
            eprintln!("回滚失败: {e}");
            std::process::exit(1);
        }
    }
}

/// toggle-provider: 启用/禁用供应商
fn cmd_toggle_provider(app: &str, id: &str, enabled: &str) {
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
    let enable = match enabled {
        "on" | "true" => true,
        "off" | "false" => false,
        _ => {
            eprintln!("错误: 第三个参数必须是 on 或 off");
            std::process::exit(1);
        }
    };
    match db.set_provider_enabled(app, id, enable) {
        Ok(_) => {
            let state = if enable { "启用" } else { "禁用" };
            println!("✓ 供应商 '{id}' ({app}) 已{state}");
        }
        Err(e) => {
            eprintln!("操作失败: {e}");
            std::process::exit(1);
        }
    }
}

/// preview-conversion: 预览协议转换
fn cmd_preview_conversion(from: &str, to: &str, payload: &str, _base_url: Option<&str>) {
    let body: serde_json::Value = match serde_json::from_str(payload) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("解析 JSON payload 失败: {e}");
            std::process::exit(1);
        }
    };
    let result = match (from, to) {
        ("anthropic", "openai_chat") => cc_switch_core::proxy::providers::transform::anthropic_to_openai(body),
        ("openai_chat", "anthropic") => cc_switch_core::proxy::providers::transform::openai_to_anthropic(body),
        _ => {
            eprintln!("错误: 不支持的转换路径: {from} → {to}");
            eprintln!("支持的路径: anthropic → openai_chat, openai_chat → anthropic");
            std::process::exit(1);
        }
    };
    match result {
        Ok(transformed) => {
            println!("协议转换预览:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  源格式: {from}");
            println!("  目标格式: {to}");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            let pretty = serde_json::to_string_pretty(&transformed)
                .unwrap_or_else(|_| "序列化失败".to_string());
            println!("{pretty}");
        }
        Err(e) => {
            eprintln!("协议转换失败: {from} → {to}: {e}");
            std::process::exit(1);
        }
    }
}

/// proxy-trace: 代理链路跟踪
fn cmd_proxy_trace(app: &str, model: &str) {
    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async {
        let status = match cc_switch_core::services::ProxyService::get_status().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("错误: 获取代理状态失败: {e}");
                std::process::exit(1);
            }
        };
        if !status.running {
            eprintln!("错误: 代理未运行，无法 trace");
            std::process::exit(1);
        }
        println!("代理链路跟踪 ({app}, 模型: {model}):");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  代理地址: {}:{}", status.address, status.port);
        println!("  应用: {app}");
        println!("  模型: {model}");
        println!("  跟踪端点: POST http://{}:{}/v1/chat/completions", status.address, status.port);
        println!();
        println!("  使用以下 curl 命令发送测试请求:");
        println!();
        println!("  curl -s http://{}:{}/v1/chat/completions \\", status.address, status.port);
        println!("    -H 'Content-Type: application/json' \\");
        println!("    -d '{{\"model\":\"{}\",\"messages\":[{{\"role\":\"user\",\"content\":\"Hello\"}}],\"max_tokens\":10,\"stream\":false}}' | jq .", model);
        println!();
        println!("  查看代理日志:");
        println!("    tail -f ~/.cc-switch/cc-switch-daemon.log");
    });
}

/// replay-request: 重放历史请求
fn cmd_replay_request(request_id: &str) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    match db.get_request_detail(request_id) {
        Ok(Some(detail)) => {
            println!("请求详情: {}", request_id);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  时间: {}", detail.created_at.unwrap_or("?".to_string()));
            println!("  应用: {}", detail.app_type.unwrap_or("?".to_string()));
            println!("  模型: {}", detail.model.unwrap_or("?".to_string()));
            println!();
            if let Some(body) = &detail.request_body {
                println!("  请求体:");
                let pretty = serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(body).unwrap_or(serde_json::Value::String(body.clone())))
                    .unwrap_or_else(|_| body.clone());
                println!("{pretty}");
            } else {
                eprintln!("  请求体未记录（需 CC_SWITCH_LOG_BODIES=1 环境变量启用）");
            }
            println!();
            println!("  如需重放，将 curl 命令指向你的代理地址:");
            println!("  curl -s http://127.0.0.1:9090/v1/chat/completions \\");
            println!("    -H 'Content-Type: application/json' \\");
            println!("    -d '<以上请求体>'");
        }
        Ok(None) => {
            eprintln!("请求 {request_id} 不存在");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("查询失败: {e}");
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
    println!("    reload                      热重载代理配置（不重启）");
    println!("    auth-token [set|clear] [--token T]  设置/清除代理访问令牌");
    println!("    acl <list|add|remove> [--cidr C]  IP 白名单管理");
    println!("    smoke-test [APP]             协议转换烟雾测试");
    println!("    export-yaml <PATH>           导出数据库配置为 YAML");
    println!("    diff <PATH>                 对比 YAML 与当前配置");
    println!("    rollback                    回滚到上一个 apply 前备份");
    println!("    toggle-provider <APP> <ID> <on|off>  启用/禁用供应商");
    println!("    preview-conversion --from F --to F --payload JSON [--base-url U]");
    println!("    proxy-trace <APP> --model M  代理链路跟踪指南");
    println!("    replay-request <REQUEST_ID>  重放历史请求");
    println!();
    println!("  代理核心能力:");
    println!("    sort-providers <APP> --order JSON   调整供应商排序");
    println!("    import-live <APP>                   从 Live 配置导入供应商");
    println!("    read-live <APP>                     读取 Live 配置文件内容");
    println!("    fetch-models --base-url U --api-key K [--full-url] [--models-path P]");
    println!("    sync-live                           同步数据库供应商到 Live 配置");
    println!("    proxy-config <get|set> [--config JSON]");
    println!("    global-proxy-config <get|set> [--config JSON]");
    println!("    app-proxy-config <get|set> <APP> [--config JSON]");
    println!("    cost-multiplier <get|set> <APP> [--value V]");
    println!("    pricing-source <get|set> <APP> [--value V]");
    println!("    takeover-status                     检测 Live 配置接管状态");
    println!();
    println!("  代理运维与监控:");
    println!("    circuit-breaker-stats <APP> <ID>    查看熔断器/供应商状态");
    println!("    provider-health <APP> <ID>          查看供应商健康状态");
    println!("    failover-available <APP>            列出可加入故障转移的供应商");
    println!("    config-snippet <get|set|extract> <APP> [--snippet JSON]");
    println!("    usage-by-app [--days N]             按应用查看用量");
    println!("    request-logs [--page N] [--page-size N] [--app A] [--provider P] [--model M] [--status S]");
    println!("    request-detail <REQUEST_ID>         查看请求详情");
    println!("    check-limits <APP> <ID>             检查供应商用量限额");
    println!("    backup-delete <NAME>                删除备份");
    println!("    backup-rename <OLD> <NEW>           重命名备份");
    println!("    endpoint <list|add|remove> <APP> <ID> [--url URL]");
    println!();
    println!("  附带功能:");
    println!("    add-mcp <ID> <NAME> --command CMD [--args JSON] [--env JSON]");
    println!("    remove-mcp <ID>                    删除 MCP 服务器");
    println!("    toggle-mcp <ID> <APP> <on|off>     启用/禁用 MCP");
    println!("    test-mcp <ID>                      测试 MCP 连接");
    println!("    add-prompt <APP> <ID> <NAME> [--content C] [--file F]");
    println!("    remove-prompt <APP> <ID>           删除 Prompt");
    println!("    enable-prompt <APP> <ID> <on|off>  启用/禁用 Prompt");
    println!("    list-skills [APP]                  列出已安装 Skills");
    println!("    remove-skill <ID> [APP]            卸载 Skill");
    println!("    toggle-skill <ID> <APP> <on|off>   启用/禁用 Skill");
    println!("    check-env                          检查环境变量冲突");
    println!("    list-sessions [APP] [--limit N]    列出会话");
    println!("    usage-trends [--days N]            查看用量趋势");
    println!("    provider-stats [--days N]          查看供应商统计");
    println!("    model-stats [--days N]             查看模型统计");
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
    current: &cc_switch_core::AppSettings,
    _default: &cc_switch_core::AppSettings,
) -> cc_switch_core::AppSettings {
    current.clone()
}

/// 格式化 Token 数量为可读字符串（K/M/B）
fn format_tokens(count: u64) -> String {
    if count >= 1_000_000_000 {
        format!("{:.2}B", count as f64 / 1_000_000_000.0)
    } else if count >= 1_000_000 {
        format!("{:.2}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

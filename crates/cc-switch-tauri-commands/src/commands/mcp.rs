#![allow(non_snake_case)]

use indexmap::IndexMap;
use std::collections::HashMap;

use serde::Serialize;
use tauri::State;

use cc_switch_core::app_config::AppType;
use cc_switch_core::claude_mcp;
use cc_switch_core::services::McpService;
use cc_switch_core::store::AppState;

/// 获取 Claude MCP 状态
#[tauri::command]
pub async fn get_claude_mcp_status() -> Result<claude_mcp::McpStatus, String> {
    claude_mcp::get_mcp_status().map_err(|e| e.to_string())
}

/// 读取 mcp.json 文本内容
#[tauri::command]
pub async fn read_claude_mcp_config() -> Result<Option<String>, String> {
    claude_mcp::read_mcp_json().map_err(|e| e.to_string())
}

/// 新增或更新一个 MCP 服务器条目
#[tauri::command]
pub async fn upsert_claude_mcp_server(id: String, spec: serde_json::Value) -> Result<bool, String> {
    claude_mcp::upsert_mcp_server(&id, spec).map_err(|e| e.to_string())
}

/// 删除一个 MCP 服务器条目
#[tauri::command]
pub async fn delete_claude_mcp_server(id: String) -> Result<bool, String> {
    claude_mcp::delete_mcp_server(&id).map_err(|e| e.to_string())
}

/// 校验命令是否在 PATH 中可用（不执行）
#[tauri::command]
pub async fn validate_mcp_command(cmd: String) -> Result<bool, String> {
    claude_mcp::validate_command_in_path(&cmd).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct McpConfigResponse {
    pub config_path: String,
    pub servers: HashMap<String, serde_json::Value>,
}

/// 获取 MCP 配置（来自 ~/.cc-switch/config.json）
use std::str::FromStr;

#[tauri::command]
#[allow(deprecated)] // 兼容层命令，内部调用已废弃的 Service 方法
pub async fn get_mcp_config(
    state: State<'_, AppState>,
    app: String,
) -> Result<McpConfigResponse, String> {
    let config_path = cc_switch_core::config::get_app_config_path()
        .to_string_lossy()
        .to_string();
    let app_ty = AppType::from_str(&app).map_err(|e| e.to_string())?;
    let servers = McpService::get_servers(&state, app_ty).map_err(|e| e.to_string())?;
    Ok(McpConfigResponse {
        config_path,
        servers,
    })
}

/// 在 config.json 中新增或更新一个 MCP 服务器定义
/// [已废弃] 该命令仍然使用旧的分应用API，会转换为统一结构
#[tauri::command]
pub async fn upsert_mcp_server_in_config(
    state: State<'_, AppState>,
    app: String,
    id: String,
    spec: serde_json::Value,
    sync_other_side: Option<bool>,
) -> Result<bool, String> {
    use cc_switch_core::app_config::McpServer;

    let app_ty = AppType::from_str(&app).map_err(|e| e.to_string())?;

    // 读取现有的服务器（如果存在）
    let existing_server = {
        let servers = state.db.get_all_mcp_servers().map_err(|e| e.to_string())?;
        servers.get(&id).cloned()
    };

    // 构建新的统一服务器结构
    let mut new_server = if let Some(mut existing) = existing_server {
        // 更新现有服务器
        existing.server = spec.clone();
        existing.apps.set_enabled_for(&app_ty, true);
        existing
    } else {
        // 创建新服务器
        let mut apps = cc_switch_core::app_config::McpApps::default();
        apps.set_enabled_for(&app_ty, true);

        // 尝试从 spec 中提取 name，否则使用 id
        let name = spec
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();

        McpServer {
            id: id.clone(),
            name,
            server: spec,
            apps,
            description: None,
            homepage: None,
            docs: None,
            tags: Vec::new(),
        }
    };

    // 如果 sync_other_side 为 true，也启用其他应用
    if sync_other_side.unwrap_or(false) {
        new_server.apps.claude = true;
        new_server.apps.codex = true;
        new_server.apps.gemini = true;
        new_server.apps.opencode = true;
    }

    McpService::upsert_server(&state, new_server)
        .map(|_| true)
        .map_err(|e| e.to_string())
}

/// 在 config.json 中删除一个 MCP 服务器定义
#[tauri::command]
pub async fn delete_mcp_server_in_config(
    state: State<'_, AppState>,
    _app: String, // 参数保留用于向后兼容，但在统一结构中不再需要
    id: String,
) -> Result<bool, String> {
    McpService::delete_server(&state, &id).map_err(|e| e.to_string())
}

/// 设置启用状态并同步到客户端配置
#[tauri::command]
#[allow(deprecated)] // 兼容层命令，内部调用已废弃的 Service 方法
pub async fn set_mcp_enabled(
    state: State<'_, AppState>,
    app: String,
    id: String,
    enabled: bool,
) -> Result<bool, String> {
    let app_ty = AppType::from_str(&app).map_err(|e| e.to_string())?;
    McpService::set_enabled(&state, app_ty, &id, enabled).map_err(|e| e.to_string())
}

// ============================================================================
// v3.7.0 新增：统一 MCP 管理命令
// ============================================================================

use cc_switch_core::app_config::McpServer;

/// 获取所有 MCP 服务器（统一结构）
#[tauri::command]
pub async fn get_mcp_servers(
    state: State<'_, AppState>,
) -> Result<IndexMap<String, McpServer>, String> {
    McpService::get_all_servers(&state).map_err(|e| e.to_string())
}

/// 添加或更新 MCP 服务器
#[tauri::command]
pub async fn upsert_mcp_server(
    state: State<'_, AppState>,
    server: McpServer,
) -> Result<(), String> {
    McpService::upsert_server(&state, server).map_err(|e| e.to_string())
}

/// 删除 MCP 服务器
#[tauri::command]
pub async fn delete_mcp_server(state: State<'_, AppState>, id: String) -> Result<bool, String> {
    McpService::delete_server(&state, &id).map_err(|e| e.to_string())
}

/// 切换 MCP 服务器在指定应用的启用状态
#[tauri::command]
pub async fn toggle_mcp_app(
    state: State<'_, AppState>,
    server_id: String,
    app: String,
    enabled: bool,
) -> Result<(), String> {
    let app_ty = AppType::from_str(&app).map_err(|e| e.to_string())?;
    McpService::toggle_app(&state, &server_id, app_ty, enabled).map_err(|e| e.to_string())
}

/// 从所有应用导入 MCP 服务器（复用已有的导入逻辑）
#[tauri::command]
pub async fn import_mcp_from_apps(state: State<'_, AppState>) -> Result<usize, String> {
    // 后端按应用 best-effort 导入：单个应用坏配置不阻断其余应用，
    // 但失败会被聚合并返回——坏文件不能再被 `unwrap_or(0)` 吞成"导入 0 个"。
    McpService::import_from_all_apps(&state).map_err(|e| e.to_string())
}

// ============================================================================
// MCP 连接测试
// ============================================================================

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConnectionTestResult {
    pub success: bool,
    /// "stdio" | "http" | "sse"
    pub transport: String,
    /// 测试耗时（毫秒）
    pub duration_ms: u64,
    /// 测试过程中收集到的消息
    pub message: Option<String>,
    /// stderr 摘要（仅 stdio 模式可能附带）
    pub stderr_tail: Option<String>,
    /// 错误信息
    pub error: Option<String>,
}

impl McpConnectionTestResult {
    fn ok(
        transport: &str,
        duration_ms: u64,
        message: Option<String>,
        stderr_tail: Option<String>,
    ) -> Self {
        Self {
            success: true,
            transport: transport.to_string(),
            duration_ms,
            message,
            stderr_tail,
            error: None,
        }
    }

    fn err(transport: &str, duration_ms: u64, error: String) -> Self {
        Self {
            success: false,
            transport: transport.to_string(),
            duration_ms,
            message: None,
            stderr_tail: None,
            error: Some(error),
        }
    }
}

/// 探测 stdio MCP 服务器：启动子进程并验证其能正常初始化。
///
/// 该命令会启动用户配置的 command+args，向 stdin 写入一个最小的
/// `initialize` 请求（与 MCP 协议一致），期望在限定时间内收到
/// 包含 `result.serverInfo` 的 `initialize` 响应或任何携带 jsonrpc 字段的
/// 响应。成功条件是子进程成功退出 0 或在超时后被干净杀掉。
#[tauri::command]
pub async fn test_mcp_connection(
    state: State<'_, AppState>,
    id: String,
) -> Result<McpConnectionTestResult, String> {
    let started = std::time::Instant::now();

    let server_opt = {
        let servers = state.db.get_all_mcp_servers().map_err(|e| e.to_string())?;
        servers.get(&id).cloned()
    };

    let Some(server) = server_opt else {
        return Ok(McpConnectionTestResult::err(
            "unknown",
            started.elapsed().as_millis() as u64,
            format!("未找到 ID 为 `{id}` 的 MCP 服务器"),
        ));
    };

    let spec = &server.server;
    let transport = match spec.get("type").and_then(|v| v.as_str()).unwrap_or("stdio") {
        "http" => "http",
        "sse" => "sse",
        _ => "stdio",
    };

    match transport {
        "http" | "sse" => test_http_transport(spec, transport, started).await,
        _ => test_stdio_transport(spec, started).await,
    }
}

async fn test_stdio_transport(
    spec: &serde_json::Value,
    started: std::time::Instant,
) -> Result<McpConnectionTestResult, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let command = spec
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "缺少 `command` 字段".to_string())?;
    let args: Vec<String> = spec
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let cwd = spec
        .get("cwd")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // 合并环境变量
    let env_map: std::collections::HashMap<String, String> = spec
        .get("env")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let mut cmd = Command::new(command);
    cmd.args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(dir) = cwd.as_ref() {
        if !dir.is_empty() {
            cmd.current_dir(dir);
        }
    }
    for (k, v) in &env_map {
        cmd.env(k, v);
    }

    // 启动并写入最小 initialize 请求
    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(err) => {
            return Ok(McpConnectionTestResult::err(
                "stdio",
                started.elapsed().as_millis() as u64,
                format!("无法启动命令 `{command}`: {err}"),
            ));
        }
    };

    let initialize_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "cc-switch",
                "version": "test"
            }
        }
    });
    let mut body = initialize_payload.to_string();
    body.push('\n');
    if let Some(stdin) = child.stdin.as_mut() {
        if let Err(err) = stdin.write_all(body.as_bytes()) {
            let _ = child.kill();
            return Ok(McpConnectionTestResult::err(
                "stdio",
                started.elapsed().as_millis() as u64,
                format!("写入 initialize 请求失败: {err}"),
            ));
        }
    }

    // 短暂等待并回收
    let wait_ms: u64 = 1500;
    let status_result = tauri::async_runtime::spawn_blocking(move || {
        // 使用 try_wait 轮询避免永久阻塞
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(wait_ms);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return Ok::<_, String>((Some(status), child)),
                Ok(None) => {
                    if std::time::Instant::now() >= deadline {
                        // 超时：杀掉进程并返回
                        let _ = child.kill();
                        let _ = child.wait();
                        return Ok((None, child));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(err) => return Err(format!("轮询子进程失败: {err}")),
            }
        }
    })
    .await
    .map_err(|e| format!("执行测试任务失败: {e}"))??;

    let (status, mut child) = status_result;
    let mut stderr_buf = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        use std::io::Read;
        let _ = stderr.read_to_string(&mut stderr_buf);
    }

    let elapsed = started.elapsed().as_millis() as u64;
    let stderr_tail: Option<String> = if stderr_buf.is_empty() {
        None
    } else {
        let trimmed = stderr_buf
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        Some(trimmed)
    };

    match status {
        Some(status) if status.success() => Ok(McpConnectionTestResult::ok(
            "stdio",
            elapsed,
            Some(format!("子进程已成功响应并退出，状态 {status}")),
            stderr_tail,
        )),
        Some(status) => Ok(McpConnectionTestResult::err(
            "stdio",
            elapsed,
            format!("子进程异常退出，状态 {status}"),
        )),
        None => {
            // 超时杀掉，但若 stderr 中包含 MCP 协议响应仍视为成功
            let looks_like_mcp = stderr_buf.contains("jsonrpc")
                || stderr_buf.contains("serverInfo")
                || stderr_buf.contains("capabilities");
            if looks_like_mcp {
                Ok(McpConnectionTestResult::ok(
                    "stdio",
                    elapsed,
                    Some("子进程在限定时间内产生了 MCP 协议输出，已自动终止".to_string()),
                    stderr_tail,
                ))
            } else {
                Ok(McpConnectionTestResult::err(
                    "stdio",
                    elapsed,
                    "子进程在 1.5s 内未退出，且未输出 MCP 协议内容".to_string(),
                ))
            }
        }
    }
}

async fn test_http_transport(
    spec: &serde_json::Value,
    transport: &str,
    started: std::time::Instant,
) -> Result<McpConnectionTestResult, String> {
    let url = spec
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("缺少 `url` 字段（{transport}）"))?;
    if url.is_empty() {
        return Ok(McpConnectionTestResult::err(
            transport,
            started.elapsed().as_millis() as u64,
            "`url` 不能为空".to_string(),
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    // 构造基本请求，附带 Authorization（如果有）
    let mut req = client.get(url);
    if let Some(headers) = spec.get("headers").and_then(|v| v.as_object()) {
        for (k, v) in headers {
            if let Some(value) = v.as_str() {
                req = req.header(k, value);
            }
        }
    }
    if let Some(bearer) = spec.get("bearer_token").and_then(|v| v.as_str()) {
        if !bearer.is_empty() {
            req = req.bearer_auth(bearer);
        }
    }

    let response_result = req.send().await;
    let elapsed = started.elapsed().as_millis() as u64;
    match response_result {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() || status.is_redirection() {
                Ok(McpConnectionTestResult::ok(
                    transport,
                    elapsed,
                    Some(format!(
                        "HTTP 响应 {} {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("")
                    )),
                    None,
                ))
            } else {
                Ok(McpConnectionTestResult::err(
                    transport,
                    elapsed,
                    format!(
                        "HTTP {} {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("")
                    ),
                ))
            }
        }
        Err(err) => Ok(McpConnectionTestResult::err(
            transport,
            elapsed,
            format!("HTTP 请求失败: {err}"),
        )),
    }
}

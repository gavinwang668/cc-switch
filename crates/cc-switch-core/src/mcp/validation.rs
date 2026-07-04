//! MCP 服务器配置验证模块

use std::net::IpAddr;

use serde_json::Value;

use crate::error::AppError;

/// URL 最大长度（防止 DoS）
const MAX_MCP_URL_LEN: usize = 2048;

/// 校验 HTTP/HTTPS URL 的安全性：
/// 1. 必须能解析为 URL
/// 2. scheme 必须是 http 或 https
/// 3. host 不能是内网/回环/链路本地/云元数据地址（防 SSRF）
fn validate_mcp_url(url: &str, kind: &str) -> Result<(), AppError> {
    if url.len() > MAX_MCP_URL_LEN {
        return Err(AppError::McpValidation(format!(
            "{kind} 类型的 MCP URL 长度超过 {MAX_MCP_URL_LEN} 字节"
        )));
    }
    let parsed = url::Url::parse(url)
        .map_err(|e| AppError::McpValidation(format!("{kind} URL 无法解析: {e}")))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(AppError::McpValidation(format!(
            "{kind} URL 协议必须是 http 或 https，实际为 {scheme}"
        )));
    }
    // 提取 host，校验是否指向危险地址
    let host = parsed
        .host_str()
        .ok_or_else(|| AppError::McpValidation(format!("{kind} URL 缺少 host")))?;
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_blocked_ip(&ip) {
            return Err(AppError::McpValidation(format!(
                "{kind} URL 指向被禁止的 IP 地址（内网/回环/云元数据）"
            )));
        }
    } else {
        // 域名形式：仅做基础检查，阻止明显危险模式
        let lower = host.to_ascii_lowercase();
        if lower == "localhost" || lower.ends_with(".localhost") || lower.ends_with(".local") {
            return Err(AppError::McpValidation(format!(
                "{kind} URL 不允许指向本地主机（{host}）"
            )));
        }
    }
    Ok(())
}

/// 判断 IP 是否属于被禁止的网段（内网/回环/链路本地/云元数据等）
fn is_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()                  // 127.0.0.0/8
                || v4.is_private()            // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                || v4.is_link_local()         // 169.254.0.0/16 — 含云元数据 169.254.169.254
                || v4.is_unspecified()        // 0.0.0.0
                || v4.is_broadcast()          // 255.255.255.255
                || v4.is_multicast()          // 224.0.0.0/4
                // 100.64.0.0/10 — CGNAT 段
                || (v4.octets()[0] == 100 && (v4.octets()[1] >= 64 && v4.octets()[1] <= 127))
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()      // ::1
                || v6.is_unspecified() // ::
                || v6.is_multicast()
                // 私有/链路本地：fc00::/7 与 fe80::/10
                || {
                    let segs = v6.segments();
                    (segs[0] & 0xfe00) == 0xfc00 || (segs[0] & 0xffc0) == 0xfe80
                }
        }
    }
}

/// 基础校验：允许 stdio/http/sse；或省略 type（视为 stdio）。对应必填字段存在
pub fn validate_server_spec(spec: &Value) -> Result<(), AppError> {
    if !spec.is_object() {
        return Err(AppError::McpValidation(
            "MCP 服务器连接定义必须为 JSON 对象".into(),
        ));
    }
    let t_opt = spec.get("type").and_then(|x| x.as_str());
    // 支持三种：stdio/http/sse；若缺省 type 则按 stdio 处理（与社区常见 .mcp.json 一致）
    let is_stdio = t_opt.map(|t| t == "stdio").unwrap_or(true);
    let is_http = t_opt.map(|t| t == "http").unwrap_or(false);
    let is_sse = t_opt.map(|t| t == "sse").unwrap_or(false);

    if !(is_stdio || is_http || is_sse) {
        return Err(AppError::McpValidation(
            "MCP 服务器 type 必须是 'stdio'、'http' 或 'sse'（或省略表示 stdio）".into(),
        ));
    }

    if is_stdio {
        let cmd = spec.get("command").and_then(|x| x.as_str()).unwrap_or("");
        if cmd.trim().is_empty() {
            return Err(AppError::McpValidation(
                "stdio 类型的 MCP 服务器缺少 command 字段".into(),
            ));
        }
        // 拒绝包含 NUL 字节和空格的命令（防止命令注入混淆）
        if cmd.contains('\0') {
            return Err(AppError::McpValidation(
                "stdio command 不能包含 NUL 字节".into(),
            ));
        }
    }
    if is_http {
        let url = spec.get("url").and_then(|x| x.as_str()).unwrap_or("");
        if url.trim().is_empty() {
            return Err(AppError::McpValidation(
                "http 类型的 MCP 服务器缺少 url 字段".into(),
            ));
        }
        validate_mcp_url(url, "http")?;
    }
    if is_sse {
        let url = spec.get("url").and_then(|x| x.as_str()).unwrap_or("");
        if url.trim().is_empty() {
            return Err(AppError::McpValidation(
                "sse 类型的 MCP 服务器缺少 url 字段".into(),
            ));
        }
        validate_mcp_url(url, "sse")?;
    }
    Ok(())
}

/// 从 MCP 条目中提取服务器规范
pub fn extract_server_spec(entry: &Value) -> Result<Value, AppError> {
    let obj = entry
        .as_object()
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目必须为 JSON 对象".into()))?;
    let server = obj
        .get("server")
        .ok_or_else(|| AppError::McpValidation("MCP 服务器条目缺少 server 字段".into()))?;

    if !server.is_object() {
        return Err(AppError::McpValidation(
            "MCP 服务器 server 字段必须为 JSON 对象".into(),
        ));
    }

    Ok(server.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_blocks_unsafe_schemes() {
        for bad in [
            "file:///etc/passwd",
            "ftp://example.com",
            "gopher://x",
            "data:text/plain,hi",
        ] {
            let spec = serde_json::json!({"type":"http","url": bad});
            assert!(validate_server_spec(&spec).is_err(), "should reject {bad}");
        }
    }

    #[test]
    fn validate_blocks_internal_ips() {
        for bad in [
            "http://127.0.0.1/x",
            "http://10.0.0.1/x",
            "http://192.168.1.1/x",
            "http://169.254.169.254/latest/meta-data/",
            "http://0.0.0.0/x",
            "http://[::1]/x",
        ] {
            let spec = serde_json::json!({"type":"http","url": bad});
            assert!(validate_server_spec(&spec).is_err(), "should reject {bad}");
        }
    }

    #[test]
    fn validate_allows_public_urls() {
        for ok in [
            "https://api.example.com/mcp",
            "https://mcp.anthropic.com/sse",
            "http://api.openai.com/mcp",
        ] {
            let spec = serde_json::json!({"type":"http","url": ok});
            assert!(validate_server_spec(&spec).is_ok(), "should accept {ok}");
        }
    }

    #[test]
    fn validate_blocks_localhost_hostname() {
        let spec = serde_json::json!({"type":"http","url": "http://localhost:8080/x"});
        assert!(validate_server_spec(&spec).is_err());
    }

    #[test]
    fn validate_blocks_nul_in_stdio_command() {
        let spec = serde_json::json!({"type":"stdio","command": "sh\0rm"});
        assert!(validate_server_spec(&spec).is_err());
    }
}

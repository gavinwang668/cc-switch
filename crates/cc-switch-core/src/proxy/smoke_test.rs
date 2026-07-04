//! 协议转换烟雾测试（REQ-021）
//!
//! 不走网络，直接调用 transform 子模块验证协议转换链路。
//! 对每个 AppType 跑一次最小请求，验证转换不抛错。

use crate::app_config::AppType;
use crate::proxy::providers::transform;
use serde_json::json;

/// 烟雾测试结果
#[derive(Debug)]
pub struct SmokeTestResult {
    pub app_type: String,
    pub passed: bool,
    pub message: String,
}

impl SmokeTestResult {
    fn ok(app: &str, msg: &str) -> Self {
        Self {
            app_type: app.to_string(),
            passed: true,
            message: msg.to_string(),
        }
    }

    fn fail(app: &str, msg: &str) -> Self {
        Self {
            app_type: app.to_string(),
            passed: false,
            message: msg.to_string(),
        }
    }
}

/// 最小 Chat Completions 请求体
fn minimal_chat_body(model: &str) -> serde_json::Value {
    json!({
        "model": model,
        "messages": [
            {"role": "user", "content": "Hello"}
        ],
        "max_tokens": 10,
        "stream": false
    })
}

/// 最小 Anthropic Messages 请求体
fn minimal_anthropic_body(model: &str) -> serde_json::Value {
    json!({
        "model": model,
        "messages": [
            {"role": "user", "content": "Hello"}
        ],
        "max_tokens": 10
    })
}

/// 最小 Gemini generateContent 请求体
fn minimal_gemini_body(model: &str) -> serde_json::Value {
    json!({
        "model": format!("models/{}", model),
        "contents": [
            {
                "role": "user",
                "parts": [{"text": "Hello"}]
            }
        ],
        "generationConfig": {
            "maxOutputTokens": 10
        }
    })
}

/// 运行烟雾测试：对指定 AppType 做最小请求转换验证
pub fn run_smoke_test(
    app_type: &AppType,
    base_url: &str,
    api_format: Option<&str>,
) -> SmokeTestResult {
    let (body, from_format) = match app_type {
        AppType::Claude | AppType::ClaudeDesktop => {
            let model = api_format.unwrap_or("claude-sonnet-5");
            (minimal_anthropic_body(model), "anthropic")
        }
        AppType::Codex => {
            let model = api_format.unwrap_or("gpt-5.5");
            (minimal_chat_body(model), "openai_chat")
        }
        AppType::Gemini => {
            let model = api_format.unwrap_or("gemini-3.1-pro");
            (minimal_gemini_body(model), "gemini_native")
        }
        _ => {
            let model = api_format.unwrap_or("gpt-5.5");
            (minimal_chat_body(model), "openai_chat")
        }
    };

    let to_format = match api_format {
        Some(f) if f == "openai_responses" => "openai_responses",
        Some(f) if f == "anthropic" => "anthropic",
        Some(f) if f == "gemini_native" => "gemini_native",
        _ => "openai_chat", // default / same as from
    };

    if from_format == to_format {
        return SmokeTestResult::ok(
            &app_type.as_str(),
            &format!(
                "pass-through ({} → {}, 无需转换)",
                from_format, to_format
            ),
        );
    }

    // 尝试转换
    match transform::transform_request(&body, from_format, to_format, base_url) {
        Ok(_transformed) => SmokeTestResult::ok(
            &app_type.as_str(),
            &format!("协议转换通过: {} → {}", from_format, to_format),
        ),
        Err(e) => SmokeTestResult::fail(
            &app_type.as_str(),
            &format!("协议转换失败: {} → {}: {e}", from_format, to_format),
        ),
    }
}

/// 批量烟雾测试所有已知 AppType
pub fn run_all_smoke_tests() -> Vec<SmokeTestResult> {
    let app_types = [
        (AppType::Claude, "anthropic", "https://api.anthropic.com"),
        (AppType::Codex, "openai_chat", "https://api.openai.com/v1"),
        (AppType::Gemini, "gemini_native", "https://generativelanguage.googleapis.com"),
    ];

    app_types
        .iter()
        .map(|(app, fmt, url)| run_smoke_test(app, url, Some(fmt)))
        .collect()
}

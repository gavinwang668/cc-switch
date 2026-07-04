//! 协议转换烟雾测试（REQ-021）
//!
//! 不走网络，直接调用 transform 子模块验证协议转换链路。
//! 对每个 AppType 跑一次最小请求，验证转换不抛错。

use crate::proxy::providers::transform;
use serde_json::json;

/// 烟雾测试结果
#[derive(Debug)]
pub struct SmokeTestResult {
    pub app_type: String,
    pub passed: bool,
    pub message: String,
}

/// 运行单个协议的烟雾测试
pub fn run_smoke_test(
    from_fmt: &str,
    to_fmt: &str,
    model: &str,
) -> SmokeTestResult {
    let body = minimal_body(from_fmt, model);

    if from_fmt == to_fmt {
        return SmokeTestResult {
            app_type: format!("{from_fmt} → {to_fmt}"),
            passed: true,
            message: format!("pass-through（无需转换，{model}）"),
        };
    }

    let result = match (from_fmt, to_fmt) {
        ("anthropic", "openai_chat") => transform::anthropic_to_openai(body),
        ("openai_chat", "anthropic") => transform::openai_to_anthropic(body),
        _ => {
            return SmokeTestResult {
                app_type: format!("{from_fmt} → {to_fmt}"),
                passed: false,
                message: format!("不支持的转换路径: {from_fmt} → {to_fmt}"),
            };
        }
    };

    match result {
        Ok(_) => SmokeTestResult {
            app_type: format!("{from_fmt} → {to_fmt}"),
            passed: true,
            message: format!("协议转换通过（{model}）"),
        },
        Err(e) => SmokeTestResult {
            app_type: format!("{from_fmt} → {to_fmt}"),
            passed: false,
            message: format!("转换失败: {e}"),
        },
    }
}

fn minimal_body(format: &str, model: &str) -> serde_json::Value {
    match format {
        "anthropic" => json!({
            "model": model,
            "messages": [{"role": "user", "content": "Hello"}],
            "max_tokens": 10
        }),
        _ => json!({
            "model": model,
            "messages": [{"role": "user", "content": "Hello"}],
            "max_tokens": 10
        }),
    }
}

/// 批量烟雾测试（常见转换路径）
pub fn run_all_smoke_tests() -> Vec<SmokeTestResult> {
    vec![
        run_smoke_test("anthropic", "openai_chat", "claude-sonnet-5"),
        run_smoke_test("openai_chat", "anthropic", "gpt-5.5"),
        run_smoke_test("openai_chat", "openai_chat", "gpt-5.5"),
        run_smoke_test("anthropic", "anthropic", "claude-sonnet-5"),
    ]
}

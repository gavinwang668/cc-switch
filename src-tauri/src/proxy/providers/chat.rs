//! OpenAI Chat Completions Provider Adapter
//!
//! 处理 OpenAI Chat Completions 格式客户端请求，支持向所有后端格式的转换。
//!
//! ## API 格式
//! - "openai_chat" (默认): OpenAI Chat Completions 格式，直接透传
//! - "anthropic": Anthropic Messages API 格式，需要格式转换
//! - "openai_responses": OpenAI Responses API 格式，需要格式转换
//! - "gemini_native": Google Gemini Native generateContent 格式，需要格式转换
//! - "bedrock_native": Amazon Bedrock Converse API 格式，需要格式转换
//!
//! ## 认证模式
//! - 默认 Bearer token，从 OPENAI_API_KEY 读取

use super::{AuthInfo, AuthStrategy, ProviderAdapter};
use crate::provider::Provider;
use crate::proxy::error::ProxyError;
use serde_json::Value;

/// 获取 Chat 供应商的 API 格式
///
/// 优先级：
/// 1. meta.api_format (SSOT)
/// 2. settings_config.api_format
/// 3. 默认 "openai_chat"
pub fn get_chat_api_format(provider: &Provider) -> &'static str {
    if let Some(meta) = provider.meta.as_ref() {
        if let Some(api_format) = meta.api_format.as_deref() {
            return match api_format {
                "openai_chat" => "openai_chat",
                "anthropic" => "anthropic",
                "openai_responses" => "openai_responses",
                "gemini_native" => "gemini_native",
                "bedrock_native" => "bedrock_native",
                _ => "openai_chat",
            };
        }
    }

    if let Some(api_format) = provider
        .settings_config
        .get("api_format")
        .and_then(|v| v.as_str())
    {
        return match api_format {
            "openai_chat" => "openai_chat",
            "anthropic" => "anthropic",
            "openai_responses" => "openai_responses",
            "gemini_native" => "gemini_native",
            "bedrock_native" => "bedrock_native",
            _ => "openai_chat",
        };
    }

    "openai_chat"
}

/// Chat 适配器
pub struct ChatAdapter;

impl ChatAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Get the API format string for a provider
    fn get_api_format(&self, provider: &Provider) -> &'static str {
        get_chat_api_format(provider)
    }
}

impl ProviderAdapter for ChatAdapter {
    fn name(&self) -> &'static str {
        "Chat"
    }

    fn extract_base_url(&self, provider: &Provider) -> Result<String, ProxyError> {
        // 读取 OPENAI_BASE_URL
        let env = &provider.settings_config;
        if let Some(base_url) = env
            .get("env")
            .and_then(|e| e.get("OPENAI_BASE_URL"))
            .and_then(|v| v.as_str())
        {
            return Ok(base_url.to_string());
        }
        // 回退到默认 OpenAI API 地址
        Ok("https://api.openai.com".to_string())
    }

    fn extract_auth(&self, provider: &Provider) -> Option<AuthInfo> {
        let env = &provider.settings_config;
        let key = env
            .get("env")
            .and_then(|e| e.get("OPENAI_API_KEY"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                env.get("env")
                    .and_then(|e| e.get("ANTHROPIC_API_KEY"))
                    .and_then(|v| v.as_str())
            })
            .or_else(|| {
                env.get("env")
                    .and_then(|e| e.get("GEMINI_API_KEY"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("");

        if key.is_empty() {
            return None;
        }

        // 根据 api_format 选择认证方式
        let api_format = self.get_api_format(provider);
        let strategy = match api_format {
            "gemini_native" => {
                if key.starts_with("ya29.") || key.starts_with('{') {
                    AuthStrategy::GoogleOAuth
                } else {
                    AuthStrategy::Google
                }
            }
            "anthropic" => {
                // 检测是否为中转服务 (仅 Bearer)
                if env
                    .get("auth_mode")
                    .and_then(|v| v.as_str())
                    == Some("bearer_only")
                {
                    AuthStrategy::ClaudeAuth
                } else {
                    AuthStrategy::Anthropic
                }
            }
            _ => AuthStrategy::Bearer,
        };

        Some(AuthInfo::new(key.to_string(), strategy))
    }

    fn build_url(&self, base_url: &str, endpoint: &str) -> String {
        let base_trimmed = base_url.trim_end_matches('/');
        let endpoint_trimmed = endpoint.trim_start_matches('/');

        // Chat 的 base_url 可能是：
        // - 纯 origin: https://api.openai.com (需要自动补 /v1)
        // - 已含 /v1: https://api.openai.com/v1 (直接拼接)
        let already_has_v1 = base_trimmed.ends_with("/v1");

        let mut url = if already_has_v1 {
            format!("{base_trimmed}/{endpoint_trimmed}")
        } else {
            // 检查 base_url 是否看起来像 OpenAI 风格的 origin
            let is_origin = !base_trimmed.contains("/v1/")
                && base_trimmed
                    .chars()
                    .filter(|&c| c == '/')
                    .count()
                    <= 2; // http://host or https://host or https://host/path
            if is_origin {
                format!("{base_trimmed}/v1/{endpoint_trimmed}")
            } else {
                format!("{base_trimmed}/{endpoint_trimmed}")
            }
        };

        // 去除重复的 /v1/v1
        while url.contains("/v1/v1") {
            url = url.replace("/v1/v1", "/v1");
        }

        url
    }

    fn get_auth_headers(
        &self,
        auth: &AuthInfo,
    ) -> Result<Vec<(http::HeaderName, http::HeaderValue)>, ProxyError> {
        use super::adapter::auth_header_value as hv;
        use http::{HeaderName, HeaderValue};
        let bearer = format!("Bearer {}", auth.api_key);
        Ok(match auth.strategy {
            AuthStrategy::Google => vec![(
                HeaderName::from_static("x-goog-api-key"),
                hv(&auth.api_key)?,
            )],
            AuthStrategy::GoogleOAuth => {
                let token = auth.access_token.as_ref().unwrap_or(&auth.api_key);
                vec![
                    (
                        HeaderName::from_static("authorization"),
                        hv(&format!("Bearer {token}"))?,
                    ),
                    (
                        HeaderName::from_static("x-goog-api-client"),
                        HeaderValue::from_static("GeminiCLI/1.0"),
                    ),
                ]
            }
            AuthStrategy::Anthropic => {
                vec![(HeaderName::from_static("x-api-key"), hv(&auth.api_key)?)]
            }
            AuthStrategy::ClaudeAuth => {
                vec![(HeaderName::from_static("authorization"), hv(&bearer)?)]
            }
            _ => vec![(
                HeaderName::from_static("authorization"),
                hv(&bearer)?,
            )],
        })
    }

    fn needs_transform(&self, provider: &Provider) -> bool {
        let api_format = self.get_api_format(provider);
        api_format != "openai_chat"
    }

    fn transform_request(
        &self,
        body: Value,
        provider: &Provider,
    ) -> Result<Value, ProxyError> {
        let api_format = self.get_api_format(provider);

        match api_format {
            "openai_chat" => Ok(body),
            "anthropic" => {
                // Chat Completions → Anthropic Messages
                super::transform::openai_to_anthropic(body)
            }
            "openai_responses" => {
                // Chat Completions → Responses
                super::transform_codex_chat::chat_completion_to_response(body)
            }
            "gemini_native" => {
                // Chat Completions → Anthropic → Gemini
                let anthropic = super::transform::openai_to_anthropic(body)?;
                super::transform_gemini::anthropic_to_gemini(anthropic)
            }
            "bedrock_native" => {
                // Chat Completions → Anthropic → Bedrock
                let anthropic = super::transform::openai_to_anthropic(body)?;
                super::transform_bedrock::anthropic_to_bedrock(anthropic)
            }
            _ => Ok(body),
        }
    }

    fn transform_response(&self, body: Value) -> Result<Value, ProxyError> {
        // 检测响应格式并转换回 OpenAI Chat Completions 格式
        if body.get("candidates").is_some() || body.get("promptFeedback").is_some() {
            // Gemini → Anthropic → Chat
            let anthropic = super::transform_gemini::gemini_to_anthropic(body)?;
            super::transform::anthropic_to_openai(anthropic)
        } else if body.get("stopReason").is_some()
            || body.get("modelId").is_some()
            || body.get("contentBlockDelta").is_some()
            || body.get("contentBlockStart").is_some()
        {
            // Bedrock → Anthropic → Chat
            let anthropic = super::transform_bedrock::bedrock_to_anthropic(body)?;
            super::transform::anthropic_to_openai(anthropic)
        } else if body.get("type").is_some() && body.get("content").is_some() {
            // Anthropic → Chat
            super::transform::anthropic_to_openai(body)
        } else if body.get("output").is_some() {
            // Responses → Anthropic → Chat
            let anthropic = super::transform_responses::responses_to_anthropic(body)?;
            super::transform::anthropic_to_openai(anthropic)
        } else {
            // 已经是 Chat Completions 格式
            Ok(body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_provider(config: Value) -> Provider {
        Provider {
            id: "test-chat".to_string(),
            name: "Test Chat Provider".to_string(),
            settings_config: config,
            website_url: None,
            category: None,
            created_at: None,
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    #[test]
    fn test_get_chat_api_format_default() {
        let provider = create_provider(json!({
            "env": { "OPENAI_API_KEY": "sk-test" }
        }));
        assert_eq!(get_chat_api_format(&provider), "openai_chat");
    }

    #[test]
    fn test_get_chat_api_format_anthropic() {
        let provider = create_provider(json!({
            "env": { "OPENAI_API_KEY": "sk-test" },
            "api_format": "anthropic"
        }));
        assert_eq!(get_chat_api_format(&provider), "anthropic");
    }

    #[test]
    fn test_chat_adapter_needs_transform() {
        let adapter = ChatAdapter::new();

        let chat_provider = create_provider(json!({
            "env": { "OPENAI_API_KEY": "sk-test" }
        }));
        assert!(!adapter.needs_transform(&chat_provider));

        let anthropic_provider = create_provider(json!({
            "env": { "OPENAI_API_KEY": "sk-test" },
            "api_format": "anthropic"
        }));
        assert!(adapter.needs_transform(&anthropic_provider));
    }

    #[test]
    fn test_chat_adapter_name() {
        let adapter = ChatAdapter::new();
        assert_eq!(adapter.name(), "Chat");
    }
}

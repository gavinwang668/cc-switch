
use serde_json::{json, Value};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderMeta {
    #[serde(rename = "apiFormat", skip_serializing_if = "Option::is_none")]
    pub api_format: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Provider {
    pub meta: Option<ProviderMeta>,
    pub settings_config: Value,
}

fn is_chat_wire_api(api_format: &str) -> bool {
    api_format == "openai_chat"
}

fn codex_provider_uses_chat_completions(provider: &Provider) -> bool {
    println!("Testing provider: {:?}", provider);
    
    if let Some(api_format) = provider
        .meta
        .as_ref()
        .and_then(|meta| meta.api_format.as_deref())
        .or_else(|| {
            provider
                .settings_config
                .get("api_format")
                .and_then(|v| v.as_str())
        })
        .or_else(|| {
            provider
                .settings_config
                .get("apiFormat")
                .and_then(|v| v.as_str())
        })
    {
        println!("Found api_format: {}", api_format);
        return is_chat_wire_api(api_format);
    }
    false
}

fn main() {
    // 测试 1: 只有 meta.api_format
    let p1 = Provider {
        meta: Some(ProviderMeta {
            api_format: Some("openai_chat".to_string()),
        }),
        settings_config: json!({}),
    };
    let result1 = codex_provider_uses_chat_completions(&p1);
    println!("Test 1 result (meta with api_format): {}", result1);

    // 测试 2: 只有 settings_config.api_format
    let p2 = Provider {
        meta: None,
        settings_config: json!({"api_format": "openai_chat"}),
    };
    let result2 = codex_provider_uses_chat_completions(&p2);
    println!("Test 2 result (settings_config api_format): {}", result2);

    // 测试 3: 只有 settings_config.apiFormat (驼峰)
    let p3 = Provider {
        meta: None,
        settings_config: json!({"apiFormat": "openai_chat"}),
    };
    let result3 = codex_provider_uses_chat_completions(&p3);
    println!("Test 3 result (settings_config apiFormat): {}", result3);
}

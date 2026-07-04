//! AWS Bedrock SigV4 Request Signing
//!
//! 为 Bedrock Converse API 请求添加 AWS Signature Version 4 认证。
//! 使用纯 Rust 实现（hmac + sha2），无需依赖 AWS SDK。
//! 从 provider.settings_config.env 中读取 AWS 凭证，
//! 对 HTTP 请求进行 SigV4 签名后发送到 Bedrock 端点。

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use http::HeaderMap;
use std::str::FromStr;

type HmacSha256 = Hmac<Sha256>;

/// AWS 凭证信息
#[derive(Debug, Clone)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub region: String,
}

impl AwsCredentials {
    /// 从 Provider 的 settings_config.env 中提取 AWS 凭证
    ///
    /// 支持的环境变量名：
    /// - `AWS_ACCESS_KEY_ID` / `aws_access_key_id`
    /// - `AWS_SECRET_ACCESS_KEY` / `aws_secret_access_key`
    /// - `AWS_SESSION_TOKEN` / `aws_session_token`（可选，用于临时凭证）
    /// - `AWS_REGION` / `aws_region`
    pub fn from_env_config(provider: &crate::provider::Provider) -> Option<Self> {
        let env = provider.settings_config.get("env")?;

        let get_str = |keys: &[&str]| -> Option<String> {
            for key in keys {
                if let Some(val) = env.get(key).and_then(|v| v.as_str()) {
                    let trimmed = val.trim();
                    if !trimmed.is_empty() {
                        // 跳过模板占位符 ${...}
                        if trimmed.starts_with("${") && trimmed.ends_with('}') {
                            continue;
                        }
                        return Some(trimmed.to_string());
                    }
                }
            }
            None
        };

        let access_key_id = get_str(&[
            "AWS_ACCESS_KEY_ID",
            "aws_access_key_id",
        ])?;

        let secret_access_key = get_str(&[
            "AWS_SECRET_ACCESS_KEY",
            "aws_secret_access_key",
        ])?;

        let region = get_str(&[
            "AWS_REGION",
            "aws_region",
        ])
        .unwrap_or_else(|| "us-east-1".to_string());

        let session_token = get_str(&[
            "AWS_SESSION_TOKEN",
            "aws_session_token",
        ]);

        Some(Self {
            access_key_id,
            secret_access_key,
            session_token,
            region,
        })
    }
}

/// 检测是否为 Bedrock 提供商
///
/// 两个判定条件（任一满足即可）：
/// 1. settings_config.env.CLAUDE_CODE_USE_BEDROCK == "1"
/// 2. meta.api_format == "bedrock_native"
pub fn is_bedrock_provider(provider: &crate::provider::Provider) -> bool {
    if provider
        .settings_config
        .get("env")
        .and_then(|e| e.get("CLAUDE_CODE_USE_BEDROCK"))
        .and_then(|v| v.as_str())
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        return true;
    }

    if let Some(ref meta) = provider.meta {
        if meta.api_format.as_deref() == Some("bedrock_native") {
            return true;
        }
    }

    false
}

/// 从 Provider 中提取 AWS 凭证
pub fn get_aws_credentials(provider: &crate::provider::Provider) -> Option<AwsCredentials> {
    AwsCredentials::from_env_config(provider)
}

// ═══════════════════════════════════════════════════════════
// SigV4 核心实现（纯 Rust，无外部依赖）
// ═══════════════════════════════════════════════════════════

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("valid HMAC key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn derive_signing_key(secret: &str, date_stamp: &str, region: &str, service: &str) -> Vec<u8> {
    let secret_key = format!("AWS4{}", secret);
    let k_date = hmac_sha256(secret_key.as_bytes(), date_stamp.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    hmac_sha256(&k_service, b"aws4_request")
}

/// 从完整 URL 中提取规范化的 URI 路径
fn canonical_uri(url: &str) -> String {
    let uri: http::Uri = url.parse().unwrap_or_else(|_| {
        http::Uri::from_str("/").unwrap()
    });
    let path = uri.path();
    if path.is_empty() { "/".to_string() } else { path.to_string() }
}

/// 从 URL 中提取规范化的查询字符串（按键排序）
fn canonical_query_string(url: &str) -> String {
    let uri: http::Uri = match url.parse() {
        Ok(u) => u,
        Err(_) => return String::new(),
    };
    let query = match uri.query() {
        Some(q) => q,
        None => return String::new(),
    };
    let mut pairs: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    pairs.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
}

/// 构建规范化请求头字符串和签名头列表
///
/// 返回 `(canonical_headers_string, signed_headers_string)`
fn build_canonical_headers(all: &[(String, String)]) -> (String, String) {
    let mut normalized: Vec<(String, String)> = all.iter()
        .map(|(k, v)| (k.to_lowercase(), v.trim().to_string()))
        .collect();
    normalized.sort_by(|a, b| a.0.cmp(&b.0));

    let canon = normalized.iter()
        .map(|(k, v)| format!("{}:{}", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    let signed = normalized.iter()
        .map(|(k, _)| k.clone())
        .collect::<Vec<_>>()
        .join(";");

    (canon + "\n", signed)
}

/// 对 HTTP 请求进行 SigV4 签名
///
/// # 参数
/// * `method` - HTTP 方法（GET, POST 等）
/// * `url` - 完整的请求 URL
/// * `headers` - 请求头（名称-值对）
/// * `body` - 请求体
/// * `creds` - AWS 凭证
///
/// # 返回
/// 签名后的 headers（包含 Authorization, X-Amz-Date, X-Amz-Security-Token 等）
pub fn sign_request(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: &[u8],
    creds: &AwsCredentials,
) -> Result<Vec<(String, String)>, String> {
    let now = Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = now.format("%Y%m%d").to_string();

    // 收集并补全必须的 headers
    let mut all: Vec<(String, String)> = headers.to_vec();

    let has_content_type = all.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
    if !has_content_type {
        all.push(("content-type".to_string(), "application/json".to_string()));
    }

    let has_host = all.iter().any(|(k, _)| k.eq_ignore_ascii_case("host"));
    if !has_host {
        let uri: http::Uri = url.parse().map_err(|e| format!("Invalid URL: {e}"))?;
        if let Some(host) = uri.host() {
            let val = match uri.port_u16() {
                Some(p) if p != 443 && p != 80 => format!("{}:{}", host, p),
                _ => host.to_string(),
            };
            all.push(("host".to_string(), val));
        }
    }

    all.push(("x-amz-date".to_string(), amz_date.clone()));

    if let Some(ref token) = creds.session_token {
        all.push(("x-amz-security-token".to_string(), token.clone()));
    }

    // ── Step 1: Canonical Request ──────────────────────
    let payload_hash = sha256_hex(body);
    let (canon_headers_str, signed_headers_str) = build_canonical_headers(&all);

    let canonical_request = format!(
        "{}\n{}\n{}\n{}{}\n{}",
        method.to_uppercase(),
        canonical_uri(url),
        canonical_query_string(url),
        canon_headers_str,
        signed_headers_str,
        payload_hash,
    );

    // ── Step 2: String to Sign ──────────────────────────
    let algorithm = "AWS4-HMAC-SHA256";
    let credential_scope = format!("{}/{}/{}/aws4_request", date_stamp, creds.region, "bedrock");
    let canonical_request_hash = sha256_hex(canonical_request.as_bytes());

    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        algorithm,
        amz_date,
        credential_scope,
        canonical_request_hash,
    );

    // ── Step 3: Derive Signing Key ──────────────────────
    let signing_key = derive_signing_key(
        &creds.secret_access_key,
        &date_stamp,
        &creds.region,
        "bedrock",
    );

    // ── Step 4: Calculate Signature ─────────────────────
    let signature = hex::encode(hmac_sha256(&signing_key, string_to_sign.as_bytes()));

    // ── Step 5: Build Authorization Header ──────────────
    let authorization = format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        algorithm,
        creds.access_key_id,
        credential_scope,
        signed_headers_str,
        signature,
    );

    all.push(("Authorization".to_string(), authorization));

    Ok(all)
}

/// 对 HTTP 请求进行 SigV4 签名（使用 http::HeaderMap 版本）
///
/// 专为 forwarder.rs 调用设计，直接操作 http 类型。
///
/// # 参数
/// * `method` - HTTP 方法
/// * `url` - 完整的请求 URL
/// * `headers` - 原始请求头
/// * `body` - 请求体字节
/// * `creds` - AWS 凭证
///
/// # 返回
/// 签名后的完整 HeaderMap（包含 Authorization, X-Amz-Date 等）
pub fn sign_http_request(
    method: &http::Method,
    url: &str,
    headers: &HeaderMap,
    body: &[u8],
    creds: &AwsCredentials,
) -> Result<HeaderMap, String> {
    // 将 HeaderMap 转为 Vec<(String, String)>
    let pairs: Vec<(String, String)> = headers.iter()
        .filter_map(|(name, value)| {
            value.to_str().ok().map(|v| (name.to_string(), v.to_string()))
        })
        .collect();

    let signed = sign_request(method.as_str(), url, &pairs, body, creds)?;

    let mut result = HeaderMap::new();
    for (name, value) in signed {
        let header_name = http::HeaderName::from_str(&name)
            .map_err(|e| format!("Invalid header name '{}': {}", name, e))?;
        let header_value = http::HeaderValue::from_str(&value)
            .map_err(|e| format!("Invalid header value for '{}': {}", value, e))?;
        result.insert(header_name, header_value);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{Provider, ProviderMeta};
    use serde_json::json;

    fn create_bedrock_provider(env: serde_json::Value) -> Provider {
        Provider {
            id: "test-bedrock".to_string(),
            name: "Test Bedrock".to_string(),
            settings_config: json!({ "env": env }),
            website_url: None,
            category: Some("cloud_provider".to_string()),
            created_at: None,
            sort_index: None,
            notes: None,
            meta: Some(ProviderMeta {
                api_format: Some("bedrock_native".to_string()),
                ..Default::default()
            }),
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    #[test]
    fn test_is_bedrock_provider() {
        let provider = create_bedrock_provider(json!({
            "CLAUDE_CODE_USE_BEDROCK": "1"
        }));
        assert!(is_bedrock_provider(&provider));

        let non_bedrock = create_bedrock_provider(json!({
            "ANTHROPIC_API_KEY": "sk-test"
        }));
        assert!(!is_bedrock_provider(&non_bedrock));
    }

    #[test]
    fn test_aws_credentials_from_env_config() {
        let provider = create_bedrock_provider(json!({
            "AWS_ACCESS_KEY_ID": "AKIAIOSFODNN7EXAMPLE",
            "AWS_SECRET_ACCESS_KEY": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "AWS_REGION": "us-west-2"
        }));

        let creds = AwsCredentials::from_env_config(&provider).unwrap();
        assert_eq!(creds.access_key_id, "AKIAIOSFODNN7EXAMPLE");
        assert_eq!(
            creds.secret_access_key,
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
        );
        assert_eq!(creds.region, "us-west-2");
        assert!(creds.session_token.is_none());
    }

    #[test]
    fn test_aws_credentials_template_placeholders_skipped() {
        let provider = create_bedrock_provider(json!({
            "AWS_ACCESS_KEY_ID": "${AWS_ACCESS_KEY_ID}",
            "AWS_SECRET_ACCESS_KEY": "${AWS_SECRET_ACCESS_KEY}",
            "AWS_REGION": "us-west-2"
        }));

        let creds = AwsCredentials::from_env_config(&provider);
        assert!(creds.is_none(), "Template placeholders should be skipped");
    }

    #[test]
    fn test_aws_credentials_with_session_token() {
        let provider = create_bedrock_provider(json!({
            "AWS_ACCESS_KEY_ID": "AKIAIOSFODNN7EXAMPLE",
            "AWS_SECRET_ACCESS_KEY": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "AWS_SESSION_TOKEN": "FwoGZXIvYXdzEBYaDNHX",
            "AWS_REGION": "eu-west-1"
        }));

        let creds = AwsCredentials::from_env_config(&provider).unwrap();
        assert_eq!(
            creds.session_token,
            Some("FwoGZXIvYXdzEBYaDNHX".to_string())
        );
        assert_eq!(creds.region, "eu-west-1");
    }

    #[test]
    fn test_aws_credentials_missing_access_key() {
        let provider = create_bedrock_provider(json!({
            "AWS_SECRET_ACCESS_KEY": "secret",
            "AWS_REGION": "us-east-1"
        }));

        let creds = AwsCredentials::from_env_config(&provider);
        assert!(creds.is_none(), "Missing access key should return None");
    }

    #[test]
    fn test_aws_credentials_default_region() {
        let provider = create_bedrock_provider(json!({
            "AWS_ACCESS_KEY_ID": "AKIAIOSFODNN7EXAMPLE",
            "AWS_SECRET_ACCESS_KEY": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
        }));

        let creds = AwsCredentials::from_env_config(&provider).unwrap();
        assert_eq!(creds.region, "us-east-1", "Default region should be us-east-1");
    }

    #[test]
    fn test_sign_request_produces_auth_header() {
        let creds = AwsCredentials {
            access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            session_token: None,
            region: "us-east-1".to_string(),
        };

        let headers = vec![
            ("content-type".to_string(), "application/json".to_string()),
            ("host".to_string(), "bedrock-runtime.us-east-1.amazonaws.com".to_string()),
        ];

        let body = b"{}";

        let result = sign_request(
            "POST",
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/test/converse-stream",
            &headers,
            body,
            &creds,
        );

        assert!(result.is_ok(), "Signing should succeed: {:?}", result.err());
        let signed_headers = result.unwrap();

        // Should contain Authorization header
        let has_auth = signed_headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("authorization"));
        assert!(has_auth, "Signed headers should contain Authorization");

        // Should contain X-Amz-Date header
        let has_date = signed_headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("x-amz-date"));
        assert!(has_date, "Signed headers should contain X-Amz-Date");

        // Should contain Content-Type and Host (preserved from input)
        let has_content_type = signed_headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("content-type"));
        assert!(has_content_type, "Signed headers should contain Content-Type");

        let has_host = signed_headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("host"));
        assert!(has_host, "Signed headers should contain Host");
    }

    #[test]
    fn test_sign_request_signature_format() {
        let creds = AwsCredentials {
            access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            session_token: None,
            region: "us-east-1".to_string(),
        };

        let headers = vec![
            ("content-type".to_string(), "application/json".to_string()),
            ("host".to_string(), "bedrock-runtime.us-east-1.amazonaws.com".to_string()),
        ];

        let result = sign_request(
            "POST",
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/anthropic.claude-3-sonnet-20240229-v1:0/converse-stream",
            &headers,
            b"{}",
            &creds,
        );

        assert!(result.is_ok());
        let signed = result.unwrap();

        // Verify Authorization header format
        let auth = signed.iter()
            .find(|(n, _)| n == "Authorization")
            .expect("Should have Authorization header");

        assert!(auth.1.starts_with("AWS4-HMAC-SHA256 Credential="),
            "Authorization should start with algorithm and Credential");
        assert!(auth.1.contains("/aws4_request"),
            "Authorization should contain the credential scope terminator");
        assert!(auth.1.contains("SignedHeaders="),
            "Authorization should contain SignedHeaders");
        assert!(auth.1.contains("Signature="),
            "Authorization should contain Signature");
    }
}

//! Bedrock Native format conversion module.
//!
//! Converts Anthropic Messages requests to Amazon Bedrock Converse API requests,
//! and Bedrock Converse API responses back to Anthropic Messages responses
//! for Claude-compatible clients.

use crate::proxy::error::ProxyError;
use serde_json::{json, Map, Value};

/// Convert Anthropic Messages API request to Bedrock Converse API request.
pub fn anthropic_to_bedrock(body: Value) -> Result<Value, ProxyError> {
    let mut result = json!({});

    // Handle system prompt
    if let Some(system) = body.get("system") {
        let system_parts = match system {
            Value::String(s) => vec![json!({ "text": s })],
            Value::Array(arr) => arr
                .iter()
                .map(|block| match block.get("type").and_then(|v| v.as_str()) {
                    Some("text") => json!({ "text": block.get("text").unwrap_or(&Value::Null) }),
                    _ => json!({ "text": "" }),
                })
                .collect(),
            _ => vec![],
        };
        if !system_parts.is_empty() {
            result["system"] = json!(system_parts);
        }
    }

    // Handle messages
    if let Some(messages) = body.get("messages").and_then(|v| v.as_array()) {
        let bedrock_messages: Result<Vec<Value>, ProxyError> = messages
            .iter()
            .map(|msg| convert_anthropic_message_to_bedrock(msg))
            .collect();
        result["messages"] = json!(bedrock_messages?);
    }

    // Handle inference configuration
    let mut inference_config = Map::new();

    if let Some(max_tokens) = body.get("max_tokens").and_then(|v| v.as_u64()) {
        inference_config.insert("maxTokens".to_string(), json!(max_tokens));
    }

    if let Some(temperature) = body.get("temperature").and_then(|v| v.as_f64()) {
        inference_config.insert("temperature".to_string(), json!(temperature));
    }

    if let Some(top_p) = body.get("top_p").and_then(|v| v.as_f64()) {
        inference_config.insert("topP".to_string(), json!(top_p));
    }

    if let Some(top_k) = body.get("top_k").and_then(|v| v.as_u64()) {
        inference_config.insert("topK".to_string(), json!(top_k));
    }

    if let Some(stop_sequences) = body.get("stop_sequences").and_then(|v| v.as_array()) {
        inference_config.insert("stopSequences".to_string(), json!(stop_sequences));
    }

    // Map anthropic thinking budget to bedrock
    if let Some(thinking) = body.get("thinking") {
        if let Some(budget_tokens) = thinking.get("budget_tokens").and_then(|v| v.as_u64()) {
            // Bedrock doesn't have direct thinking budget, but we can preserve it in metadata
        }
    }

    if !inference_config.is_empty() {
        result["inferenceConfig"] = json!(inference_config);
    }

    // Handle tool configuration
    if let Some(tools) = body.get("tools").and_then(|v| v.as_array()) {
        let tool_config = build_bedrock_tool_config(tools)?;
        if !tool_config.is_empty() {
            result["toolConfig"] = json!(tool_config);
        }
    }

    Ok(result)
}

/// Convert a single Anthropic message to Bedrock message format.
fn convert_anthropic_message_to_bedrock(msg: &Value) -> Result<Value, ProxyError> {
    let role = msg
        .get("role")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ProxyError::TransformError("Missing message role".to_string()))?;

    let bedrock_role = match role {
        "user" => "user",
        "assistant" => "assistant",
        _ => "user",
    };

    let mut content = Vec::new();

    if let Some(blocks) = msg.get("content").and_then(|v| v.as_array()) {
        for block in blocks {
            let converted = convert_anthropic_content_block_to_bedrock(block)?;
            content.extend(converted);
        }
    } else if let Some(text) = msg.get("content").and_then(|v| v.as_str()) {
        content.push(json!({ "text": text }));
    }

    Ok(json!({
        "role": bedrock_role,
        "content": content
    }))
}

/// Convert an Anthropic content block to Bedrock content blocks.
fn convert_anthropic_content_block_to_bedrock(block: &Value) -> Result<Vec<Value>, ProxyError> {
    let mut result = Vec::new();

    match block.get("type").and_then(|v| v.as_str()) {
        Some("text") => {
            if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                result.push(json!({ "text": text }));
            }
        }
        Some("image") => {
            if let Some(source) = block.get("source") {
                if let Some(media_type) = source.get("media_type").and_then(|v| v.as_str()) {
                    if let Some(data) = source.get("data").and_then(|v| v.as_str()) {
                        result.push(json!({
                            "image": {
                                "format": media_type.strip_prefix("image/").unwrap_or("png"),
                                "source": {
                                    "bytes": data
                                }
                            }
                        }));
                    }
                }
            }
        }
        Some("tool_use") => {
            let name = block
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let input = block.get("input").cloned().unwrap_or(json!({}));
            let tool_use_id = block.get("id").and_then(|v| v.as_str());

            let mut tool_use = json!({
                "toolUse": {
                    "name": name,
                    "input": input
                }
            });

            if let Some(id) = tool_use_id {
                tool_use["toolUse"]["toolUseId"] = json!(id);
            }

            result.push(tool_use);
        }
        Some("tool_result") => {
            let tool_use_id = block
                .get("tool_use_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let mut content = Vec::new();
            if let Some(content_blocks) = block.get("content").and_then(|v| v.as_array()) {
                for content_block in content_blocks {
                    if let Some(text) = content_block.get("text").and_then(|v| v.as_str()) {
                        content.push(json!({ "text": text }));
                    } else if let Some(image) = content_block.get("image") {
                        if let Some(source) = image.get("source") {
                            if let Some(media_type) = source.get("media_type").and_then(|v| v.as_str()) {
                                if let Some(data) = source.get("data").and_then(|v| v.as_str()) {
                                    content.push(json!({
                                        "image": {
                                            "format": media_type.strip_prefix("image/").unwrap_or("png"),
                                            "source": {
                                                "bytes": data
                                            }
                                        }
                                    }));
                                }
                            }
                        }
                    }
                }
            } else if let Some(text) = block.get("content").and_then(|v| v.as_str()) {
                content.push(json!({ "text": text }));
            }

            let mut tool_result = json!({
                "toolResult": {
                    "toolUseId": tool_use_id,
                    "content": content
                }
            });

            if block.get("is_error").and_then(|v| v.as_bool()) == Some(true) {
                tool_result["toolResult"]["status"] = json!("error");
            }

            result.push(tool_result);
        }
        Some("thinking") | Some("redacted_thinking") => {
            // Bedrock handles thinking internally or via extended fields
            // We'll preserve these as text blocks for compatibility
            if let Some(thinking) = block.get("thinking").and_then(|v| v.as_str()) {
                result.push(json!({ "text": thinking }));
            }
            if let Some(signature) = block.get("signature").and_then(|v| v.as_str()) {
                result.push(json!({ "text": signature }));
            }
        }
        _ => {}
    }

    Ok(result)
}

/// Build Bedrock tool configuration from Anthropic tools.
fn build_bedrock_tool_config(tools: &[Value]) -> Result<Map<String, Value>, ProxyError> {
    let mut tool_config = Map::new();
    let mut tool_specs = Vec::new();

    for tool in tools {
        if let Some(tool_type) = tool.get("type").and_then(|v| v.as_str()) {
            if tool_type == "custom" || tool_type == "function" {
                let name = tool
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let description = tool.get("description").and_then(|v| v.as_str());
                let input_schema = tool.get("input_schema").cloned().unwrap_or(json!({}));

                let mut tool_spec = json!({
                    "toolSpec": {
                        "name": name,
                        "inputSchema": {
                            "json": input_schema
                        }
                    }
                });

                if let Some(desc) = description {
                    tool_spec["toolSpec"]["description"] = json!(desc);
                }

                tool_specs.push(tool_spec);
            }
        }
    }

    if !tool_specs.is_empty() {
        tool_config.insert("tools".to_string(), json!(tool_specs));
    }

    Ok(tool_config)
}

/// Convert Bedrock Converse API response to Anthropic Messages API response.
pub fn bedrock_to_anthropic(body: Value) -> Result<Value, ProxyError> {
    let mut result = json!({
        "id": body.get("responseId").and_then(|v| v.as_str()).unwrap_or(""),
        "type": "message",
        "role": "assistant",
        "content": [],
        "model": body.get("model").and_then(|v| v.as_str()).unwrap_or(""),
        "stop_reason": "end_turn",
        "stop_sequence": Value::Null,
        "usage": build_anthropic_usage(body.get("usage"))
    });

    // Process output message
    if let Some(output) = body.get("output") {
        if let Some(message) = output.get("message") {
            if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                let mut anthropic_content = Vec::new();
                for block in content {
                    let converted = convert_bedrock_content_block_to_anthropic(block)?;
                    anthropic_content.extend(converted);
                }
                result["content"] = json!(anthropic_content);
            }
        }
    }

    // Process stop reason
    if let Some(stop_reason) = body.get("stopReason").and_then(|v| v.as_str()) {
        result["stop_reason"] = json!(match stop_reason {
            "end_turn" => "end_turn",
            "tool_use" => "tool_use",
            "stop_sequence" => "stop_sequence",
            "max_tokens" => "max_tokens",
            "content_filtered" => "content_filtered",
            _ => "end_turn",
        });
    }

    Ok(result)
}

/// Convert Bedrock content blocks to Anthropic content blocks.
fn convert_bedrock_content_block_to_anthropic(block: &Value) -> Result<Vec<Value>, ProxyError> {
    let mut result = Vec::new();

    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
        result.push(json!({
            "type": "text",
            "text": text
        }));
    }

    if let Some(tool_use) = block.get("toolUse") {
        let name = tool_use
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let input = tool_use.get("input").cloned().unwrap_or(json!({}));
        let tool_use_id = tool_use.get("toolUseId").and_then(|v| v.as_str()).unwrap_or("");

        result.push(json!({
            "type": "tool_use",
            "id": tool_use_id,
            "name": name,
            "input": input
        }));
    }

    if let Some(image) = block.get("image") {
        if let Some(format) = image.get("format").and_then(|v| v.as_str()) {
            if let Some(source) = image.get("source") {
                if let Some(bytes) = source.get("bytes").and_then(|v| v.as_str()) {
                    result.push(json!({
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": format!("image/{}", format),
                            "data": bytes
                        }
                    }));
                }
            }
        }
    }

    Ok(result)
}

/// Build Anthropic usage metadata from Bedrock usage.
fn build_anthropic_usage(usage: Option<&Value>) -> Value {
    let mut result = json!({
        "input_tokens": 0,
        "output_tokens": 0
    });

    if let Some(usage) = usage {
        if let Some(input_tokens) = usage.get("inputTokens").and_then(|v| v.as_u64()) {
            result["input_tokens"] = json!(input_tokens);
        }
        if let Some(output_tokens) = usage.get("outputTokens").and_then(|v| v.as_u64()) {
            result["output_tokens"] = json!(output_tokens);
        }
    }

    result
}

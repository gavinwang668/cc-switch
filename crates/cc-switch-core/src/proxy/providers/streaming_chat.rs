//! Anthropic SSE → OpenAI Chat Completions SSE 流式转换
//!
//! 当 Chat 客户端连接非 Chat 后端时，先将后端 SSE 转为 Anthropic SSE
//!（已有 create_anthropic_sse_stream_from_* 系列函数），再由本模块将
//! Anthropic SSE 转为 Chat Completions SSE 返回给客户端。

use crate::proxy::sse::{strip_sse_field, take_sse_block};
use bytes::Bytes;
use futures::stream::Stream;
use serde_json::{json, Value};
use std::collections::HashMap;

/// 将 Anthropic SSE 流转为 OpenAI Chat Completions SSE 流。
///
/// 输出符合 OpenAI Chat Completions 流式规范：
/// - 首 chunk 带 `delta.role: "assistant"`
/// - 正文 chunk 带 `delta.content`
/// - tool_use chunk 带 `delta.tool_calls`
/// - 尾 chunk 带 `finish_reason` 和 `usage`
/// - 流结束标记 `[DONE]`
pub fn create_chat_sse_stream_from_anthropic<E: std::error::Error + Send + 'static>(
    stream: impl Stream<Item = Result<Bytes, E>> + Send + 'static,
) -> impl Stream<Item = Result<Bytes, std::io::Error>> + Send {
    async_stream::stream! {
        let mut buffer = String::new();
        let mut chat_id = String::new();
        let mut chat_model = String::new();
        let mut pending_finish_reason: Option<String> = None;
        let mut pending_usage: Option<Value> = None;
        let mut has_sent_final_chunk = false;

        // 工具调用状态：跟踪每个 content block index → tool info
        let mut tool_states: HashMap<u32, ToolState> = HashMap::new();
        // 下一个工具调用在 choices[0].delta.tool_calls 数组中的 index
        let mut next_tool_call_index: usize = 0;

        for await result in stream {
            let data = match result {
                Ok(data) => data,
                Err(e) => {
                    log::error!("[Chat/Anthropic→Chat] 上游流错误: {e}");
                    let _ = stream_error_event(&mut buffer).await;
                    break;
                }
            };

            let chunk = String::from_utf8_lossy(&data);
            buffer.push_str(&chunk);

            while let Some(block) = take_sse_block(&mut buffer) {
                let mut event_name = "";
                let mut data_str = "";

                for line in block.lines() {
                    if let Some(evt) = strip_sse_field(line, "event") {
                        event_name = evt.trim();
                    } else if let Some(data) = strip_sse_field(line, "data") {
                        data_str = data;
                    }
                }

                if data_str.is_empty() {
                    continue;
                }

                let event: Value = match serde_json::from_str(data_str) {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("[Chat/Anthropic→Chat] 解析 SSE data 失败: {e}, data: {data_str}");
                        continue;
                    }
                };

                match event_name {
                    "message_start" => {
                        if let Some(msg) = event.get("message") {
                            chat_id = msg.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            chat_model = msg.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        }

                        // 发送首 chunk：含 role
                        let initial = build_chat_chunk(
                            &chat_id,
                            &chat_model,
                            Some("assistant"),
                            None,
                            None,
                            None,
                            None,
                        );
                        let sse = format_sse_chunk(&initial);
                        yield Ok(Bytes::from(sse));
                    }
                    "content_block_start" => {
                        let index = event.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        if let Some(cb) = event.get("content_block") {
                            let cb_type = cb.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            if cb_type == "tool_use" {
                                let tool_id = cb.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let tool_name = cb.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();

                                let tool_index = next_tool_call_index;
                                next_tool_call_index += 1;

                                tool_states.insert(index, ToolState {
                                    tool_index,
                                    id: tool_id.clone(),
                                    name: tool_name.clone(),
                                });

                                // 发送 tool_call 首 chunk（id + function name）
                                let tool_delta = json!([{
                                    "index": tool_index,
                                    "id": tool_id,
                                    "type": "function",
                                    "function": {
                                        "name": tool_name,
                                        "arguments": ""
                                    }
                                }]);
                                let chunk = build_chat_chunk(
                                    &chat_id, &chat_model, None, None,
                                    Some(&tool_delta), None, None,
                                );
                                let sse = format_sse_chunk(&chunk);
                                yield Ok(Bytes::from(sse));
                            }
                            // text 类型的 content_block_start 不需要发送任何 chat chunk
                        }
                    }
                    "content_block_delta" => {
                        let index = event.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        if let Some(delta) = event.get("delta") {
                            let delta_type = delta.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            match delta_type {
                                "text_delta" => {
                                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                        let chunk = build_chat_chunk(
                                            &chat_id, &chat_model, None,
                                            Some(text), None, None, None,
                                        );
                                        let sse = format_sse_chunk(&chunk);
                                        yield Ok(Bytes::from(sse));
                                    }
                                }
                                "input_json_delta" => {
                                    if let Some(partial) = delta.get("partial_json").and_then(|v| v.as_str()) {
                                        if let Some(ts) = tool_states.get(&index) {
                                            let tool_delta = json!([{
                                                "index": ts.tool_index,
                                                "function": {
                                                    "arguments": partial
                                                }
                                            }]);
                                            let chunk = build_chat_chunk(
                                                &chat_id, &chat_model, None,
                                                None, Some(&tool_delta), None, None,
                                            );
                                            let sse = format_sse_chunk(&chunk);
                                            yield Ok(Bytes::from(sse));
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    "content_block_stop" => {
                        // content block 结束，清理工具状态
                        let index = event.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        tool_states.remove(&index);
                    }
                    "message_delta" => {
                        if let Some(delta) = event.get("delta") {
                            pending_finish_reason = delta
                                .get("stop_reason")
                                .and_then(|v| v.as_str())
                                .map(|s| map_anthropic_stop_to_openai(s));
                        }
                        if let Some(usage) = event.get("usage") {
                            pending_usage = Some(usage.clone());
                        }
                    }
                    "message_stop" => {
                        // 发送最终 chunk：含 finish_reason 和 usage
                        if !has_sent_final_chunk {
                            let finish = pending_finish_reason.take();
                            let usage = pending_usage.take();

                            let openai_usage = usage.map(|u| {
                                json!({
                                    "prompt_tokens": u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                                    "completion_tokens": u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                                    "total_tokens": u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                                        + u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                                })
                            });

                            let chunk = build_chat_chunk(
                                &chat_id, &chat_model, None,
                                None, None, finish.as_deref(), openai_usage.as_ref(),
                            );
                            let sse = format_sse_chunk(&chunk);
                            yield Ok(Bytes::from(sse));

                            // 发送 [DONE]
                            yield Ok(Bytes::from("data: [DONE]\n\n"));
                            has_sent_final_chunk = true;
                        }
                    }
                    "error" => {
                        log::error!(
                            "[Chat/Anthropic→Chat] 上游错误: {}",
                            event.get("error").and_then(|e| e.get("message")).and_then(|v| v.as_str()).unwrap_or("unknown")
                        );
                        let err = json!({
                            "error": {
                                "message": event.get("error").and_then(|e| e.get("message")).and_then(|v| v.as_str()).unwrap_or("Stream error"),
                                "type": "stream_error"
                            }
                        });
                        let sse = format!("data: {}\n\n", serde_json::to_string(&err).unwrap_or_default());
                        yield Ok(Bytes::from(sse));
                        yield Ok(Bytes::from("data: [DONE]\n\n"));
                        has_sent_final_chunk = true;
                        break;
                    }
                    "ping" => {
                        // 忽略 ping 事件
                    }
                    _ => {
                        // 忽略未知事件
                    }
                }
            }
        }

        // 流自然结束时确保发送尾 chunk + [DONE]
        if !has_sent_final_chunk {
            let finish = pending_finish_reason.take();
            let usage = pending_usage.take();

            let openai_usage = usage.map(|u| {
                json!({
                    "prompt_tokens": u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    "completion_tokens": u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                    "total_tokens": u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                        + u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                })
            });

            let chunk = build_chat_chunk(
                &chat_id, &chat_model, None,
                None, None, finish.as_deref(), openai_usage.as_ref(),
            );
            let sse = format_sse_chunk(&chunk);
            yield Ok(Bytes::from(sse));
            yield Ok(Bytes::from("data: [DONE]\n\n"));
        }
    }
}

/// 流错误事件 — 仅在 buffer 仍有未完成 SSE 块时发送
async fn stream_error_event(_buffer: &mut String) {}

// --------------------------------------------------------------------------
// Tool state
// --------------------------------------------------------------------------

struct ToolState {
    tool_index: usize,
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    name: String,
}

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

fn build_chat_chunk(
    id: &str,
    model: &str,
    role: Option<&str>,
    content: Option<&str>,
    tool_calls: Option<&Value>,
    finish_reason: Option<&str>,
    usage: Option<&Value>,
) -> Value {
    let mut delta = json!({});
    if let Some(r) = role {
        delta["role"] = json!(r);
    }
    if let Some(c) = content {
        delta["content"] = json!(c);
    }
    if let Some(tc) = tool_calls {
        delta["tool_calls"] = tc.clone();
    }

    let mut choice = json!({
        "index": 0,
        "delta": delta,
    });
    if let Some(fr) = finish_reason {
        choice["finish_reason"] = json!(fr);
    }

    let mut chunk = json!({
        "id": id,
        "object": "chat.completion.chunk",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [choice],
    });
    if let Some(u) = usage {
        chunk["usage"] = u.clone();
    }

    chunk
}

fn format_sse_chunk(chunk: &Value) -> String {
    let json_str = serde_json::to_string(chunk).unwrap_or_default();
    format!("data: {}\n\n", json_str)
}

fn map_anthropic_stop_to_openai(stop_reason: &str) -> String {
    match stop_reason {
        "end_turn" => "stop",
        "max_tokens" => "length",
        "stop_sequence" => "stop",
        "tool_use" => "tool_calls",
        other => {
            log::warn!("[Chat/Anthropic→Chat] 未知 stop_reason: {other}");
            "stop"
        }
    }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use futures::StreamExt;
    use serde_json::Value;

    async fn collect_chat_events(input: &str) -> Vec<Value> {
        let upstream = stream::iter(vec![Ok::<_, std::io::Error>(Bytes::from(
            input.as_bytes().to_vec(),
        ))]);
        let converted = create_chat_sse_stream_from_anthropic(upstream);
        futures::pin_mut!(converted);

        let mut events = Vec::new();
        while let Some(Ok(chunk)) = converted.next().await {
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if line == "data: [DONE]" {
                    events.push(json!({"[DONE]": true}));
                } else if let Some(data) = line.strip_prefix("data: ") {
                    if let Ok(evt) = serde_json::from_str::<Value>(data) {
                        events.push(evt);
                    }
                }
            }
        }
        events
    }

    #[tokio::test]
    async fn test_simple_text_conversion() {
        let sse = "\
event: message_start
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-3\",\"content\":[]}}

event: content_block_start
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\" world\"}}

event: content_block_stop
data: {\"type\":\"content_block_stop\",\"index\":0}

event: message_delta
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"input_tokens\":10,\"output_tokens\":2}}

event: message_stop
data: {\"type\":\"message_stop\"}
";

        let events = collect_chat_events(sse).await;
        assert!(!events.is_empty(), "应该产生事件");

        // 首 chunk 应有 role
        let first = &events[0];
        assert_eq!(first["choices"][0]["delta"]["role"], "assistant");

        // 检测文本内容
        let all_content: String = events.iter()
            .filter_map(|e| e["choices"][0]["delta"]["content"].as_str())
            .collect();
        assert_eq!(all_content, "Hello world");

        // 尾 chunk 应有 finish_reason
        let last_meaningful = events.iter()
            .rev()
            .find(|e| e["choices"][0]["finish_reason"].as_str().is_some())
            .expect("应有 finish_reason 的 chunk");
        assert_eq!(last_meaningful["choices"][0]["finish_reason"], "stop");

        // 应有 [DONE]
        assert!(events.last().unwrap().get("[DONE]").is_some());
    }

    #[tokio::test]
    async fn test_tool_use_conversion() {
        let sse = "\
event: message_start
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_2\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-3\",\"content\":[]}}

event: content_block_start
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Let me check\"}}

event: content_block_stop
data: {\"type\":\"content_block_stop\",\"index\":0}

event: content_block_start
data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_01\",\"name\":\"get_weather\",\"input\":{}}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"location\\\":\"}}

event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"Tokyo\\\"}\"}}

event: content_block_stop
data: {\"type\":\"content_block_stop\",\"index\":1}

event: message_delta
data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"input_tokens\":15,\"output_tokens\":10}}

event: message_stop
data: {\"type\":\"message_stop\"}
";

        let events = collect_chat_events(sse).await;

        // 检测文本
        let all_content: String = events.iter()
            .filter_map(|e| e["choices"][0]["delta"]["content"].as_str())
            .collect();
        assert_eq!(all_content, "Let me check");

        // 检测 tool_call
        let has_tool_call = events.iter().any(|e| {
            e["choices"][0]["delta"]["tool_calls"].as_array().is_some_and(|a| !a.is_empty())
        });
        assert!(has_tool_call, "应有 tool_call delta");

        // finish_reason 应为 tool_calls
        let last = events.iter()
            .rev()
            .find(|e| e["choices"][0]["finish_reason"].as_str().is_some())
            .expect("应有 finish_reason");
        assert_eq!(last["choices"][0]["finish_reason"], "tool_calls");
    }
}

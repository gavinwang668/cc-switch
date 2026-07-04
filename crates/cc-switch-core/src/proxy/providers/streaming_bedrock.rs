//! Amazon Bedrock Converse API streaming conversion module.
//!
//! Converts Bedrock `converseStream` chunks into Anthropic-style
//! SSE events for Claude-compatible clients.

use super::transform_bedrock::{build_anthropic_usage, synthesize_tool_call_id};
use crate::proxy::sse::{append_utf8_safe, strip_sse_field, take_sse_block};
use bytes::Bytes;
use futures::stream::{Stream, StreamExt};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
struct ToolCallState {
    name: String,
    args: String,
    has_sent_start: bool,
}

fn map_stop_reason(reason: Option<&str>, has_tool_use: bool) -> &'static str {
    if has_tool_use {
        return "tool_use";
    }

    match reason {
        Some("stop_sequence") => "stop_sequence",
        Some("max_tokens") => "max_tokens",
        Some("tool_use") => "tool_use",
        Some("content_filtered") => "stop_sequence",
        Some("end_turn") | _ => "end_turn",
    }
}

/// 创建从 Bedrock ConverseStream 到 Anthropic SSE 的转换流
pub fn create_anthropic_sse_stream_from_bedrock<E: std::error::Error + Send + 'static>(
    stream: impl Stream<Item = Result<Bytes, E>> + Send + 'static,
) -> impl Stream<Item = Result<Bytes, std::io::Error>> + Send {
    async_stream::stream! {
        let mut buffer = String::new();
        let mut utf8_remainder: Vec<u8> = Vec::new();
        let mut message_id: Option<String> = None;
        let mut current_model: Option<String> = None;
        let mut has_sent_message_start = false;
        let mut has_tool_use = false;
        let mut tool_calls: HashMap<String, ToolCallState> = HashMap::new();
        let mut current_tool_call_id: Option<String> = None;
        let mut open_tool_indices: HashSet<u32> = HashSet::new();
        let mut next_tool_index: u32 = 0;
        let mut tool_index_by_id: HashMap<String, u32> = HashMap::new();
        let mut accumulated_usage: Option<Value> = None;

        tokio::pin!(stream);

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    append_utf8_safe(&mut buffer, &mut utf8_remainder, &bytes);

                    // SSE 事件由 \n\n 分隔
                    while let Some(block) = take_sse_block(&mut buffer) {
                        if block.trim().is_empty() {
                            continue;
                        }

                        // 解析 SSE 块：提取 event: 和 data: 行
                        let mut event_type: Option<String> = None;
                        let mut data_parts: Vec<String> = Vec::new();

                        for line in block.lines() {
                            if let Some(evt) = strip_sse_field(line, "event") {
                                event_type = Some(evt.trim().to_string());
                            } else if let Some(d) = strip_sse_field(line, "data") {
                                data_parts.push(d.to_string());
                            }
                        }

                        if data_parts.is_empty() {
                            continue;
                        }

                        let data_str = data_parts.join("");
                        let data: Value = match serde_json::from_str(&data_str) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };

                        // 处理不同的 Bedrock 事件类型
                        let result = process_bedrock_event(
                            &data,
                            &mut message_id,
                            &mut current_model,
                            &mut has_sent_message_start,
                            &mut has_tool_use,
                            &mut tool_calls,
                            &mut current_tool_call_id,
                            &mut open_tool_indices,
                            &mut next_tool_index,
                            &mut tool_index_by_id,
                            &mut accumulated_usage,
                        );

                        for sse_line in result {
                            yield Ok(Bytes::from(sse_line));
                        }
                    }
                }
                Err(_) => {
                    // 简单地忽略错误，保持流继续
                    continue;
                }
            }
        }
    }
}

fn process_bedrock_event(
    data: &Value,
    message_id: &mut Option<String>,
    current_model: &mut Option<String>,
    has_sent_message_start: &mut bool,
    has_tool_use: &mut bool,
    tool_calls: &mut HashMap<String, ToolCallState>,
    current_tool_call_id: &mut Option<String>,
    open_tool_indices: &mut HashSet<u32>,
    next_tool_index: &mut u32,
    tool_index_by_id: &mut HashMap<String, u32>,
    accumulated_usage: &mut Option<Value>,
) -> Vec<String> {
    let mut result = Vec::new();

    // 检查消息开始
    if let Some(trace_id) = data.get("traceId").and_then(|v| v.as_str()) {
        if message_id.is_none() {
            *message_id = Some(trace_id.to_string());
        }
    }

    if let Some(model_id) = data.get("modelId").and_then(|v| v.as_str()) {
        if current_model.is_none() {
            *current_model = Some(model_id.to_string());
        }
    }

    // 发送消息开始事件（如果尚未发送）
    if !*has_sent_message_start {
        if let (Some(id), Some(model)) = (message_id.as_ref(), current_model.as_ref()) {
            result.push(format!(
                "event: message_start\ndata: {}\n\n",
                json!({
                    "type": "message_start",
                    "message": {
                        "id": id,
                        "type": "message",
                        "role": "assistant",
                        "content": [],
                        "model": model,
                        "stop_reason": Value::Null,
                        "stop_sequence": Value::Null,
                        "usage": {
                            "input_tokens": 0,
                            "output_tokens": 0
                        }
                    }
                })
            ));
            *has_sent_message_start = true;
        }
    }

    // 处理内容块增量
    if let Some(content_block_delta) = data.get("contentBlockDelta") {
        if let Some(delta) = content_block_delta.get("delta") {
            if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                // 文本增量
                result.push(format!(
                    "event: content_block_delta\ndata: {}\n\n",
                    json!({
                        "type": "content_block_delta",
                        "index": 0,
                        "delta": {
                            "type": "text_delta",
                            "text": text
                        }
                    })
                ));
            }
        }
    }

    // 处理内容块开始
    if let Some(content_block_start) = data.get("contentBlockStart") {
        if let Some(start) = content_block_start.get("start") {
            if let Some(tool_use) = start.get("toolUse") {
                // 工具调用开始
                let tool_use_id = tool_use
                    .get("toolUseId")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(ToString::to_string)
                    .unwrap_or_else(synthesize_tool_call_id);

                let name = tool_use
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // 获取或分配工具索引
                let tool_index = if let Some(existing) = tool_index_by_id.get(&tool_use_id) {
                    *existing
                } else {
                    let idx = *next_tool_index;
                    *next_tool_index += 1;
                    tool_index_by_id.insert(tool_use_id.clone(), idx);
                    idx
                };

                // 保存工具调用状态
                tool_calls.insert(
                    tool_use_id.clone(),
                    ToolCallState {
                        name: name.clone(),
                        args: String::new(),
                        has_sent_start: false,
                    },
                );

                *current_tool_call_id = Some(tool_use_id.clone());
                open_tool_indices.insert(tool_index);
                *has_tool_use = true;

                // 发送工具调用开始事件
                result.push(format!(
                    "event: content_block_start\ndata: {}\n\n",
                    json!({
                        "type": "content_block_start",
                        "index": tool_index,
                        "content_block": {
                            "type": "tool_use",
                            "id": tool_use_id,
                            "name": name,
                            "input": {}
                        }
                    })
                ));
            }
        }
    }

    // 处理工具使用增量
    if let Some(content_block_delta) = data.get("contentBlockDelta") {
        if let Some(delta) = content_block_delta.get("delta") {
            if let Some(tool_use) = delta.get("toolUse") {
                if let Some(input) = tool_use.get("input").and_then(|v| v.as_str()) {
                    // 工具输入增量
                    if let Some(tool_id) = current_tool_call_id.as_ref() {
                        if let Some(state) = tool_calls.get_mut(tool_id) {
                            state.args.push_str(input);

                            // 获取工具索引
                            if let Some(&tool_index) = tool_index_by_id.get(tool_id) {
                                // 发送工具调用增量事件
                                result.push(format!(
                                    "event: content_block_delta\ndata: {}\n\n",
                                    json!({
                                        "type": "content_block_delta",
                                        "index": tool_index,
                                        "delta": {
                                            "type": "input_json_delta",
                                            "partial_json": input
                                        }
                                    })
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // 处理内容块完成
    if let Some(content_block_stop) = data.get("contentBlockStop") {
        if let Some(tool_id) = current_tool_call_id.as_ref() {
            if let Some(&tool_index) = tool_index_by_id.get(tool_id) {
                open_tool_indices.remove(&tool_index);

                // 发送工具调用完成事件
                result.push(format!(
                    "event: content_block_stop\ndata: {}\n\n",
                    json!({
                        "type": "content_block_stop",
                        "index": tool_index
                    })
                ));
            }
        }
    }

    // 处理消息完成
    if let Some(message_stop) = data.get("messageStop") {
        let stop_reason = message_stop
            .get("stopReason")
            .and_then(|v| v.as_str())
            .unwrap_or("end_turn");

        // 更新累积使用量
        if let Some(usage) = data.get("usage") {
            *accumulated_usage = Some(usage.clone());
        }

        // 发送消息完成事件
        let anthropic_usage = build_anthropic_usage(accumulated_usage.as_ref());
        result.push(format!(
            "event: message_delta\ndata: {}\n\n",
            json!({
                "type": "message_delta",
                "delta": {
                    "stop_reason": map_stop_reason(Some(stop_reason), *has_tool_use),
                    "stop_sequence": Value::Null
                },
                "usage": anthropic_usage
            })
        ));

        result.push("event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string());
    }

    // 处理 metadata 中的 usage
    if let Some(metadata) = data.get("metadata") {
        if let Some(usage) = metadata.get("usage") {
            *accumulated_usage = Some(usage.clone());

            // 发送 ping 事件以保持连接活跃
            result.push("event: ping\ndata: {\"type\": \"ping\"}\n\n".to_string());
        }
    }

    result
}

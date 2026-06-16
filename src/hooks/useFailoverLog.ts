import { useCallback, useEffect, useRef, useState } from "react";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import type { FailoverEvent, FailoverReason } from "@/types/failover";

/**
 * 监听后端 `provider-switched` 事件（source = "failover"），
 * 累积形成前端内存中的故障转移事件流。
 *
 * - 默认最多保留 MAX_EVENTS 条，超出后丢弃最早的（FIFO）
 * - 整个应用共享一个事件列表（通过全局 ref 复用）
 * - 提供手动 `clear` 接口供 UI 重置
 */

const MAX_EVENTS = 500;

interface FailoverEventPayload {
  appType: string;
  providerId: string;
  providerName?: string;
  source?: string;
  reason?: string;
  result?: string;
  durationMs?: number;
  errorMessage?: string;
}

const globalEvents: FailoverEvent[] = [];
const listeners = new Set<() => void>();

function notify() {
  for (const cb of listeners) cb();
}

function pushEvent(event: FailoverEvent) {
  globalEvents.unshift(event);
  if (globalEvents.length > MAX_EVENTS) {
    globalEvents.length = MAX_EVENTS;
  }
  notify();
}

function inferReason(reason: string | undefined): FailoverReason {
  if (!reason) return "unknown";
  const r = reason.toLowerCase();
  if (r.includes("consecutive") || r.includes("fail"))
    return "consecutive_failures";
  if (r.includes("timeout")) return "timeout";
  if (r.includes("stream")) return "stream_error";
  if (r.includes("non_stream") || r.includes("non-stream"))
    return "non_stream_error";
  return "unknown";
}

function inferResult(
  result: string | undefined,
): "success" | "skipped" | "failed" {
  if (!result) return "success";
  if (result === "failed") return "failed";
  if (result === "skipped") return "skipped";
  return "success";
}

let counter = 0;
function nextId(): string {
  counter += 1;
  return `${Date.now()}-${counter}`;
}

export function useFailoverLog() {
  // 每个 hook 实例订阅通知，但只有一份全局数据
  const [, force] = useState(0);
  const ref = useRef(force);
  ref.current = force;

  useEffect(() => {
    const cb = () => ref.current((n) => n + 1);
    listeners.add(cb);
    return () => {
      listeners.delete(cb);
    };
  }, []);

  useTauriEvent<FailoverEventPayload>("provider-switched", (payload) => {
    if (payload?.source !== "failover") return;
    pushEvent({
      id: nextId(),
      timestamp: new Date().toISOString(),
      appType: payload.appType,
      providerId: payload.providerId,
      providerName: payload.providerName ?? payload.providerId,
      reason: inferReason(payload.reason),
      result: inferResult(payload.result),
      durationMs: payload.durationMs,
      errorMessage: payload.errorMessage,
    });
  });

  const clear = useCallback(() => {
    globalEvents.length = 0;
    notify();
  }, []);

  return {
    events: globalEvents,
    clear,
  };
}

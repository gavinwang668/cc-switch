/**
 * Failover 事件类型定义
 *
 * 记录代理在运行期间发生的故障转移事件。前端通过监听
 * `provider-switched` 事件（source = "failover"）累积而成，
 * 不依赖后端持久化存储（重启后会清空）。
 */

export type FailoverReason =
  | "consecutive_failures"
  | "timeout"
  | "stream_error"
  | "non_stream_error"
  | "unknown";

export type FailoverResult = "success" | "skipped" | "failed";

export interface FailoverEvent {
  /** 自增 ID，仅前端使用 */
  id: string;
  /** ISO 时间戳 */
  timestamp: string;
  /** 应用类型：claude / codex / gemini */
  appType: string;
  /** 切换到的目标 Provider ID */
  providerId: string;
  /** 切换到的目标 Provider 名称 */
  providerName: string;
  /** 触发原因（若后端提供则使用，否则推断） */
  reason: FailoverReason;
  /** 切换结果 */
  result: FailoverResult;
  /** 持续时间（毫秒），由后端或前端估算 */
  durationMs?: number;
  /** 错误信息（如果切换失败） */
  errorMessage?: string;
}

export interface FailoverLogFilter {
  appType?: string;
  reason?: FailoverReason;
  result?: FailoverResult;
  search?: string;
}

import { save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { sessionsApi } from "@/lib/api/sessions";
import type { SessionMessage, SessionMeta } from "@/types";

export type ExportFormat = "markdown" | "json";

export interface ExportOptions {
  format: ExportFormat;
  /** 包含的会话（单个或多个） */
  sessions: SessionMeta[];
}

export interface ExportResult {
  filePath: string;
  format: ExportFormat;
  sessions: number;
  messages: number;
}

/**
 * 将单个会话的消息数组格式化为 Markdown。
 */
function formatSessionAsMarkdown(
  session: SessionMeta,
  messages: SessionMessage[],
): string {
  const lines: string[] = [];
  const title = session.title?.trim() || session.sessionId;
  lines.push(`# ${title}`);
  lines.push("");

  // 元信息
  const meta: string[] = [];
  meta.push(`- Provider: ${session.providerId}`);
  meta.push(`- Session ID: ${session.sessionId}`);
  if (session.projectDir) meta.push(`- Project: ${session.projectDir}`);
  if (session.createdAt) {
    meta.push(`- Created: ${new Date(session.createdAt).toISOString()}`);
  }
  if (session.lastActiveAt) {
    meta.push(
      `- Last Active: ${new Date(session.lastActiveAt).toISOString()}`,
    );
  }
  lines.push(...meta);
  lines.push("");
  lines.push("---");
  lines.push("");

  // 消息主体
  for (const message of messages) {
    const role = (message.role || "unknown").toUpperCase();
    const ts = message.ts
      ? ` (${new Date(message.ts).toISOString()})`
      : "";
    lines.push(`## ${role}${ts}`);
    lines.push("");
    const content = message.content?.trim() ?? "";
    if (content) {
      // 简单按代码块切分，避免与文本混淆
      const parts = content.split(/(```[\s\S]*?```)/g);
      for (const part of parts) {
        if (part.startsWith("```")) {
          lines.push(part);
          lines.push("");
        } else {
          lines.push(part);
          lines.push("");
        }
      }
    } else {
      lines.push("_(empty)_");
      lines.push("");
    }
  }

  return lines.join("\n");
}

/**
 * 将会话格式化为 JSON。
 */
function formatSessionAsJson(
  session: SessionMeta,
  messages: SessionMessage[],
) {
  return {
    sessionId: session.sessionId,
    providerId: session.providerId,
    title: session.title,
    summary: session.summary,
    projectDir: session.projectDir,
    createdAt: session.createdAt
      ? new Date(session.createdAt).toISOString()
      : null,
    lastActiveAt: session.lastActiveAt
      ? new Date(session.lastActiveAt).toISOString()
      : null,
    sourcePath: session.sourcePath,
    messages: messages.map((m) => ({
      role: m.role,
      content: m.content,
      ts: m.ts ? new Date(m.ts).toISOString() : null,
    })),
  };
}

function suggestFileName(format: ExportFormat, sessions: SessionMeta[]): string {
  const stamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
  if (sessions.length === 1) {
    const base = (sessions[0].title || sessions[0].sessionId || "session")
      .replace(/[\\/:*?"<>|]/g, "_")
      .slice(0, 60);
    return `${base}-${stamp}.${format === "markdown" ? "md" : "json"}`;
  }
  return `cc-switch-sessions-${stamp}.${format === "markdown" ? "md" : "json"}`;
}

/**
 * 收集所有会话的消息，并按所选格式生成导出内容。
 */
export async function buildSessionExport(options: ExportOptions): Promise<{
  content: string;
  fileName: string;
  totalMessages: number;
}> {
  const { format, sessions } = options;
  let totalMessages = 0;

  if (format === "markdown") {
    const parts: string[] = [];
    for (let i = 0; i < sessions.length; i += 1) {
      const session = sessions[i];
      if (!session.sourcePath) continue;
      const messages = await sessionsApi.getMessages(
        session.providerId,
        session.sourcePath,
      );
      totalMessages += messages.length;
      parts.push(formatSessionAsMarkdown(session, messages));
      if (i < sessions.length - 1) {
        parts.push("\n---\n\n");
      }
    }
    return {
      content: parts.join(""),
      fileName: suggestFileName(format, sessions),
      totalMessages,
    };
  }

  // JSON
  const jsonPayload: unknown[] = [];
  for (const session of sessions) {
    if (!session.sourcePath) continue;
    const messages = await sessionsApi.getMessages(
      session.providerId,
      session.sourcePath,
    );
    totalMessages += messages.length;
    jsonPayload.push(formatSessionAsJson(session, messages));
  }
  return {
    content: JSON.stringify(
      {
        exportedAt: new Date().toISOString(),
        sessions: jsonPayload,
      },
      null,
      2,
    ),
    fileName: suggestFileName(format, sessions),
    totalMessages,
  };
}

/**
 * 触发系统保存对话框并将内容写入磁盘。
 */
export async function exportSessions(options: ExportOptions): Promise<ExportResult | null> {
  const { content, fileName, totalMessages } = await buildSessionExport(options);
  const filters =
    options.format === "markdown"
      ? [{ name: "Markdown", extensions: ["md", "markdown"] }]
      : [{ name: "JSON", extensions: ["json"] }];
  const filePath = await save({
    defaultPath: fileName,
    filters,
  });
  if (!filePath) return null;
  await writeTextFile(filePath, content);
  return {
    filePath,
    format: options.format,
    sessions: options.sessions.length,
    messages: totalMessages,
  };
}

function writeTextFile(filePath: string, content: string): Promise<string> {
  return invoke<string>("write_text_file", { filePath, content });
}

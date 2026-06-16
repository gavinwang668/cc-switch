import { useEffect, useRef } from "react";

export type ShortcutModifiers = {
  /** Ctrl 键（macOS 也会同时按 Meta，但在 macOS 上自动映射为 Meta） */
  ctrl?: boolean;
  /** Meta 键（macOS 的 ⌘） */
  meta?: boolean;
  /** Shift 键 */
  shift?: boolean;
  /** Alt / Option 键 */
  alt?: boolean;
};

export interface ShortcutSpec extends ShortcutModifiers {
  /**
   * 主键。可以是单字符（如 "k"）或特殊键名（"escape"、"enter"、"arrowup" 等）。
   * 不区分大小写。
   */
  key: string;
  /**
   * 触发后执行的回调。返回 false 表示事件未被消费，允许冒泡到下一个监听器。
   */
  action: (event: KeyboardEvent) => void | boolean | Promise<void>;
  /**
   * 是否在输入框（input/textarea/contenteditable）聚焦时禁用。
   * 默认为 true —— 大多数快捷键不应该打断用户输入。
   */
  disableInInputs?: boolean;
  /**
   * 简短描述，用于将来的快捷键设置面板。
   */
  description?: string;
}

export interface ShortcutGroup {
  id: string;
  title: string;
  shortcuts: ShortcutSpec[];
}

function isMacLike() {
  if (typeof navigator === "undefined") return false;
  return /Mac|iPhone|iPad/i.test(
    navigator.platform || navigator.userAgent || "",
  );
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName.toLowerCase();
  if (tag === "input" || tag === "textarea" || tag === "select") return true;
  if (target.isContentEditable) return true;
  return false;
}

function matchesShortcut(
  event: KeyboardEvent,
  spec: ShortcutSpec,
  mac: boolean,
): boolean {
  const key = event.key.toLowerCase();
  const expected = spec.key.toLowerCase();

  if (key !== expected) return false;

  const ctrlOrMetaWanted = spec.ctrl || spec.meta;
  const ctrlOrMetaPressed = event.ctrlKey || event.metaKey;
  if (ctrlOrMetaWanted) {
    if (mac) {
      // macOS 上把 Ctrl/统一看作 Meta
      if (!event.metaKey && !event.ctrlKey) return false;
    } else if (!event.ctrlKey) {
      return false;
    }
  } else if (ctrlOrMetaPressed) {
    // 用户没声明修饰键但按下了修饰键，不匹配
    return false;
  }

  if (spec.shift !== undefined && spec.shift !== event.shiftKey) return false;
  if (spec.alt !== undefined && spec.alt !== event.altKey) return false;

  return true;
}

/**
 * 通用键盘快捷键 hook。
 * 多个 hook 同时挂载时，每个 hook 独立监听 keydown，匹配的回调会按注册顺序消费。
 */
export function useKeyboardShortcuts(
  shortcuts: ShortcutSpec[] | ShortcutGroup[],
  options: { enabled?: boolean } = {},
) {
  const { enabled = true } = options;
  const mac = isMacLike();
  // 使用 ref 避免在每次 action 变化时重新绑定事件
  const listRef = useRef(shortcuts);
  listRef.current = shortcuts;

  useEffect(() => {
    if (!enabled) return;
    if (typeof window === "undefined") return;

    const handler = (event: KeyboardEvent) => {
      // 忽略自动重复（按住一个键不松）
      if (event.repeat && !event.altKey && !event.shiftKey) {
        // 仍允许修饰键组合通过
        if (!(event.ctrlKey || event.metaKey)) return;
      }

      const list: ShortcutSpec[] = (() => {
        const out: ShortcutSpec[] = [];
        for (const entry of listRef.current) {
          if (Array.isArray((entry as ShortcutGroup).shortcuts)) {
            out.push(...(entry as ShortcutGroup).shortcuts);
          } else {
            out.push(entry as ShortcutSpec);
          }
        }
        return out;
      })();

      for (const spec of list) {
        if (!matchesShortcut(event, spec, mac)) continue;
        if ((spec.disableInInputs ?? true) && isEditableTarget(event.target)) {
          continue;
        }
        const ret = spec.action(event);
        if (ret === false) continue;
        // 默认消费事件，避免重复触发
        event.preventDefault();
        event.stopPropagation();
        break;
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [enabled, mac]);
}

/** 装饰：在 UI 展示时使用 */
export function formatShortcut(spec: ShortcutSpec): string {
  const mac = isMacLike();
  const parts: string[] = [];
  if (spec.ctrl) parts.push(mac ? "⌃" : "Ctrl");
  if (spec.meta) parts.push(mac ? "⌘" : "Meta");
  if (spec.alt) parts.push(mac ? "⌥" : "Alt");
  if (spec.shift) parts.push(mac ? "⇧" : "Shift");
  parts.push(spec.key.length === 1 ? spec.key.toUpperCase() : spec.key);
  return parts.join(mac ? "" : "+");
}

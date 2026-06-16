import { useCallback, useEffect, useRef, useState } from "react";

export type NotificationLevel = "info" | "success" | "warning" | "error";

export interface SystemNotificationOptions {
  title: string;
  body?: string;
  level?: NotificationLevel;
  /** 自定义图标 URL（可选） */
  icon?: string;
  /** 点击通知时的回调（在 webview 内） */
  onClick?: () => void;
  /** 自动关闭时长（毫秒）；0 表示不自动关闭 */
  autoCloseMs?: number;
  /** tag 用来去重；同 tag 的通知会被合并 */
  tag?: string;
  /** 静默模式（在支持的平台上） */
  silent?: boolean;
}

export type NotificationPermissionState =
  | "default"
  | "granted"
  | "denied"
  | "unsupported";

interface UseSystemNotificationResult {
  /** 当前通知权限状态 */
  permission: NotificationPermissionState;
  /** 是否已请求过权限（避免重复弹窗） */
  hasRequested: boolean;
  /** 请求通知权限；如已授予或拒绝则立刻 resolve */
  requestPermission: () => Promise<NotificationPermissionState>;
  /**
   * 弹出一条系统通知。
   * - 当权限为 granted 时显示原生通知
   * - 当权限为 default/denied 时回退为 Sonner toast
   * - 当 API 不可用时同样回退
   */
  notify: (options: SystemNotificationOptions) => Promise<void>;
}

const PERMISSION_STORAGE_KEY = "cc-switch.notifications.permissionAsked";

function readAskedFlag(): boolean {
  if (typeof window === "undefined") return false;
  try {
    return window.localStorage.getItem(PERMISSION_STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

function writeAskedFlag() {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(PERMISSION_STORAGE_KEY, "1");
  } catch {
    /* ignore */
  }
}

function detectSupport(): NotificationPermissionState {
  if (typeof window === "undefined") return "unsupported";
  if (!("Notification" in window)) return "unsupported";
  return (Notification.permission as NotificationPermissionState) ?? "default";
}

async function importSonner() {
  // 动态 import 避免循环依赖
  const mod = await import("sonner");
  return mod.toast;
}

const LEVEL_TO_SONNER: Record<NotificationLevel, string> = {
  info: "info",
  success: "success",
  warning: "warning",
  error: "error",
};

/**
 * 系统通知 hook。
 *
 * - 优先使用 Web Notification API（在 Tauri WebView 中会显示为原生通知）
 * - 不支持 / 未授权时回退为应用内 Sonner toast
 */
export function useSystemNotification(): UseSystemNotificationResult {
  const [permission, setPermission] = useState<NotificationPermissionState>(
    () => detectSupport(),
  );
  const [hasRequested, setHasRequested] = useState<boolean>(() =>
    readAskedFlag(),
  );
  const inFlight = useRef<Promise<NotificationPermissionState> | null>(null);

  useEffect(() => {
    setPermission(detectSupport());
    setHasRequested(readAskedFlag());
  }, []);

  const requestPermission =
    useCallback(async (): Promise<NotificationPermissionState> => {
      if (typeof window === "undefined" || !("Notification" in window)) {
        setPermission("unsupported");
        return "unsupported";
      }
      if (inFlight.current) return inFlight.current;
      const promise = (async () => {
        try {
          const result = await Notification.requestPermission();
          const normalized =
            (result as NotificationPermissionState) ?? "default";
          setPermission(normalized);
          writeAskedFlag();
          setHasRequested(true);
          return normalized;
        } catch {
          setPermission("denied");
          writeAskedFlag();
          setHasRequested(true);
          return "denied";
        } finally {
          inFlight.current = null;
        }
      })();
      inFlight.current = promise;
      return promise;
    }, []);

  const fallbackToast = useCallback(
    async (options: SystemNotificationOptions) => {
      try {
        const toast = await importSonner();
        const level = options.level ?? "info";
        const fn = toast[
          LEVEL_TO_SONNER[level] as keyof typeof toast
        ] as typeof toast.info;
        fn(options.title, {
          description: options.body,
          duration: options.autoCloseMs ?? 4000,
        });
      } catch {
        // 静默忽略：toast 也不可用时直接吞掉
      }
    },
    [],
  );

  const notify = useCallback(
    async (options: SystemNotificationOptions) => {
      if (
        typeof window === "undefined" ||
        !("Notification" in window) ||
        permission !== "granted"
      ) {
        await fallbackToast(options);
        return;
      }
      try {
        const notification = new Notification(options.title, {
          body: options.body,
          tag: options.tag,
          icon: options.icon,
          silent: options.silent,
        });
        if (options.autoCloseMs && options.autoCloseMs > 0) {
          window.setTimeout(() => notification.close(), options.autoCloseMs);
        }
        if (options.onClick) {
          notification.onclick = () => {
            try {
              options.onClick?.();
            } finally {
              window.focus();
              notification.close();
            }
          };
        }
      } catch {
        // 部分平台（如 macOS）在无用户交互时构造会抛错，降级到 toast
        await fallbackToast(options);
      }
    },
    [fallbackToast, permission],
  );

  return { permission, hasRequested, requestPermission, notify };
}

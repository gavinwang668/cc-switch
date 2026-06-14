import { useCallback, useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { extractErrorMessage } from "@/utils/errorUtils";

export interface UseWindowControlsParams {
  /**
   * 同步窗口装饰状态所需的设置数据：
   * - `useAppWindowControls`: 是否使用应用内窗口控件（Linux 风格）
   * 传入 `null` 或 `undefined` 表示设置数据尚未加载，hook 会跳过装饰同步。
   */
  useAppWindowControls?: boolean;
  settingsLoaded: boolean;
}

export interface UseWindowControlsResult {
  isWindowMaximized: boolean;
  minimize: () => Promise<void>;
  toggleMaximize: () => Promise<void>;
  close: () => Promise<void>;
}

/**
 * 集中管理 Tauri 窗口控制：
 * - 监听窗口尺寸变化同步最大化状态
 * - 根据 `useAppWindowControls` 切换窗口装饰
 * - 暴露最小化 / 最大化 / 关闭的统一接口（错误统一 toast）
 */
export function useWindowControls({
  useAppWindowControls,
  settingsLoaded,
}: UseWindowControlsParams): UseWindowControlsResult {
  const { t } = useTranslation();
  const [isWindowMaximized, setIsWindowMaximized] = useState(false);

  // 窗口尺寸变化时同步最大化状态
  useEffect(() => {
    let active = true;
    let unlistenResize: (() => void) | undefined;

    const setupWindowStateSync = async () => {
      try {
        const currentWindow = getCurrentWindow();
        const syncWindowMaximizedState = async () => {
          const maximized = await currentWindow.isMaximized();
          if (active) {
            setIsWindowMaximized(maximized);
          }
        };

        await syncWindowMaximizedState();
        unlistenResize = await currentWindow.onResized(() => {
          void syncWindowMaximizedState();
        });
      } catch (error) {
        console.error(
          "[useWindowControls] Failed to sync window maximized state",
          error,
        );
      }
    };

    void setupWindowStateSync();
    return () => {
      active = false;
      unlistenResize?.();
    };
  }, []);

  // 设置变更时同步窗口装饰
  useEffect(() => {
    // settingsData 未加载时跳过，避免用 fallback 覆盖 Rust 侧已设好的装饰状态
    if (!settingsLoaded) return;

    const syncWindowDecorations = async () => {
      try {
        await getCurrentWindow().setDecorations(!useAppWindowControls);
      } catch (error) {
        console.error(
          "[useWindowControls] Failed to update window decorations",
          error,
        );
      }
    };

    void syncWindowDecorations();
  }, [useAppWindowControls, settingsLoaded]);

  const notifyError = useCallback(
    (error: unknown) => {
      toast.error(
        t("notifications.windowControlFailed", {
          defaultValue: "窗口控制失败：{{error}}",
          error: extractErrorMessage(error),
        }),
      );
    },
    [t],
  );

  const minimize = useCallback(async () => {
    try {
      await getCurrentWindow().minimize();
    } catch (error) {
      console.error("[useWindowControls] Failed to minimize window", error);
      notifyError(error);
    }
  }, [notifyError]);

  const toggleMaximize = useCallback(async () => {
    try {
      const currentWindow = getCurrentWindow();
      await currentWindow.toggleMaximize();
      setIsWindowMaximized(await currentWindow.isMaximized());
    } catch (error) {
      console.error("[useWindowControls] Failed to toggle maximize", error);
      notifyError(error);
    }
  }, [notifyError]);

  const close = useCallback(async () => {
    try {
      await getCurrentWindow().close();
    } catch (error) {
      console.error("[useWindowControls] Failed to close window", error);
      notifyError(error);
    }
  }, [notifyError]);

  return {
    isWindowMaximized,
    minimize,
    toggleMaximize,
    close,
  };
}

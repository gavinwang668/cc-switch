import React, { useEffect, useRef, useState } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { DatabaseUpgrade } from "./components/DatabaseUpgrade";
import { UpdateProvider } from "./contexts/UpdateContext";
import "./index.css";
// 导入国际化配置
import i18n from "./i18n";
import { QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "@/components/theme-provider";
import { queryClient } from "@/lib/query";
import { Toaster } from "@/components/ui/sonner";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { RouterProvider } from "@/lib/router";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { message } from "@tauri-apps/plugin-dialog";
import { exit } from "@tauri-apps/plugin-process";

// 根据平台添加 body class，便于平台特定样式
try {
  const ua = navigator.userAgent || "";
  const plat = (navigator.platform || "").toLowerCase();
  const isMac = /mac/i.test(ua) || plat.includes("mac");
  if (isMac) {
    document.body.classList.add("is-mac");
  }
} catch {
  // 忽略平台检测失败
}

// 配置加载错误payload类型
interface ConfigLoadErrorPayload {
  path?: string;
  error?: string;
  /** "db_version_too_new" 表示数据库版本过新，渲染应用内升级恢复界面 */
  kind?: string;
}

/**
 * 处理配置加载失败：显示错误消息并强制退出应用
 * 不给用户"取消"选项，因为配置损坏时应用无法正常运行
 */
async function handleConfigLoadError(
  payload: ConfigLoadErrorPayload | null,
): Promise<void> {
  const path = payload?.path ?? "~/.cc-switch/config.json";
  const detail = payload?.error ?? "Unknown error";

  await message(
    i18n.t("errors.configLoadFailedMessage", {
      path,
      detail,
      defaultValue:
        "无法读取配置文件：\n{{path}}\n\n错误详情：\n{{detail}}\n\n请手动检查 JSON 是否有效，或从同目录的备份文件（如 config.json.bak）恢复。\n\n应用将退出以便您进行修复。",
    }),
    {
      title: i18n.t("errors.configLoadFailedTitle", {
        defaultValue: "配置加载失败",
      }),
      kind: "error",
    },
  );

  await exit(1);
}

/**
 * 启动期初始化守卫组件
 *
 * 在 React 树挂载后才异步查询后端 init error，避免 IPC 调用阻塞首屏渲染。
 * 如果后端报告配置损坏，弹出系统对话框后退出应用。
 */
function BootstrapGuard({ children }: { children: React.ReactNode }) {
  const checkedRef = useRef(false);
  const [dbUpgradePayload, setDbUpgradePayload] =
    useState<ConfigLoadErrorPayload | null>(null);

  useEffect(() => {
    if (checkedRef.current) return;
    checkedRef.current = true;

    let cancelled = false;

    const checkInitError = async () => {
      try {
        const initError = (await invoke(
          "get_init_error",
        )) as ConfigLoadErrorPayload | null;
        if (cancelled) return;
        if (initError?.kind === "db_version_too_new") {
          setDbUpgradePayload(initError);
          return;
        }
        if (initError && (initError.path || initError.error)) {
          await handleConfigLoadError(initError);
        }
      } catch (e) {
        // 忽略拉取错误，继续正常渲染
        if (!cancelled) {
          console.error("拉取初始化错误失败", e);
        }
      }
    };

    void checkInitError();

    return () => {
      cancelled = true;
    };
  }, []);

  // 监听后端的配置加载错误事件（运行时侧推送）
  useEffect(() => {
    let unsubscribe: (() => void) | undefined;

    try {
      const promise = listen("configLoadError", async (evt) => {
        const payload = evt.payload as ConfigLoadErrorPayload | null;
        if (payload?.kind === "db_version_too_new") {
          setDbUpgradePayload(payload);
          return;
        }
        await handleConfigLoadError(payload);
      });
      promise.then((fn) => {
        unsubscribe = fn;
      });
    } catch (e) {
      console.error("订阅 configLoadError 事件失败", e);
    }

    return () => {
      if (unsubscribe) unsubscribe();
    };
  }, []);

  if (dbUpgradePayload) {
    return <DatabaseUpgrade payload={dbUpgradePayload} />;
  }

  return <>{children}</>;
}

// ─── 立即挂载 React 树（不等待任何 IPC） ───
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <ThemeProvider defaultTheme="system" storageKey="cc-switch-theme">
          <UpdateProvider>
            <RouterProvider>
              <BootstrapGuard>
                <App />
              </BootstrapGuard>
              <Toaster />
            </RouterProvider>
          </UpdateProvider>
        </ThemeProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  </React.StrictMode>,
);

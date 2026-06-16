import type { ReactNode } from "react";
import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { I18nextProvider } from "react-i18next";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { useWindowControls } from "@/hooks/useWindowControls";

// Setup i18n for toast messages
const testI18n = i18n.createInstance();
testI18n.use(initReactI18next).init({
  lng: "zh",
  fallbackLng: "zh",
  resources: {
    zh: {
      translation: {
        "notifications.windowControlFailed": "窗口控制失败：{{error}}",
      },
    },
  },
  interpolation: { escapeValue: false },
});

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const minimizeMock = vi.fn();
const toggleMaximizeMock = vi.fn();
const closeMock = vi.fn();
const isMaximizedMock = vi.fn();
const setDecorationsMock = vi.fn();
const onResizedUnlisten = vi.fn();
const onResizedMock = vi.fn(() => onResizedUnlisten);

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: minimizeMock,
    toggleMaximize: toggleMaximizeMock,
    close: closeMock,
    isMaximized: isMaximizedMock,
    setDecorations: setDecorationsMock,
    onResized: onResizedMock,
  }),
}));

function wrapper({ children }: { children: ReactNode }) {
  return (
    <I18nextProvider i18n={testI18n}>
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    </I18nextProvider>
  );
}

describe("useWindowControls", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    isMaximizedMock.mockResolvedValue(false);
    minimizeMock.mockResolvedValue(undefined);
    toggleMaximizeMock.mockResolvedValue(undefined);
    closeMock.mockResolvedValue(undefined);
    setDecorationsMock.mockResolvedValue(undefined);
  });

  it("返回初始未最大化状态", () => {
    const { result } = renderHook(
      () => useWindowControls({ useAppWindowControls: false, settingsLoaded: true }),
      { wrapper },
    );
    expect(result.current.isWindowMaximized).toBe(false);
  });

  it("minimize 调用窗口最小化", async () => {
    const { result } = renderHook(
      () => useWindowControls({ useAppWindowControls: false, settingsLoaded: true }),
      { wrapper },
    );
    await act(async () => {
      await result.current.minimize();
    });
    expect(minimizeMock).toHaveBeenCalledOnce();
  });

  it("toggleMaximize 调用切换最大化并同步状态", async () => {
    isMaximizedMock.mockResolvedValue(true);
    const { result } = renderHook(
      () => useWindowControls({ useAppWindowControls: false, settingsLoaded: true }),
      { wrapper },
    );
    await act(async () => {
      await result.current.toggleMaximize();
    });
    expect(toggleMaximizeMock).toHaveBeenCalledOnce();
    expect(isMaximizedMock).toHaveBeenCalled();
  });

  it("close 调用窗口关闭", async () => {
    const { result } = renderHook(
      () => useWindowControls({ useAppWindowControls: false, settingsLoaded: true }),
      { wrapper },
    );
    await act(async () => {
      await result.current.close();
    });
    expect(closeMock).toHaveBeenCalledOnce();
  });

  it("装饰同步: useAppWindowControls=true 时设置 decorations=false", async () => {
    renderHook(
      () => useWindowControls({ useAppWindowControls: true, settingsLoaded: true }),
      { wrapper },
    );
    // useEffect 在渲染后异步执行，稍等微任务
    await vi.waitFor(() => {
      expect(setDecorationsMock).toHaveBeenCalledWith(false);
    });
  });

  it("装饰同步: useAppWindowControls=false 时设置 decorations=true", async () => {
    renderHook(
      () => useWindowControls({ useAppWindowControls: false, settingsLoaded: true }),
      { wrapper },
    );
    await vi.waitFor(() => {
      expect(setDecorationsMock).toHaveBeenCalledWith(true);
    });
  });

  it("settingsLoaded=false 时跳过装饰同步", async () => {
    renderHook(
      () => useWindowControls({ useAppWindowControls: false, settingsLoaded: false }),
      { wrapper },
    );
    // 等待一小段时间确认 setDecorations 未被调用
    await vi.waitFor(() => {
      expect(setDecorationsMock).not.toHaveBeenCalled();
    });
  });
});
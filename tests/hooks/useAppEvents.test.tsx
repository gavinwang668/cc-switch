import type { ReactNode } from "react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { I18nextProvider } from "react-i18next";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { useAppEvents } from "@/hooks/useAppEvents";

const testI18n = i18n.createInstance();
testI18n.use(initReactI18next).init({
  lng: "zh",
  fallbackLng: "zh",
  resources: { zh: { translation: {} } },
  interpolation: { escapeValue: false },
});

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

// Mock providers API
const onSwitchedOff = vi.fn();
const onSwitchedMock = vi.fn(async (_handler: unknown) => onSwitchedOff);
const updateTrayMenuMock = vi.fn();
const getAllMock = vi.fn();

vi.mock("@/lib/api", () => ({
  providersApi: {
    onSwitched: (handler: (event: unknown) => void) => onSwitchedMock(handler),
    getAll: () => getAllMock(),
    updateTrayMenu: () => updateTrayMenuMock(),
  },
}));

// Mock env API
const checkAllEnvConflictsMock = vi.fn();
const checkEnvConflictsMock = vi.fn();
vi.mock("@/lib/api/env", () => ({
  checkAllEnvConflicts: (...args: unknown[]) => checkAllEnvConflictsMock(...args),
  checkEnvConflicts: (...args: unknown[]) => checkEnvConflictsMock(...args),
}));

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_migration_result") return Promise.resolve(false);
    if (cmd === "get_skills_migration_result") return Promise.resolve(null);
    return Promise.resolve(null);
  }),
}));

// Mock useTauriEvent
vi.mock("@/hooks/useTauriEvent", () => ({
  useTauriEvent: vi.fn((_event: string, handler: (...args: unknown[]) => void) => {
    // Store handlers for external triggering
    storedHandlers.set(_event, handler);
  }),
}));

const storedHandlers = new Map<string, (...args: unknown[]) => void>();

export function triggerTauriEvent(event: string, payload: unknown) {
  const handler = storedHandlers.get(event);
  if (handler) handler(payload);
}

function wrapper({ children }: { children: ReactNode }) {
  return (
    <I18nextProvider i18n={testI18n}>
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    </I18nextProvider>
  );
}

describe("useAppEvents", () => {
  const refetchMock = vi.fn();
  const setEnvConflictsMock = vi.fn();
  const setShowEnvBannerMock = vi.fn();

  const defaultParams = {
    activeApp: "codex" as const,
    refetch: refetchMock,
    setEnvConflicts: setEnvConflictsMock,
    setShowEnvBanner: setShowEnvBannerMock,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    storedHandlers.clear();
    refetchMock.mockResolvedValue(undefined);
    checkAllEnvConflictsMock.mockResolvedValue({});
    checkEnvConflictsMock.mockResolvedValue([]);
  });

  afterEach(() => {
    sessionStorage.removeItem("env_banner_dismissed");
  });

  it("注册 provider switch 事件监听", () => {
    renderHook(() => useAppEvents(defaultParams), { wrapper });
    expect(onSwitchedMock).toHaveBeenCalled();
  });

  it("providersApi.onSwitched 返回的 off 函数在卸载时调用", async () => {
    const { unmount } = renderHook(() => useAppEvents(defaultParams), { wrapper });
    // 等待异步 setupListener 完成后再卸载
    await vi.waitFor(() => {
      expect(onSwitchedMock).toHaveBeenCalled();
    });
    unmount();
    expect(onSwitchedOff).toHaveBeenCalled();
  });

  it("universal-provider-synced 事件触发后 updateTrayMenu", async () => {
    renderHook(() => useAppEvents(defaultParams), { wrapper });
    triggerTauriEvent("universal-provider-synced", null);
    await waitFor(() => {
      expect(updateTrayMenuMock).toHaveBeenCalled();
    });
  });

  it("启动时检查环境冲突，无冲突时不设置 banner", async () => {
    renderHook(() => useAppEvents(defaultParams), { wrapper });
    await waitFor(() => {
      expect(checkAllEnvConflictsMock).toHaveBeenCalled();
    });
    expect(setEnvConflictsMock).not.toHaveBeenCalled();
    expect(setShowEnvBannerMock).not.toHaveBeenCalled();
  });

  it("启动时检查环境冲突，有冲突时设置 banner", async () => {
    checkAllEnvConflictsMock.mockResolvedValue({
      codex: [{ varName: "API_KEY", sourcePath: "/path/to/config", currentValue: "123", expectedValue: "456" }],
    });
    renderHook(() => useAppEvents(defaultParams), { wrapper });
    await waitFor(() => {
      expect(setEnvConflictsMock).toHaveBeenCalled();
    });
  });
});
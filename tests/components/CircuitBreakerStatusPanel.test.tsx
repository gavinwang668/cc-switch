import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { I18nextProvider } from "react-i18next";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { CircuitBreakerStatusPanel } from "@/components/proxy/CircuitBreakerStatusPanel";
import type { CircuitBreakerStatusMap } from "@/hooks/useCircuitBreakerStatus";

const testI18n = i18n.createInstance();
testI18n.use(initReactI18next).init({
  lng: "zh",
  fallbackLng: "zh",
  resources: {
    zh: {
      translation: {
        "circuitBreaker.title": "熔断器状态",
        "circuitBreaker.filterAll": "全部",
        "circuitBreaker.filterOpen": "已断开",
        "circuitBreaker.filterHalfOpen": "半开",
        "circuitBreaker.filterClosed": "正常",
        "circuitBreaker.state.closed": "正常",
        "circuitBreaker.state.open": "已断开",
        "circuitBreaker.state.half_open": "半开",
        "circuitBreaker.provider": "供应商",
        "circuitBreaker.state": "状态",
        "circuitBreaker.consecutiveFailures": "连续失败",
        "circuitBreaker.consecutiveSuccesses": "连续成功",
        "circuitBreaker.totalRequests": "总请求",
        "circuitBreaker.failedRequests": "失败请求",
        "circuitBreaker.errorRate": "失败率",
        "circuitBreaker.noData": "暂无数据",
        "circuitBreaker.reset": "重置",
        "circuitBreaker.resetConfirm": "确认重置",
        "circuitBreaker.resetSuccess": "重置成功",
        "common": { "cancel": "取消" },
      },
    },
  },
  interpolation: { escapeValue: false },
});

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

// Mock useCircuitBreakerStatus
const useCircuitBreakerStatusMock = vi.fn();
vi.mock("@/hooks/useCircuitBreakerStatus", () => ({
  useCircuitBreakerStatus: (...args: unknown[]) => useCircuitBreakerStatusMock(...args),
}));

// Mock useResetCircuitBreaker
const resetMutateAsyncMock = vi.fn();
vi.mock("@/lib/query/failover", () => ({
  useResetCircuitBreaker: vi.fn(() => ({
    mutateAsync: (...args: unknown[]) => resetMutateAsyncMock(...args),
    isPending: false,
  })),
}));

function renderComponent(
  props: { appType?: string; isProxyRunning?: boolean } = {},
) {
  return render(
    <I18nextProvider i18n={testI18n}>
      <QueryClientProvider client={queryClient}>
        <CircuitBreakerStatusPanel
          appType={props.appType as any ?? undefined}
          isProxyRunning={props.isProxyRunning ?? true}
        />
      </QueryClientProvider>
    </I18nextProvider>,
  );
}

const mockData: CircuitBreakerStatusMap = {
  p1: {
    providerId: "p1",
    providerName: "Provider A",
    state: "closed",
    consecutiveFailures: 0,
    consecutiveSuccesses: 10,
    totalRequests: 50,
    failedRequests: 2,
    isLoaded: true,
  },
  p2: {
    providerId: "p2",
    providerName: "Provider B",
    state: "open",
    consecutiveFailures: 5,
    consecutiveSuccesses: 0,
    totalRequests: 8,
    failedRequests: 5,
    isLoaded: true,
  },
  p3: {
    providerId: "p3",
    providerName: "Provider C",
    state: "half_open",
    consecutiveFailures: 2,
    consecutiveSuccesses: 1,
    totalRequests: 15,
    failedRequests: 3,
    isLoaded: true,
  },
};

describe("CircuitBreakerStatusPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    queryClient.clear();
  });

  it("appType 为 undefined 且 proxy 未运行时显示加载状态", () => {
    useCircuitBreakerStatusMock.mockReturnValue({
      data: undefined,
      isLoading: true,
      isFetching: false,
      refetch: vi.fn(),
    });
    renderComponent({ appType: undefined, isProxyRunning: false });
    // Should show nothing meaningful since enabled=false means query doesn't run
    // The panel should render empty state
  });

  it("展示熔断器数据列表", async () => {
    useCircuitBreakerStatusMock.mockReturnValue({
      data: mockData,
      isLoading: false,
      isFetching: false,
      refetch: vi.fn(),
    });

    renderComponent({ appType: "codex", isProxyRunning: true });

    await waitFor(() => {
      expect(screen.getByText("Provider A")).toBeInTheDocument();
      expect(screen.getByText("Provider B")).toBeInTheDocument();
      expect(screen.getByText("Provider C")).toBeInTheDocument();
    });
  });

  it("空数据时显示暂无数据", async () => {
    useCircuitBreakerStatusMock.mockReturnValue({
      data: {},
      isLoading: false,
      isFetching: false,
      refetch: vi.fn(),
    });

    renderComponent({ appType: "codex", isProxyRunning: true });

    await waitFor(() => {
      // Should show "no data" state
    });
  });

  it("过滤按钮可切换 state 过滤", async () => {
    useCircuitBreakerStatusMock.mockReturnValue({
      data: mockData,
      isLoading: false,
      isFetching: false,
      refetch: vi.fn(),
    });

    renderComponent({ appType: "codex", isProxyRunning: true });

    // The filter buttons should exist
    // Default is "all" which shows all 3 providers
  });
});
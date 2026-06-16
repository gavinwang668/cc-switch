import type { ReactNode } from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { useCircuitBreakerStatus } from "@/hooks/useCircuitBreakerStatus";
import type { AppId } from "@/lib/api";

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const getAllMock = vi.fn();
vi.mock("@/lib/api", () => ({
  providersApi: {
    getAll: (...args: unknown[]) => getAllMock(...args),
  },
}));

const getCircuitBreakerStatsMock = vi.fn();
vi.mock("@/lib/api/failover", () => ({
  failoverApi: {
    getCircuitBreakerStats: (...args: unknown[]) => getCircuitBreakerStatsMock(...args),
  },
}));

function wrapper({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}

const mockProviders = {
  p1: { id: "p1", name: "Provider 1" },
  p2: { id: "p2", name: "Provider 2" },
} as Record<string, { id: string; name: string }>;

const mockStatsClosed = {
  state: "closed" as const,
  consecutiveFailures: 0,
  consecutiveSuccesses: 5,
  totalRequests: 20,
  failedRequests: 1,
};

const mockStatsOpen = {
  state: "open" as const,
  consecutiveFailures: 5,
  consecutiveSuccesses: 0,
  totalRequests: 10,
  failedRequests: 5,
};

describe("useCircuitBreakerStatus", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    queryClient.clear();
  });

  it("空 providers 时返回空 map", async () => {
    getAllMock.mockResolvedValue({});

    const { result } = renderHook(
      () => useCircuitBreakerStatus("codex" as AppId, true),
      { wrapper },
    );

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });
    expect(Object.keys(result.current.data ?? {})).toHaveLength(0);
  });

  it("返回所有 provider 的熔断器状态", async () => {
    getAllMock.mockResolvedValue(mockProviders);
    getCircuitBreakerStatsMock
      .mockResolvedValueOnce(mockStatsClosed)
      .mockResolvedValueOnce(mockStatsOpen);

    const { result } = renderHook(
      () => useCircuitBreakerStatus("codex" as AppId, true),
      { wrapper },
    );

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    const data = result.current.data!;
    expect(Object.keys(data)).toHaveLength(2);
    expect(data.p1.state).toBe("closed");
    expect(data.p1.providerName).toBe("Provider 1");
    expect(data.p2.state).toBe("open");
    expect(data.p2.consecutiveFailures).toBe(5);
  });

  it("某个 provider stats 返回 null 时跳过", async () => {
    getAllMock.mockResolvedValue(mockProviders);
    getCircuitBreakerStatsMock
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce(mockStatsOpen);

    const { result } = renderHook(
      () => useCircuitBreakerStatus("codex" as AppId, true),
      { wrapper },
    );

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    const data = result.current.data!;
    expect(Object.keys(data)).toHaveLength(1);
    expect(data.p2.state).toBe("open");
  });

  it("某个 provider stats 请求异常时跳过", async () => {
    getAllMock.mockResolvedValue(mockProviders);
    getCircuitBreakerStatsMock
      .mockRejectedValueOnce(new Error("network error"))
      .mockResolvedValueOnce(mockStatsClosed);

    const { result } = renderHook(
      () => useCircuitBreakerStatus("codex" as AppId, true),
      { wrapper },
    );

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    const data = result.current.data!;
    expect(Object.keys(data)).toHaveLength(1);
    expect(data.p2.providerName).toBe("Provider 2");
  });
});
import { useQuery } from "@tanstack/react-query";
import { failoverApi } from "@/lib/api/failover";
import type { AppId } from "@/lib/api";
import { providersApi } from "@/lib/api";

export interface CircuitBreakerStatusEntry {
  providerId: string;
  providerName: string;
  state: "closed" | "open" | "half_open";
  consecutiveFailures: number;
  consecutiveSuccesses: number;
  totalRequests: number;
  failedRequests: number;
  isLoaded: boolean;
}

export interface CircuitBreakerStatusMap {
  [providerId: string]: CircuitBreakerStatusEntry;
}

const REFRESH_INTERVAL_MS = 5_000;

/**
 * 汇总一个 App 下所有供应商的熔断器状态。
 *
 * - 自动跳过数据库中不存在的 provider（已被删除但旧缓存还在）
 * - 单一 query 即可订阅整个应用的状态，UI 直接渲染
 * - 每 5 秒自动刷新一次（与 useProviderHealth 保持一致）
 */
export function useCircuitBreakerStatus(appType: AppId, enabled = true) {
  return useQuery({
    queryKey: ["circuitBreakerStatus", appType],
    enabled: enabled && !!appType,
    refetchInterval: REFRESH_INTERVAL_MS,
    queryFn: async (): Promise<CircuitBreakerStatusMap> => {
      const providers = await providersApi.getAll(appType);
      const entries = await Promise.all(
        Object.values(providers).map(async (provider) => {
          try {
            const stats = await failoverApi.getCircuitBreakerStats(
              provider.id,
              appType,
            );
            if (!stats) {
              return null;
            }
            return {
              providerId: provider.id,
              providerName: provider.name,
              state: stats.state,
              consecutiveFailures: stats.consecutiveFailures,
              consecutiveSuccesses: stats.consecutiveSuccesses,
              totalRequests: stats.totalRequests,
              failedRequests: stats.failedRequests,
              isLoaded: true,
            } satisfies CircuitBreakerStatusEntry;
          } catch {
            return null;
          }
        }),
      );

      const map: CircuitBreakerStatusMap = {};
      for (const entry of entries) {
        if (entry) {
          map[entry.providerId] = entry;
        }
      }
      return map;
    },
  });
}

import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import type { CostEstimate, BudgetConfig, BudgetAlert } from "@/types/budget";

export function useCostEstimation() {
  const costQuery = useQuery<CostEstimate>({
    queryKey: ["cost-estimation"],
    queryFn: async () => {
      try {
        return await invoke<CostEstimate>("get_cost_estimation");
      } catch {
        return {
          totalCost: 0,
          dailyCost: 0,
          weeklyCost: 0,
          monthlyCost: 0,
          byProvider: {},
          byModel: {},
          trend: [],
        };
      }
    },
    refetchInterval: 60000,
  });

  return {
    costEstimate: costQuery.data,
    isLoading: costQuery.isLoading,
    error: costQuery.error,
    refetch: costQuery.refetch,
  };
}

export function useBudgetConfig() {
  const configQuery = useQuery<BudgetConfig>({
    queryKey: ["budget-config"],
    queryFn: async () => {
      try {
        return await invoke<BudgetConfig>("get_budget_config");
      } catch {
        return {
          monthlyBudget: 100,
          alertThresholds: [50, 80, 90],
          currency: "USD",
          enabled: false,
        };
      }
    },
  });

  return {
    config: configQuery.data,
    isLoading: configQuery.isLoading,
    refetch: configQuery.refetch,
  };
}

export function useBudgetAlerts() {
  const alertsQuery = useQuery<BudgetAlert[]>({
    queryKey: ["budget-alerts"],
    queryFn: async () => {
      try {
        return await invoke<BudgetAlert[]>("get_budget_alerts");
      } catch {
        return [];
      }
    },
    refetchInterval: 30000,
  });

  return {
    alerts: alertsQuery.data,
    isLoading: alertsQuery.isLoading,
    refetch: alertsQuery.refetch,
  };
}

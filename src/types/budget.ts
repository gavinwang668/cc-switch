export interface BudgetConfig {
  monthlyBudget: number;
  alertThresholds: number[];
  currency: string;
  enabled: boolean;
}

export interface CostEstimate {
  totalCost: number;
  dailyCost: number;
  weeklyCost: number;
  monthlyCost: number;
  byProvider: Record<string, number>;
  byModel: Record<string, number>;
  trend: CostDataPoint[];
}

export interface CostDataPoint {
  date: string;
  cost: number;
  provider?: string;
  model?: string;
}

export interface BudgetAlert {
  id: string;
  threshold: number;
  triggered: boolean;
  triggeredAt?: string;
  message: string;
}

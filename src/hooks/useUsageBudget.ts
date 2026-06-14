import { useCallback, useEffect, useState } from "react";

const STORAGE_KEY = "cc-switch.usageBudget.v1";

export type BudgetPeriod = "day" | "week" | "month";

export interface UsageBudget {
  /** 周期内成本上限（USD），0 表示未启用 */
  amountUsd: number;
  /** 触发告警的百分比，0-100 */
  thresholdPercent: number;
  /** 周期粒度 */
  period: BudgetPeriod;
  /** 是否在已超出预算时持续提示 */
  alertWhenExceeded: boolean;
}

export const DEFAULT_BUDGET: UsageBudget = {
  amountUsd: 0,
  thresholdPercent: 80,
  period: "month",
  alertWhenExceeded: true,
};

function readBudget(): UsageBudget {
  if (typeof window === "undefined") return DEFAULT_BUDGET;
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_BUDGET;
    const parsed = JSON.parse(raw) as Partial<UsageBudget>;
    return {
      ...DEFAULT_BUDGET,
      ...parsed,
    };
  } catch {
    return DEFAULT_BUDGET;
  }
}

function writeBudget(budget: UsageBudget) {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(budget));
  } catch {
    // 静默失败即可
  }
}

/**
 * 读取并维护"使用量预算"配置。
 * 配置写入 localStorage，便于在设置对话框内即时修改。
 */
export function useUsageBudget() {
  const [budget, setBudgetState] = useState<UsageBudget>(() => readBudget());

  // 跨 Tab 同步
  useEffect(() => {
    if (typeof window === "undefined") return;
    const handler = (event: StorageEvent) => {
      if (event.key !== STORAGE_KEY) return;
      setBudgetState(readBudget());
    };
    window.addEventListener("storage", handler);
    return () => window.removeEventListener("storage", handler);
  }, []);

  const setBudget = useCallback((next: UsageBudget) => {
    setBudgetState(next);
    writeBudget(next);
  }, []);

  const resetBudget = useCallback(() => {
    setBudgetState(DEFAULT_BUDGET);
    writeBudget(DEFAULT_BUDGET);
  }, []);

  return { budget, setBudget, resetBudget };
}

export interface BudgetStatus {
  /** 预算是否启用（amountUsd > 0） */
  enabled: boolean;
  /** 实际花费 */
  spent: number;
  /** 配置的上限 */
  limit: number;
  /** 实际花费/上限，0–1+ */
  ratio: number;
  /** 是否已达到告警阈值 */
  reachedThreshold: boolean;
  /** 是否超出预算 */
  exceeded: boolean;
  /** 周期粒度 */
  period: BudgetPeriod;
}

/**
 * 根据当前周期估算应纳入预算的时间范围 (start, end) 时间戳。
 * 与 UsageDashboard 的周期一致：day=自然日, week=本周, month=本月。
 */
export function getBudgetWindow(period: BudgetPeriod, now: Date = new Date()) {
  const end = new Date(now);
  end.setHours(23, 59, 59, 999);
  const start = new Date(now);
  start.setHours(0, 0, 0, 0);

  if (period === "day") {
    return { start, end };
  }

  if (period === "week") {
    // 周一到周日（中国习惯）
    const day = (start.getDay() + 6) % 7; // 0..6, 0 = Monday
    start.setDate(start.getDate() - day);
    return { start, end };
  }

  // month
  start.setDate(1);
  return { start, end };
}

export function buildBudgetStatus(
  budget: UsageBudget,
  spent: number,
): BudgetStatus {
  const limit = Math.max(0, budget.amountUsd);
  const ratio = limit > 0 ? spent / limit : 0;
  const threshold = Math.max(0, Math.min(100, budget.thresholdPercent)) / 100;
  return {
    enabled: limit > 0,
    spent,
    limit,
    ratio,
    reachedThreshold: limit > 0 && ratio >= threshold && ratio < 1,
    exceeded: limit > 0 && ratio >= 1,
    period: budget.period,
  };
}

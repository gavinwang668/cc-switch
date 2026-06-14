import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { AlertTriangle, CheckCircle2, Settings2 } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
  buildBudgetStatus,
  type UsageBudget,
} from "@/hooks/useUsageBudget";

interface UsageBudgetBannerProps {
  budget: UsageBudget;
  spent: number;
  onConfigure: () => void;
  onDismissThreshold?: () => void;
  thresholdDismissed: boolean;
  onDismissExceeded?: () => void;
  exceededDismissed: boolean;
}

const currency = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

export function UsageBudgetBanner({
  budget,
  spent,
  onConfigure,
  onDismissThreshold,
  thresholdDismissed,
  onDismissExceeded,
  exceededDismissed,
}: UsageBudgetBannerProps) {
  const { t } = useTranslation();
  const status = useMemo(() => buildBudgetStatus(budget, spent), [budget, spent]);

  if (!status.enabled) return null;

  const percentRaw = Math.min(status.ratio, 1.5) * 100;
  const percent = Math.max(0, Math.min(100, percentRaw));
  const over = status.ratio > 1;

  let tone: "ok" | "warn" | "danger" = "ok";
  if (over) tone = "danger";
  else if (status.reachedThreshold) tone = "warn";

  const showThresholdAlert =
    tone === "warn" && !thresholdDismissed;
  const showExceededAlert = tone === "danger" && !exceededDismissed;

  if (!showThresholdAlert && !showExceededAlert) {
    return null;
  }

  const headingKey = showExceededAlert
    ? "usage.budget.exceededTitle"
    : "usage.budget.thresholdTitle";

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, y: -6 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -6 }}
        transition={{ duration: 0.2 }}
        className={cn(
          "rounded-xl border p-3 flex flex-col gap-2 shadow-sm",
          tone === "warn" &&
            "border-amber-500/30 bg-amber-50/40 dark:bg-amber-950/20",
          tone === "danger" &&
            "border-red-500/40 bg-red-50/40 dark:bg-red-950/30",
        )}
      >
        <div className="flex items-start gap-3">
          {tone === "danger" ? (
            <AlertTriangle className="h-4 w-4 mt-0.5 text-red-500" />
          ) : (
            <CheckCircle2 className="h-4 w-4 mt-0.5 text-amber-500" />
          )}
          <div className="flex-1 space-y-1">
            <p className="text-sm font-medium leading-tight">
              {t(headingKey, {
                defaultValue: showExceededAlert
                  ? "已超出本周期预算"
                  : "已接近本周期预算上限",
              })}
            </p>
            <p className="text-xs text-muted-foreground">
              {t("usage.budget.summary", {
                defaultValue:
                  "已花费 {{spent}} / 上限 {{limit}} （{{percent}}%）",
                spent: currency.format(status.spent),
                limit: currency.format(status.limit),
                percent: percentRaw.toFixed(1),
              })}
            </p>
          </div>
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7"
              onClick={onConfigure}
              title={t("usage.budget.configure", { defaultValue: "预算设置" })}
            >
              <Settings2 className="h-3.5 w-3.5" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={
                showExceededAlert ? onDismissExceeded : onDismissThreshold
              }
              className="h-7 text-xs"
            >
              {t("usage.budget.dismiss", { defaultValue: "知道了" })}
            </Button>
          </div>
        </div>
        <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
          <div
            className={cn(
              "h-full rounded-full transition-all",
              tone === "warn" && "bg-amber-500",
              tone === "danger" && "bg-red-500",
              tone === "ok" && "bg-emerald-500",
            )}
            style={{ width: `${percent}%` }}
          />
        </div>
      </motion.div>
    </AnimatePresence>
  );
}

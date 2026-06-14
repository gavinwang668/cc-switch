import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Settings2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  DEFAULT_BUDGET,
  type BudgetPeriod,
  type UsageBudget,
} from "@/hooks/useUsageBudget";

interface UsageBudgetDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  budget: UsageBudget;
  onSave: (next: UsageBudget) => void;
  onReset: () => void;
}

const PERIODS: BudgetPeriod[] = ["day", "week", "month"];

export function UsageBudgetDialog({
  open,
  onOpenChange,
  budget,
  onSave,
  onReset,
}: UsageBudgetDialogProps) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState<UsageBudget>(budget);

  useEffect(() => {
    if (open) setDraft(budget);
  }, [budget, open]);

  const handleSave = () => {
    const sanitized: UsageBudget = {
      amountUsd: Math.max(0, Number.isFinite(draft.amountUsd) ? draft.amountUsd : 0),
      thresholdPercent: Math.max(
        0,
        Math.min(100, Math.round(draft.thresholdPercent)),
      ),
      period: draft.period,
      alertWhenExceeded: !!draft.alertWhenExceeded,
    };
    onSave(sanitized);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Settings2 className="h-4 w-4" />
            {t("usage.budget.title", { defaultValue: "成本预算与告警" })}
          </DialogTitle>
          <DialogDescription>
            {t("usage.budget.description", {
              defaultValue:
                "设置一个周期性的成本上限，到达阈值时会在使用量页面顶部提示，避免月底超出预算。",
            })}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="space-y-1.5">
            <Label htmlFor="budget-amount">
              {t("usage.budget.amount", { defaultValue: "预算上限 (USD)" })}
            </Label>
            <Input
              id="budget-amount"
              type="number"
              min={0}
              step="0.01"
              value={Number.isFinite(draft.amountUsd) ? draft.amountUsd : 0}
              onChange={(event) =>
                setDraft((prev) => ({
                  ...prev,
                  amountUsd: Number(event.target.value),
                }))
              }
              placeholder={t("usage.budget.amountPlaceholder", {
                defaultValue: "0 = 关闭预算",
              })}
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1.5">
              <Label htmlFor="budget-period">
                {t("usage.budget.period", { defaultValue: "统计周期" })}
              </Label>
              <Select
                value={draft.period}
                onValueChange={(value) =>
                  setDraft((prev) => ({ ...prev, period: value as BudgetPeriod }))
                }
              >
                <SelectTrigger id="budget-period">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PERIODS.map((period) => (
                    <SelectItem key={period} value={period}>
                      {t(`usage.budget.period.${period}`, {
                        defaultValue:
                          period === "day"
                            ? "每天"
                            : period === "week"
                              ? "每周"
                              : "每月",
                      })}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="budget-threshold">
                {t("usage.budget.threshold", { defaultValue: "告警阈值 (%)" })}
              </Label>
              <Input
                id="budget-threshold"
                type="number"
                min={0}
                max={100}
                step={1}
                value={draft.thresholdPercent}
                onChange={(event) =>
                  setDraft((prev) => ({
                    ...prev,
                    thresholdPercent: Number(event.target.value),
                  }))
                }
              />
            </div>
          </div>

          <div className="flex items-center justify-between rounded-md border border-border bg-muted/30 p-3">
            <div className="space-y-0.5">
              <Label htmlFor="budget-alert-when-exceeded" className="text-sm">
                {t("usage.budget.continuousAlert", {
                  defaultValue: "超额时持续提示",
                })}
              </Label>
              <p className="text-xs text-muted-foreground">
                {t("usage.budget.continuousAlertHint", {
                  defaultValue:
                    "关闭后仅在首次到达阈值时提示一次，再次超额不再重复提醒。",
                })}
              </p>
            </div>
            <Switch
              id="budget-alert-when-exceeded"
              checked={draft.alertWhenExceeded}
              onCheckedChange={(value) =>
                setDraft((prev) => ({ ...prev, alertWhenExceeded: value }))
              }
            />
          </div>
        </div>

        <DialogFooter className="gap-2">
          <Button
            variant="ghost"
            onClick={() => {
              setDraft(DEFAULT_BUDGET);
              onReset();
              onOpenChange(false);
            }}
          >
            {t("usage.budget.reset", { defaultValue: "恢复默认" })}
          </Button>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("common.cancel", { defaultValue: "取消" })}
          </Button>
          <Button onClick={handleSave}>
            {t("common.save", { defaultValue: "保存" })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

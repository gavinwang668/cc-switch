import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useBudgetConfig } from "@/hooks/useCostEstimation";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { toast } from "sonner";
import { Save } from "lucide-react";

export const BudgetAlertSettings: React.FC = () => {
  const { t } = useTranslation();
  const { config, refetch } = useBudgetConfig();
  const [enabled, setEnabled] = useState(config?.enabled ?? false);
  const [monthlyBudget, setMonthlyBudget] = useState(
    config?.monthlyBudget ?? 100,
  );
  const [thresholds, setThresholds] = useState(
    config?.alertThresholds.join(", ") ?? "50, 80, 90",
  );
  const [currency, setCurrency] = useState(config?.currency ?? "USD");
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    setSaving(true);
    try {
      const thresholdArray = thresholds
        .split(",")
        .map((t) => parseInt(t.trim()))
        .filter((n) => !isNaN(n));

      await invoke("save_budget_config", {
        config: {
          enabled,
          monthlyBudget,
          alertThresholds: thresholdArray,
          currency,
        },
      });

      toast.success(t("budgetAlert.saveSuccess"));
      refetch();
    } catch (error) {
      toast.error(t("budgetAlert.saveFailed"));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("budgetAlert.title")}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex items-center space-x-2">
          <Switch
            id="budget-enabled"
            checked={enabled}
            onCheckedChange={setEnabled}
          />
          <Label htmlFor="budget-enabled">{t("budgetAlert.enabled")}</Label>
        </div>

        {enabled && (
          <>
            <div className="space-y-2">
              <Label htmlFor="monthly-budget">
                {t("budgetAlert.monthlyBudget")}
              </Label>
              <Input
                id="monthly-budget"
                type="number"
                value={monthlyBudget}
                onChange={(e) => setMonthlyBudget(Number(e.target.value))}
                min={0}
              />
              <p className="text-xs text-muted-foreground">
                {t("budgetAlert.monthlyBudgetHint")}
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="thresholds">{t("budgetAlert.thresholds")}</Label>
              <Input
                id="thresholds"
                type="text"
                value={thresholds}
                onChange={(e) => setThresholds(e.target.value)}
                placeholder="50, 80, 90"
              />
              <p className="text-xs text-muted-foreground">
                {t("budgetAlert.thresholdsHint")}
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="currency">{t("budgetAlert.currency")}</Label>
              <Input
                id="currency"
                type="text"
                value={currency}
                onChange={(e) => setCurrency(e.target.value)}
                placeholder="USD"
              />
            </div>
          </>
        )}

        <Button onClick={handleSave} disabled={saving} className="w-full">
          <Save className="h-4 w-4 mr-2" />
          {saving ? t("common.saving") : t("common.save")}
        </Button>
      </CardContent>
    </Card>
  );
};

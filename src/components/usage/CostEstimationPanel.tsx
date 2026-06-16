import React from "react";
import { useTranslation } from "react-i18next";
import {
  useCostEstimation,
  useBudgetConfig,
  useBudgetAlerts,
} from "@/hooks/useCostEstimation";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { TrendingUp, DollarSign, AlertTriangle } from "lucide-react";

export const CostEstimationPanel: React.FC = () => {
  const { t } = useTranslation();
  const { costEstimate, isLoading: costLoading } = useCostEstimation();
  const { config: budgetConfig } = useBudgetConfig();
  const { alerts } = useBudgetAlerts();

  if (costLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="text-muted-foreground">{t("common.loading")}</div>
      </div>
    );
  }

  if (!costEstimate) {
    return (
      <Alert>
        <AlertDescription>{t("costEstimation.noData")}</AlertDescription>
      </Alert>
    );
  }

  const triggeredAlerts = alerts?.filter((a) => a.triggered) || [];

  return (
    <div className="space-y-6">
      {triggeredAlerts.length > 0 && (
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            {t("costEstimation.budgetAlert", { count: triggeredAlerts.length })}
          </AlertDescription>
        </Alert>
      )}

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              {t("costEstimation.totalCost")}
            </CardTitle>
            <DollarSign className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              ${costEstimate.totalCost.toFixed(2)}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              {t("costEstimation.dailyCost")}
            </CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              ${costEstimate.dailyCost.toFixed(2)}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              {t("costEstimation.weeklyCost")}
            </CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              ${costEstimate.weeklyCost.toFixed(2)}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">
              {t("costEstimation.monthlyCost")}
            </CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              ${costEstimate.monthlyCost.toFixed(2)}
            </div>
            {budgetConfig?.monthlyBudget && (
              <p className="text-xs text-muted-foreground mt-1">
                {t("costEstimation.budget")}: ${budgetConfig.monthlyBudget}
              </p>
            )}
          </CardContent>
        </Card>
      </div>

      {Object.keys(costEstimate.byProvider).length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>{t("costEstimation.byProvider")}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {Object.entries(costEstimate.byProvider).map(
                ([provider, cost]) => (
                  <div
                    key={provider}
                    className="flex justify-between items-center"
                  >
                    <span className="text-sm">{provider}</span>
                    <span className="text-sm font-medium">
                      ${cost.toFixed(2)}
                    </span>
                  </div>
                ),
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {Object.keys(costEstimate.byModel).length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>{t("costEstimation.byModel")}</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {Object.entries(costEstimate.byModel).map(([model, cost]) => (
                <div key={model} className="flex justify-between items-center">
                  <span className="text-sm">{model}</span>
                  <span className="text-sm font-medium">
                    ${cost.toFixed(2)}
                  </span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

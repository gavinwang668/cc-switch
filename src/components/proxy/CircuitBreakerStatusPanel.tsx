import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Activity,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  RefreshCw,
  Loader2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";
import type { AppId } from "@/lib/api";
import { useResetCircuitBreaker } from "@/lib/query/failover";
import {
  useCircuitBreakerStatus,
  type CircuitBreakerStatusEntry,
} from "@/hooks/useCircuitBreakerStatus";
import { extractErrorMessage } from "@/utils/errorUtils";

interface CircuitBreakerStatusPanelProps {
  /**
   * 当前应用类型。若为 undefined 表示代理未运行/未指定 app，面板自动禁用。
   */
  appType: AppId | undefined;
  /**
   * 代理服务是否正在运行（决定是否发起轮询）。
   */
  isProxyRunning: boolean;
}

type StateFilter = "all" | "open" | "half_open" | "closed";

/**
 * 熔断器实时状态面板
 *
 * 展示每个供应商的熔断器状态、连续失败/成功次数、累计统计，
 * 并支持手动重置单个熔断器（重置后会自动恢复高优先级供应商）。
 */
export function CircuitBreakerStatusPanel({
  appType,
  isProxyRunning,
}: CircuitBreakerStatusPanelProps) {
  const { t } = useTranslation();
  const [filter, setFilter] = useState<StateFilter>("all");

  const enabled = Boolean(appType) && isProxyRunning;
  const { data, isLoading, isFetching, refetch } = useCircuitBreakerStatus(
    appType as AppId,
    enabled,
  );
  const resetCircuitBreaker = useResetCircuitBreaker();

  const entries = useMemo(() => {
    if (!data) return [] as CircuitBreakerStatusEntry[];
    return Object.values(data);
  }, [data]);

  const filtered = useMemo(() => {
    if (filter === "all") return entries;
    return entries.filter((e) => e.state === filter);
  }, [entries, filter]);

  const counts = useMemo(() => {
    return {
      total: entries.length,
      open: entries.filter((e) => e.state === "open").length,
      halfOpen: entries.filter((e) => e.state === "half_open").length,
      closed: entries.filter((e) => e.state === "closed").length,
    };
  }, [entries]);

  const handleReset = async (providerId: string, providerName: string) => {
    if (!appType) return;
    try {
      await resetCircuitBreaker.mutateAsync({
        providerId,
        appType,
      });
      toast.success(
        t("circuitBreaker.status.resetSuccess", {
          name: providerName,
          defaultValue: `已重置 ${providerName} 的熔断器`,
        }),
        { closeButton: true },
      );
    } catch (error) {
      toast.error(
        t("circuitBreaker.status.resetFailed", {
          defaultValue: "重置熔断器失败：{{error}}",
          error: extractErrorMessage(error),
        }),
      );
    }
  };

  if (!appType) {
    return (
      <div className="rounded-lg border border-border bg-muted/30 p-4 text-sm text-muted-foreground">
        {t("circuitBreaker.status.noAppSelected", {
          defaultValue: "请先选择一个应用以查看熔断器状态",
        })}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-2">
        <div>
          <h3 className="text-base font-semibold">
            {t("circuitBreaker.status.title", {
              defaultValue: "熔断器实时状态",
            })}
          </h3>
          <p className="text-xs text-muted-foreground mt-1">
            {t("circuitBreaker.status.subtitle", {
              defaultValue:
                "查看各供应商熔断器状态并手动重置。开启代理后每 5 秒自动刷新。",
            })}
          </p>
        </div>
        <Button
          size="sm"
          variant="outline"
          onClick={() => refetch()}
          disabled={!enabled || isFetching}
        >
          {isFetching ? (
            <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
          ) : (
            <RefreshCw className="mr-1.5 h-3.5 w-3.5" />
          )}
          {t("common.refresh", { defaultValue: "刷新" })}
        </Button>
      </div>

      {/* 顶部统计卡片 */}
      <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
        <SummaryStat
          icon={<Activity className="h-4 w-4" />}
          label={t("circuitBreaker.status.total", { defaultValue: "总数" })}
          value={counts.total}
          variant="default"
        />
        <SummaryStat
          icon={<CheckCircle2 className="h-4 w-4 text-green-500" />}
          label={t("circuitBreaker.status.closed", { defaultValue: "关闭" })}
          value={counts.closed}
          variant="success"
        />
        <SummaryStat
          icon={<AlertTriangle className="h-4 w-4 text-yellow-500" />}
          label={t("circuitBreaker.status.halfOpen", {
            defaultValue: "半开",
          })}
          value={counts.halfOpen}
          variant="warning"
        />
        <SummaryStat
          icon={<XCircle className="h-4 w-4 text-red-500" />}
          label={t("circuitBreaker.status.open", { defaultValue: "打开" })}
          value={counts.open}
          variant="danger"
        />
      </div>

      {/* 状态过滤器 */}
      <div className="flex flex-wrap items-center gap-1.5 text-xs">
        {(["all", "open", "half_open", "closed"] as StateFilter[]).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`rounded-full border px-3 py-1 transition-colors ${
              filter === f
                ? "border-primary bg-primary/10 text-primary"
                : "border-border bg-background text-muted-foreground hover:bg-muted/50"
            }`}
          >
            {filterLabel(f, t)}
          </button>
        ))}
      </div>

      {/* 供应商列表 */}
      {!enabled ? (
        <div className="rounded-lg border border-border bg-muted/30 p-4 text-sm text-muted-foreground">
          {t("circuitBreaker.status.proxyNotRunning", {
            defaultValue: "代理未运行，启动代理后即可查看熔断器状态。",
          })}
        </div>
      ) : isLoading ? (
        <div className="flex items-center gap-2 text-sm text-muted-foreground py-4">
          <Loader2 className="h-4 w-4 animate-spin" />
          {t("circuitBreaker.status.loading", { defaultValue: "加载中..." })}
        </div>
      ) : filtered.length === 0 ? (
        <div className="rounded-lg border border-border bg-muted/30 p-4 text-sm text-muted-foreground">
          {entries.length === 0
            ? t("circuitBreaker.status.empty", {
                defaultValue: "该应用下暂无供应商。",
              })
            : t("circuitBreaker.status.noMatch", {
                defaultValue: "没有匹配当前过滤器的供应商。",
              })}
        </div>
      ) : (
        <div className="space-y-2">
          {filtered.map((entry) => (
            <CircuitBreakerRow
              key={entry.providerId}
              entry={entry}
              appType={appType}
              onReset={() => handleReset(entry.providerId, entry.providerName)}
              isPending={
                resetCircuitBreaker.isPending &&
                resetCircuitBreaker.variables?.providerId === entry.providerId
              }
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface CircuitBreakerRowProps {
  entry: CircuitBreakerStatusEntry;
  appType: AppId;
  onReset: () => void;
  isPending: boolean;
}

function CircuitBreakerRow({
  entry,
  onReset,
  isPending,
}: CircuitBreakerRowProps) {
  const { t } = useTranslation();
  const variant = stateVariant(entry.state);
  const successRate =
    entry.totalRequests > 0
      ? (
          ((entry.totalRequests - entry.failedRequests) / entry.totalRequests) *
          100
        ).toFixed(1)
      : "—";

  return (
    <div className="rounded-lg border border-border bg-card/50 p-3 transition-colors hover:bg-muted/30">
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 min-w-0">
          <variant.Icon className={`h-4 w-4 flex-shrink-0 ${variant.color}`} />
          <div className="min-w-0">
            <p className="text-sm font-medium truncate">{entry.providerName}</p>
            <p className="text-xs text-muted-foreground">{variant.label}</p>
          </div>
        </div>
        <Button
          size="sm"
          variant="outline"
          onClick={onReset}
          disabled={isPending || entry.state === "closed"}
          title={
            entry.state === "closed"
              ? t("circuitBreaker.status.cannotResetClosed", {
                  defaultValue: "关闭状态无需重置",
                })
              : t("circuitBreaker.status.resetTooltip", {
                  defaultValue: "重置熔断器并尝试恢复",
                })
          }
        >
          {isPending ? (
            <Loader2 className="mr-1.5 h-3.5 w-3.5 animate-spin" />
          ) : (
            <RefreshCw className="mr-1.5 h-3.5 w-3.5" />
          )}
          {t("circuitBreaker.status.reset", { defaultValue: "重置" })}
        </Button>
      </div>
      <div className="mt-2 grid grid-cols-2 gap-2 sm:grid-cols-4 text-xs">
        <Metric
          label={t("circuitBreaker.status.consecutiveFailures", {
            defaultValue: "连续失败",
          })}
          value={entry.consecutiveFailures}
          highlight={entry.consecutiveFailures > 0}
        />
        <Metric
          label={t("circuitBreaker.status.consecutiveSuccesses", {
            defaultValue: "连续成功",
          })}
          value={entry.consecutiveSuccesses}
        />
        <Metric
          label={t("circuitBreaker.status.totalRequests", {
            defaultValue: "总请求",
          })}
          value={entry.totalRequests}
        />
        <Metric
          label={t("circuitBreaker.status.successRate", {
            defaultValue: "成功率",
          })}
          value={`${successRate}%`}
        />
      </div>
    </div>
  );
}

interface MetricProps {
  label: string;
  value: number | string;
  highlight?: boolean;
}

function Metric({ label, value, highlight }: MetricProps) {
  return (
    <div className="rounded-md border border-border/60 bg-background/40 px-2 py-1">
      <p className="text-[10px] text-muted-foreground">{label}</p>
      <p
        className={`text-sm font-medium ${
          highlight ? "text-red-500" : "text-foreground"
        }`}
      >
        {value}
      </p>
    </div>
  );
}

interface SummaryStatProps {
  icon: React.ReactNode;
  label: string;
  value: number;
  variant: "default" | "success" | "warning" | "danger";
}

function SummaryStat({ icon, label, value, variant }: SummaryStatProps) {
  const colorMap: Record<SummaryStatProps["variant"], string> = {
    default: "border-border bg-card/50",
    success: "border-green-500/30 bg-green-500/5",
    warning: "border-yellow-500/30 bg-yellow-500/5",
    danger: "border-red-500/30 bg-red-500/5",
  };
  return (
    <div
      className={`flex items-center gap-2 rounded-lg border p-3 ${colorMap[variant]}`}
    >
      {icon}
      <div>
        <p className="text-[10px] text-muted-foreground">{label}</p>
        <p className="text-lg font-semibold">{value}</p>
      </div>
    </div>
  );
}

function filterLabel(
  filter: StateFilter,
  t: (key: string, opts?: { defaultValue?: string }) => string,
): string {
  switch (filter) {
    case "all":
      return t("circuitBreaker.status.filterAll", { defaultValue: "全部" });
    case "open":
      return t("circuitBreaker.status.open", { defaultValue: "打开" });
    case "half_open":
      return t("circuitBreaker.status.halfOpen", { defaultValue: "半开" });
    case "closed":
      return t("circuitBreaker.status.closed", { defaultValue: "关闭" });
  }
}

function stateVariant(state: "closed" | "open" | "half_open") {
  switch (state) {
    case "closed":
      return {
        Icon: CheckCircle2,
        color: "text-green-500",
        label: "Closed",
      };
    case "half_open":
      return {
        Icon: AlertTriangle,
        color: "text-yellow-500",
        label: "Half-Open",
      };
    case "open":
      return {
        Icon: XCircle,
        color: "text-red-500",
        label: "Open",
      };
  }
}

import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Activity,
  CheckCircle2,
  AlertTriangle,
  XCircle,
  RefreshCw,
  Filter,
  Trash2,
  ScrollText,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useFailoverLog } from "@/hooks/useFailoverLog";
import type { FailoverEvent, FailoverReason, FailoverResult } from "@/types/failover";

type AppFilter = "all" | "claude" | "codex" | "gemini";

/**
 * Failover 事件日志面板
 *
 * 展示近期故障转移事件（默认最近 500 条），支持：
 * - 按应用类型过滤
 * - 按结果/原因过滤
 * - 关键词搜索
 * - 一键清空
 */
export function FailoverLogPanel() {
  const { t } = useTranslation();
  const { events, clear } = useFailoverLog();
  const [appFilter, setAppFilter] = useState<AppFilter>("all");
  const [resultFilter, setResultFilter] = useState<"all" | FailoverResult>("all");
  const [search, setSearch] = useState("");

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    return events.filter((e) => {
      if (appFilter !== "all" && e.appType !== appFilter) return false;
      if (resultFilter !== "all" && e.result !== resultFilter) return false;
      if (q) {
        const haystack = `${e.providerName} ${e.providerId} ${e.errorMessage ?? ""}`.toLowerCase();
        if (!haystack.includes(q)) return false;
      }
      return true;
    });
  }, [events, appFilter, resultFilter, search]);

  const counts = useMemo(() => {
    return {
      total: events.length,
      success: events.filter((e) => e.result === "success").length,
      failed: events.filter((e) => e.result === "failed").length,
      skipped: events.filter((e) => e.result === "skipped").length,
    };
  }, [events]);

  const handleClear = () => {
    if (events.length === 0) return;
    if (window.confirm(t("failoverLog.confirmClear", { defaultValue: "确认清空全部故障转移日志？" }))) {
      clear();
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-2">
        <div>
          <h3 className="text-base font-semibold">
            {t("failoverLog.title", { defaultValue: "Failover 事件日志" })}
          </h3>
          <p className="text-xs text-muted-foreground mt-1">
            {t("failoverLog.subtitle", {
              defaultValue: "展示代理运行期间的故障转移事件，仅保存在内存中。",
            })}
          </p>
        </div>
        <Button
          size="sm"
          variant="outline"
          onClick={handleClear}
          disabled={events.length === 0}
        >
          <Trash2 className="mr-1.5 h-3.5 w-3.5" />
          {t("failoverLog.clear", { defaultValue: "清空" })}
        </Button>
      </div>

      {/* 顶部统计卡片 */}
      <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
        <SummaryStat
          icon={<Activity className="h-4 w-4" />}
          label={t("failoverLog.total", { defaultValue: "总数" })}
          value={counts.total}
        />
        <SummaryStat
          icon={<CheckCircle2 className="h-4 w-4 text-green-500" />}
          label={t("failoverLog.success", { defaultValue: "成功" })}
          value={counts.success}
        />
        <SummaryStat
          icon={<AlertTriangle className="h-4 w-4 text-yellow-500" />}
          label={t("failoverLog.skipped", { defaultValue: "跳过" })}
          value={counts.skipped}
        />
        <SummaryStat
          icon={<XCircle className="h-4 w-4 text-red-500" />}
          label={t("failoverLog.failed", { defaultValue: "失败" })}
          value={counts.failed}
        />
      </div>

      {/* 过滤栏 */}
      <div className="flex flex-wrap items-center gap-2">
        <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
          <Filter className="h-3.5 w-3.5" />
          {t("failoverLog.filter", { defaultValue: "过滤" })}
        </div>
        <Select value={appFilter} onValueChange={(v) => setAppFilter(v as AppFilter)}>
          <SelectTrigger className="h-8 w-32 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("failoverLog.appAll", { defaultValue: "全部应用" })}</SelectItem>
            <SelectItem value="claude">Claude</SelectItem>
            <SelectItem value="codex">Codex</SelectItem>
            <SelectItem value="gemini">Gemini</SelectItem>
          </SelectContent>
        </Select>
        <Select value={resultFilter} onValueChange={(v) => setResultFilter(v as "all" | FailoverResult)}>
          <SelectTrigger className="h-8 w-32 text-xs">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t("failoverLog.resultAll", { defaultValue: "全部结果" })}</SelectItem>
            <SelectItem value="success">{t("failoverLog.success", { defaultValue: "成功" })}</SelectItem>
            <SelectItem value="skipped">{t("failoverLog.skipped", { defaultValue: "跳过" })}</SelectItem>
            <SelectItem value="failed">{t("failoverLog.failed", { defaultValue: "失败" })}</SelectItem>
          </SelectContent>
        </Select>
        <Input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder={t("failoverLog.searchPlaceholder", {
            defaultValue: "搜索 Provider 名称或 ID",
          })}
          className="h-8 w-48 text-xs"
        />
        {(appFilter !== "all" || resultFilter !== "all" || search) && (
          <Button
            variant="ghost"
            size="sm"
            className="h-8 text-xs"
            onClick={() => {
              setAppFilter("all");
              setResultFilter("all");
              setSearch("");
            }}
          >
            <RefreshCw className="mr-1.5 h-3.5 w-3.5" />
            {t("failoverLog.resetFilter", { defaultValue: "重置" })}
          </Button>
        )}
      </div>

      {/* 事件列表 */}
      {events.length === 0 ? (
        <div className="rounded-lg border border-border bg-muted/30 p-6 text-sm text-muted-foreground flex flex-col items-center gap-2">
          <ScrollText className="h-6 w-6" />
          <span>
            {t("failoverLog.empty", {
              defaultValue: "暂无故障转移事件。代理运行后会在此显示。",
            })}
          </span>
        </div>
      ) : filtered.length === 0 ? (
        <div className="rounded-lg border border-border bg-muted/30 p-4 text-sm text-muted-foreground">
          {t("failoverLog.noMatch", { defaultValue: "没有匹配当前过滤条件的事件。" })}
        </div>
      ) : (
        <div className="space-y-1.5 max-h-[420px] overflow-y-auto pr-1">
          {filtered.map((event) => (
            <FailoverRow key={event.id} event={event} />
          ))}
        </div>
      )}
    </div>
  );
}

interface SummaryStatProps {
  icon: React.ReactNode;
  label: string;
  value: number;
}

function SummaryStat({ icon, label, value }: SummaryStatProps) {
  return (
    <div className="flex items-center gap-2 rounded-lg border border-border bg-card/50 p-3">
      {icon}
      <div>
        <p className="text-[10px] text-muted-foreground">{label}</p>
        <p className="text-lg font-semibold">{value}</p>
      </div>
    </div>
  );
}

function FailoverRow({ event }: { event: FailoverEvent }) {
  const { t } = useTranslation();
  const variant = resultVariant(event.result);
  return (
    <div className="rounded-md border border-border bg-card/30 p-2.5 text-xs hover:bg-muted/30">
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2 min-w-0">
          <variant.Icon className={`h-3.5 w-3.5 flex-shrink-0 ${variant.color}`} />
          <span className="font-medium truncate">{event.providerName}</span>
          <span className="text-muted-foreground">·</span>
          <span className="text-muted-foreground capitalize">{event.appType}</span>
        </div>
        <time className="text-muted-foreground tabular-nums">
          {formatTime(event.timestamp)}
        </time>
      </div>
      <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-muted-foreground">
        <span>{reasonLabel(event.reason, t)}</span>
        {typeof event.durationMs === "number" && (
          <>
            <span>·</span>
            <span>{event.durationMs}ms</span>
          </>
        )}
        {event.errorMessage && (
          <>
            <span>·</span>
            <span className="text-red-500 truncate max-w-[280px]" title={event.errorMessage}>
              {event.errorMessage}
            </span>
          </>
        )}
      </div>
    </div>
  );
}

function resultVariant(result: FailoverResult) {
  switch (result) {
    case "success":
      return { Icon: CheckCircle2, color: "text-green-500" };
    case "skipped":
      return { Icon: AlertTriangle, color: "text-yellow-500" };
    case "failed":
      return { Icon: XCircle, color: "text-red-500" };
  }
}

function reasonLabel(
  reason: FailoverReason,
  t: (key: string, opts?: { defaultValue?: string }) => string,
): string {
  switch (reason) {
    case "consecutive_failures":
      return t("failoverLog.reason.consecutiveFailures", { defaultValue: "连续失败" });
    case "timeout":
      return t("failoverLog.reason.timeout", { defaultValue: "超时" });
    case "stream_error":
      return t("failoverLog.reason.streamError", { defaultValue: "流式错误" });
    case "non_stream_error":
      return t("failoverLog.reason.nonStreamError", { defaultValue: "非流式错误" });
    case "unknown":
    default:
      return t("failoverLog.reason.unknown", { defaultValue: "未知原因" });
  }
}

function formatTime(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return iso;
  }
}

import { useTranslation } from "react-i18next";
import { CheckCircle2, AlertCircle, Activity } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import type { McpConnectionTestResult } from "@/lib/api/mcp";

interface McpTestResultDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  serverName: string;
  result: McpConnectionTestResult;
}

/**
 * MCP 连接测试结果展示对话框
 *
 * - 成功 / 失败分别用不同色块展示
 * - 附带传输类型、耗时与 stderr 摘要
 */
export function McpTestResultDialog({
  open,
  onOpenChange,
  serverName,
  result,
}: McpTestResultDialogProps) {
  const { t } = useTranslation();
  const isSuccess = result.success;
  const transportLabel = transportDisplay(result.transport);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {isSuccess ? (
              <CheckCircle2 className="h-4 w-4 text-green-500" />
            ) : (
              <AlertCircle className="h-4 w-4 text-red-500" />
            )}
            {t("mcp.testResult.title", { defaultValue: "连接测试结果" })}
          </DialogTitle>
          <DialogDescription>
            {t("mcp.testResult.subtitle", {
              defaultValue: "目标 MCP 服务器：{{name}}",
              name: serverName,
            })}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3 py-2">
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div className="rounded-md border border-border bg-muted/30 p-2.5">
              <p className="text-[10px] uppercase text-muted-foreground">
                {t("mcp.testResult.transport", { defaultValue: "传输方式" })}
              </p>
              <p className="mt-1 font-medium">{transportLabel}</p>
            </div>
            <div className="rounded-md border border-border bg-muted/30 p-2.5">
              <p className="text-[10px] uppercase text-muted-foreground">
                {t("mcp.testResult.duration", { defaultValue: "耗时" })}
              </p>
              <p className="mt-1 font-medium tabular-nums">
                {result.durationMs} ms
              </p>
            </div>
          </div>

          <div
            className={`rounded-md border p-3 text-sm ${
              isSuccess
                ? "border-green-500/30 bg-green-50/40 dark:bg-green-950/20"
                : "border-red-500/30 bg-red-50/40 dark:bg-red-950/20"
            }`}
          >
            {isSuccess ? (
              <p className="flex items-start gap-2">
                <Activity className="mt-0.5 h-3.5 w-3.5 text-green-500 flex-shrink-0" />
                <span className="break-words">
                  {result.message ||
                    t("mcp.testResult.successDefault", {
                      defaultValue: "连接测试通过",
                    })}
                </span>
              </p>
            ) : (
              <p className="flex items-start gap-2">
                <AlertCircle className="mt-0.5 h-3.5 w-3.5 text-red-500 flex-shrink-0" />
                <span className="break-words">
                  {result.error ||
                    t("mcp.testResult.failedDefault", {
                      defaultValue: "连接测试失败",
                    })}
                </span>
              </p>
            )}
          </div>

          {result.stderrTail && (
            <details className="rounded-md border border-border bg-muted/20 p-2 text-xs">
              <summary className="cursor-pointer text-muted-foreground">
                {t("mcp.testResult.stderr", { defaultValue: "stderr 摘要" })}
              </summary>
              <pre className="mt-2 max-h-40 overflow-auto whitespace-pre-wrap break-all text-foreground/80">
                {result.stderrTail}
              </pre>
            </details>
          )}
        </div>

        <DialogFooter>
          <Button onClick={() => onOpenChange(false)}>
            {t("common.close", { defaultValue: "关闭" })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function transportDisplay(transport: McpConnectionTestResult["transport"]) {
  switch (transport) {
    case "stdio":
      return "stdio";
    case "http":
      return "HTTP";
    case "sse":
      return "SSE";
    default:
      return transport;
  }
}

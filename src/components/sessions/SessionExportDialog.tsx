import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Download, FileJson, FileText, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { exportSessions, type ExportFormat } from "@/lib/api/sessionExport";
import type { SessionMeta } from "@/types";
import { extractErrorMessage } from "@/utils/errorUtils";

interface SessionExportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  sessions: SessionMeta[];
}

/**
 * 会话导出对话框
 *
 * - 支持 Markdown 和 JSON 两种格式
 * - 支持单个或多个会话批量导出
 */
export function SessionExportDialog({
  open,
  onOpenChange,
  sessions,
}: SessionExportDialogProps) {
  const { t } = useTranslation();
  const [format, setFormat] = useState<ExportFormat>("markdown");
  const [isExporting, setIsExporting] = useState(false);

  const handleExport = async () => {
    if (sessions.length === 0) return;
    setIsExporting(true);
    try {
      const result = await exportSessions({ format, sessions });
      if (result) {
        toast.success(
          t("sessionExport.success", {
            defaultValue: "已导出 {{sessions}} 个会话 ({{messages}} 条消息)",
            sessions: result.sessions,
            messages: result.messages,
          }),
          { closeButton: true, description: result.filePath },
        );
        onOpenChange(false);
      }
    } catch (error) {
      toast.error(
        t("sessionExport.failed", {
          defaultValue: "导出失败：{{error}}",
          error: extractErrorMessage(error),
        }),
      );
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>
            {t("sessionExport.title", { defaultValue: "导出会话" })}
          </DialogTitle>
          <DialogDescription>
            {t("sessionExport.description", {
              defaultValue: "将会话内容导出为 Markdown 或 JSON 文件。",
            })}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="text-sm text-muted-foreground">
            {t("sessionExport.selectedCount", {
              defaultValue: "已选择 {{count}} 个会话",
              count: sessions.length,
            })}
          </div>

          <div className="space-y-2">
            <Label>
              {t("sessionExport.format", { defaultValue: "导出格式" })}
            </Label>
            <div className="grid grid-cols-2 gap-2">
              <FormatOption
                active={format === "markdown"}
                onClick={() => setFormat("markdown")}
                icon={<FileText className="h-4 w-4" />}
                title="Markdown"
                description={t("sessionExport.markdownHint", {
                  defaultValue: "适合阅读和分享",
                })}
              />
              <FormatOption
                active={format === "json"}
                onClick={() => setFormat("json")}
                icon={<FileJson className="h-4 w-4" />}
                title="JSON"
                description={t("sessionExport.jsonHint", {
                  defaultValue: "适合备份与二次处理",
                })}
              />
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="ghost"
            onClick={() => onOpenChange(false)}
            disabled={isExporting}
          >
            {t("common.cancel", { defaultValue: "取消" })}
          </Button>
          <Button
            onClick={() => void handleExport()}
            disabled={isExporting || sessions.length === 0}
          >
            {isExporting ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                {t("sessionExport.exporting", { defaultValue: "导出中..." })}
              </>
            ) : (
              <>
                <Download className="mr-2 h-4 w-4" />
                {t("sessionExport.export", { defaultValue: "导出" })}
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

interface FormatOptionProps {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  title: string;
  description: string;
}

function FormatOption({
  active,
  onClick,
  icon,
  title,
  description,
}: FormatOptionProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`flex items-start gap-2 rounded-md border p-3 text-left transition-colors ${
        active
          ? "border-primary bg-primary/5 ring-1 ring-primary"
          : "border-border hover:bg-muted/30"
      }`}
    >
      <div className="mt-0.5 text-muted-foreground">{icon}</div>
      <div className="min-w-0">
        <p className="text-sm font-medium">{title}</p>
        <p className="text-xs text-muted-foreground truncate">{description}</p>
      </div>
    </button>
  );
}

import React from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle, XCircle, Loader, AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  useMcpConnectionTest,
} from "@/hooks/useMcpConnectionTest";

interface McpConnectionTestProps {
  serverId: string;
  serverName?: string;
}

export const McpConnectionTest: React.FC<McpConnectionTestProps> = ({
  serverId,
  serverName: _serverName,
}) => {
  const { t } = useTranslation();
  const { testStatus, testResult, testConnection, resetTest } =
    useMcpConnectionTest();

  const handleTest = async () => {
    await testConnection(serverId);
  };

  const getStatusIcon = () => {
    switch (testStatus) {
      case "testing":
        return <Loader className="h-4 w-4 animate-spin text-blue-500" />;
      case "success":
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case "error":
        return <XCircle className="h-4 w-4 text-red-500" />;
      default:
        return null;
    }
  };

  const getStatusBadge = () => {
    switch (testStatus) {
      case "testing":
        return (
          <Badge variant="secondary" className="gap-1">
            <Loader className="h-3 w-3 animate-spin" />
            {t("mcp.test.testing")}
          </Badge>
        );
      case "success":
        return (
          <Badge variant="default" className="gap-1 bg-green-500">
            <CheckCircle className="h-3 w-3" />
            {t("mcp.test.success")}
          </Badge>
        );
      case "error":
        return (
          <Badge variant="destructive" className="gap-1">
            <XCircle className="h-3 w-3" />
            {t("mcp.test.failed")}
          </Badge>
        );
      default:
        return null;
    }
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={handleTest}
          disabled={testStatus === "testing"}
          className="gap-2"
        >
          {testStatus === "testing" ? (
            <Loader className="h-4 w-4 animate-spin" />
          ) : (
            <CheckCircle className="h-4 w-4" />
          )}
          {t("mcp.test.button")}
        </Button>

        {getStatusBadge()}

        {testResult && (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={resetTest}
            className="text-xs"
          >
            {t("mcp.test.clear")}
          </Button>
        )}
      </div>

      {testResult && (
        <div
          className={`rounded-lg border p-4 ${
            testResult.success
              ? "border-green-200 bg-green-50 dark:border-green-900 dark:bg-green-950"
              : "border-red-200 bg-red-50 dark:border-red-900 dark:bg-red-950"
          }`}
        >
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {getStatusIcon()}
                <span className="font-medium">
                  {testResult.success
                    ? t("mcp.test.successTitle")
                    : t("mcp.test.failedTitle")}
                </span>
              </div>
              <span className="text-sm text-muted-foreground">
                {formatDuration(testResult.durationMs)}
              </span>
            </div>

            <div className="text-sm space-y-1">
              <div className="flex gap-2">
                <span className="font-medium text-muted-foreground">
                  {t("mcp.test.transport")}:
                </span>
                <Badge variant="outline" className="text-xs">
                  {testResult.transport}
                </Badge>
              </div>

              {testResult.message && (
                <div className="flex gap-2">
                  <span className="font-medium text-muted-foreground">
                    {t("mcp.test.message")}:
                  </span>
                  <span>{testResult.message}</span>
                </div>
              )}

              {testResult.error && (
                <div className="flex gap-2 items-start">
                  <AlertCircle className="h-4 w-4 text-red-500 mt-0.5 flex-shrink-0" />
                  <div className="flex-1">
                    <span className="font-medium text-red-700 dark:text-red-400">
                      {t("mcp.test.error")}:
                    </span>
                    <pre className="mt-1 text-xs bg-red-100 dark:bg-red-900 p-2 rounded overflow-x-auto">
                      {testResult.error}
                    </pre>
                  </div>
                </div>
              )}

              {testResult.stderr && (
                <div className="flex gap-2 items-start">
                  <span className="font-medium text-muted-foreground">
                    {t("mcp.test.stderr")}:
                  </span>
                  <pre className="flex-1 text-xs bg-gray-100 dark:bg-gray-900 p-2 rounded overflow-x-auto">
                    {testResult.stderr}
                  </pre>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

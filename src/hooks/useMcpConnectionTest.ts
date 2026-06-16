import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { McpConnectionTestResult } from "@/types/mcp";

export type TestStatus = "idle" | "testing" | "success" | "error";

export function useMcpConnectionTest() {
  const [testStatus, setTestStatus] = useState<TestStatus>("idle");
  const [testResult, setTestResult] = useState<McpConnectionTestResult | null>(
    null,
  );

  const testConnection = useCallback(async (serverId: string) => {
    setTestStatus("testing");
    setTestResult(null);

    try {
      const result = await invoke<McpConnectionTestResult>(
        "test_mcp_connection",
        {
          serverId,
        },
      );
      setTestResult(result);
      setTestStatus(result.success ? "success" : "error");
      return result;
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      const errorResult = {
        success: false,
        error: errorMsg,
        durationMs: 0,
      };
      setTestResult(errorResult);
      setTestStatus("error");
      return errorResult;
    }
  }, []);

  const resetTest = useCallback(() => {
    setTestStatus("idle");
    setTestResult(null);
  }, []);

  return {
    testStatus,
    testResult,
    testConnection,
    resetTest,
  };
}

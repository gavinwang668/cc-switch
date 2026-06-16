import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { I18nextProvider } from "react-i18next";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { SessionExportDialog } from "@/components/sessions/SessionExportDialog";
import type { SessionMeta } from "@/types";

const testI18n = i18n.createInstance();
testI18n.use(initReactI18next).init({
  lng: "zh",
  fallbackLng: "zh",
  resources: {
    zh: {
      translation: {
        "sessionExport.title": "导出会话",
        "sessionExport.description": "将会话内容导出为 Markdown 或 JSON 文件。",
        "sessionExport.selectedCount": "已选择 {{count}} 个会话",
        "sessionExport.format": "导出格式",
        "sessionExport.markdownHint": "适合阅读和分享",
        "sessionExport.jsonHint": "适合备份与二次处理",
        "sessionExport.export": "导出",
        "sessionExport.exporting": "导出中...",
        "sessionExport.success": "已导出 {{sessions}} 个会话 ({{messages}} 条消息)",
        "sessionExport.failed": "导出失败：{{error}}",
        "common.cancel": "取消",
      },
    },
  },
  interpolation: { escapeValue: false },
});

const exportSessionsMock = vi.fn();
vi.mock("@/lib/api/sessionExport", () => ({
  exportSessions: (...args: unknown[]) => exportSessionsMock(...args),
}));

const mockSessions = [
  { providerId: "p1", sessionId: "s1", title: "Session 1", createdAt: 1000, sourcePath: "/path/to/session1.json" },
  { providerId: "p2", sessionId: "s2", title: "Session 2", createdAt: 2000, sourcePath: "/path/to/session2.json" },
] as SessionMeta[];

function renderComponent(
  open = true,
  sessions: SessionMeta[] = mockSessions,
  onOpenChange = vi.fn(),
) {
  return render(
    <I18nextProvider i18n={testI18n}>
      <SessionExportDialog
        open={open}
        onOpenChange={onOpenChange}
        sessions={sessions}
      />
    </I18nextProvider>,
  );
}

describe("SessionExportDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("open=false 时不渲染", () => {
    const { container } = renderComponent(false, mockSessions);
    expect(container.textContent).toBe("");
  });

  it("显示已选择的会话数量", async () => {
    renderComponent(true, mockSessions);
    await waitFor(() => {
      expect(screen.getByText("已选择 2 个会话")).toBeInTheDocument();
    });
  });

  it("默认导出格式为 Markdown", async () => {
    renderComponent(true, mockSessions);
    await waitFor(() => {
      // Markdown should be active by default
      expect(screen.getByText("Markdown")).toBeInTheDocument();
    });
  });

  it("点击导出按钮调用 exportSessions", async () => {
    const onOpenChange = vi.fn();
    exportSessionsMock.mockResolvedValue({
      sessions: 2,
      messages: 8,
      filePath: "/tmp/export.md",
    });

    renderComponent(true, mockSessions, onOpenChange);
    const user = userEvent.setup();

    await waitFor(() => {
      expect(screen.getByText("导出")).toBeInTheDocument();
    });

    await user.click(screen.getByText("导出"));

    await waitFor(() => {
      expect(exportSessionsMock).toHaveBeenCalledWith({
        format: "markdown",
        sessions: mockSessions,
      });
    });
  });

  it("无会话时导出按钮禁用", async () => {
    renderComponent(true, []);
    await waitFor(() => {
      const exportBtn = screen.getByText("导出").closest("button");
      expect(exportBtn).toBeDisabled();
    });
  });

  it("导出失败时显示错误 toast", async () => {
    exportSessionsMock.mockRejectedValue(new Error("IO error"));

    renderComponent(true, mockSessions);
    const user = userEvent.setup();

    await waitFor(() => {
      expect(screen.getByText("导出")).toBeInTheDocument();
    });

    await user.click(screen.getByText("导出"));
    // Error toast should be shown - wait for async
    await vi.waitFor(() => {
      expect(exportSessionsMock).toHaveBeenCalled();
    });
  });
});
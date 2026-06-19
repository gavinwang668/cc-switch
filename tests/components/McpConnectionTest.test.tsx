import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { McpConnectionTest } from "@/components/mcp/McpConnectionTest";
import type { McpConnectionTestResult } from "@/types/mcp";
import type { TestStatus } from "@/hooks/useMcpConnectionTest";

// Mock useMcpConnectionTest hook
const mockTestConnection = vi.fn();
const mockResetTest = vi.fn();

interface MockHookReturn {
  testStatus: TestStatus;
  testResult: McpConnectionTestResult | null;
  testConnection: typeof mockTestConnection;
  resetTest: typeof mockResetTest;
}

const mockUseMcpConnectionTest = vi.fn(
  (): MockHookReturn => ({
    testStatus: "idle",
    testResult: null,
    testConnection: mockTestConnection,
    resetTest: mockResetTest,
  }),
);

vi.mock("@/hooks/useMcpConnectionTest", () => ({
  useMcpConnectionTest: () => mockUseMcpConnectionTest(),
}));

// Mock useTranslation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("McpConnectionTest", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseMcpConnectionTest.mockReturnValue({
      testStatus: "idle",
      testResult: null,
      testConnection: mockTestConnection,
      resetTest: mockResetTest,
    });
  });

  it("renders test button with correct text", () => {
    render(<McpConnectionTest serverId="test-server" />);
    
    const button = screen.getByRole("button", { name: /mcp\.test\.button/i });
    expect(button).toBeInTheDocument();
  });

  it("calls testConnection when button is clicked", () => {
    render(<McpConnectionTest serverId="test-server" serverName="Test Server" />);
    
    const button = screen.getByRole("button", { name: /mcp\.test\.button/i });
    fireEvent.click(button);
    
    expect(mockTestConnection).toHaveBeenCalledWith("test-server");
  });

  it("disables button during testing", () => {
    mockUseMcpConnectionTest.mockReturnValue({
      testStatus: "testing",
      testResult: null,
      testConnection: mockTestConnection,
      resetTest: mockResetTest,
    });

    render(<McpConnectionTest serverId="test-server" />);
    
    const button = screen.getByRole("button", { name: /mcp\.test\.button/i });
    expect(button).toBeDisabled();
  });

  it("shows success badge when test succeeds", () => {
    mockUseMcpConnectionTest.mockReturnValue({
      testStatus: "success",
      testResult: {
        success: true,
        durationMs: 150,
        transport: "stdio",
        message: "Connection successful",
      } as McpConnectionTestResult,
      testConnection: mockTestConnection,
      resetTest: mockResetTest,
    });

    render(<McpConnectionTest serverId="test-server" />);
    
    expect(screen.getByText("mcp.test.success")).toBeInTheDocument();
  });

  it("shows error badge when test fails", () => {
    mockUseMcpConnectionTest.mockReturnValue({
      testStatus: "error",
      testResult: {
        success: false,
        durationMs: 500,
        error: "Connection timeout",
      } as McpConnectionTestResult,
      testConnection: mockTestConnection,
      resetTest: mockResetTest,
    });

    render(<McpConnectionTest serverId="test-server" />);
    
    expect(screen.getByText("mcp.test.failed")).toBeInTheDocument();
  });

  it("shows clear button when test result exists", () => {
    mockUseMcpConnectionTest.mockReturnValue({
      testStatus: "success",
      testResult: {
        success: true,
        durationMs: 150,
      } as McpConnectionTestResult,
      testConnection: mockTestConnection,
      resetTest: mockResetTest,
    });

    render(<McpConnectionTest serverId="test-server" />);
    
    const clearButton = screen.getByRole("button", { name: /mcp\.test\.clear/i });
    expect(clearButton).toBeInTheDocument();
    
    fireEvent.click(clearButton);
    expect(mockResetTest).toHaveBeenCalled();
  });

  it("formats duration correctly for milliseconds", () => {
    mockUseMcpConnectionTest.mockReturnValue({
      testStatus: "success",
      testResult: {
        success: true,
        durationMs: 450,
      } as McpConnectionTestResult,
      testConnection: mockTestConnection,
      resetTest: mockResetTest,
    });

    render(<McpConnectionTest serverId="test-server" />);
    
    expect(screen.getByText("450ms")).toBeInTheDocument();
  });

  it("formats duration correctly for seconds", () => {
    mockUseMcpConnectionTest.mockReturnValue({
      testStatus: "success",
      testResult: {
        success: true,
        durationMs: 2500,
      } as McpConnectionTestResult,
      testConnection: mockTestConnection,
      resetTest: mockResetTest,
    });

    render(<McpConnectionTest serverId="test-server" />);
    
    expect(screen.getByText("2.50s")).toBeInTheDocument();
  });
});

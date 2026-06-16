export interface McpServer {
  id: string;
  name: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
  enabled: boolean;
  description?: string;
  icon?: string;
  createdAt: number;
  updatedAt: number;
}

export interface McpConnectionTestResult {
  success: boolean;
  transport?: string;
  message?: string;
  error?: string;
  stderr?: string;
  durationMs: number;
}

export interface McpValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

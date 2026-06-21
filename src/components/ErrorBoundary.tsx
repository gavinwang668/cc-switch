import React, { Component } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";

interface ErrorBoundaryProps {
  children: React.ReactNode;
  /** 自定义错误消息，默认使用 i18n key */
  fallbackTitle?: string;
  fallbackDescription?: string;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: React.ErrorInfo | null;
}

export class ErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
    this.setState({ errorInfo });
    // 输出到控制台便于调试
    console.error("[ErrorBoundary] 捕获到渲染错误:", error, errorInfo);
  }

  handleRetry = (): void => {
    this.setState({ hasError: false, error: null, errorInfo: null });
  };

  render() {
    if (this.state.hasError) {
      const { fallbackTitle, fallbackDescription } = this.props;
      const title =
        fallbackTitle ??
        "应用遇到错误";
      const description =
        fallbackDescription ??
        "渲染界面时发生了未预期的错误。请尝试重启应用。如果问题持续出现，请检查 ~/.cc-switch/logs/cc-switch.log 获取详细日志。";

      return (
        <div className="flex items-center justify-center h-screen bg-background text-foreground p-8">
          <div className="flex flex-col items-center max-w-md text-center space-y-6">
            <div className="rounded-full bg-destructive/10 p-4">
              <AlertTriangle className="w-10 h-10 text-destructive" />
            </div>

            <div className="space-y-2">
              <h1 className="text-xl font-semibold">{title}</h1>
              <p className="text-sm text-muted-foreground leading-relaxed">
                {description}
              </p>
            </div>

            {this.state.error && (
              <details className="w-full">
                <summary className="text-xs text-muted-foreground cursor-pointer hover:text-foreground transition-colors">
                  查看错误详情
                </summary>
                <pre className="mt-2 p-3 bg-muted rounded-lg text-xs text-left overflow-auto max-h-48 whitespace-pre-wrap break-all">
                  {this.state.error.message}
                  {"\n\n"}
                  {this.state.error.stack}
                </pre>
              </details>
            )}

            <Button
              onClick={this.handleRetry}
              variant="outline"
              className="gap-2"
            >
              <RefreshCw className="w-4 h-4" />
              重试
            </Button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

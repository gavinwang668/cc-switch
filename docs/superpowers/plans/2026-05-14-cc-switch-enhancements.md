# CC Switch 功能增强实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 系统性增强 CC Switch 的架构可维护性、功能完整性、代码质量和安全性，并实现完整的 API 协议转换矩阵和 Linux CLI 模式

**Architecture:** 分 7 个阶段执行：P0 架构重构 → P1 核心功能增强 → P2 用户体验提升 → P3 代码质量与安全 → P4 多协议支持 → P5 Linux CLI 模式 → P6 补充测试。每个阶段产出可独立验证的增量。

**Tech Stack:** React 18 + TypeScript + Tailwind CSS + shadcn/ui + Tauri 2 (Rust) + Vitest + React Query + Clap (CLI)

---

## 多协议与 CLI 当前状态分析

### 已实现的转换函数（后端）
✅ `transform.rs`: Anthropic ↔ OpenAI Chat Completions  
✅ `transform_responses.rs`: Anthropic ↔ OpenAI Responses API  
✅ `transform_gemini.rs`: Anthropic ↔ Gemini Native  
✅ `transform_bedrock.rs`: Anthropic ↔ Bedrock Converse API  
✅ `transform_codex_chat.rs`: Codex Responses ↔ Chat Completions  

### 前端 API 格式支持现状
- **Claude**: ✅ 支持 apiFormat 选择器（anthropic/openai_chat/openai_responses/gemini_native/bedrock）
- **Codex**: ✅ 支持 apiFormat 选择器（openai_responses/openai_chat）
- **Gemini**: ❌ 无 apiFormat 选择器（固定使用 Gemini Native）
- **Claude Desktop**: ❌ 无 apiFormat 选择器（固定使用 Anthropic）
- **OpenCode/OpenClaw/Hermes**: 各自独立配置体系，不使用 apiFormat

### Linux CLI 模式
❌ 完全缺失，需要从头实现

---

## Phase 0: 架构重构（高优先级）

### Task 1: 拆分 App.tsx — 提取路由系统

**Files:**
- Create: `src/lib/router.tsx`
- Create: `src/hooks/useAppRouter.ts`
- Modify: `src/App.tsx`

- [ ] **Step 1: 创建路由类型和上下文**

创建 `src/lib/router.tsx`，定义 View 类型、路由上下文和 Provider：

```tsx
import { createContext, useContext, useState, useCallback, type ReactNode } from "react";

export type View =
  | "providers"
  | "settings"
  | "prompts"
  | "skills"
  | "skillsDiscovery"
  | "mcp"
  | "agents"
  | "universal"
  | "sessions"
  | "workspace"
  | "openclawEnv"
  | "openclawTools"
  | "openclawAgents"
  | "hermesMemory";

const VALID_VIEWS: View[] = [
  "providers", "settings", "prompts", "skills", "skillsDiscovery",
  "mcp", "agents", "universal", "sessions", "workspace",
  "openclawEnv", "openclawTools", "openclawAgents", "hermesMemory",
];

const VIEW_STORAGE_KEY = "cc-switch-last-view";

const getInitialView = (): View => {
  const saved = localStorage.getItem(VIEW_STORAGE_KEY) as View | null;
  if (saved && VALID_VIEWS.includes(saved)) return saved;
  return "providers";
};

interface RouterState {
  currentView: View;
  navigate: (view: View) => void;
  goBack: () => void;
  settingsDefaultTab: string;
  setSettingsDefaultTab: (tab: string) => void;
}

const RouterContext = createContext<RouterState | null>(null);

export function RouterProvider({ children }: { children: ReactNode }) {
  const [currentView, setCurrentView] = useState<View>(getInitialView);
  const [settingsDefaultTab, setSettingsDefaultTab] = useState("general");

  const navigate = useCallback((view: View) => {
    setCurrentView(view);
    localStorage.setItem(VIEW_STORAGE_KEY, view);
  }, []);

  const goBack = useCallback(() => {
    setCurrentView((prev) =>
      prev === "skillsDiscovery" ? "skills" : "providers"
    );
  }, []);

  return (
    <RouterContext.Provider
      value={{ currentView, navigate, goBack, settingsDefaultTab, setSettingsDefaultTab }}
    >
      {children}
    </RouterContext.Provider>
  );
}

export function useRouter() {
  const ctx = useContext(RouterContext);
  if (!ctx) throw new Error("useRouter must be used within RouterProvider");
  return ctx;
}
```

- [ ] **Step 2: 创建 useAppRouter hook 封装导航逻辑**

创建 `src/hooks/useAppRouter.ts`，封装各视图的导航方法：

```ts
import { useRouter } from "@/lib/router";
import type { View } from "@/lib/router";

export function useAppRouter() {
  const router = useRouter();

  return {
    ...router,
    openSettings: (tab = "general") => {
      router.setSettingsDefaultTab(tab);
      router.navigate("settings");
    },
    openPrompts: () => router.navigate("prompts"),
    openSkills: () => router.navigate("skills"),
    openSkillsDiscovery: () => router.navigate("skillsDiscovery"),
    openMcp: () => router.navigate("mcp"),
    openAgents: () => router.navigate("agents"),
    openUniversal: () => router.navigate("universal"),
    openSessions: () => router.navigate("sessions"),
    openWorkspace: () => router.navigate("workspace"),
    openOpenclawEnv: () => router.navigate("openclawEnv"),
    openOpenclawTools: () => router.navigate("openclawTools"),
    openOpenclawAgents: () => router.navigate("openclawAgents"),
    openHermesMemory: () => router.navigate("hermesMemory"),
    openProviders: () => router.navigate("providers"),
  };
}
```

- [ ] **Step 3: 重构 App.tsx 使用新路由系统**

将 App.tsx 中的 `currentView`/`setCurrentView`/`settingsDefaultTab` 替换为 `useAppRouter()`，删除内联的 View 类型和 localStorage 逻辑。

- [ ] **Step 4: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib/router.tsx src/hooks/useAppRouter.ts src/App.tsx
git commit -m "refactor: extract router system from App.tsx"
```

---

### Task 2: 拆分 App.tsx — 提取全局事件监听

**Files:**
- Create: `src/hooks/useAppEvents.ts`
- Modify: `src/App.tsx`

- [ ] **Step 1: 创建 useAppEvents hook**

将 App.tsx 中 6+ 个 useEffect 事件监听（provider switch、universal-provider-synced、webdav-sync-status-updated、proxy-official-warning、env conflict check、migration check、skills migration check）抽取到 `src/hooks/useAppEvents.ts`。

- [ ] **Step 2: 在 App.tsx 中调用 useAppEvents**

替换所有内联 useEffect 为单行调用。

- [ ] **Step 3: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/hooks/useAppEvents.ts src/App.tsx
git commit -m "refactor: extract global event listeners from App.tsx"
```

---

### Task 3: 拆分 App.tsx — 提取窗口控制逻辑

**Files:**
- Create: `src/hooks/useWindowControls.ts`
- Modify: `src/App.tsx`

- [ ] **Step 1: 创建 useWindowControls hook**

将窗口最小化/最大化/关闭、窗口装饰同步、最大化状态追踪等逻辑抽取到 `src/hooks/useWindowControls.ts`。

- [ ] **Step 2: 在 App.tsx 中调用 useWindowControls**

- [ ] **Step 3: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/hooks/useWindowControls.ts src/App.tsx
git commit -m "refactor: extract window controls from App.tsx"
```

---

## Phase 1: 核心功能增强

### Task 4: Failover — 增加熔断器实时状态面板

**Files:**
- Create: `src/components/proxy/CircuitBreakerStatusPanel.tsx`
- Create: `src/hooks/useCircuitBreakerStatus.ts`
- Modify: `src/components/proxy/ProxyPanel.tsx`

- [ ] **Step 1: 创建 useCircuitBreakerStatus hook**

封装获取各 Provider 熔断器状态的查询逻辑，支持定时刷新。

- [ ] **Step 2: 创建 CircuitBreakerStatusPanel 组件**

展示每个 Provider 的熔断器状态（Closed/Open/HalfOpen）、连续失败次数、上次打开时间，支持手动重置。

- [ ] **Step 3: 集成到 ProxyPanel**

在代理面板中添加熔断器状态 tab。

- [ ] **Step 4: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/proxy/CircuitBreakerStatusPanel.tsx src/hooks/useCircuitBreakerStatus.ts src/components/proxy/ProxyPanel.tsx
git commit -m "feat: add circuit breaker real-time status panel"
```

---

### Task 5: Failover — 增加 Failover 事件日志

**Files:**
- Create: `src/components/proxy/FailoverLogPanel.tsx`
- Create: `src/hooks/useFailoverLog.ts`
- Create: `src/types/failover.ts`
- Modify: `src/components/proxy/ProxyPanel.tsx`

- [ ] **Step 1: 定义 Failover 日志类型**

创建 `src/types/failover.ts`，定义 `FailoverEvent` 接口（时间戳、源 Provider、目标 Provider、触发原因、结果）。

- [ ] **Step 2: 创建 useFailoverLog hook**

查询和订阅 Failover 事件。

- [ ] **Step 3: 创建 FailoverLogPanel 组件**

展示 Failover 事件时间线，支持过滤和搜索。

- [ ] **Step 4: 集成到 ProxyPanel**

- [ ] **Step 5: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/types/failover.ts src/hooks/useFailoverLog.ts src/components/proxy/FailoverLogPanel.tsx src/components/proxy/ProxyPanel.tsx
git commit -m "feat: add failover event log panel"
```

---

### Task 6: Session — 增加会话导出功能

**Files:**
- Create: `src/components/sessions/SessionExportDialog.tsx`
- Create: `src/lib/api/sessionExport.ts`
- Modify: `src/components/sessions/SessionManagerPage.tsx`

- [ ] **Step 1: 创建 sessionExport API 模块**

支持导出为 Markdown 和 JSON 格式。

- [ ] **Step 2: 创建 SessionExportDialog 组件**

选择导出格式、范围（单个/批量），触发下载。

- [ ] **Step 3: 集成到 SessionManagerPage**

在工具栏添加导出按钮。

- [ ] **Step 4: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib/api/sessionExport.ts src/components/sessions/SessionExportDialog.tsx src/components/sessions/SessionManagerPage.tsx
git commit -m "feat: add session export to markdown/json"
```

---

### Task 7: MCP — 增加 MCP 连接测试功能

**Files:**
- Create: `src/components/mcp/McpConnectionTest.tsx`
- Create: `src/hooks/useMcpConnectionTest.ts`
- Modify: `src/components/mcp/McpFormModal.tsx`

- [ ] **Step 1: 创建 useMcpConnectionTest hook**

封装 MCP 服务器连接测试逻辑（启动 → 握手 → 超时检测）。

- [ ] **Step 2: 创建 McpConnectionTest 组件**

展示测试进度和结果（成功/失败/超时），显示错误详情。

- [ ] **Step 3: 集成到 McpFormModal**

在 MCP 编辑弹窗中添加"测试连接"按钮。

- [ ] **Step 4: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/components/mcp/McpConnectionTest.tsx src/hooks/useMcpConnectionTest.ts src/components/mcp/McpFormModal.tsx
git commit -m "feat: add MCP connection test"
```

---

### Task 8: Usage — 增加成本估算和预算告警

**Files:**
- Create: `src/components/usage/CostEstimationPanel.tsx`
- Create: `src/components/usage/BudgetAlertSettings.tsx`
- Create: `src/hooks/useCostEstimation.ts`
- Create: `src/types/budget.ts`
- Modify: `src/components/settings/SettingsPage.tsx`

- [ ] **Step 1: 定义预算类型**

创建 `src/types/budget.ts`，定义 `BudgetConfig`（月度预算、告警阈值）和 `CostEstimate` 接口。

- [ ] **Step 2: 创建 useCostEstimation hook**

基于用量统计和 costMultiplier 计算成本估算。

- [ ] **Step 3: 创建 CostEstimationPanel 组件**

展示成本趋势、按 Provider/Model 分组的成本分布。

- [ ] **Step 4: 创建 BudgetAlertSettings 组件**

设置月度预算和告警阈值。

- [ ] **Step 5: 集成到设置页和用量统计**

- [ ] **Step 6: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add src/types/budget.ts src/hooks/useCostEstimation.ts src/components/usage/CostEstimationPanel.tsx src/components/usage/BudgetAlertSettings.tsx src/components/settings/SettingsPage.tsx
git commit -m "feat: add cost estimation and budget alerts"
```

---

## Phase 2: 用户体验提升

### Task 9: 增加键盘快捷键系统

**Files:**
- Create: `src/hooks/useKeyboardShortcuts.ts`
- Create: `src/components/settings/KeyboardShortcutsSettings.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: 创建 useKeyboardShortcuts hook**

定义快捷键映射表，支持 Cmd/Ctrl+数字切换 App、Cmd/Ctrl+P 打开 Prompts 等。

- [ ] **Step 2: 创建 KeyboardShortcutsSettings 组件**

展示和自定义快捷键列表。

- [ ] **Step 3: 在 App.tsx 中注册全局快捷键**

- [ ] **Step 4: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/hooks/useKeyboardShortcuts.ts src/components/settings/KeyboardShortcutsSettings.tsx src/App.tsx
git commit -m "feat: add keyboard shortcuts system"
```

---

### Task 10: 增加系统通知

**Files:**
- Create: `src/hooks/useSystemNotifications.ts`
- Create: `src/components/settings/NotificationSettings.tsx`
- Create: `src/types/notification.ts`
- Modify: `src/hooks/useAppEvents.ts`

- [ ] **Step 1: 定义通知类型和偏好**

创建 `src/types/notification.ts`，定义 `NotificationEvent` 和 `NotificationPreferences`。

- [ ] **Step 2: 创建 useSystemNotifications hook**

封装 Tauri 通知 API，支持事件过滤和偏好配置。

- [ ] **Step 3: 创建 NotificationSettings 组件**

配置哪些事件触发系统通知。

- [ ] **Step 4: 在 useAppEvents 中集成通知**

Failover 触发、Provider 故障等关键事件发送系统通知。

- [ ] **Step 5: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/types/notification.ts src/hooks/useSystemNotifications.ts src/components/settings/NotificationSettings.tsx src/hooks/useAppEvents.ts
git commit -m "feat: add system notification support"
```

---

### Task 11: i18n 支持懒加载

**Files:**
- Modify: `src/i18n/index.ts`

- [ ] **Step 1: 重构 i18n 为动态导入**

将静态 import 改为 `i18next-http-backend` 或动态 `import()`，按需加载语言包。

- [ ] **Step 2: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/i18n/index.ts
git commit -m "perf: lazy-load i18n locale bundles"
```

---

## Phase 3: 代码质量与安全

### Task 12: 清理废弃 API

**Files:**
- Modify: `src/lib/api/mcp.ts`
- Modify: `src/lib/api/config.ts`

- [ ] **Step 1: 搜索所有 @deprecated 方法的调用点**

确认 `getConfig`、`upsertServerInConfig`、`deleteServerInConfig`、`toggleAppInConfig`、`getCommonConfigSnippet` (旧版)、`setCommonConfigSnippet` (旧版) 的所有调用者已迁移。

- [ ] **Step 2: 迁移残留调用到新 API**

- [ ] **Step 3: 删除废弃方法**

- [ ] **Step 4: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/lib/api/mcp.ts src/lib/api/config.ts
git commit -m "chore: remove deprecated API methods"
```

---

### Task 13: 清理 console.log

**Files:**
- Modify: 50 files in `src/`

- [ ] **Step 1: 安装 eslint-plugin-no-console（如未安装）或配置现有规则**

- [ ] **Step 2: 将 console.error 替换为结构化日志**

保留 console.error 用于真正的错误场景，移除调试用的 console.log/warn。

- [ ] **Step 3: 运行 lint 验证**

Run: `pnpm format:check`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: clean up console.log statements"
```

---

### Task 14: 补充核心模块测试

**Files:**
- Create: `tests/hooks/useAppRouter.test.tsx`
- Create: `tests/hooks/useAppEvents.test.tsx`
- Create: `tests/hooks/useWindowControls.test.tsx`
- Create: `tests/hooks/useCircuitBreakerStatus.test.tsx`
- Create: `tests/components/CircuitBreakerStatusPanel.test.tsx`
- Create: `tests/components/SessionExportDialog.test.tsx`
- Create: `tests/components/McpConnectionTest.test.tsx`

- [ ] **Step 1: 编写 useAppRouter 测试**

验证导航、goBack、localStorage 持久化。

- [ ] **Step 2: 编写 useAppEvents 测试**

验证事件监听注册/清理、回调触发。

- [ ] **Step 3: 编写 useWindowControls 测试**

验证窗口控制方法调用。

- [ ] **Step 4: 编写 CircuitBreakerStatusPanel 测试**

验证状态展示、重置按钮交互。

- [ ] **Step 5: 编写 SessionExportDialog 测试**

验证格式选择、导出触发。

- [ ] **Step 6: 运行所有测试**

Run: `pnpm test:unit`
Expected: ALL PASS

- [ ] **Step 7: Commit**

```bash
git add tests/
git commit -m "test: add tests for router, events, window controls, and new features"
```

---

### Task 15: API Key 安全增强

**Files:**
- Create: `src/lib/api/keychain.ts`
- Modify: `src/lib/api/providers.ts`
- Modify: `src-tauri/src/commands/provider.rs`

- [ ] **Step 1: 创建 keychain API 模块**

封装 Tauri 的系统 Keychain 访问（使用 tauri-plugin-keychain 或自定义命令）。

- [ ] **Step 2: 修改 Provider 保存逻辑**

保存 Provider 时，API Key 优先存入系统 Keychain，数据库只存引用 ID。

- [ ] **Step 3: 修改 Provider 读取逻辑**

读取 Provider 时，从 Keychain 获取 API Key。

- [ ] **Step 4: 添加 API Key 有效性检测**

在 Provider 编辑表单中增加"验证 Key"按钮。

- [ ] **Step 5: 运行 typecheck 验证**

Run: `pnpm typecheck`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/lib/api/keychain.ts src/lib/api/providers.ts src-tauri/src/commands/provider.rs
git commit -m "feat: secure API key storage with system keychain"
```

---

## Phase 4: 多协议支持 — Gemini API 格式选择

### Task 16: 扩展 Gemini 类型定义

**Files:**
- Modify: `src/types.ts`

- [ ] **Step 1: 添加 GeminiApiFormat 类型**

在 `src/types.ts` 中找到 `CodexApiFormat` 定义位置，在其后添加：

```typescript
export type GeminiApiFormat = 
  | "gemini_native"
  | "openai_chat"
  | "openai_responses"
  | "anthropic";
```

- [ ] **Step 2: 在 ProviderMeta 中添加 geminiApiFormat 字段**

```typescript
  geminiApiFormat?: GeminiApiFormat;
```

- [ ] **Step 3: 提交**

```bash
git add src/types.ts
git commit -m "feat(types): add GeminiApiFormat type definition"
```

---

### Task 17: 创建 Gemini 表单 API 格式选择器

**Files:**
- Modify: `src/components/providers/forms/GeminiFormFields.tsx`

- [ ] **Step 1: 在组件 props 中添加 geminiApiFormat 相关回调**

```typescript
  geminiApiFormat?: GeminiApiFormat;
  onGeminiApiFormatChange?: (format: GeminiApiFormat) => void;
```

- [ ] **Step 2: 在表单中添加 API 格式选择器 UI**

选项：Gemini Native（默认）、OpenAI Chat Completions、OpenAI Responses API、Anthropic Messages API

- [ ] **Step 3: 提交**

```bash
git add src/components/providers/forms/GeminiFormFields.tsx
git commit -m "feat(gemini): add API format selector to Gemini form"
```

---

### Task 18: 后端 Gemini API 格式处理

**Files:**
- Modify: `src-tauri/src/proxy/providers/gemini.rs`
- Modify: `src-tauri/src/proxy/handlers.rs`

- [ ] **Step 1: 在 gemini.rs 中添加 get_gemini_api_format 函数**

```rust
pub fn get_gemini_api_format(provider: &Provider) -> String {
    provider.meta.as_ref()
        .and_then(|meta| meta.get("geminiApiFormat"))
        .and_then(|v| v.as_str())
        .unwrap_or("gemini_native")
        .to_string()
}

pub fn gemini_api_format_needs_transform(format: &str) -> bool {
    format != "gemini_native"
}
```

- [ ] **Step 2: 在 handlers.rs 中添加 Gemini 格式转换逻辑**

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/proxy/providers/gemini.rs src-tauri/src/proxy/handlers.rs
git commit -m "feat(gemini): add backend API format handling"
```

---

## Phase 5: 多协议支持 — Claude Desktop API 格式选择

### Task 19: 扩展 Claude Desktop 类型定义

**Files:**
- Modify: `src/types.ts`

- [ ] **Step 1: 添加 ClaudeDesktopApiFormat 类型**

```typescript
export type ClaudeDesktopApiFormat = 
  | "anthropic"
  | "openai_chat"
  | "openai_responses"
  | "gemini_native"
  | "bedrock";
```

- [ ] **Step 2: 在 ProviderMeta 中添加 claudeDesktopApiFormat 字段**

```typescript
  claudeDesktopApiFormat?: ClaudeDesktopApiFormat;
```

- [ ] **Step 3: 提交**

```bash
git add src/types.ts
git commit -m "feat(types): add ClaudeDesktopApiFormat type definition"
```

---

### Task 20: 创建 Claude Desktop 表单 API 格式选择器

**Files:**
- Modify: `src/components/providers/forms/ClaudeDesktopProviderForm.tsx`

- [ ] **Step 1: 添加 API 格式选择器**

选项：Anthropic（默认）、OpenAI Chat Completions、OpenAI Responses API、Gemini Native、Amazon Bedrock

- [ ] **Step 2: 提交**

```bash
git add src/components/providers/forms/ClaudeDesktopProviderForm.tsx
git commit -m "feat(claude-desktop): add API format selector"
```

---

### Task 21: 后端 Claude Desktop API 格式处理

**Files:**
- Modify: `src-tauri/src/proxy/providers/claude.rs`

- [ ] **Step 1: 添加 get_claude_desktop_api_format 函数**

```rust
pub fn get_claude_desktop_api_format(provider: &Provider) -> String {
    provider.meta.as_ref()
        .and_then(|meta| meta.get("claudeDesktopApiFormat"))
        .and_then(|v| v.as_str())
        .unwrap_or("anthropic")
        .to_string()
}
```

- [ ] **Step 2: 在请求处理中添加格式转换**

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/proxy/providers/claude.rs
git commit -m "feat(claude-desktop): add backend API format handling"
```

---

## Phase 6: Linux CLI 模式实现

### Task 22: 创建 CLI 二进制入口

**Files:**
- Create: `src-tauri/src/bin/cc-switch-cli.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 在 Cargo.toml 中添加 CLI 二进制目标**

```toml
[[bin]]
name = "cc-switch-cli"
path = "src/bin/cc-switch-cli.rs"
```

添加依赖：
```toml
clap = { version = "4.5", features = ["derive"] }
```

- [ ] **Step 2: 创建 CLI 入口文件**

定义子命令：`start`、`stop`、`list-providers`、`add-provider`、`remove-provider`、`switch-provider`、`status`、`config`

- [ ] **Step 3: 测试编译**

```bash
cd src-tauri && cargo build --bin cc-switch-cli
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/src/bin/cc-switch-cli.rs
git commit -m "feat(cli): add CLI binary entry point with command structure"
```

---

### Task 23: 提取核心逻辑模块

**Files:**
- Create: `src-tauri/src/core/mod.rs`
- Create: `src-tauri/src/core/database.rs`
- Create: `src-tauri/src/core/provider_manager.rs`

- [ ] **Step 1: 创建 core 模块**

将不依赖 Tauri 的核心逻辑（数据库初始化、供应商管理）提取到 `core` 模块。

- [ ] **Step 2: 提取数据库初始化逻辑**

- [ ] **Step 3: 提取供应商管理逻辑**

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/core/
git commit -m "refactor: extract core logic modules for CLI reuse"
```

---

### Task 24: 实现 CLI 命令逻辑

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`

- [ ] **Step 1: 实现 list_providers 命令**

表格输出：ID、App、Name、Base URL

- [ ] **Step 2: 实现 add_provider 命令**

- [ ] **Step 3: 实现 remove_provider / switch_provider 命令**

- [ ] **Step 4: 实现 start_proxy_server（复用代理服务器逻辑）**

- [ ] **Step 5: 实现 status / config 命令**

- [ ] **Step 6: 提交**

```bash
git add src-tauri/src/bin/cc-switch-cli.rs
git commit -m "feat(cli): implement all CLI command logic"
```

---

## 执行顺序

1. **Task 1-3**: 架构重构（App.tsx 拆分）— 降低后续开发难度
2. **Task 4-5**: Failover 增强 — 核心功能补全
3. **Task 6-7**: Session/MCP 增强 — 用户体验提升
4. **Task 8**: Usage 成本估算 — 高价值功能
5. **Task 9-10**: 快捷键/通知 — 效率提升
6. **Task 11**: i18n 懒加载 — 性能优化
7. **Task 12-13**: 代码清理 — 技术债
8. **Task 14**: 测试补充 — 质量保障
9. **Task 15**: API Key 安全 — 安全增强
10. **Task 16-18**: Gemini 多协议支持 — 协议转换矩阵
11. **Task 19-21**: Claude Desktop 多协议支持 — 协议转换矩阵
12. **Task 22-24**: Linux CLI 模式 — 无头服务器支持

---

## 审计记录

> **2026-06-14 首次审计** — 完成率 95.8%（23/24），Task 7/8/9/10/13/14/15 存在问题
>
> **2026-06-14 晚间重审** — 完成率 95.8%（23/24 完全实现 + 1 部分实现），详见 [`docs/plan-audit-report-2026-06-14.md`](../../plan-audit-report-2026-06-14.md)
>
> **24 个任务的最终状态：**
> - ✅ 已完成：Task 1-14, 16-24（23 个）
> - ⚠️ 部分实现：**Task 15（API Key 安全增强 — Keychain 基础设施已搭建但未集成到 Provider 保存/读取流程）**
> - ⚠️ 偏差：Task 8 集成位置变更（UsageDashboard 而非 SettingsPage）、Task 10 hook 命名单复数差异、Task 24 CLI validated_app 仅支持 claude/codex/gemini

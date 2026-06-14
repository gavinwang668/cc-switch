# API 模式扩展实施计划

## 概述

本计划旨在为多个应用添加缺失的 API 模式支持，确保用户可以灵活选择合适的 API 协议与不同供应商通信。

## 当前问题

1. **Hermes**：缺少 Google (Gemini) API 模式（OpenClaw 已有 "google-generative-ai"）
2. **Gemini**：缺少 Amazon (Bedrock) 和 Anthropic API 模式
3. **Codex**：缺少 Amazon, Anthropic 和 Google (Gemini) API 模式
4. **Claude Desktop**：缺少 Google (Gemini) 和 Amazon API 模式
5. **Claude CLI**：缺少 Amazon API 模式

## 实施步骤

### 第一阶段：类型定义和配置常量扩展

#### 1.1 更新类型定义 (`src/types.ts`)

- 扩展 `ClaudeApiFormat` 类型，添加 `amazon_bedrock` 选项
- 扩展 `CodexApiFormat` 类型，添加更多选项
- 为其他工具添加相应的类型定义

#### 1.2 更新 Hermes 配置常量 (`src/config/hermesProviderPresets.ts`)

- 在 `HermesApiMode` 类型中添加 `"google_generative_ai"`
- 在 `hermesApiModes` 数组中添加对应的选项

#### 1.3 更新 Claude Desktop 常量 (`src/config/claudeDesktopProviderPresets.ts`)

- 添加新的 API 格式选项

### 第二阶段：前端表单组件更新

#### 2.1 HermesFormFields (`src/components/providers/forms/HermesFormFields.tsx`)

- 更新 API 模式下拉菜单，包含新增的 Google 选项

#### 2.2 GeminiFormFields (`src/components/providers/forms/GeminiFormFields.tsx`)

- 添加 API 模式选择功能
- 支持 Amazon Bedrock 和 Anthropic 模式

#### 2.3 CodexFormFields (`src/components/providers/forms/CodexFormFields.tsx`)

- 扩展 API 格式选项
- 支持 Amazon, Anthropic, Google 模式

#### 2.4 ClaudeFormFields (`src/components/providers/forms/ClaudeFormFields.tsx`)

- 添加 Amazon Bedrock API 格式选项

#### 2.5 ClaudeDesktopProviderForm (`src/components/providers/forms/ClaudeDesktopProviderForm.tsx`)

- 添加 Google 和 Amazon API 格式选项

### 第三阶段：后端配置处理更新

#### 3.1 Hermes 配置处理 (`src-tauri/src/hermes_config.rs`)

- 确保新的 API 模式能够正确写入 `config.yaml`
- 保持现有配置读取/写入逻辑不变

#### 3.2 Codex 配置处理 (`src-tauri/src/codex_config.rs`)

- 更新 TOML 配置处理，支持新的 API 格式
- 确保更新时保留现有配置

#### 3.3 Claude Desktop 配置处理 (`src-tauri/src/claude_desktop_config.rs`)

- 更新配置处理逻辑以支持新 API 模式

#### 3.4 Gemini 配置处理 (`src-tauri/src/gemini_config.rs`)

- 添加对多种 API 模式的支持

### 第四阶段：代理层格式转换（如需要）

#### 4.1 检查并更新代理处理代码

- 确保代理层能够正确处理新增的 API 格式
- 可能需要在 `src-tauri/src/proxy/` 中添加格式转换逻辑

## 配置文件安全原则

所有配置更新操作都遵循以下原则：
1. **先读取后更新**：在写入新配置前，先读取现有配置
2. **部分更新**：只修改需要更新的部分，保留其他内容
3. **备份机制**：更新前创建备份，确保可以回滚

## 实施检查清单

- [ ] 类型定义更新完毕
- [ ] 前端表单组件支持新 API 模式
- [ ] 后端配置读写支持新模式
- [ ] 现有功能测试通过
- [ ] 新增模式功能验证通过
- [ ] 配置文件更新测试（保留现有内容）

## 相关文件

主要需要修改的文件列表：
1. `src/types.ts` - 类型定义
2. `src/config/hermesProviderPresets.ts` - Hermes 配置常量
3. `src/config/claudeProviderPresets.ts` - Claude 配置常量
4. `src/config/codexProviderPresets.ts` - Codex 配置常量
5. `src/config/geminiProviderPresets.ts` - Gemini 配置常量
6. `src/components/providers/forms/*.tsx` - 各表单组件
7. `src-tauri/src/*_config.rs` - 后端配置处理

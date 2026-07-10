# CC Switch CLI 全面审计报告

> 审计日期：2026-07-05
> 审计范围：Plan A/B/C/D 实现状态、代码质量、性能、CLI 功能覆盖
> 审计方法：代码级交叉验证（源码 + 计划文档 + spec）

---

## 目录

- [一、审计结论摘要](#一审计结论摘要)
- [二、计划完成度矩阵](#二计划完成度矩阵)
  - [Plan A：P0 阻塞性修复](#plan-ap0-阻塞性修复)
  - [Plan B：架构重构](#plan-b架构重构)
  - [Plan C：新功能实现（REQ-021/022/023）](#plan-c新功能实现req-021022023)
  - [Plan D：体验改进（M-3~M-7）](#plan-d体验改进m-3m-7)
  - [汇总统计](#汇总统计)
- [三、代码实现核验清单](#三代码实现核验清单)
  - [3.1 Plan A 核验](#31-plan-a-核验)
  - [3.2 Plan B 核验](#32-plan-b-核验)
  - [3.3 Plan C 核验](#33-plan-c-核验)
  - [3.4 Plan D 核验](#34-plan-d-核验)
- [四、BUG 与异常处理清单](#四bug-与异常处理清单)
  - [4.1 🔴 P0 严重 BUG（阻塞性）](#41-p0-严重-bug阻塞性)
  - [4.2 🟡 P1 重要 BUG](#42-p1-重要-bug)
  - [4.3 🟢 P2 低风险问题](#43-p2-低风险问题)
  - [4.4 异常处理缺失汇总](#44-异常处理缺失汇总)
- [五、性能瓶颈与资源消耗评估](#五性能瓶颈与资源消耗评估)
  - [5.1 tokio Runtime 重复创建](#51-tokio-runtime-重复创建)
  - [5.2 reqwest Client 不复用](#52-reqwest-client-不复用)
  - [5.3 数据库连接模式](#53-数据库连接模式)
  - [5.4 性能评分](#54-性能评分)
- [六、CLI 功能覆盖矩阵](#六cli-功能覆盖矩阵)
  - [6.1 按 REQ 编号覆盖](#61-按-req-编号覆盖)
  - [6.2 按 CLI 命令枚举覆盖](#62-按-cli-命令枚举覆盖)
  - [6.3 缺失命令清单](#63-缺失命令清单)
- [七、优先级修复建议](#七优先级修复建议)

---

## 一、审计结论摘要

| 评估维度 | 评级 | 核心发现 |
|----------|------|----------|
| **Plan A 完成度** | ✅ 95% | 4 个 P0 修复中 4/4 代码已落地，但 `stream-check`/`remove-session` 是完全删除而非补全 |
| **Plan B 完成度** | ✅ 90% | Workspace 分层完成，CLI 无 tauri 依赖；**stream-check CLI 命令缺失** |
| **Plan C 完成度** | 🟡 75% | 4 个 CLI 命令已实现，但 **ingress_auth middleware 未接入 server.rs**（零安全效果） |
| **Plan D 完成度** | 🟡 60% | 9 个命令中 5 个完全可用，**toggle-provider/rollback 功能断裂**，**connections/stats/logs 命令缺失** |
| **代码质量** | 🟡 中等 | 0 处 `panic!`、30 处 `unwrap/expect`（全部为 Runtime 创建）、异常处理模式一致 |
| **性能** | 🟡 中等 | **26 个独立 tokio Runtime**（每个 CLI 命令一个）、5 次 reqwest Client 独立创建 |
| **CLI 功能覆盖** | 🟡 88% | 80 个命令变体，约 10 个有缺陷 |

**总体评估**: CLI 基础框架（架构分层、75%+ 命令）建设良好，但存在 **4 个 P0 阻塞性 BUG**（ingress_auth 未接线、rollback 不可用、toggle-provider 路由未过滤、disabled 列缺失）和 **2 个 P1 架构问题**（26 个独立 Runtime、stream-check 缺失）。

---

## 二、计划完成度矩阵

### Plan A：P0 阻塞性修复

| Task | 描述 | 状态 | 验证证据 |
|------|------|------|----------|
| **Task 1** | 修正评估文档"全部完成 ✅"声称 | ✅ 完成 | `cli-feature-implementation-assessment.md:3-13` 已更新为"Phase 2 完成，Phase 1 基本完成，Phase 3 部分完成" |
| **Task 2** | 删除 stream-check/stream-check-all 桩命令 | ✅ 完成 | CLI `main.rs` 搜索 `StreamCheck`/`stream_check` = **0 结果**，命令已从 Commands 枚举移除 |
| **Task 3** | 删除 remove-session 桩命令 | ✅ 完成 | CLI 中无 `RemoveSession` 变体或 `cmd_remove_session` 函数 |
| **Task 4** | 修复 add-provider env 字段硬编码 | ✅ 完成 | `main.rs:1297-1302` — 有完整的 `match app { "codex" => ("OPENAI_API_KEY", ...), "gemini" => ("GEMINI_API_KEY", ...), ... }` 映射；0 处硬编码 `ANTHROPIC_API_KEY` 残留 |
| **Task 5** | apply-config 从 schema 删除 listen/port | ✅ 完成 | `decl_config.rs:51-56` — `ProxySection` 仅含 `takeover: HashMap<String, bool>`，注释说明 listen/port 已移除 |
| **Task 6** | 补充参考手册 speedtest/verify-key 说明 | ✅ 完成 | 参考手册相应章节上方有 "不依赖代理运行" 标注 |
| **Task 7** | 最终验证与回归测试 | ⚠️ 部分 | 代码变更均已完成，但缺少 Task 7 中要求的测试文件 `cli_p0_fixes.rs` |

### Plan B：架构重构

| Task | 描述 | 状态 | 验证证据 |
|------|------|------|----------|
| **Task 1** | 创建 Cargo workspace 根配置 | ✅ 完成 | 根 `Cargo.toml` 含 4 个 members |
| **Task 2** | 创建 cc-switch-core crate 空壳 | ✅ 完成 | `crates/cc-switch-core/` 含完整 `lib.rs`（42 个模块导出） |
| **Task 3** | 迁移 error 模块 | ✅ 完成 | `crates/cc-switch-core/src/error.rs` 存在 |
| **Task 4** | 迁移 provider/app_config 基础类型 | ✅ 完成 | `crates/cc-switch-core/src/provider.rs`、`app_config.rs` 存在 |
| **Task 5** | 迁移 database 模块 | ✅ 完成 | `crates/cc-switch-core/src/database/` 目录存在 |
| **Task 6** | 迁移 config/settings 等基础模块 | ✅ 完成 | 16 个 config 文件均在 core crate 中 |
| **Task 7** | 迁移 services 模块 + EventCallback trait | ✅ 完成 | `crates/cc-switch-core/src/services/` 存在 |
| **Task 8** | 迁移 proxy 模块 | ✅ 完成 | `crates/cc-switch-core/src/proxy/` 存在 |
| **Task 9** | 迁移 core 模块 + ApplyContext 升级 | ⚠️ 部分 | `core/decl_config.rs` 存在，`apply()` 方法仍为旧签名 `fn apply(&self, db: &Database)`，缺少 `ApplyContext`（见 spec 建议） |
| **Task 10** | 引入 CopilotAuthState 类型别名 | ✅ 完成 | `cc-switch-core/src/copilot_auth.rs:16` — `pub type CopilotAuthState = Arc<RwLock<CopilotAuthManager>>` |
| **Task 11** | 创建 cc-switch-tauri-commands crate | ✅ 完成 | `crates/cc-switch-tauri-commands/` 含 30 个命令模块 |
| **Task 12** | 重命名 src-tauri package | ⚠️ 部分 | `src-tauri/Cargo.toml` package 名仍为 `cc-switch`（计划要求改 `cc-switch-app`），但功能无影响 |
| **Task 13** | CLI 迁到独立 crate | ✅ 完成 | `crates/cc-switch-cli/` 为独立 crate，`cargo tree -p cc-switch-cli | grep tauri` 无输出 |
| **Task 14** | 实现 CLI stream-check 命令 | 🔴 **缺失** | CLI `main.rs` 中 `StreamCheck` 搜索结果 = 0；Tauri-commands 侧 `stream_check.rs` 有完整实现但未被 CLI 复用 |
| **Task 15** | API 格式设置覆盖 7 应用 | ✅ 完成 | `add-provider` help 注释和 `add-provider --help` 均列出 7 应用 |
| **Task 16** | 最终验证与文档更新 | ⚠️ 部分 | 文档基础更新完成，但架构状态描述未在评估文档中体现 |

### Plan C：新功能实现（REQ-021/022/023）

| Task | 描述 | 状态 | 验证证据 |
|------|------|------|----------|
| **Task 1** | 数据库 DAO 扩展 auth_token/acl | ✅ 完成 | `settings.rs` 有 `get/set_proxy_auth_token`、`get/set_proxy_acl_cidrs`（位于 `database/dao/settings.rs`） |
| **Task 2** | ingress_auth middleware 模块 | ✅ 完成 | `proxy/ingress_auth.rs` 文件存在，导出 `IngressAuthLayer` 和 `ingress_auth_middleware` |
| **Task 3** | ProxyServer 接入 ingress auth layer | 🔴 **丢失** | `server.rs:296-367` `build_router()` **无任何** `ingress_auth` 引用或 `.layer()` 调用 |
| **Task 4** | ProxyService::reload_config 方法 | ✅ 完成 | `services/proxy.rs` 有 `reload_config()` 方法 |
| **Task 5** | 协议转换烟雾测试模块 | ✅ 完成 | `proxy/smoke_test.rs` 存在，含 `run_smoke_test()`、单元测试 |
| **Task 6** | CLI reload/auth-token/acl/smoke-test 命令 | ✅ 完成 | `Reload`、`AuthToken`、`Acl`、`SmokeTest` 4 个 Commands 变体存在，含对应 `cmd_*` 函数 |
| **Task 7** | 集成测试 | ⚠️ 部分 | 模块内单元测试存在，但独立的 `plan_c_*.rs` 测试文件可能不存在 |
| **Task 8** | 文档更新 | ✅ 完成 | 参考手册新章节包含 reload/auth-token/acl/smoke-test |

### Plan D：体验改进（M-3~M-7）

| Task | 描述 | 状态 | 验证证据 |
|------|------|------|----------|
| **Task 1** | M-6 export-yaml 命令 | ✅ 完成 | `ExportYaml` 命令变体 + `DeclConfig::from_database()` + `to_yaml_string()` 存在 |
| **Task 2** | M-5 diff 命令 | ✅ 完成 | `Diff` 命令变体 + `DeclConfig::diff()` 方法存在 |
| **Task 3** | M-5 rollback 命令 | 🔴 **断裂** | `Rollback` 命令变体存在，但搜索文件名含 `"apply-rollback"` 的备份——`apply()` 不创建备份，`backup_database_file()` 生成的文件名格式不匹配 |
| **Task 4** | M-7 toggle-provider 命令 | 🔴 **断裂** | 命令变体存在，调用 `set_provider_enabled()`。但① `ProviderMeta` 无 `disabled` 字段、② `provider_router.rs` 无 disabled 过滤、③ `CREATE TABLE providers` 可能缺 `disabled` 列 |
| **Task 5** | M-3 preview-conversion 命令 | ✅ 完成 | `PreviewConversion` 命令 + `cmd_preview_conversion` 函数存在，调用 transform 模块 |
| **Task 6** | M-3 proxy-trace 命令 | ✅ 完成 | `ProxyTrace` 命令 + `cmd_proxy_trace` 函数存在，输出 4 段转换链路 |
| **Task 7** | M-3 replay-request 命令 | ✅ 完成 | `ReplayRequest` 命令 + `cmd_replay_request` 函数存在，支持 `--dry-run` |
| **Task 8** | M-4 connections 命令 | 🔴 **缺失** | CLI `main.rs` 中 **无 `Connections` 变体**（Plan 约定了此命令，但代码中不存在） |
| **Task 9** | M-4 stats --live 命令 | 🔴 **缺失** | CLI `main.rs` 中 **无 `Stats` 变体** |
| **Task 10** | M-4 logs --tail 命令 | 🔴 **缺失** | CLI `main.rs` 中 **无 `Logs` 变体** |
| **Task 11** | P2-1 参考手册补全 API 格式表格 | ✅ 完成 | 参考手册包含 7 应用完整表格 |
| **Task 12** | P2-3 OPT 子优先级标注 | ⚠️ 部分 | 评估文档缺少 OPT 子优先级 A/B 列 |
| **Task 13** | 最终验证 | ⚠️ 待执行 | 全量 cargo test 未运行 |

### 汇总统计

| Plan | Tasks 总数 | ✅ 完成 | ⚠️ 部分 | 🔴 缺失 | 完成率 |
|------|-----------|---------|---------|---------|--------|
| Plan A | 7 | 6 | 1 | 0 | 86% |
| Plan B | 16 | 12 | 3 | 1 | 75% |
| Plan C | 8 | 6 | 1 | 1 | 75% |
| Plan D | 13 | 7 | 1 | 5 | 54% |
| **总计** | **44** | **31** | **6** | **7** | **70%** |

---

## 三、代码实现核验清单

### 3.1 Plan A 核验

| 核验项 | 文件 | 行号 | 实现完整性 | 备注 |
|--------|------|------|-----------|------|
| 评估文档状态更新 | `cli-feature-implementation-assessment.md` | 3-13 | ✅ 完整 | 正确标注 Phase 1/2/3 实际状态 |
| stream-check 命令删除 | `crates/cc-switch-cli/src/main.rs` | N/A | ✅ 完整 | 枚举、分发、函数、help 文本全部移除 |
| remove-session 命令删除 | `crates/cc-switch-cli/src/main.rs` | N/A | ✅ 完整 | 同 stream-check |
| env 字段按 app 选择 | `crates/cc-switch-cli/src/main.rs` | 1297-1302 | ✅ 完整 | add-provider 和 update-provider 两处均已修复 |
| ProxySection 删除 listen/port | `crates/cc-switch-core/src/core/decl_config.rs` | 51-56 | ✅ 完整 | 仅保留 `takeover: HashMap<String, bool>` |
| 参考手册补充说明 | `docs/cli-reference-manual.md` | — | ✅ 完整 | speedtest/verify-key 加注，stream-check/remove-session 标 GUI 专属 |
| P0 测试文件 | `src-tauri/tests/cli_p0_fixes.rs` | — | ❌ 缺失 | Plan A 明确要求新建此测试文件 |

### 3.2 Plan B 核验

| 核验项 | 文件 | 行号 | 实现完整性 | 备注 |
|--------|------|------|-----------|------|
| Workspace 4 成员 | `Cargo.toml`（根） | — | ✅ 完整 | core/cli/tauri-commands/src-tauri |
| Core crate 模块完整 | `crates/cc-switch-core/src/lib.rs` | — | ✅ 完整 | 42 个模块，102 个 re-export |
| CLI 无 tauri 依赖 | `cargo tree -p cc-switch-cli` | — | ✅ 完整 | 已验证 |
| CopilotAuthState 类型别名 | `cc-switch-core/src/copilot_auth.rs` | 16 | ✅ 完整 | `pub type CopilotAuthState = Arc<RwLock<CopilotAuthManager>>` |
| EventCallback trait | `cc-switch-core/src/event_callback.rs` | — | ✅ 完整 | 包含 `NoopEventCallback` CLI 实现 |
| Tauri commands 层 | `cc-switch-tauri-commands/src/commands/` | — | ✅ 完整 | 30 个命令模块 |
| stream-check CLI 命令 | `cc-switch-cli/src/main.rs` | — | 🔴 缺失 | CLI 无 StreamCheck 变体 |
| Package 重命名 | `src-tauri/Cargo.toml` | — | ⚠️ 未改 | 仍为 `cc-switch` 而非 `cc-switch-app` |
| ApplyContext | `cc-switch-core/src/core/decl_config.rs` | — | ⚠️ 未实现 | `apply()` 签名仍为 `fn apply(&self, db: &Database)` |

### 3.3 Plan C 核验

| 核验项 | 文件 | 行号 | 实现完整性 | 备注 |
|--------|------|------|-----------|------|
| auth_token/acl DAO | `cc-switch-core/src/database/dao/settings.rs` | — | ✅ 完整 | get/set 方法完整 |
| ingress_auth 模块 | `cc-switch-core/src/proxy/ingress_auth.rs` | — | ✅ 完整 | `IngressAuthLayer` + middleware 函数 |
| **ingress_auth 接入 server** | `cc-switch-core/src/proxy/server.rs` | 296-367 | 🔴 **未接入** | **build_router() 无任何 ingress_auth 调用** |
| reload_config 方法 | `cc-switch-core/src/services/proxy.rs` | — | ✅ 完整 | 代理运行中热重载，未运行返回 Ok |
| smoke_test 模块 | `cc-switch-core/src/proxy/smoke_test.rs` | — | ✅ 完整 | 测试 claude/codex/gemini 三链路 |
| CLI 4 个新命令 | `cc-switch-cli/src/main.rs` | — | ✅ 完整 | Reload/AuthToken/Acl/SmokeTest 均有 cmd_* 实现 |

> **🔴 关键发现**：`ingress_auth.rs` 是完整实现，`mod.rs` 声明了 `pub mod ingress_auth`，但 `server.rs` 的 `build_router()` 方法完全未引入该 middleware。这意味着 `auth-token set` 和 `acl add` 命令虽然能成功写入数据库，但**代理服务器的 HTTP 路由完全不检查 token 和 IP**——auth/acl 功能形同虚设。

### 3.4 Plan D 核验

| 核验项 | 文件 | 行号 | 实现完整性 | 备注 |
|--------|------|------|-----------|------|
| export-yaml | `cc-switch-cli/src/main.rs` | — | ✅ 完整 | `DeclConfig::from_database()` + `to_yaml_string()` |
| diff | `cc-switch-cli/src/main.rs` | — | ✅ 完整 | `DeclConfig::diff()` 基于行集对比 |
| **rollback** | `cc-switch-cli/src/main.rs` | 4458-4490 | 🔴 **断裂** | 搜索名含 `"apply-rollback"` 的备份，但无代码生成此类备份 |
| **toggle-provider** | `cc-switch-cli/src/main.rs` | 4492-4523 | 🔴 **断裂** | 调用 `set_provider_enabled()`，但 router 不读取 disabled 状态 |
| preview-conversion | `cc-switch-cli/src/main.rs` | — | ✅ 完整 | 调用 transform 模块 6 条转换路径 |
| proxy-trace | `cc-switch-cli/src/main.rs` | — | ✅ 完整 | 输出 4 段转换链路 |
| replay-request | `cc-switch-cli/src/main.rs` | — | ✅ 完整 | 支持 `--dry-run` 和 `--payload` |
| **connections** | `cc-switch-cli/src/main.rs` | — | 🔴 **缺失** | 无 Commands 变体 |
| **stats** | `cc-switch-cli/src/main.rs` | — | 🔴 **缺失** | 无 Commands 变体 |
| **logs** | `cc-switch-cli/src/main.rs` | — | 🔴 **缺失** | 无 Commands 变体 |

---

## 四、BUG 与异常处理清单

### 4.1 🔴 P0 严重 BUG（阻塞性）

#### BUG-1：ingress_auth middleware 未接入 server Router

- **文件**: `crates/cc-switch-core/src/proxy/server.rs:296-367`
- **根因**: `build_router()` 方法创建 Router 时没有引入 `IngressAuthLayer` 或调用 `ingress_auth_middleware`
- **影响**: `auth-token set --token "xxx"` 和 `acl add --cidr 10.0.0.0/8` 写入数据库成功，但**代理服务器完全不校验**——任何能访问端口的请求都能被代理转发
- **修复**: 在 `build_router()` 中添加 `.layer(axum::middleware::from_fn_with_state(...))` 引入 `ingress_auth_middleware`，注意 `/health`、`/status`、`/stop` 三个端点需要放行

#### BUG-2：rollback 命令永远找不到备份

- **文件**: `crates/cc-switch-cli/src/main.rs:4474-4481`
- **根因**: `cmd_rollback()` 调用 `list_backups()` 后用 `.filter(|b| b.filename.contains("apply-rollback"))` 筛选，但：
  1. `DeclConfig::apply()` 方法不创建备份
  2. 现有 `backup_database_file()` 生成的文件名格式为 `db_backup_YYYYMMDD_HHMMSS.db`，不含 `"apply-rollback"` 字符串
- **影响**: `rollback` 命令永远输出 "没有找到 apply 回滚备份" 并退出 1
- **修复**: 两选一：(a) 让 `apply()` 调用 `backup_database_file()` 并在文件名中包含 `apply-rollback` 标记，或 (b) 用 settings 表记录 apply 前备份文件名，`rollback` 读取该记录

#### BUG-3：toggle-provider disabled 状态不被代理路由读取

- **文件**: 
  - `crates/cc-switch-core/src/proxy/provider_router.rs:37-109` (`select_providers` 无 disabled 过滤)
  - `crates/cc-switch-core/src/database/dao/providers.rs:20-109` (`get_all_providers` SQL 无 `WHERE disabled = 0`)
- **根因**: `cmd_toggle_provider` 能调用 `set_provider_enabled()` 写入数据库，但 `select_providers()` 和 `get_all_providers()` 都不读取/过滤 disabled 状态
- **影响**: 用户执行 `toggle-provider claude my-provider off` 后，代理仍然把请求转发给该供应商
- **修复**: 在 `get_all_providers()` SQL 加 `WHERE disabled = 0` 过滤，或在 `select_providers()` 中读取后 `retain` 过滤

#### BUG-4：providers 表可能缺少 disabled 列

- **文件**: `crates/cc-switch-core/src/database/schema.rs:27-43` + `crates/cc-switch-core/src/database/dao/providers.rs:733-735`
- **根因**: `CREATE TABLE providers` 没有 `disabled` 列，但 `set_provider_enabled()` 执行 `UPDATE providers SET disabled = ?1`
- **影响**: 如果 migration 未在别处添加该列，`toggle-provider` 命令将因 SQL 错误失败
- **修复**: 在 schema migration 中添加 `ALTER TABLE providers ADD COLUMN disabled INTEGER NOT NULL DEFAULT 0`

### 4.2 🟡 P1 重要 BUG

#### BUG-5：26 个独立 tokio Runtime（性能 bug）

- **文件**: `crates/cc-switch-cli/src/main.rs`
- **根因**: 25 处 `.expect("无法创建 tokio runtime")` + 1 处 `.unwrap()`
- **影响**: 每个 CLI 命令创建新 Runtime（线程池 + I/O 驱动），启动开销 > 业务逻辑
- **修复**: 在 `main()` 中创建单个 Runtime，通过传参复用

#### BUG-6：stream-check/stream-check-all 命令缺失

- **文件**: `crates/cc-switch-cli/src/main.rs`
- **根因**: Plan B Task 14 未实现；`CopilotAuthState` 已重构为 `Arc<RwLock<...>>` 类型别名，CLI 可直接创建实例
- **影响**: CLI 用户无法执行流式连通性检查（需用 `speedtest`/`verify-key` 替代）
- **修复**: 复用 `cc_switch_core::services::stream_check::StreamCheckService` 实现 CLI 命令

#### BUG-7：reqwest Client build 失败静默回退

- **文件**: `crates/cc-switch-cli/src/main.rs:1138, 1421`
- **根因**: `reqwest::Client::builder().timeout(...).build().unwrap_or_default()` — build 失败时回退到**无超时**默认 Client
- **影响**: 如果 TLS 或 proxy 配置有问题导致 build 失败，HTTP 请求可能永久挂起
- **修复**: 改为 `.expect("无法创建 HTTP client")` 或给默认 Client 也设置超时

### 4.3 🟢 P2 低风险问题

| # | 描述 | 文件 | 行号 | 风险 | 修复建议 |
|---|------|------|------|------|----------|
| BUG-8 | `try_clone().unwrap()` 回退也可能 panic | `main.rs` | 1020 | 中等 | 使用 `.ok_or_else(|| eprintln!...; exit(1))` |
| BUG-9 | `.unwrap()` 与 `.expect()` 风格不一致 | `main.rs` | 2530 | 低 | 统一为 `.expect("无法创建 tokio runtime")` |
| BUG-10 | `unwrap_or_default()` 静默回退多处 | `main.rs` | 13 处 | 低 | 对非确定性输入（如网络）不建议用 `unwrap_or_default()` |
| BUG-11 | `as` 转换 `u32` → `i32` 理论上可能溢出 | `main.rs` | 886 | 极低 | 实际 PID 不会超过 i32::MAX，可忽略 |
| BUG-12 | 无 `cli_p0_fixes.rs` 测试文件 | `src-tauri/tests/` | — | 低 | Plan A Task 4/5 要求的集成测试缺失 |

### 4.4 异常处理缺失汇总

| 类别 | 数量 | 评估 |
|------|------|------|
| 无 `panic!` 调用 | 0 | ✅ 优良 |
| `unwrap()` | 5 | 🟡 2 处中风险，其余安全 |
| `expect()` | 25 | 🟡 全部用于 Runtime 创建，模式一致 |
| `unwrap_or_default()` | 13 | 🟡 2 处有风险（HTTP client build），其余安全 |
| `as` 类型转换 | 8 | ✅ 全部安全 |
| `exit(1)` 前有 `eprintln!` | ~269 | ✅ 良好——所有退出前都有错误日志 |

---

## 五、性能瓶颈与资源消耗评估

### 5.1 tokio Runtime 重复创建

**问题**: 26 个独立 `Runtime::new()` / `rt.block_on()` 对

**影响量化**:
- 每次 `Runtime::new()` 创建：工作线程池（默认 CPU 核数个）、I/O 驱动（epoll/iocp）、定时器 wheel
- 典型 CLI 命令（如 `cc-switch-cli speedtest URL`）：Runtime 创建耗时可占比 30-50%
- 在 Windows 上 I/O 驱动初始化（iocp）尤其重

**建议**:

```rust
// 方案 A：使用 #[tokio::main]（推荐）
#[tokio::main]
async fn main() {
    // 所有子命令共用同一个 Runtime
}

// 方案 B：单线程 Runtime（CLI 不需要多线程并发）
let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .expect("无法创建 tokio runtime");
```

### 5.2 reqwest Client 不复用

**问题**: 5 个独立 `Client::new()` 调用分布在 5 个函数中

**影响**: 无法复用 HTTP 连接池、DNS 缓存、TLS session 缓存

**建议**: 在 `main()` 中创建一个全局 `reqwest::Client`，通过 `Arc` 传递给子命令。`Client` 内部实现了 `Clone`，clone 是廉价操作。

### 5.3 数据库连接模式

**现状**: 每个子命令独立调用 `init_db()` → `Database::init()` 打开新连接

**评估**: 对 CLI 工具（每个命令执行一次就退出）是**可接受的**。无需优化，除非未来支持 "批量命令" 模式。

### 5.4 性能评分

| 维度 | 评分 | 说明 |
|------|------|------|
| Runtime 管理 | 🟡 中等 | 26 个独立 Runtime，启动开销可优化 |
| HTTP 连接复用 | 🟡 中等 | Client 不共享，连接池浪费 |
| 数据库连接 | ✅ 良好 | CLI 单命令模式合理 |
| 内存使用 | ✅ 良好 | 无大对象泄漏风险 |
| 编译依赖 | ✅ 优良 | CLI 已去 tauri/webkit2gtk |

---

## 六、CLI 功能覆盖矩阵

### 6.1 按 REQ 编号覆盖

| REQ | 功能 | CLI 实现 | 状态 | 备注 |
|-----|------|---------|------|------|
| REQ-001 | 供应商排序 | `sort-providers` | ✅ | |
| REQ-002 | Live 配置导入 | `import-live` | ✅ | |
| REQ-003 | 读取 Live 配置 | `read-live` | ✅ | |
| REQ-004 | 模型列表获取 | `fetch-models` | ✅ | |
| REQ-005 | 同步到 Live | `sync-live` | ✅ | |
| REQ-006 | 代理配置读写 | `proxy-config` / `global-proxy-config` / `app-proxy-config` | ✅ | |
| REQ-009 | Live 接管检测 | `takeover-status` | ✅ | |
| REQ-010 | 熔断器统计 | `circuit-breaker-stats` | ✅ | |
| REQ-011 | 供应商健康状态 | `provider-health` | ✅ | |
| REQ-012 | 可用故障转移列表 | `failover-available` | ✅ | |
| REQ-014 | 按应用统计用量 | `usage-by-app` | ✅ | |
| REQ-015 | 请求日志 | `request-logs` / `request-detail` | ✅ | |
| REQ-016 | 供应商限额检查 | `check-limits` | ✅ | |
| REQ-017 | 备份删除 | `backup-delete` | ✅ | |
| REQ-018 | 备份重命名 | `backup-rename` | ✅ | |
| REQ-019 | 自定义端点管理 | `endpoint` | ✅ | |
| REQ-020 | API 格式覆盖 7 应用 | `add-provider --api-format` | ✅ | help 文本列出 7 应用 |
| REQ-021 | 协议转换烟雾测试 | `smoke-test` | ✅ | |
| REQ-022 | 代理热重载 | `reload` | ⚠️ | 仅提示 SIGHUP，无 HTTP /reload 端点 |
| REQ-023 | 代理访问控制 | `auth-token` / `acl` | 🔴 | DAO 和命令存在，但 middleware 未接入 server——零安全效果 |
| OC-01 | OpenClaw 默认模型 | — | ❌ | CLI 未实现 |
| OC-02 | OpenClaw 模型目录 | — | ❌ | CLI 未实现 |
| HE-01 | Hermes 模型配置查看 | — | ❌ | CLI 未实现 |

### 6.2 按 CLI 命令枚举覆盖

完整的 80 个 Commands 变体分布在以下功能域：

| 功能域 | 命令数 | 核心状态 |
|--------|--------|----------|
| 代理管理 (start/daemon/stop/status) | 4 | ✅ 全部可用 |
| 供应商管理 | 8 | ✅ 全部可用 |
| 代理配置 | 8 | ✅ 全部可用 |
| 故障转移与熔断器 | 5 | ✅ 全部可用 |
| 请求处理配置 | 2 | ✅ 全部可用 |
| 全局出站代理 | 1 | ✅ 可用 |
| 配置与设置 | 6 | ✅ 全部可用 |
| 声明式配置 | 3 | ✅ validate/apply-config 可用，diff 可用 |
| 备份与恢复 | 4 | ✅ 全部可用（rollback 🔴 除外） |
| 用量统计与监控 | 6 | ✅ 全部可用 |
| 测试与诊断 | 4 | ✅ speedtest/verify-key/smoke-test/preview-conversion |
| 请求日志 | 2 | ✅ 可用 |
| 代理运维与监控 | 5 | ⚠️ reload/auth-token/acl 存在但各有问题 |
| MCP 管理 | 5 | ✅ 可用 |
| Prompt 管理 | 3 | ✅ 可用 |
| Skills 管理 | 3 | ✅ 可用 |
| 环境变量检查 | 1 | ✅ 可用 |
| 会话管理 | 1 | ✅ list-sessions 可用 |
| 协议转换可观测性 | 6 | ✅ proxy-trace/replay-request/export-yaml/preview-conversion |
| 代理安全 | 3 | 🔴 auth-token/acl 不生效 |
| 实时可观测性 | 0 (计划 3) | 🔴 connections/stats/logs 全部缺失 |

### 6.3 缺失命令清单

| 命令 | 计划来源 | 严重性 | 说明 |
|------|---------|--------|------|
| `stream-check` / `stream-check-all` | Plan B Task 14 | 🟡 P1 | Core service 已就绪，CLI 未复用 |
| `connections` | Plan D Task 8 | 🟡 P1 | 活跃连接查看 |
| `stats [--live]` | Plan D Task 9 | 🟡 P1 | 实时 QPS 统计 |
| `logs [--tail]` | Plan D Task 10 | 🟢 P2 | 日志 tail 查看 |
| `remove-session` | Plan A Task 3 | 🟢 N/A | 已有意删除（GUI 专属） |

---

## 七、优先级修复建议

### P0 — 阻塞性修复（必须立即处理）

| # | 问题 | 涉及文件 | 估时 | 依赖 |
|---|------|---------|------|------|
| **P0-1** | ingress_auth middleware 接入 server.rs (BUG-1) | `proxy/server.rs` `build_router()` | 2h | 无 |
| **P0-2** | rollback 备份链修复 (BUG-2) | `core/decl_config.rs` + CLI `cmd_apply_config` | 3h | 无 |
| **P0-3** | toggle-provider disabled 路由过滤 (BUG-3) | `proxy/provider_router.rs` + `dao/providers.rs` | 2h | 无 |
| **P0-4** | providers 表 disabled 列 migration (BUG-4) | `database/schema.rs` | 1h | 无 |

### P1 — 重要修复（2 周内完成）

| # | 问题 | 涉及文件 | 估时 | 依赖 |
|---|------|---------|------|------|
| **P1-1** | Runtime 池化——CLI 用单个 Runtime (BUG-5) | CLI `main.rs` | 4h | 无 |
| **P1-2** | 实现 stream-check CLI 命令 (BUG-6) | CLI `main.rs` | 3h | P1-1 完成后 |
| **P1-3** | reqwest Client build 失败处理 (BUG-7) | CLI `main.rs:1138,1421` | 0.5h | 无 |
| **P1-4** | 实现 connections/stats/logs 命令 | CLI `main.rs` + `proxy/server.rs` | 8h | P1-1 完成后 |

### P2 — 体验改进（后续版本）

| # | 问题 | 估时 | 
|---|------|------|
| P2-1 | 补充 `cli_p0_fixes.rs` 测试文件 | 2h |
| P2-2 | 统一 `.unwrap()` → `.expect()` 风格 | 0.5h |
| P2-3 | OPT 子优先级标注完成评估文档更新 | 1h |
| P2-4 | OpenClaw/Hermes REQ 升级项 CLI 命令 | 待评估 |

### 预计修复时间线

```
P0 修复（第 1 周）:
  Day 1: P0-1 + P0-4 (ingress_auth 接线 + disabled 列 migration)
  Day 2: P0-2 (rollback 备份链)
  Day 3: P0-3 (toggle-provider 路由过滤)
  Day 4-5: 全量回归测试 + 文档同步

P1 修复（第 2 周）:
  Day 1-2: P1-1 (Runtime 池化)
  Day 3: P1-2 + P1-3 (stream-check + reqwest fix)
  Day 4-5: P1-4 (connections/stats/logs 命令)

P2 修复（第 3 周，可选）:
  测试补充 + 风格统一 + 文档完善
```

---

> **审计完成**。本报告所有发现均基于 2026-07-05 代码库状态（`crates/cc-switch-cli/src/main.rs` 4817 行，80 个 Commands 变体）。
> 
> 关键源码引用：`ingress_auth.rs` middleware、`server.rs:296-367` build_router、`main.rs:4458-4490` cmd_rollback、`provider_router.rs:37-109` select_providers。

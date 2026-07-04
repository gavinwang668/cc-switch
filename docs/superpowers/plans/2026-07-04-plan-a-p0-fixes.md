# Plan A: P0 阻塞性修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复评估文档与代码不一致的阻塞性问题，包括修正"全部完成 ✅"声称、处理 4 个桩命令、修复 add-provider env 硬编码、apply-config 代理字段止血。

**Architecture:** 纯文档与代码点状修复，不涉及架构重构。所有修改保持现有 lib crate 结构不变，仅修正错误内容与添加缺失逻辑。

**Tech Stack:** Rust（cc-switch-cli 二进制）、Markdown（评估文档与参考手册）、clap（命令行解析）

**关联 Spec:** [docs/superpowers/specs/2026-07-04-cli-feature-review-design.md](file:///f:/workspace/trae/cc-switch/docs/superpowers/specs/2026-07-04-cli-feature-review-design.md) §七.1（P0-1~P0-4）

---

## File Structure

| 文件 | 操作 | 责任 |
|---|---|---|
| `docs/cli-feature-implementation-assessment.md` | 修改 | 修正"全部完成 ✅"声称 |
| `docs/cli-reference-manual.md` | 修改 | 标注桩命令为"GUI 专属"、补 speedtest/verify-key 说明 |
| `src-tauri/src/bin/cc-switch-cli.rs` | 修改 | 删除桩命令、修复 env 硬编码、补 list-providers Base URL 读取 |
| `src-tauri/src/core/decl_config.rs` | 修改 | 删除 ProxySection.listen/port 字段止血 |
| `src-tauri/tests/cli_p0_fixes.rs` | 新建 | P0 修复的集成测试 |

---

## Task 1: 修正评估文档"全部完成 ✅"声称（P0-1）

**Files:**
- Modify: `docs/cli-feature-implementation-assessment.md:3-9`

- [ ] **Step 1: 阅读当前声称**

Read [docs/cli-feature-implementation-assessment.md:1-15](file:///f:/workspace/trae/cc-switch/docs/cli-feature-implementation-assessment.md)

确认第 3 行与第 9 行的声称内容：
- 第 3 行：`状态：三阶段全部完成`
- 第 9 行：`实现状态：Phase 1（REQ-001~009）✅、Phase 2（REQ-010~019）✅、Phase 3（OPT 系列关键功能）✅ 已全部实现`

- [ ] **Step 2: 修改第 3 行状态**

将第 3 行：

```markdown
> 文档日期：2026-06-27 ｜ 最后更新：2026-06-27 ｜ 状态：三阶段全部完成
```

改为：

```markdown
> 文档日期：2026-06-27 ｜ 最后更新：2026-07-04 ｜ 状态：Phase 2 完成，Phase 1 基本完成，Phase 3 部分完成
```

- [ ] **Step 3: 修改第 9 行实现状态**

将第 9 行：

```markdown
> **实现状态**：Phase 1（REQ-001~009）✅、Phase 2（REQ-010~019）✅、Phase 3（OPT 系列关键功能）✅ 已全部实现。完整命令参考见 `docs/cli-reference-manual.md`。
```

改为：

```markdown
> **实现状态**（2026-07-04 修订）：
> - Phase 2（REQ-010~019）✅ 完成
> - Phase 1（REQ-001~009）🟡 基本完成 — `apply-config` 代理字段（listen/port/takeover）只校验不应用，详见参考手册"已知限制"
> - Phase 3（OPT 关键功能）🔴 部分完成 — `stream-check` / `stream-check-all` / `remove-session` 为桩实现，已标注为"GUI 专属"，详见参考手册对应章节
> - `add-provider` env 字段名按 `--app` 自动选择（claude→ANTHROPIC_*、codex→OPENAI_*、gemini→GEMINI_*）
>
> 完整命令参考见 `docs/cli-reference-manual.md`。本评估文档的分类与优先级修订详见 `docs/superpowers/specs/2026-07-04-cli-feature-review-design.md`。
```

- [ ] **Step 4: 验证修改**

Read 修改后的文件前 15 行，确认第 3 行与第 9 行已更新。

- [ ] **Step 5: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add docs/cli-feature-implementation-assessment.md
git commit -m "docs(cli): 修正评估文档'全部完成 ✅'声称，标注 Phase 1/3 实际状态

- Phase 2 完成
- Phase 1 基本完成（apply-config 代理字段待补）
- Phase 3 部分完成（4 个桩命令已标注 GUI 专属）
- add-provider env 字段按 app 选择

关联 spec: docs/superpowers/specs/2026-07-04-cli-feature-review-design.md §七.1 P0-1"
```

---

## Task 2: 从 CLI 删除 stream-check / stream-check-all 桩命令（P0-3 第 1 部分）

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:30-545`（Commands 枚举）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:700-712`（match 分发）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:3989-4004`（cmd_stream_check / cmd_stream_check_all 函数）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:4104-4105`（help 文本）
- Modify: `docs/cli-reference-manual.md`（标注 GUI 专属）

- [ ] **Step 1: 删除 Commands 枚举中的 StreamCheck 与 StreamCheckAll 变体**

Read [src-tauri/src/bin/cc-switch-cli.rs:530-545](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

删除第 534~542 行：

```rust
    /// 流式检查供应商
    StreamCheck {
        /// 应用类型
        app: String,
        /// 供应商 ID
        id: String,
    },
    /// 流式检查全部供应商
    StreamCheckAll,
```

- [ ] **Step 2: 删除 match 分发的 StreamCheck 与 StreamCheckAll 分支**

Read [src-tauri/src/bin/cc-switch-cli.rs:700-715](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

删除第 711~712 行：

```rust
        Commands::StreamCheck { app, id } => cmd_stream_check(app.clone(), id.clone()),
        Commands::StreamCheckAll => cmd_stream_check_all(),
```

- [ ] **Step 3: 删除 cmd_stream_check 与 cmd_stream_check_all 函数**

Read [src-tauri/src/bin/cc-switch-cli.rs:3985-4010](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

删除第 3989~4004 行（含函数前的注释行）：

```rust
/// stream-check: 流式检查供应商
fn cmd_stream_check(app: String, id: String) {
    if let Err(e) = validated_app(&app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
    eprintln!("流式检查需要代理服务器运行中且 CopilotAuthState 初始化，当前 CLI 环境不支持。");
    eprintln!("请使用 speedtest 或 verify-key 命令进行基本连通性测试。");
    eprintln!("应用: {app}, 供应商: {id}");
}

/// stream-check-all: 流式检查全部供应商
fn cmd_stream_check_all() {
    eprintln!("流式检查需要代理服务器运行中且 CopilotAuthState 初始化，当前 CLI 环境不支持。");
    eprintln!("请使用 speedtest 或 verify-key 命令进行基本连通性测试。");
}
```

- [ ] **Step 4: 删除 help 文本中的 stream-check 行**

Read [src-tauri/src/bin/cc-switch-cli.rs:4100-4110](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

删除第 4104~4105 行：

```rust
    println!("    stream-check <APP> <ID>            流式检查（需代理运行）");
    println!("    stream-check-all                   流式检查全部（需代理运行）");
```

- [ ] **Step 5: 验证编译**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo check --bin cc-switch-cli
```

Expected: 编译通过，无 `StreamCheck` / `StreamCheckAll` 相关警告

- [ ] **Step 6: 标注参考手册 stream-check 章节为 GUI 专属**

Read [docs/cli-reference-manual.md:1900-1925](file:///f:/workspace/trae/cc-switch/docs/cli-reference-manual.md)

在第 1913 行（stream-check 章节标题下方）插入：

```markdown
> ⚠️ **GUI 专属命令**：此命令依赖代理运行时的 `CopilotAuthState`，CLI 已移除此命令。CLI 用户请使用 `speedtest` 或 `verify-key` 进行基本连通性测试。
```

同样在 stream-check-all 章节插入相同提示。

- [ ] **Step 7: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/bin/cc-switch-cli.rs docs/cli-reference-manual.md
git commit -m "fix(cli): 移除 stream-check/stream-check-all 桩命令

依赖 CopilotAuthState 的命令在 CLI 架构上不可行，已从命令枚举删除。
参考手册对应章节标注为 GUI 专属。

关联 spec: §七.1 P0-3"
```

---

## Task 3: 从 CLI 删除 remove-session 桩命令（P0-3 第 2 部分）

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:511-515`（Commands 枚举）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:707`（match 分发）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:3867-3872`（cmd_remove_session 函数）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:4100`（help 文本）
- Modify: `docs/cli-reference-manual.md`（标注 GUI 专属）

- [ ] **Step 1: 删除 Commands 枚举中的 RemoveSession 变体**

Read [src-tauri/src/bin/cc-switch-cli.rs:505-520](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

删除第 511~515 行：

```rust
    /// 删除会话
    RemoveSession {
        /// 会话 ID
        id: String,
    },
```

- [ ] **Step 2: 删除 match 分发的 RemoveSession 分支**

Read [src-tauri/src/bin/cc-switch-cli.rs:705-715](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

删除第 707 行：

```rust
        Commands::RemoveSession { id } => cmd_remove_session(id.clone()),
```

- [ ] **Step 3: 删除 cmd_remove_session 函数**

Read [src-tauri/src/bin/cc-switch-cli.rs:3865-3875](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

删除第 3867~3872 行：

```rust
/// remove-session: 删除会话
fn cmd_remove_session(id: String) {
    eprintln!("删除会话需要提供 provider_id 和 source_path，当前 CLI 命令暂不支持完整参数。");
    eprintln!("请使用 GUI 删除会话，或手动删除会话文件。");
    eprintln!("会话 ID: {id}");
}
```

- [ ] **Step 4: 删除 help 文本中的 remove-session 行**

Read [src-tauri/src/bin/cc-switch-cli.rs:4098-4102](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

删除第 4100 行：

```rust
    println!("    remove-session <ID>                删除会话（需 GUI）");
```

- [ ] **Step 5: 验证编译**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo check --bin cc-switch-cli
```

Expected: 编译通过，无 `RemoveSession` 相关警告

- [ ] **Step 6: 标注参考手册 remove-session 章节为 GUI 专属**

Grep `docs/cli-reference-manual.md` 找到 remove-session 章节，在标题下方插入：

```markdown
> ⚠️ **GUI 专属命令**：此命令需要 `provider_id` 与 `source_path` 完整参数，CLI 已移除此命令。CLI 用户请手动删除会话文件，或使用 GUI。
```

- [ ] **Step 7: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/bin/cc-switch-cli.rs docs/cli-reference-manual.md
git commit -m "fix(cli): 移除 remove-session 桩命令

需要 provider_id + source_path 完整参数的命令在 CLI 不实用，已从命令枚举删除。
参考手册对应章节标注为 GUI 专属。

关联 spec: §七.1 P0-3"
```

---

## Task 4: 修复 add-provider env 字段硬编码（P0-4）

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:1178-1190`（env 字段插入逻辑）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs:78-84`（api_format 注释，顺便补全）
- Test: `src-tauri/tests/cli_p0_fixes.rs`

- [ ] **Step 1: 写失败测试 — env 字段按 app 选择**

Create `src-tauri/tests/cli_p0_fixes.rs`:

```rust
// 测试 add-provider 的 env 字段名按 app 类型选择
// 由于 cmd_add_provider 是私有函数，这里通过子进程调用 CLI 二进制测试

use std::process::Command;

#[cfg(test)]
mod env_field_tests {
    use super::*;

    fn cli_binary() -> String {
        env!("CARGO_BIN_EXE_cc-switch-cli").to_string()
    }

    #[test]
    fn test_add_provider_codex_uses_openai_env() {
        // 临时数据库路径
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let db_path = tmp.path().to_str().unwrap();

        // 用 CC_SWITCH_HOME 环境变量重定向数据库
        let tmp_dir = tempfile::tempdir().unwrap();
        let home = tmp_dir.path().to_str().unwrap();

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "codex",
                "test-openai",
                "Test OpenAI",
                "--api-key",
                "sk-test",
                "--base-url",
                "https://api.openai.com/v1",
            ])
            .output()
            .expect("Failed to execute CLI");

        assert!(
            output.status.success(),
            "add-provider failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        // 验证写入的 env 字段名
        let db = cc_switch_lib::Database::init_at(home).unwrap();
        let provider = db.get_provider("codex", "test-openai").unwrap().unwrap();
        let env = provider.settings_config.get("env").unwrap();
        assert!(env.get("OPENAI_API_KEY").is_some(), "应为 OPENAI_API_KEY");
        assert!(env.get("OPENAI_BASE_URL").is_some(), "应为 OPENAI_BASE_URL");
        assert!(
            env.get("ANTHROPIC_API_KEY").is_none(),
            "不应为 ANTHROPIC_API_KEY"
        );
    }

    #[test]
    fn test_add_provider_gemini_uses_gemini_env() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let home = tmp_dir.path().to_str().unwrap();

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "gemini",
                "test-gemini",
                "Test Gemini",
                "--api-key",
                "test-key",
                "--base-url",
                "https://generativelanguage.googleapis.com/v1",
            ])
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success());

        let db = cc_switch_lib::Database::init_at(home).unwrap();
        let provider = db.get_provider("gemini", "test-gemini").unwrap().unwrap();
        let env = provider.settings_config.get("env").unwrap();
        assert!(env.get("GEMINI_API_KEY").is_some(), "应为 GEMINI_API_KEY");
        assert!(env.get("GEMINI_BASE_URL").is_some(), "应为 GEMINI_BASE_URL");
    }

    #[test]
    fn test_add_provider_claude_uses_anthropic_env() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let home = tmp_dir.path().to_str().unwrap();

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "claude",
                "test-claude",
                "Test Claude",
                "--api-key",
                "sk-ant-test",
                "--base-url",
                "https://api.anthropic.com",
            ])
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success());

        let db = cc_switch_lib::Database::init_at(home).unwrap();
        let provider = db.get_provider("claude", "test-claude").unwrap().unwrap();
        let env = provider.settings_config.get("env").unwrap();
        assert!(
            env.get("ANTHROPIC_API_KEY").is_some(),
            "应为 ANTHROPIC_API_KEY"
        );
        assert!(
            env.get("ANTHROPIC_BASE_URL").is_some(),
            "应为 ANTHROPIC_BASE_URL"
        );
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_p0_fixes -- --nocapture
```

Expected: 测试失败，因为当前 env 字段硬编码为 `ANTHROPIC_API_KEY` / `ANTHROPIC_BASE_URL`，对 codex/gemini 应用写入错误字段名

- [ ] **Step 3: 修改 add-provider 的 env 字段插入逻辑**

Read [src-tauri/src/bin/cc-switch-cli.rs:1175-1200](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

将第 1178~1190 行：

```rust
    let mut env = serde_json::Map::new();
    if let Some(key) = api_key {
        env.insert(
            "ANTHROPIC_API_KEY".to_string(),
            serde_json::Value::String(key.to_string()),
        );
    }
    if let Some(url) = base_url {
        env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            serde_json::Value::String(url.to_string()),
        );
    }
```

替换为：

```rust
    // 按 app 类型选择正确的 env 字段名
    let (key_field, url_field) = match app.as_str() {
        "claude" | "claude-desktop" => ("ANTHROPIC_API_KEY", "ANTHROPIC_BASE_URL"),
        "codex" | "opencode" => ("OPENAI_API_KEY", "OPENAI_BASE_URL"),
        "gemini" => ("GEMINI_API_KEY", "GEMINI_BASE_URL"),
        "openclaw" => ("OPENCLAW_API_KEY", "OPENCLAW_BASE_URL"),
        "hermes" => ("HERMES_API_KEY", "HERMES_BASE_URL"),
        _ => {
            eprintln!("错误: 不支持的应用类型: {app}");
            std::process::exit(1);
        }
    };

    let mut env = serde_json::Map::new();
    if let Some(key) = api_key {
        env.insert(
            key_field.to_string(),
            serde_json::Value::String(key.to_string()),
        );
    }
    if let Some(url) = base_url {
        env.insert(
            url_field.to_string(),
            serde_json::Value::String(url.to_string()),
        );
    }
```

> 注：`app` 变量在此函数作用域内可用。若 `app` 是 `&str` 而非 `String`，将 `app.as_str()` 改为 `app`。实施时先 Read 上下文确认变量类型。

- [ ] **Step 4: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_p0_fixes -- --nocapture
```

Expected: 三个测试全部通过

- [ ] **Step 5: 同步修复 update-provider 命令的 env 字段硬编码**

Grep `src-tauri/src/bin/cc-switch-cli.rs` 查找 `update-provider` 函数中的 `ANTHROPIC_API_KEY` 引用，按相同模式修复。

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
rg -n "ANTHROPIC_API_KEY|ANTHROPIC_BASE_URL" src/bin/cc-switch-cli.rs
```

对 `cmd_update_provider` 中的相同硬编码按 Step 3 的模式修复。

- [ ] **Step 6: 同步修复 list-providers 的 Base URL 读取**

Read [src-tauri/src/bin/cc-switch-cli.rs:1135-1145](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

将第 1138~1139 行（含周边上下文）：

```rust
.or_else(|| provider.settings_config.pointer("/env/ANTHROPIC_BASE_URL"))
.or_else(|| provider.settings_config.pointer("/env/BASE_URL"))
```

替换为按 app 类型尝试多个字段名：

```rust
let base_url = provider
    .settings_config
    .pointer("/env/ANTHROPIC_BASE_URL")
    .or_else(|| provider.settings_config.pointer("/env/OPENAI_BASE_URL"))
    .or_else(|| provider.settings_config.pointer("/env/GEMINI_BASE_URL"))
    .or_else(|| provider.settings_config.pointer("/env/OPENCLAW_BASE_URL"))
    .or_else(|| provider.settings_config.pointer("/env/HERMES_BASE_URL"))
    .or_else(|| provider.settings_config.pointer("/env/BASE_URL"))
    .and_then(|v| v.as_str())
    .unwrap_or("-");
```

> 注：实施时 Read 上下文确认变量名与原代码风格一致。

- [ ] **Step 7: 补充测试覆盖 update-provider 与 list-providers**

在 `src-tauri/tests/cli_p0_fixes.rs` 追加：

```rust
    #[test]
    fn test_update_provider_codex_preserves_openai_env() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let home = tmp_dir.path().to_str().unwrap();

        // 先添加
        let _ = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "codex",
                "test-update",
                "Test",
                "--api-key",
                "old-key",
            ])
            .output()
            .unwrap();

        // 更新
        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "update-provider",
                "codex",
                "test-update",
                "--api-key",
                "new-key",
            ])
            .output()
            .expect("Failed to execute CLI");

        assert!(output.status.success());

        let db = cc_switch_lib::Database::init_at(home).unwrap();
        let provider = db.get_provider("codex", "test-update").unwrap().unwrap();
        let env = provider.settings_config.get("env").unwrap();
        assert_eq!(
            env.get("OPENAI_API_KEY").and_then(|v| v.as_str()),
            Some("new-key")
        );
    }

    #[test]
    fn test_list_providers_shows_base_url_for_codex() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let home = tmp_dir.path().to_str().unwrap();

        let _ = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "codex",
                "test-list",
                "Test",
                "--api-key",
                "k",
                "--base-url",
                "https://api.openai.com/v1",
            ])
            .output()
            .unwrap();

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["list-providers", "codex"])
            .output()
            .expect("Failed to execute CLI");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("https://api.openai.com/v1"),
            "list-providers 应显示 OpenAI base URL，实际输出: {stdout}"
        );
    }
```

- [ ] **Step 8: 运行全部测试**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_p0_fixes -- --nocapture
```

Expected: 5 个测试全部通过

- [ ] **Step 9: 验证编译无警告**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo clippy --bin cc-switch-cli
```

Expected: 无新增警告

- [ ] **Step 10: 更新 AddProvider.api_format 字段注释**

Read [src-tauri/src/bin/cc-switch-cli.rs:75-90](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

将 api_format 字段注释更新为：

```rust
    /// API 格式（按应用支持情况选择）
    /// - claude: anthropic / openai_chat / openai_responses
    /// - claude-desktop: anthropic / openai_chat / openai_responses / gemini_native / bedrock
    /// - codex: openai_responses / openai_chat
    /// - gemini: gemini_native / openai_chat / openai_responses / anthropic
    /// - opencode: openai_chat / openai_responses（待确认）
    /// - openclaw: 待确认（详见 REQ-020 实现）
    /// - hermes: 待确认（详见 REQ-020 实现）
    #[arg(long)]
    api_format: Option<String>,
```

- [ ] **Step 11: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/bin/cc-switch-cli.rs src-tauri/tests/cli_p0_fixes.rs
git commit -m "fix(cli): add-provider env 字段名按 app 类型选择

- claude/claude-desktop → ANTHROPIC_API_KEY/ANTHROPIC_BASE_URL
- codex/opencode → OPENAI_API_KEY/OPENAI_BASE_URL
- gemini → GEMINI_API_KEY/GEMINI_BASE_URL
- openclaw → OPENCLAW_API_KEY/OPENCLAW_BASE_URL
- hermes → HERMES_API_KEY/HERMES_BASE_URL

同步修复 update-provider 与 list-providers 的 Base URL 读取。
新增 5 个集成测试覆盖 env 字段正确性。

关联 spec: §七.1 P0-4"
```

---

## Task 5: apply-config 代理字段止血 — 从 schema 删除 listen/port（P0-2 方案 B）

**Files:**
- Modify: `src-tauri/src/core/decl_config.rs:30-50`（ProxySection 结构体）
- Modify: `docs/cli-reference-manual.md:1338-1385`（YAML 示例与说明）
- Test: `src-tauri/tests/cli_p0_fixes.rs`（追加测试）

- [ ] **Step 1: 写失败测试 — YAML 不再接受 listen/port**

在 `src-tauri/tests/cli_p0_fixes.rs` 追加：

```rust
mod apply_config_tests {
    use super::*;
    use std::process::Command;

    fn cli_binary() -> String {
        env!("CARGO_BIN_EXE_cc-switch-cli").to_string()
    }

    #[test]
    fn test_apply_config_rejects_listen_field() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let home = tmp_dir.path().to_str().unwrap();
        let yaml_path = tmp_dir.path().join("test.yaml");

        std::fs::write(
            &yaml_path,
            r#"
proxy:
  listen: 0.0.0.0
  port: 9091
providers: []
"#,
        )
        .unwrap();

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["validate", yaml_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute CLI");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success() || stderr.contains("unknown field"),
            "validate 应拒绝 listen/port 字段，实际 stderr: {stderr}"
        );
    }

    #[test]
    fn test_apply_config_accepts_takeover_only() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let home = tmp_dir.path().to_str().unwrap();
        let yaml_path = tmp_dir.path().join("test.yaml");

        std::fs::write(
            &yaml_path,
            r#"
proxy:
  takeover:
    claude: true
providers: []
"#,
        )
        .unwrap();

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["validate", yaml_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute CLI");

        assert!(
            output.status.success(),
            "validate 应接受 takeover 字段，stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_p0_fixes apply_config -- --nocapture
```

Expected: `test_apply_config_rejects_listen_field` 失败（当前 schema 接受 listen 字段）

- [ ] **Step 3: 修改 ProxySection 结构体删除 listen/port 字段**

Read [src-tauri/src/core/decl_config.rs:30-60](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs)

将第 30~39 行：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProxySection {
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub takeover: HashMap<String, bool>,
}
```

替换为：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProxySection {
    /// 仅保留 takeover（可应用）。listen/port 已移除：使用 CC_SWITCH_LISTEN/CC_SWITCH_PORT 环境变量或 proxy-config 命令设置。
    #[serde(default)]
    pub takeover: HashMap<String, bool>,
}
```

- [ ] **Step 4: 修改 ProxySection 的 Default 实现**

Read [src-tauri/src/core/decl_config.rs:41-50](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs)

将第 41~49 行：

```rust
impl Default for ProxySection {
    fn default() -> Self {
        Self {
            listen: default_listen(),
            port: default_port(),
            takeover: HashMap::new(),
        }
    }
}
```

替换为：

```rust
impl Default for ProxySection {
    fn default() -> Self {
        Self {
            takeover: HashMap::new(),
        }
    }
}
```

- [ ] **Step 5: 删除 default_listen 与 default_port 函数**

Grep [src-tauri/src/core/decl_config.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs) 查找 `fn default_listen` 与 `fn default_port`，删除这两个函数（已无引用）。

- [ ] **Step 6: 添加 deny_unknown_fields 防止 listen/port 静默接受**

修改 ProxySection 的派生属性：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct ProxySection {
    /// 仅保留 takeover（可应用）。listen/port 已移除：使用 CC_SWITCH_LISTEN/CC_SWITCH_PORT 环境变量或 proxy-config 命令设置。
    #[serde(default)]
    pub takeover: HashMap<String, bool>,
}
```

> 注：若 `DeclConfig` 根结构也需要 `deny_unknown_fields`，按相同模式添加。但根结构添加可能破坏向前兼容，仅 ProxySection 添加即可。

- [ ] **Step 7: 检查 validate 命令是否调用 deserialize**

Read `cmd_validate` 函数（Grep `fn cmd_validate` in [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)）

确认 validate 命令调用 `serde_yaml::from_str::<DeclConfig>`。若调用，则 `deny_unknown_fields` 会让 listen/port 字段在 validate 阶段就报错。

- [ ] **Step 8: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_p0_fixes apply_config -- --nocapture
```

Expected: 2 个 apply_config 测试通过

- [ ] **Step 9: 运行全部测试确保无回归**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test
```

Expected: 所有测试通过（特别注意 decl_config 已有测试，可能需要同步更新）

- [ ] **Step 10: 更新参考手册 YAML 示例**

Read [docs/cli-reference-manual.md:1338-1390](file:///f:/workspace/trae/cc-switch/docs/cli-reference-manual.md)

将 YAML 示例中的 `proxy:` 段：

```yaml
proxy:
  listen: 127.0.0.1
  port: 9090
  takeover:
    claude: true
```

改为：

```yaml
proxy:
  takeover:
    claude: true
# 注：listen/port 已移除，请使用 CC_SWITCH_LISTEN 与 CC_SWITCH_PORT 环境变量，或 proxy-config 命令设置
```

- [ ] **Step 11: 更新参考手册"已知限制"章节**

Read [docs/cli-reference-manual.md:1383-1385](file:///f:/workspace/trae/cc-switch/docs/cli-reference-manual.md)

将原"不会被 apply-config 实际应用"说明改为：

```markdown
> **已移除字段**（2026-07-04）：`proxy.listen` 与 `proxy.port` 已从 YAML schema 删除。请使用 `CC_SWITCH_LISTEN` / `CC_SWITCH_PORT` 环境变量、`proxy-config` 命令或 `takeover` 命令设置。`proxy.takeover` 仍可在 YAML 中配置并通过 `apply-config` 应用。
```

- [ ] **Step 12: 验证编译**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo check --bin cc-switch-cli
cargo clippy --bin cc-switch-cli
```

Expected: 编译通过，无 `default_listen` / `default_port` 相关错误

- [ ] **Step 13: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/core/decl_config.rs docs/cli-reference-manual.md src-tauri/tests/cli_p0_fixes.rs
git commit -m "fix(decl-config): 移除 ProxySection.listen/port 字段止血

apply-config 之前只校验不应用 listen/port，造成静默失败。
止血方案：从 YAML schema 删除字段，添加 deny_unknown_fields，
validate 阶段即报错。

takeover 字段保留，仍可通过 apply-config 应用。
listen/port 请使用 CC_SWITCH_LISTEN/CC_SWITCH_PORT 环境变量。

后续 Plan B 将升级为 ApplyContext 方案 A，真正应用代理字段。

关联 spec: §七.1 P0-2 方案 B"
```

---

## Task 6: 补充参考手册 speedtest/verify-key 说明（P2-1 顺便完成）

**Files:**
- Modify: `docs/cli-reference-manual.md`（speedtest/verify-key 章节）

- [ ] **Step 1: 找到 speedtest 与 verify-key 章节**

Run:

```bash
cd f:/workspace/trae/cc-switch
rg -n "^## (speedtest|verify-key)" docs/cli-reference-manual.md
```

- [ ] **Step 2: 在 speedtest 章节添加"不依赖代理运行"说明**

在 speedtest 章节标题下方插入：

```markdown
> ℹ️ **不依赖代理运行**：此命令直接向目标 URL 发送 HTTP 请求，不需要代理服务器在运行。适合首次配置时测试供应商连通性。
```

- [ ] **Step 3: 在 verify-key 章节添加相同说明**

在 verify-key 章节标题下方插入：

```markdown
> ℹ️ **不依赖代理运行**：此命令直接向供应商 API 发送认证请求，不需要代理服务器在运行。
```

- [ ] **Step 4: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add docs/cli-reference-manual.md
git commit -m "docs(cli): 补充 speedtest/verify-key 不依赖代理运行的说明

关联 spec: §七.3 P2-1"
```

---

## Task 7: 最终验证与回归测试

**Files:**
- 无修改，仅运行验证

- [ ] **Step 1: 运行全部 Rust 测试**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test
```

Expected: 所有测试通过

- [ ] **Step 2: 运行 clippy 检查**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo clippy --all-targets
```

Expected: 无新增警告

- [ ] **Step 3: 运行 cargo fmt 检查**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo fmt --check
```

Expected: 无格式问题（如有，运行 `cargo fmt` 修复）

- [ ] **Step 4: 运行前端 typecheck（虽然未改前端，确认无破坏）**

Run:

```bash
cd f:/workspace/trae/cc-switch
pnpm typecheck
```

Expected: 通过

- [ ] **Step 5: 手动验证 CLI 命令**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo build --bin cc-switch-cli
./target/debug/cc-switch-cli.exe help
```

Expected: help 输出中不再包含 `stream-check`、`stream-check-all`、`remove-session`

Run:

```bash
./target/debug/cc-switch-cli.exe add-provider codex test-1 "Test" --api-key sk-test --base-url https://api.openai.com/v1
./target/debug/cc-switch-cli.exe list-providers codex
```

Expected: list-providers 输出显示 `https://api.openai.com/v1` 作为 Base URL

- [ ] **Step 6: 手动验证 apply-config**

创建测试 YAML `test.yaml`:

```yaml
proxy:
  takeover:
    claude: true
providers: []
```

Run:

```bash
./target/debug/cc-switch-cli.exe validate test.yaml
```

Expected: 校验通过

创建错误 YAML `test-bad.yaml`:

```yaml
proxy:
  listen: 0.0.0.0
providers: []
```

Run:

```bash
./target/debug/cc-switch-cli.exe validate test-bad.yaml
```

Expected: 校验失败，提示 `unknown field listen`

- [ ] **Step 7: Commit 最终验证记录**

如有格式修复：

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo fmt
git add .
git commit -m "style: cargo fmt 修复格式"
```

---

## Self-Review

### Spec 覆盖检查

| Spec §七.1 项 | 对应 Task | 状态 |
|---|---|---|
| P0-1 修正评估文档声称 | Task 1 | ✅ |
| P0-2 apply-config 方案 B 止血 | Task 5 | ✅ |
| P0-3 处理桩命令（stream-check/stream-check-all/remove-session） | Task 2 + Task 3 | ✅ |
| P0-4 修复 env 硬编码 | Task 4 | ✅ |
| P2-1 speedtest/verify-key 说明（顺便） | Task 6 | ✅ |
| 最终验证 | Task 7 | ✅ |

### Placeholder 扫描

- 所有代码块均含完整实现，无 TBD/TODO
- 所有文件路径均为绝对路径或相对项目根的路径
- 所有命令均含 expected 输出说明

### 类型一致性

- `ProxySection` 删除 listen/port 后，所有引用已更新（Default、decl_config.rs 内部）
- `add-provider` env 字段名映射函数与 `update-provider`、`list-providers` 保持一致
- 测试中 `Database::init_at(home)` 函数名需在实施时确认（可能是 `init` + 环境变量，或 `init_at` 接受路径参数）

### 已知风险

1. **Database::init_at 函数名**：测试中假设存在 `Database::init_at(home)` 接受路径参数。实施时若该函数不存在，需查 `Database::init` 签名，可能需要先设置 `CC_SWITCH_HOME` 环境变量再调用 `Database::init`。
2. **tempfile 依赖**：测试使用 `tempfile` crate，需确认 `src-tauri/Cargo.toml` 的 `[dev-dependencies]` 已包含。若无，需先添加。
3. **CLI 二进制测试**：`env!("CARGO_BIN_EXE_cc-switch-cli")` 需要 `[[bin]]` 配置正确。若无法解析，改用 `std::env::current_exe()` 推导路径。

实施时如遇上述问题，先 Grep 确认现有 API 再调整测试代码。

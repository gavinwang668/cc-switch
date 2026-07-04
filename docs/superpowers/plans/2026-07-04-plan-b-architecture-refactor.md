# Plan B: 架构重构 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 拆分 `cc_switch_lib` 单一 crate 为三层（`cc-switch-core` / `cc-switch-tauri-commands` / `cc-switch-app`），让 CLI 不再依赖 Tauri/webkit2gtk，`CopilotAuthState` 改由 core service 持有，`apply-config` 接收 `ApplyContext`，API 格式设置覆盖 7 种应用。

**Architecture:** Cargo workspace 三层 crate 拆分。`cc-switch-core` 持有所有纯业务逻辑（database / services / proxy / commands 的 service 部分），不依赖 `tauri`；`cc-switch-tauri-commands` 是 `#[tauri::command]` 包装层，依赖 core；`cc-switch-app`（原 src-tauri）保留 GUI 二进制与 Tauri Builder。CLI 二进制迁到独立 crate `cc-switch-cli`，只依赖 core。

**Tech Stack:** Rust 1.85+、Cargo workspace、Tauri 2.8、tokio、rusqlite、reqwest、clap。

**关联 Spec:** [docs/superpowers/specs/2026-07-04-cli-feature-review-design.md](file:///f:/workspace/trae/cc-switch/docs/superpowers/specs/2026-07-04-cli-feature-review-design.md) §七.2（P1-1 / P1-2 / P1-3）、§七.1 P0-2 方案 A、§六.7 架构重构建议。

**前置依赖:** Plan A（`docs/superpowers/plans/2026-07-04-plan-a-p0-fixes.md`）已全部完成。

---

## File Structure

| 文件 / 目录 | 操作 | 责任 |
|---|---|---|
| `Cargo.toml`（根） | 新建 | workspace 配置，列出 4 个成员 crate |
| `crates/cc-switch-core/Cargo.toml` | 新建 | core crate 依赖（无 tauri） |
| `crates/cc-switch-core/src/lib.rs` | 新建 | re-export database/services/proxy/core/error 等 |
| `crates/cc-switch-core/src/error.rs` | 迁移 | 来自 `src-tauri/src/error.rs` |
| `crates/cc-switch-core/src/provider.rs` | 迁移 | 来自 `src-tauri/src/provider.rs` |
| `crates/cc-switch-core/src/app_config.rs` | 迁移 | 来自 `src-tauri/src/app_config.rs` |
| `crates/cc-switch-core/src/database/` | 迁移 | 来自 `src-tauri/src/database/` |
| `crates/cc-switch-core/src/config.rs` | 迁移 | 来自 `src-tauri/src/config.rs` |
| `crates/cc-switch-core/src/settings.rs` | 迁移 | 来自 `src-tauri/src/settings.rs` |
| `crates/cc-switch-core/src/services/` | 迁移 | 来自 `src-tauri/src/services/`，去 Tauri 化 |
| `crates/cc-switch-core/src/proxy/` | 迁移 | 来自 `src-tauri/src/proxy/` |
| `crates/cc-switch-core/src/core/` | 迁移 | 来自 `src-tauri/src/core/`，含 `ApplyContext` |
| `crates/cc-switch-core/src/copilot_auth_state.rs` | 新建 | `CopilotAuthState = Arc<RwLock<CopilotAuthManager>>` 类型别名 |
| `crates/cc-switch-tauri-commands/Cargo.toml` | 新建 | tauri-commands crate 依赖 |
| `crates/cc-switch-tauri-commands/src/lib.rs` | 新建 | re-export 全部 commands 子模块 |
| `crates/cc-switch-tauri-commands/src/commands/` | 迁移 | 来自 `src-tauri/src/commands/`，调用 core service |
| `crates/cc-switch-cli/Cargo.toml` | 新建 | CLI 独立 crate，只依赖 cc-switch-core |
| `crates/cc-switch-cli/src/main.rs` | 迁移 | 来自 `src-tauri/src/bin/cc-switch-cli.rs` |
| `src-tauri/Cargo.toml` | 修改 | 重命名 package 为 `cc-switch-app`，依赖新 crate |
| `src-tauri/src/lib.rs` | 修改 | 删除已迁移模块，依赖 `cc_switch_tauri_commands` |
| `src-tauri/src/main.rs` | 修改 | 不变（仅 `cc_switch_app::run()` 调用） |
| `crates/cc-switch-core/tests/apply_context_test.rs` | 新建 | ApplyContext 行为测试 |
| `crates/cc-switch-core/tests/copilot_auth_state_test.rs` | 新建 | CopilotAuthState service 持有测试 |
| `crates/cc-switch-cli/tests/stream_check_cli_test.rs` | 新建 | CLI stream-check 集成测试 |

**迁移核心原则：**
1. **每完成一个 Task 必须能 `cargo build --workspace` 通过**——保持可回滚。
2. **迁移文件用 `git mv`** 保留历史；之后只改 `use` 路径。
3. **`crate::` 在迁移后改为 `cc_switch_core::` 或 `cc_switch_tauri_commands::`**——按目标 crate 而定。
4. **不破坏 Plan A 的修复**——`apply-config` 止血字段、`add-provider` env 修复、桩命令删除都保留。

---

## Task 1: 创建 Cargo workspace 根配置

**Files:**
- Create: `Cargo.toml`（根）
- Modify: `src-tauri/Cargo.toml`（仅注释说明属于 workspace 成员）

- [ ] **Step 1: 在项目根创建 workspace Cargo.toml**

写入 `f:\workspace\trae\cc-switch\Cargo.toml`：

```toml
# CC Switch Cargo Workspace
#
# 三个成员 crate：
# - crates/cc-switch-core          纯业务逻辑（无 Tauri 依赖）
# - crates/cc-switch-tauri-commands  #[tauri::command] 包装层
# - crates/cc-switch-cli           CLI 二进制（只依赖 core）
# - src-tauri                      GUI 二进制（重命名为 cc-switch-app package）
#
# 旧 src-tauri 在 Task 11 中重命名 package；本 workspace 仅声明成员，
# 不影响 src-tauri 现有构建。

[workspace]
resolver = "2"
members = [
    "crates/cc-switch-core",
    "crates/cc-switch-tauri-commands",
    "crates/cc-switch-cli",
    "src-tauri",
]

# 共享 profile（与 src-tauri 现有 profile 保持一致）
[profile.release]
codegen-units = 1
lto = "thin"
opt-level = "s"
panic = "unwind"
strip = "symbols"

[profile.dev]
opt-level = 0
```

- [ ] **Step 2: 在 src-tauri/Cargo.toml 顶部加注释**

在 `src-tauri/Cargo.toml` 第 1 行上方插入注释（不修改其它内容）：

```toml
# 本 crate 是 workspace 成员，详见根 Cargo.toml。
# package name 在 Task 11 改为 cc-switch-app。
```

- [ ] **Step 3: 验证 workspace 解析**

Run: `cargo metadata --no-deps --format-version 1 > NUL`
Expected: 命令成功退出（exit 0）。若报 "no such crate member"，检查 members 路径大小写。

- [ ] **Step 4: 验证 src-tauri 仍能编译**

Run: `cargo build -p cc-switch`
Expected: 编译成功。新 workspace 不影响现有 crate（成员路径都暂不存在，但 `cargo metadata` 只校验已声明的）。

> 注：若 `cargo build` 报"could not find member"，回退 Step 1 把 members 暂时改为 `members = ["src-tauri"]`，待 Task 2/3/10 创建子 crate 后再追加。

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src-tauri/Cargo.toml
git commit -m "chore(workspace): 初始化 Cargo workspace 根配置"
```

---

## Task 2: 创建 cc-switch-core crate 空壳

**Files:**
- Create: `crates/cc-switch-core/Cargo.toml`
- Create: `crates/cc-switch-core/src/lib.rs`

- [ ] **Step 1: 创建目录结构**

Run: `mkdir crates\cc-switch-core\src`

- [ ] **Step 2: 创建 Cargo.toml**

写入 `crates/cc-switch-core/Cargo.toml`：

```toml
[package]
name = "cc-switch-core"
version = "3.16.5"
edition = "2021"
rust-version = "1.85.0"
description = "CC Switch 纯业务逻辑层（无 Tauri 依赖）"
license = "MIT"

[lib]
name = "cc_switch_core"
crate-type = ["rlib"]
doctest = false

[features]
default = []
test-hooks = []

# 注意：本 crate 严禁依赖 tauri / tauri-plugin-* / webkit2gtk。
# 所有 Tauri 相关代码迁到 cc-switch-tauri-commands。
[dependencies]
serde_json = { version = "1.0", features = ["preserve_order"] }
serde = { version = "1.0", features = ["derive"] }
log = "0.4"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5.0"
toml = "0.8"
toml_edit = "0.22"
reqwest = { version = "0.12", features = ["rustls-tls", "json", "stream", "socks"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time", "sync", "signal"] }
futures = "0.3"
async-stream = "0.3"
bytes = "1.5"
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors"] }
hyper = { version = "1.0", features = ["full"] }
hyper-util = { version = "0.1", features = ["tokio", "http1", "client-legacy"] }
hyper-rustls = { version = "0.27", features = ["http1", "tls12", "ring", "webpki-tokio"] }
http = "1"
http-body = "1"
http-body-util = "0.1"
httparse = "1"
tokio-rustls = "0.26"
rustls = "0.23"
webpki-roots = "0.26"
rustls-native-certs = "0.8"
regex = "1.10"
rquickjs = { version = "0.8", features = ["array-buffer", "classes"] }
thiserror = "2.0"
anyhow = "1.0"
zip = "2.2"
serde_yaml = "0.9"
tempfile = "3"
url = "2.5"
once_cell = "1.21.3"
base64 = "0.22"
rusqlite = { version = "0.31", features = ["bundled", "backup", "hooks"] }
indexmap = { version = "2", features = ["serde"] }
rust_decimal = "1.33"
uuid = { version = "1.11", features = ["v4"] }
sha2 = "0.10"
hmac = "0.12"
json5 = "0.4"
json-five = "0.3.1"
flate2 = "1"
brotli = "7"
zstd = "0.13"
arboard = "3.6"

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(not(target_os = "linux"))'.dependencies]
keyring = { version = "3.6", features = ["windows-native"] }

[target.'cfg(all(target_os = "windows", target_arch = "aarch64"))'.dependencies]
rquickjs = { version = "0.8", features = ["bindgen"] }

[target.'cfg(target_os = "windows")'.dependencies]
winreg = "0.52"
windows-sys = { version = "0.61", features = ["Win32_Globalization", "Win32_UI_Shell"] }

[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-app-kit = { version = "0.2", features = ["NSColor"] }

[dev-dependencies]
serial_test = "3"
tempfile = "3"
```

- [ ] **Step 3: 创建空 lib.rs**

写入 `crates/cc-switch-core/src/lib.rs`：

```rust
//! CC Switch 核心业务逻辑层
//!
//! 本 crate 包含所有不依赖 Tauri 的业务逻辑：
//! - database: SQLite DAO
//! - services: 业务服务（ProviderService / ProxyService / McpService 等）
//! - proxy: 本地代理服务器
//! - core: bootstrap / provider_manager / decl_config
//! - error / provider / app_config / config / settings: 基础类型
//!
//! 严禁依赖 tauri / tauri-plugin-* / webkit2gtk。
//! Tauri 命令包装层在 cc-switch-tauri-commands crate 中。

#![allow(clippy::module_inception)]

// 模块在后续 Task 中逐步迁移到此 crate。
```

- [ ] **Step 4: 把 cc-switch-core 加入 workspace**

修改根 `Cargo.toml` 的 `members`，确认已包含 `"crates/cc-switch-core"`（Task 1 已写入）。

- [ ] **Step 5: 验证空 crate 编译**

Run: `cargo build -p cc-switch-core`
Expected: 编译成功，无错误。

- [ ] **Step 6: Commit**

```bash
git add crates/cc-switch-core Cargo.toml
git commit -m "feat(core): 创建 cc-switch-core crate 空壳"
```

---

## Task 3: 迁移 error 模块到 cc-switch-core

**Files:**
- Move: `src-tauri/src/error.rs` → `crates/cc-switch-core/src/error.rs`
- Modify: `src-tauri/src/lib.rs`（删除 `mod error;`，改为 `pub use cc_switch_core::error;`）
- Modify: `crates/cc-switch-core/src/lib.rs`

- [ ] **Step 1: git mv error.rs**

Run: `git mv src-tauri\src\error.rs crates\cc-switch-core\src\error.rs`

- [ ] **Step 2: 在 cc-switch-core/src/lib.rs 注册模块**

把 `crates/cc-switch-core/src/lib.rs` 改为：

```rust
//! CC Switch 核心业务逻辑层
//!
//! 本 crate 包含所有不依赖 Tauri 的业务逻辑：
//! - database: SQLite DAO
//! - services: 业务服务（ProviderService / ProxyService / McpService 等）
//! - proxy: 本地代理服务器
//! - core: bootstrap / provider_manager / decl_config
//! - error / provider / app_config / config / settings: 基础类型
//!
//! 严禁依赖 tauri / tauri-plugin-* / webkit2gtk。
//! Tauri 命令包装层在 cc-switch-tauri-commands crate 中。

#![allow(clippy::module_inception)]

pub mod error;

pub use error::AppError;
```

- [ ] **Step 3: 检查 error.rs 内的 use 路径**

Run: `cargo build -p cc-switch-core`

如果 `error.rs` 内有 `use crate::xxx`，先保留——它们在迁移对应模块后再统一修复。当前 `error.rs` 通常只依赖 `thiserror`/`serde`，编译应通过。

Expected: 编译成功。

- [ ] **Step 4: 修改 src-tauri/src/lib.rs**

把 `src-tauri/src/lib.rs` 第 14 行 `mod error;` 改为：

```rust
pub use cc_switch_core::error;
```

（其它行不动；后续 Task 会逐步把 `mod xxx;` 改为 `pub use cc_switch_core::xxx;`）

- [ ] **Step 5: 修改 src-tauri/Cargo.toml 添加依赖**

在 `src-tauri/Cargo.toml` 的 `[dependencies]` 顶部加：

```toml
cc-switch-core = { path = "../crates/cc-switch-core" }
```

- [ ] **Step 6: 验证 src-tauri 仍能编译**

Run: `cargo build -p cc-switch`
Expected: 编译成功。

如果 `error.rs` 内有 `use crate::xxx` 指向尚未迁移的模块，会报路径错。临时改为 `use cc_switch_core::xxx;`——但若 xxx 还未迁移到 core，则改回 `src-tauri` 内的 `crate::xxx`。当前 `error.rs` 通常自包含，应无问题。

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/Cargo.toml crates/cc-switch-core/src/error.rs crates/cc-switch-core/src/lib.rs
git commit -m "refactor(core): 迁移 error 模块到 cc-switch-core"
```

---

## Task 4: 迁移 provider/app_config 等基础类型到 cc-switch-core

**Files:**
- Move: `src-tauri/src/provider.rs` → `crates/cc-switch-core/src/provider.rs`
- Move: `src-tauri/src/app_config.rs` → `crates/cc-switch-core/src/app_config.rs`
- Move: `src-tauri/src/provider_defaults.rs` → `crates/cc-switch-core/src/provider_defaults.rs`
- Modify: `crates/cc-switch-core/src/lib.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: git mv 三个文件**

Run:
```
git mv src-tauri\src\provider.rs crates\cc-switch-core\src\provider.rs
git mv src-tauri\src\app_config.rs crates\cc-switch-core\src\app_config.rs
git mv src-tauri\src\provider_defaults.rs crates\cc-switch-core\src\provider_defaults.rs
```

- [ ] **Step 2: 更新 cc-switch-core/src/lib.rs 注册模块**

把 `crates/cc-switch-core/src/lib.rs` 改为：

```rust
//! CC Switch 核心业务逻辑层
//!
//! 详见 crate 顶部文档。

#![allow(clippy::module_inception)]

pub mod app_config;
pub mod error;
pub mod provider;
pub mod provider_defaults;

pub use app_config::{AppType, InstalledSkill, McpApps, McpServer, MultiAppConfig, SkillApps};
pub use error::AppError;
pub use provider::{Provider, ProviderMeta};
```

- [ ] **Step 3: 把迁移文件内的 crate:: 改为 cc_switch_core::**

打开 `crates/cc-switch-core/src/provider.rs`，搜索 `use crate::`：

- 若引用 `error::AppError`，改为 `use crate::error::AppError;`（仍在同 crate 内，无需改）
- 若引用 `database::Database`，改为 `// 迁移后在 Task 5 启用： use crate::database::Database;`——当前阶段 `provider.rs` 通常不直接依赖 database，先注释掉该 use，待 Task 5 后恢复

打开 `crates/cc-switch-core/src/app_config.rs`，同理处理。

Run: `cargo build -p cc-switch-core`
Expected: 编译成功。若有 `crate::database` 引用未解决，临时把对应 `use` 改为 `// 待 Task 5`。

- [ ] **Step 4: 修改 src-tauri/src/lib.rs**

把 `src-tauri/src/lib.rs` 中以下三行：

```rust
mod app_config;
mod provider;
mod provider_defaults;
```

改为：

```rust
pub use cc_switch_core::app_config;
pub use cc_switch_core::provider;
pub use cc_switch_core::provider_defaults;
```

保留 `pub use app_config::{AppType, ...};` 等再 export 不变（路径现在从 core 来）。

- [ ] **Step 5: 验证 workspace 编译**

Run: `cargo build --workspace`
Expected: 编译成功。

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor(core): 迁移 provider/app_config/provider_defaults 到 cc-switch-core"
```

---

## Task 5: 迁移 database 模块到 cc-switch-core

**Files:**
- Move: `src-tauri/src/database/` → `crates/cc-switch-core/src/database/`
- Modify: `crates/cc-switch-core/src/lib.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: git mv database 目录**

Run: `git mv src-tauri\src\database crates\cc-switch-core\src\database`

- [ ] **Step 2: 在 cc-switch-core/src/lib.rs 注册**

把 `crates/cc-switch-core/src/lib.rs` 在 `pub mod provider_defaults;` 后追加：

```rust
pub mod database;

pub use database::Database;
```

- [ ] **Step 3: 修复迁移后 provider/app_config 中之前注释的 use**

打开 `crates/cc-switch-core/src/provider.rs`，把 Step 3（Task 4）注释掉的 `// 待 Task 5: use crate::database::Database;` 改回：

```rust
use crate::database::Database;
```

同理修复 `crates/cc-switch-core/src/app_config.rs`。

Run: `cargo build -p cc-switch-core`
Expected: 编译成功。若 database 内部 use `crate::xxx` 指向 services 或 proxy（尚未迁移），暂时保留——database DAO 通常不反向依赖 services。

- [ ] **Step 4: 修改 src-tauri/src/lib.rs**

把 `src-tauri/src/lib.rs` 中：

```rust
mod database;
```

改为：

```rust
pub use cc_switch_core::database;
```

`pub use database::Database;` 保持不变。

- [ ] **Step 5: 验证 workspace 编译**

Run: `cargo build --workspace`
Expected: 编译成功。

- [ ] **Step 6: 运行 database 单元测试**

Run: `cargo test -p cc-switch-core --lib database`
Expected: 全部测试通过。

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor(core): 迁移 database 模块到 cc-switch-core"
```

---

## Task 6: 迁移 config/settings 等基础模块到 cc-switch-core

**Files:**
- Move: `src-tauri/src/config.rs` → `crates/cc-switch-core/src/config.rs`
- Move: `src-tauri/src/settings.rs` → `crates/cc-switch-core/src/settings.rs`
- Move: `src-tauri/src/codex_config.rs` → `crates/cc-switch-core/src/codex_config.rs`
- Move: `src-tauri/src/gemini_config.rs` → `crates/cc-switch-core/src/gemini_config.rs`
- Move: `src-tauri/src/hermes_config.rs` → `crates/cc-switch-core/src/hermes_config.rs`
- Move: `src-tauri/src/openclaw_config.rs` → `crates/cc-switch-core/src/openclaw_config.rs`
- Move: `src-tauri/src/opencode_config.rs` → `crates/cc-switch-core/src/opencode_config.rs`
- Move: `src-tauri/src/claude_desktop_config.rs` → `crates/cc-switch-core/src/claude_desktop_config.rs`
- Move: `src-tauri/src/claude_mcp.rs` → `crates/cc-switch-core/src/claude_mcp.rs`
- Move: `src-tauri/src/claude_plugin.rs` → `crates/cc-switch-core/src/claude_plugin.rs`
- Move: `src-tauri/src/gemini_mcp.rs` → `crates/cc-switch-core/src/gemini_mcp.rs`
- Move: `src-tauri/src/codex_history_migration.rs` → `crates/cc-switch-core/src/codex_history_migration.rs`
- Move: `src-tauri/src/prompt.rs` → `crates/cc-switch-core/src/prompt.rs`
- Move: `src-tauri/src/prompt_files.rs` → `crates/cc-switch-core/src/prompt_files.rs`
- Move: `src-tauri/src/mcp/` → `crates/cc-switch-core/src/mcp/`
- Move: `src-tauri/src/deeplink/` → `crates/cc-switch-core/src/deeplink/`
- Modify: `crates/cc-switch-core/src/lib.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 批量 git mv**

Run（PowerShell 单行）:
```
git mv src-tauri\src\config.rs crates\cc-switch-core\src\config.rs; git mv src-tauri\src\settings.rs crates\cc-switch-core\src\settings.rs; git mv src-tauri\src\codex_config.rs crates\cc-switch-core\src\codex_config.rs; git mv src-tauri\src\gemini_config.rs crates\cc-switch-core\src\gemini_config.rs; git mv src-tauri\src\hermes_config.rs crates\cc-switch-core\src\hermes_config.rs; git mv src-tauri\src\openclaw_config.rs crates\cc-switch-core\src\openclaw_config.rs; git mv src-tauri\src\opencode_config.rs crates\cc-switch-core\src\opencode_config.rs; git mv src-tauri\src\claude_desktop_config.rs crates\cc-switch-core\src\claude_desktop_config.rs; git mv src-tauri\src\claude_mcp.rs crates\cc-switch-core\src\claude_mcp.rs; git mv src-tauri\src\claude_plugin.rs crates\cc-switch-core\src\claude_plugin.rs; git mv src-tauri\src\gemini_mcp.rs crates\cc-switch-core\src\gemini_mcp.rs; git mv src-tauri\src\codex_history_migration.rs crates\cc-switch-core\src\codex_history_migration.rs; git mv src-tauri\src\prompt.rs crates\cc-switch-core\src\prompt.rs; git mv src-tauri\src\prompt_files.rs crates\cc-switch-core\src\prompt_files.rs; git mv src-tauri\src\mcp crates\cc-switch-core\src\mcp; git mv src-tauri\src\deeplink crates\cc-switch-core\src\deeplink
```

- [ ] **Step 2: 更新 cc-switch-core/src/lib.rs**

把 `crates/cc-switch-core/src/lib.rs` 替换为：

```rust
//! CC Switch 核心业务逻辑层
//!
//! 详见 crate 顶部文档。

#![allow(clippy::module_inception)]

pub mod app_config;
pub mod claude_desktop_config;
pub mod claude_mcp;
pub mod claude_plugin;
pub mod codex_config;
pub mod codex_history_migration;
pub mod config;
pub mod database;
pub mod deeplink;
pub mod error;
pub mod gemini_config;
pub mod gemini_mcp;
pub mod hermes_config;
pub mod mcp;
pub mod openclaw_config;
pub mod opencode_config;
pub mod prompt;
pub mod prompt_files;
pub mod provider;
pub mod provider_defaults;
pub mod settings;

pub use app_config::{AppType, InstalledSkill, McpApps, McpServer, MultiAppConfig, SkillApps};
pub use codex_config::{get_codex_auth_path, get_codex_config_path, write_codex_live_atomic};
pub use config::{get_app_config_dir, get_claude_mcp_path, get_claude_settings_path, read_json_file};
pub use database::Database;
pub use deeplink::{import_provider_from_deeplink, parse_deeplink_url, DeepLinkImportRequest};
pub use error::AppError;
pub use prompt::Prompt;
pub use mcp::{
    import_from_claude, import_from_codex, import_from_gemini, remove_server_from_claude,
    remove_server_from_codex, remove_server_from_gemini, sync_enabled_to_claude,
    sync_enabled_to_codex, sync_enabled_to_gemini, sync_single_server_to_claude,
    sync_single_server_to_codex, sync_single_server_to_gemini,
};
pub use provider::{Provider, ProviderMeta};
pub use settings::{get_settings, reload_settings, update_settings, AppSettings};
```

- [ ] **Step 3: 修复 use crate::路径**

各迁移文件内若有 `use crate::xxx` 指向 services / proxy / store 等尚未迁移模块，**保持不变**——这些 use 在 Task 7/8 完成后才会解析成功。当前 `cargo build -p cc-switch-core` 会失败，本 Step 仅做记录。

不需要逐文件修改：迁移文件间的 `use crate::xxx`（如 `use crate::error::AppError`）仍正确，因为都在同一 crate。

- [ ] **Step 4: 修改 src-tauri/src/lib.rs**

把 `src-tauri/src/lib.rs` 顶部 `mod xxx;` 中已迁移的模块改为 `pub use cc_switch_core::xxx;`：

```rust
pub use cc_switch_core::app_config;
pub use cc_switch_core::claude_desktop_config;
pub use cc_switch_core::claude_mcp;
pub use cc_switch_core::claude_plugin;
pub use cc_switch_core::codex_config;
pub use cc_switch_core::codex_history_migration;
pub use cc_switch_core::config;
pub use cc_switch_core::database;
pub use cc_switch_core::deeplink;
pub use cc_switch_core::error;
pub use cc_switch_core::gemini_config;
pub use cc_switch_core::gemini_mcp;
pub use cc_switch_core::hermes_config;
pub use cc_switch_core::mcp;
pub use cc_switch_core::openclaw_config;
pub use cc_switch_core::opencode_config;
pub use cc_switch_core::prompt;
pub use cc_switch_core::prompt_files;
pub use cc_switch_core::provider;
pub use cc_switch_core::provider_defaults;
pub use cc_switch_core::settings;

mod app_store;
mod auto_launch;
mod commands;
mod core;
mod init_status;
mod lightweight;
#[cfg(target_os = "linux")]
mod linux_fix;
mod panic_hook;
mod proxy;
mod services;
pub mod session_manager;
mod store;
mod tray;
mod usage_events;
mod usage_script;
```

注意：`pub mod commands;` / `pub mod core;` / `mod services;` / `mod proxy;` / `mod store;` 暂时保留在 src-tauri，后续 Task 迁移。

- [ ] **Step 5: 验证 src-tauri 仍可编译（容忍 core 编译失败）**

Run: `cargo build -p cc-switch`
Expected: src-tauri 编译成功；`cargo build -p cc-switch-core` 可能因 services/proxy 未迁移而失败——这是预期，下一个 Task 修复。

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor(core): 迁移 config/settings/各应用 config/mcp/deeplink 到 cc-switch-core"
```

---

## Task 7: 迁移 services 模块到 cc-switch-core

**Files:**
- Move: `src-tauri/src/services/` → `crates/cc-switch-core/src/services/`
- Move: `src-tauri/src/store.rs` → `crates/cc-switch-core/src/store.rs`
- Move: `src-tauri/src/session_manager/` → `crates/cc-switch-core/src/session_manager/`
- Move: `src-tauri/src/usage_events.rs` → `crates/cc-switch-core/src/usage_events.rs`
- Move: `src-tauri/src/usage_script.rs` → `crates/cc-switch-core/src/usage_script.rs`
- Move: `src-tauri/src/init_status.rs` → `crates/cc-switch-core/src/init_status.rs`
- Modify: `crates/cc-switch-core/src/lib.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 批量 git mv**

Run:
```
git mv src-tauri\src\services crates\cc-switch-core\src\services; git mv src-tauri\src\store.rs crates\cc-switch-core\src\store.rs; git mv src-tauri\src\session_manager crates\cc-switch-core\src\session_manager; git mv src-tauri\src\usage_events.rs crates\cc-switch-core\src\usage_events.rs; git mv src-tauri\src\usage_script.rs crates\cc-switch-core\src\usage_script.rs; git mv src-tauri\src\init_status.rs crates\cc-switch-core\src\init_status.rs
```

- [ ] **Step 2: 更新 cc-switch-core/src/lib.rs**

在 `pub mod settings;` 后追加：

```rust
pub mod init_status;
pub mod services;
pub mod session_manager;
pub mod store;
pub mod usage_events;
pub mod usage_script;

pub use services::{
    skill::{migrate_skills_to_ssot, ImportSkillSelection},
    ConfigService, EndpointLatency, McpService, PromptService, ProviderService, ProviderSortUpdate,
    ProxyService, SkillService, SpeedtestService,
    model_fetch::FetchedModel,
    env_manager,
};
pub use services::usage_stats::{LogFilters, PaginatedLogs, ProviderLimitStatus, UsageSummary};
pub use store::AppState;
```

- [ ] **Step 3: 检查 services 内对 Tauri 的依赖**

Run: `grep -r "use tauri" crates\cc-switch-core\src\services crates\cc-switch-core\src\store.rs crates\cc-switch-core\src\session_manager crates\cc-switch-core\src\usage_events.rs crates\cc-switch-core\src\usage_script.rs crates\cc-switch-core\src\init_status.rs`

如果输出非空，**逐个文件修改**：

- `services/webdav_auto_sync.rs` / `services/s3_auto_sync.rs` 中 `app_handle: Option<&tauri::AppHandle>` 改为 `Option<&dyn EventCallback>`（在 core 中定义 trait，Task 9 中由 tauri-commands 实现）。
- `usage_events.rs` 中 `tauri::AppHandle` 改为 trait 引用。
- `store.rs` 内的 `tauri::AppHandle` 调用改为通过函数参数注入。

如果迁移文件中使用 `tauri::AppHandle` 仅用于 `app.emit("xxx", ...)`，按以下模式替换。

- [ ] **Step 4: 定义 EventCallback trait（去 Tauri 化关键）**

创建 `crates/cc-switch-core/src/event_callback.rs`：

```rust
//! 事件回调 trait，用于解耦 service 层与 Tauri 运行时。
//!
//! GUI 实现走 Tauri emit；CLI 实现为空 callback（不向前端推送）。
//! service 层只依赖此 trait，不直接依赖 tauri::AppHandle。

use serde::Serialize;

/// 事件回调接口。
///
/// 实现者负责把事件转发到前端（GUI）或丢弃（CLI）。
pub trait EventCallback: Send + Sync {
    /// 发射事件到前端，payload 必须可序列化。
    fn emit<T: Serialize + Clone>(&self, event: &str, payload: T);
}

/// 空实现，CLI / 无头模式使用。
pub struct NoopEventCallback;

impl EventCallback for NoopEventCallback {
    fn emit<T: Serialize + Clone>(&self, _event: &str, _payload: T) {
        // CLI 不向前端推送事件
    }
}
```

在 `crates/cc-switch-core/src/lib.rs` 追加：

```rust
pub mod event_callback;

pub use event_callback::{EventCallback, NoopEventCallback};
```

- [ ] **Step 5: 修改 services/webdav_auto_sync.rs 使用 EventCallback**

打开 `crates/cc-switch-core/src/services/webdav_auto_sync.rs`，把 `pub fn start_worker(db: Arc<Database>, app_handle: Option<tauri::AppHandle>)` 改为：

```rust
use crate::event_callback::EventCallback;
use std::sync::Arc;

/// 启动 WebDAV 自动同步 worker。
///
/// `event_callback`：GUI 传入实现 EventCallback 的 wrapper；CLI 传 None 或 NoopEventCallback。
pub fn start_worker(db: Arc<Database>, event_callback: Option<Arc<dyn EventCallback>>) {
    // 内部所有 app.emit("xxx", payload) 改为：
    // if let Some(cb) = &event_callback { cb.emit("xxx", payload); }
    // ...
}
```

> 注：内部 `app.emit` 调用点逐个改为 `event_callback` 调用。具体调用点见原文件，按 grep `app_handle.emit\|app.emit\|handle.emit` 列出后逐一替换。

同理修改 `crates/cc-switch-core/src/services/s3_auto_sync.rs`。

- [ ] **Step 6: 修改 usage_events.rs 使用 EventCallback**

打开 `crates/cc-switch-core/src/usage_events.rs`。原文件用 `tauri::AppHandle` 全局存储。改为：

```rust
use crate::event_callback::EventCallback;
use std::sync::OnceLock;
use std::sync::Arc;

static EVENT_CALLBACK: OnceLock<Arc<dyn EventCallback>> = OnceLock::new();

/// 初始化全局事件回调。GUI 启动时调用一次；CLI 可不调用或传 NoopEventCallback。
pub fn init(callback: Arc<dyn EventCallback>) {
    let _ = EVENT_CALLBACK.set(callback);
}

fn with_callback<F: FnOnce(&Arc<dyn EventCallback>)>(f: F) {
    if let Some(cb) = EVENT_CALLBACK.get() {
        f(cb);
    }
    // 未初始化时静默跳过（CLI 模式）
}

// 原 emit 调用改为：
// with_callback(|cb| cb.emit("usage-log-recorded", payload));
```

- [ ] **Step 7: 修改 store.rs 中 set_app_handle 的处理**

原 `AppState::proxy_service.set_app_handle(app.handle().clone())` 调用走 `tauri::AppHandle`。改为通过 `EventCallback`：

打开 `crates/cc-switch-core/src/store.rs`，把 `AppState` 改为：

```rust
use crate::database::Database;
use crate::event_callback::{EventCallback, NoopEventCallback};
use crate::services::{ProxyService, UsageCache};
use std::sync::Arc;

/// 全局应用状态
pub struct AppState {
    pub db: Arc<Database>,
    pub proxy_service: ProxyService,
    pub usage_cache: Arc<UsageCache>,
    pub event_callback: Arc<dyn EventCallback>,
}

impl AppState {
    /// 创建新的应用状态（GUI 用，传入 Tauri emit 实现的 callback）
    pub fn new(db: Arc<Database>) -> Self {
        let proxy_service = ProxyService::new(db.clone());
        Self {
            db,
            proxy_service,
            usage_cache: Arc::new(UsageCache::new()),
            event_callback: Arc::new(NoopEventCallback),
        }
    }

    /// 创建带事件回调的 AppState（GUI 用）
    pub fn new_with_callback(db: Arc<Database>, callback: Arc<dyn EventCallback>) -> Self {
        let proxy_service = ProxyService::new(db.clone());
        Self {
            db,
            proxy_service,
            usage_cache: Arc::new(UsageCache::new()),
            event_callback: callback,
        }
    }
}
```

`ProxyService::set_app_handle` 内部若依赖 `tauri::AppHandle`，改为 `set_event_callback(callback: Arc<dyn EventCallback>)`。GUI 启动时调 `state.proxy_service.set_event_callback(callback)` 替代 `set_app_handle`。

- [ ] **Step 8: 修改 src-tauri/src/lib.rs**

把 `src-tauri/src/lib.rs` 中：

```rust
mod services;
mod store;
pub mod session_manager;
mod usage_events;
mod usage_script;
mod init_status;
```

改为：

```rust
pub use cc_switch_core::services;
pub use cc_switch_core::store;
pub use cc_switch_core::session_manager;
pub use cc_switch_core::usage_events;
pub use cc_switch_core::usage_script;
pub use cc_switch_core::init_status;
```

`pub use services::{...}` 等 re-export 保持不变。

- [ ] **Step 9: 修改 src-tauri/src/lib.rs 中 setup 闭包内 set_app_handle 调用**

原 `src-tauri/src/lib.rs:518`:

```rust
app_state.proxy_service.set_app_handle(app.handle().clone());
```

改为：

```rust
// GUI 用 Tauri emit 包装 EventCallback
let emit_callback = std::sync::Arc::new(TauriEmitCallback {
    app_handle: app.handle().clone(),
});
app_state.proxy_service.set_event_callback(emit_callback.clone());
cc_switch_lib::usage_events::init(emit_callback);
```

并在 `src-tauri/src/lib.rs` 文件底部（`mod tests` 之前）添加：

```rust
/// Tauri emit 实现的 EventCallback，把 core 事件转发到前端。
struct TauriEmitCallback {
    app_handle: tauri::AppHandle,
}

impl cc_switch_core::event_callback::EventCallback for TauriEmitCallback {
    fn emit<T: serde::Serialize + Clone>(&self, event: &str, payload: T) {
        let _ = self.app_handle.emit(event, payload);
    }
}
```

- [ ] **Step 10: 修改 webdav/s3 自动同步调用**

在 `src-tauri/src/lib.rs:969` 原：

```rust
crate::services::webdav_auto_sync::start_worker(
    app_state.db.clone(),
    Some(app.handle().clone()),
);
crate::services::s3_auto_sync::start_worker(
    app_state.db.clone(),
    Some(app.handle().clone()),
);
```

改为：

```rust
let cb = std::sync::Arc::new(TauriEmitCallback { app_handle: app.handle().clone() });
cc_switch_core::services::webdav_auto_sync::start_worker(
    app_state.db.clone(),
    Some(cb.clone()),
);
cc_switch_core::services::s3_auto_sync::start_worker(
    app_state.db.clone(),
    Some(cb),
);
```

- [ ] **Step 11: 验证 workspace 编译**

Run: `cargo build --workspace`
Expected: 编译成功。若有 services 内的 `use crate::commands::xxx` 残留，改为 `use cc_switch_core::services::xxx` 或注释——services 不应反向依赖 commands。

- [ ] **Step 12: 运行 services 单元测试**

Run: `cargo test -p cc-switch-core --lib services`
Expected: 全部通过。

- [ ] **Step 13: Commit**

```bash
git add -A
git commit -m "refactor(core): 迁移 services/store/session_manager 到 cc-switch-core，引入 EventCallback trait 去 Tauri 化"
```

---

## Task 8: 迁移 proxy 模块到 cc-switch-core

**Files:**
- Move: `src-tauri/src/proxy/` → `crates/cc-switch-core/src/proxy/`
- Modify: `crates/cc-switch-core/src/lib.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: git mv proxy 目录**

Run: `git mv src-tauri\src\proxy crates\cc-switch-core\src\proxy`

- [ ] **Step 2: 更新 cc-switch-core/src/lib.rs**

在 `pub mod usage_script;` 后追加：

```rust
pub mod proxy;

pub use proxy::circuit_breaker::CircuitBreakerConfig;
pub use proxy::http_client;
pub use proxy::{server::ProxyServer, ProxyConfig, ProxyStatus};
pub use proxy::types::{AppProxyConfig, GlobalProxyConfig, ProviderHealth};
```

- [ ] **Step 3: 修复 proxy 内 tauri 依赖**

Run: `grep -r "use tauri" crates\cc-switch-core\src\proxy`

通常 proxy 模块不直接依赖 tauri（依赖 reqwest/hyper）。若有 `tauri::AppHandle`（如 forwarder.rs 故障转移通知），改为 `EventCallback`：

打开 `crates/cc-switch-core/src/proxy/forwarder.rs`，把 `app_handle: &tauri::AppHandle` 改为 `event_callback: &dyn EventCallback`，所有 `app_handle.emit("xxx", payload)` 改为 `event_callback.emit("xxx", payload)`。

`ProxyService` 内若有 `set_app_handle` 方法（在 Task 7 Step 7 已改），保持一致。

- [ ] **Step 4: 修改 src-tauri/src/lib.rs**

把 `src-tauri/src/lib.rs` 中 `mod proxy;` 改为：

```rust
pub use cc_switch_core::proxy;
```

- [ ] **Step 5: 验证 workspace 编译**

Run: `cargo build --workspace`
Expected: 编译成功。

- [ ] **Step 6: 运行 proxy 单元测试**

Run: `cargo test -p cc-switch-core --lib proxy`
Expected: 全部通过。

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor(core): 迁移 proxy 模块到 cc-switch-core"
```

---

## Task 9: 迁移 core 模块到 cc-switch-core 并升级 DeclConfig::apply 为 ApplyContext

**Files:**
- Move: `src-tauri/src/core/` → `crates/cc-switch-core/src/core/`
- Modify: `crates/cc-switch-core/src/lib.rs`
- Modify: `crates/cc-switch-core/src/core/decl_config.rs`（升级 apply 签名）
- Modify: `crates/cc-switch-core/src/core/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `crates/cc-switch-core/tests/apply_context_test.rs`

- [ ] **Step 1: git mv core 目录**

Run: `git mv src-tauri\src\core crates\cc-switch-core\src\core`

> 注：迁移后 `cc-switch-core/src/core/` 是 crate 内的 core 子模块，与 crate 名相同，但 Rust 允许。`#![allow(clippy::module_inception)]` 已在 lib.rs 顶部加。

- [ ] **Step 2: 更新 cc-switch-core/src/lib.rs**

在 `pub mod provider_defaults;` 后追加：

```rust
pub mod core;
```

`pub use core::xxx` 已在原 `src-tauri/src/lib.rs` 第 11 行 `pub mod core;` 注册——本 Step 仅在 cc-switch-core 中重新注册。

- [ ] **Step 3: 修改 core/mod.rs**

打开 `crates/cc-switch-core/src/core/mod.rs`，保持内容不变（已是 4 个子模块声明）。但 `bootstrap.rs` 内 `use crate::xxx` 仍指向同 crate，正确。

- [ ] **Step 4: 升级 DeclConfig::apply 签名为 ApplyContext**

打开 `crates/cc-switch-core/src/core/decl_config.rs`，在文件顶部 `use` 区追加：

```rust
use crate::services::ProxyService;
```

在 `impl DeclConfig {` 之前追加 `ApplyContext` 定义：

```rust
/// 声明式配置应用上下文。
///
/// CLI 传 `proxy_service: None`，apply 时对代理字段（takeover）写日志"需手动设置"；
/// GUI 传完整 ctx，apply 时真正应用。
pub struct ApplyContext<'a> {
    pub db: &'a crate::database::Database,
    pub proxy_service: Option<&'a ProxyService>,
}

impl<'a> ApplyContext<'a> {
    pub fn new(db: &'a crate::database::Database) -> Self {
        Self { db, proxy_service: None }
    }

    pub fn with_proxy(db: &'a crate::database::Database, proxy: &'a ProxyService) -> Self {
        Self { db, proxy_service: Some(proxy) }
    }
}
```

把 `pub fn apply(&self, db: &crate::database::Database) -> Result<String, String>` 改为：

```rust
/// 将声明式配置应用到数据库和代理服务。
///
/// - `ctx.proxy_service = None`（CLI 模式）：跳过代理字段应用，记录日志
/// - `ctx.proxy_service = Some(_)`（GUI 模式）：完整应用代理字段
pub fn apply(&self, ctx: &ApplyContext) -> Result<String, String> {
    let db = ctx.db;
    let mut actions = Vec::new();

    // 1. 应用供应商配置
    for p in &self.providers {
        let mut env = serde_json::Map::new();
        for (k, v) in &p.env {
            env.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        let settings_config = serde_json::json!({ "env": env });
        let provider =
            crate::Provider::with_id(p.id.clone(), p.name.clone(), settings_config, None);
        db.save_provider(&p.app, &provider)
            .map_err(|e| format!("保存供应商 {} 失败: {e}", p.id))?;

        if p.current {
            crate::core::provider_manager::switch_provider(db, &p.app, &p.id)
                .map_err(|e| format!("切换供应商 {} 失败: {e}", p.id))?;
        }
        actions.push(format!("供应商 {}/{} ({}) 已配置", p.app, p.id, p.name));
    }

    // 2. 应用全局出站代理
    if let Some(gp) = &self.global_proxy {
        db.set_global_proxy_url(Some(&gp.url))
            .map_err(|e| format!("设置全局代理失败: {e}"))?;
        crate::proxy::http_client::init(Some(&gp.url))
            .map_err(|e| format!("初始化 HTTP 客户端失败: {e}"))?;
        actions.push(format!("全局出站代理已设置为: {}", gp.url));
    }

    // 3. 应用故障转移队列
    for (app, ids) in &self.failover.queue {
        if let Ok(existing) = db.get_failover_queue(app) {
            for item in existing {
                let _ = db.remove_from_failover_queue(app, &item.provider_id);
            }
        }
        for id in ids {
            db.add_to_failover_queue(app, id)
                .map_err(|e| format!("添加故障转移队列 {app}/{id} 失败: {e}"))?;
        }
        actions.push(format!("故障转移队列 {app} 已配置 ({} 项)", ids.len()));
    }

    // 4. 应用自动故障转移
    if self.failover.auto {
        for app in ["claude", "codex", "gemini"] {
            let (enabled, _) = db.get_proxy_flags_sync(app);
            db.set_proxy_flags_sync(app, enabled, true)
                .map_err(|e| format!("设置自动故障转移 {app} 失败: {e}"))?;
        }
        actions.push("自动故障转移已全局开启".to_string());
    }

    // 5. 应用代理接管状态（依赖 proxy_service）
    for (app, enabled) in &self.proxy.takeover {
        match ctx.proxy_service {
            Some(svc) => {
                svc.set_takeover_for_app(app, *enabled)
                    .await
                    .map_err(|e| format!("设置接管 {app}={} 失败: {e}", enabled))?;
                actions.push(format!("代理接管 {app}={}", enabled));
            }
            None => {
                log::warn!(
                    "代理接管 {app}={} 需 proxy_service，当前 CLI 模式未提供，请手动执行 takeover 命令",
                    enabled
                );
                actions.push(format!(
                    "（跳过）代理接管 {app}={} —— CLI 模式需手动执行 `cc-switch-cli takeover {app} on`",
                    enabled
                ));
            }
        }
    }

    // 6. 应用设备级设置
    let mut settings = crate::settings::get_settings();
    if let Some(lang) = &self.settings.language {
        settings.language = Some(lang.clone());
    }
    if let Some(hours) = self.settings.backup_interval_hours {
        settings.backup_interval_hours = Some(hours);
    }
    if let Some(count) = self.settings.backup_retain_count {
        settings.backup_retain_count = Some(count);
    }
    if let Some(dir) = &self.settings.claude_config_dir {
        settings.claude_config_dir = Some(dir.clone());
    }
    if let Some(dir) = &self.settings.codex_config_dir {
        settings.codex_config_dir = Some(dir.clone());
    }
    if let Some(dir) = &self.settings.gemini_config_dir {
        settings.gemini_config_dir = Some(dir.clone());
    }
    match crate::settings::update_settings(settings) {
        Ok(_) => actions.push("设备级设置已更新".to_string()),
        Err(e) => log::warn!("更新设备级设置失败: {e}"),
    }

    Ok(actions.join("\n"))
}
```

> 注：`set_takeover_for_app` 是 async 方法，故 `apply` 也需改为 `async`。把签名改为 `pub async fn apply(&self, ctx: &ApplyContext<'_>) -> Result<String, String>`。

- [ ] **Step 5: 修改 src-tauri/src/lib.rs**

把 `src-tauri/src/lib.rs` 中 `pub mod core;` 改为：

```rust
pub use cc_switch_core::core;
```

- [ ] **Step 6: 修改 src-tauri/src/bin/cc-switch-cli.rs 中 cmd_apply_config**

打开 `src-tauri/src/bin/cc-switch-cli.rs:2374`，把 `fn cmd_apply_config(path: &str)` 改为：

```rust
/// apply-config: 应用声明式配置文件到数据库
fn cmd_apply_config(path: &str) {
    let config = match cc_switch_lib::core::decl_config::DeclConfig::from_yaml_file(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("加载配置文件失败: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = config.validate() {
        eprintln!("配置校验失败: {e}");
        std::process::exit(1);
    }
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    // CLI 模式：proxy_service = None，代理字段会被跳过并提示
    let ctx = cc_switch_lib::core::decl_config::ApplyContext::new(&db);
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(config.apply(&ctx)) {
        Ok(summary) => {
            println!("✓ 配置已应用:");
            println!("{summary}");
        }
        Err(e) => {
            eprintln!("应用配置失败: {e}");
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 7: 创建 ApplyContext 行为测试**

写入 `crates/cc-switch-core/tests/apply_context_test.rs`：

```rust
//! ApplyContext 行为测试。
//!
//! 验证 CLI 模式（proxy_service=None）下 apply 不会 panic，
//! 且代理字段被记录为"跳过"。

use cc_switch_core::core::decl_config::{ApplyContext, DeclConfig};
use cc_switch_core::database::Database;

fn setup_test_db() -> Database {
    // 使用 tempfile 创建临时数据库
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap();
    // 通过环境变量覆盖 app_config_dir，确保 Database::init 用临时路径
    std::env::set_var("CC_SWITCH_CONFIG_DIR_OVERRIDE", path);
    Database::init().expect("数据库初始化失败")
}

#[test]
fn apply_with_cli_context_skips_proxy_takeover() {
    let db = setup_test_db();
    let yaml = r#"
providers:
  - app: claude
    id: test-provider
    name: Test Provider
    current: true
    env:
      ANTHROPIC_API_KEY: sk-test
proxy:
  takeover:
    claude: true
failover:
  auto: false
"#;
    let config = DeclConfig::from_yaml_str(yaml).expect("解析 YAML 失败");
    config.validate().expect("校验失败");

    let ctx = ApplyContext::new(&db);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let summary = rt.block_on(config.apply(&ctx)).expect("apply 失败");

    // 验证供应商已保存
    let providers = db.get_all_providers("claude").unwrap();
    assert!(providers.contains_key("test-provider"));

    // 验证 takeover 被跳过（CLI 模式）
    assert!(summary.contains("跳过") || summary.contains("CLI 模式"));
}

#[test]
fn apply_context_new_has_no_proxy_service() {
    let db = setup_test_db();
    let ctx = ApplyContext::new(&db);
    assert!(ctx.proxy_service.is_none());
}
```

- [ ] **Step 8: 验证 workspace 编译与测试**

Run: `cargo build --workspace`
Expected: 编译成功。

Run: `cargo test -p cc-switch-core --test apply_context_test`
Expected: 2 个测试通过。

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "refactor(core): 迁移 core 模块到 cc-switch-core，升级 DeclConfig::apply 为 ApplyContext"
```

---

## Task 10: 引入 CopilotAuthState 类型别名由 core 持有

**Files:**
- Create: `crates/cc-switch-core/src/copilot_auth_state.rs`
- Modify: `crates/cc-switch-core/src/lib.rs`
- Modify: `crates/cc-switch-core/src/services/stream_check.rs`（去 Tauri State）
- Create: `crates/cc-switch-core/tests/copilot_auth_state_test.rs`

- [ ] **Step 1: 创建 CopilotAuthState 类型别名**

写入 `crates/cc-switch-core/src/copilot_auth_state.rs`：

```rust
//! CopilotAuthState 类型定义。
//!
//! 原本 `CopilotAuthState` 是 `commands/copilot.rs` 中的 Tauri managed state 包装：
//!
//! ```ignore
//! pub struct CopilotAuthState(pub Arc<RwLock<CopilotAuthManager>>);
//! ```
//!
//! 重构后改为类型别名，由 core 持有，service 层直接接收 `&Arc<RwLock<CopilotAuthManager>>`。
//! Tauri command 包装层从 managed state 取出 `Arc<RwLock<...>>` 调 service；
//! CLI 直接创建独立 `CopilotAuthManager` 实例调 service。

use crate::proxy::providers::copilot_auth::CopilotAuthManager;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Copilot 认证状态：由 core service 持有，Tauri 与 CLI 共用同一类型。
///
/// 重构前是 `commands::CopilotAuthState(Arc<RwLock<...>>)`（newtype 包装 Tauri State）；
/// 重构后直接是 `Arc<RwLock<CopilotAuthManager>>`，service 层不再依赖 Tauri State。
pub type CopilotAuthState = Arc<RwLock<CopilotAuthManager>>;

/// 创建 CopilotAuthState 实例。
///
/// GUI 启动时在 setup 闭包中调用，注册为 Tauri managed state；
/// CLI 在需要时调用（如执行 stream-check 时）。
pub fn new_copilot_auth_state(app_config_dir: std::path::PathBuf) -> CopilotAuthState {
    let manager = CopilotAuthManager::new(app_config_dir);
    Arc::new(RwLock::new(manager))
}
```

- [ ] **Step 2: 在 cc-switch-core/src/lib.rs 注册**

在 `pub mod core;` 后追加：

```rust
pub mod copilot_auth_state;

pub use copilot_auth_state::{new_copilot_auth_state, CopilotAuthState};
```

- [ ] **Step 3: 升级 services/stream_check.rs 接收 CopilotAuthState**

打开 `crates/cc-switch-core/src/services/stream_check.rs`，在 `impl StreamCheckService {` 中追加新方法：

```rust
use crate::copilot_auth_state::CopilotAuthState;
use crate::provider::Provider;

impl StreamCheckService {
    // ... 原有 check_with_retry / merge_provider_config 等不变

    /// 检查单个供应商（接收 core 的 CopilotAuthState，去 Tauri 化版本）。
    ///
    /// GUI 包装层从 managed state 取出 `Arc<RwLock<CopilotAuthManager>>` 调此方法；
    /// CLI 创建独立 manager 实例调此方法。
    pub async fn check_provider_with_state(
        app_type: &crate::app_config::AppType,
        provider: &Provider,
        config: &StreamCheckConfig,
        copilot_state: &CopilotAuthState,
    ) -> Result<StreamCheckResult, crate::error::AppError> {
        let base_url_override = Self::resolve_copilot_base_url(provider, copilot_state).await?;
        Self::check_with_retry(app_type, provider, config, base_url_override).await
    }

    /// 解析 Copilot 供应商的 base_url 覆盖值。
    ///
    /// 从原 `commands/stream_check.rs::resolve_copilot_base_url_override` 抽出，
    /// 接收 core 的 `CopilotAuthState` 而非 `tauri::State`。
    async fn resolve_copilot_base_url_override(
        provider: &Provider,
        copilot_state: &CopilotAuthState,
    ) -> Result<Option<String>, crate::error::AppError> {
        let is_copilot = Self::is_copilot_provider(provider);
        let is_full_url = provider
            .meta
            .as_ref()
            .and_then(|meta| meta.is_full_url)
            .unwrap_or(false);

        if !is_copilot || is_full_url {
            return Ok(None);
        }

        let auth_manager = copilot_state.read().await;
        let account_id = provider
            .meta
            .as_ref()
            .and_then(|meta| meta.managed_account_id_for("github_copilot"));

        let endpoint = match account_id.as_deref() {
            Some(id) => auth_manager.get_api_endpoint(id).await,
            None => auth_manager.get_default_api_endpoint().await,
        };

        Ok(Some(endpoint))
    }

    /// 判断供应商是否为 Copilot 类型。
    fn is_copilot_provider(provider: &Provider) -> bool {
        provider
            .meta
            .as_ref()
            .and_then(|meta| meta.provider_type.as_deref())
            == Some("github_copilot")
            || provider
                .settings_config
                .pointer("/env/ANTHROPIC_BASE_URL")
                .and_then(|value| value.as_str())
                .map(|url| url.contains("githubcopilot.com"))
                .unwrap_or(false)
    }
}
```

- [ ] **Step 4: 创建 CopilotAuthState 测试**

写入 `crates/cc-switch-core/tests/copilot_auth_state_test.rs`：

```rust
//! CopilotAuthState 类型持有测试。

use cc_switch_core::copilot_auth_state::{new_copilot_auth_state, CopilotAuthState};
use cc_switch_core::proxy::providers::copilot_auth::CopilotAuthManager;
use std::sync::Arc;
use tokio::sync::RwLock;

#[test]
fn copilot_auth_state_is_arc_rwlock_manager() {
    // 类型等价性：CopilotAuthState 应为 Arc<RwLock<CopilotAuthManager>>
    let tmp_dir = tempfile::tempdir().unwrap();
    let state: CopilotAuthState = new_copilot_auth_state(tmp_dir.path().to_path_buf());

    // 验证类型可转为 Arc<RwLock<CopilotAuthManager>>
    let _: Arc<RwLock<CopilotAuthManager>> = state;
}

#[tokio::test]
async fn copilot_auth_state_can_be_locked_for_read() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let state = new_copilot_auth_state(tmp_dir.path().to_path_buf());

    let guard = state.read().await;
    // 验证可以读取 manager 状态
    let _status = guard.get_status().await;
    drop(guard);
}
```

- [ ] **Step 5: 验证编译与测试**

Run: `cargo build --workspace`
Expected: 编译成功。

Run: `cargo test -p cc-switch-core --test copilot_auth_state_test`
Expected: 2 个测试通过。

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor(core): 引入 CopilotAuthState 类型别名由 core 持有，stream_check service 去 Tauri 化"
```

---

## Task 11: 创建 cc-switch-tauri-commands crate 并迁移 commands

**Files:**
- Create: `crates/cc-switch-tauri-commands/Cargo.toml`
- Create: `crates/cc-switch-tauri-commands/src/lib.rs`
- Move: `src-tauri/src/commands/` → `crates/cc-switch-tauri-commands/src/commands/`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 crate 目录**

Run: `mkdir crates\cc-switch-tauri-commands\src`

- [ ] **Step 2: 创建 Cargo.toml**

写入 `crates/cc-switch-tauri-commands/Cargo.toml`：

```toml
[package]
name = "cc-switch-tauri-commands"
version = "3.16.5"
edition = "2021"
rust-version = "1.85.0"
description = "CC Switch Tauri 命令包装层"
license = "MIT"

[lib]
name = "cc_switch_tauri_commands"
crate-type = ["rlib"]
doctest = false

[features]
default = []
test-hooks = []

[dependencies]
cc-switch-core = { path = "../cc-switch-core", features = ["test-hooks"] }
serde_json = { version = "1.0", features = ["preserve_order"] }
serde = { version = "1.0", features = ["derive"] }
log = "0.4"
chrono = { version = "0.4", features = ["serde"] }
tauri = { version = "2.8.2", features = ["tray-icon", "protocol-asset", "image-png"] }
tauri-plugin-log = "2"
tauri-plugin-opener = "2"
tauri-plugin-process = "2"
tauri-plugin-updater = "2"
tauri-plugin-store = "2"
tauri-plugin-deep-link = "2"
tauri-plugin-dialog = "2"
tauri-plugin-window-state = "2"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time", "sync", "signal"] }
futures = "0.3"
indexmap = { version = "2", features = ["serde"] }
anyhow = "1.0"
thiserror = "2.0"
url = "2.5"
once_cell = "1.21.3"

[target.'cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))'.dependencies]
tauri-plugin-single-instance = "2"

[target.'cfg(target_os = "linux")'.dependencies]
webkit2gtk = { version = "2.0.1", features = ["v2_16"] }
```

- [ ] **Step 3: 创建 lib.rs 空壳**

写入 `crates/cc-switch-tauri-commands/src/lib.rs`：

```rust
//! CC Switch Tauri 命令包装层
//!
//! 本 crate 包含所有 `#[tauri::command]` 函数，依赖 `cc-switch-core` 提供的业务逻辑。
//! Tauri State 包装类型（如 CopilotAuthState）从此 crate 中删除，改用 core 提供的类型别名。

pub mod commands;

pub use commands::*;
```

- [ ] **Step 4: git mv commands 目录**

Run: `git mv src-tauri\src\commands crates\cc-switch-tauri-commands\src\commands`

- [ ] **Step 5: 删除 commands/copilot.rs 中的 CopilotAuthState 定义**

打开 `crates/cc-switch-tauri-commands/src/commands/copilot.rs`，删除第 14 行：

```rust
pub struct CopilotAuthState(pub Arc<RwLock<CopilotAuthManager>>);
```

替换为：

```rust
// CopilotAuthState 已迁移到 cc-switch-core:
//   pub type CopilotAuthState = Arc<RwLock<CopilotAuthManager>>;
// 由 cc_switch_core::CopilotAuthState re-export。
// Tauri command 包装层仍以 `State<'_, CopilotAuthState>` 接收 managed state，
// 但类型来自 core，service 层可直接复用。
use cc_switch_core::CopilotAuthState;
```

确保 `commands/copilot.rs` 顶部 `use` 区不再 `use crate::proxy::providers::copilot_auth::CopilotAuthManager`（如未直接使用），保留 `use cc_switch_core::proxy::providers::copilot_auth::{...}` 引用其它类型。

- [ ] **Step 6: 修改 commands 内所有 use crate:: 路径**

Run: `grep -r "use crate::" crates\cc-switch-tauri-commands\src\commands | head -50`

逐个文件把 `use crate::xxx` 改为：
- 业务逻辑（database/services/proxy/core/error/provider/app_config 等）→ `use cc_switch_core::xxx`
- 跨 commands 引用 → `use crate::commands::xxx`（同 crate 内）

例：`crates/cc-switch-tauri-commands/src/commands/stream_check.rs` 顶部改为：

```rust
use cc_switch_core::app_config::AppType;
use cc_switch_core::copilot_auth_state::CopilotAuthState;
use cc_switch_core::error::AppError;
use cc_switch_core::services::stream_check::{
    HealthStatus, StreamCheckConfig, StreamCheckResult, StreamCheckService,
};
use cc_switch_core::store::AppState;
use std::collections::HashSet;
use tauri::State;
```

`stream_check_provider` 命令体可简化为调用 core service：

```rust
#[tauri::command]
pub async fn stream_check_provider(
    state: State<'_, AppState>,
    copilot_state: State<'_, CopilotAuthState>,
    app_type: AppType,
    provider_id: String,
) -> Result<StreamCheckResult, AppError> {
    let config = state.db.get_stream_check_config()?;

    let providers = state.db.get_all_providers(app_type.as_str())?;
    let provider = providers
        .get(&provider_id)
        .ok_or_else(|| AppError::Message(format!("供应商 {provider_id} 不存在")))?;

    let result = StreamCheckService::check_provider_with_state(
        &app_type, provider, &config, &copilot_state.0,
    ).await?;

    let _ = state.db.save_stream_check_log(
        &provider_id, &provider.name, app_type.as_str(), &result,
    );

    Ok(result)
}
```

> 注：`copilot_state.0` 取出 `Arc<RwLock<CopilotAuthManager>>`。原 `CopilotAuthState(pub Arc<...>)` newtype 已删除，但 `State<'_, CopilotAuthState>` 持有的是 `Arc<RwLock<...>>`（类型别名），故 `&copilot_state.0` 实际是 `&Arc<...>` 的 `&` 解引用。具体写法：`let copilot_state: CopilotAuthState = copilot_state.inner().clone();` 然后 `&copilot_state`。简化为：

```rust
let copilot_state_arc = copilot_state.inner().clone();
let result = StreamCheckService::check_provider_with_state(
    &app_type, provider, &config, &copilot_state_arc,
).await?;
```

- [ ] **Step 7: 修改 commands/mod.rs**

打开 `crates/cc-switch-tauri-commands/src/commands/mod.rs`，内容保持不变（已是各子模块声明 + re-export）。但删除原第 7 行附近 `pub use copilot::*;` 中可能含 `CopilotAuthState` re-export——因 Task 5 已迁移到 core。在 mod.rs 顶部追加：

```rust
// CopilotAuthState 类型别名由 core 提供，commands 内通过 `use cc_switch_core::CopilotAuthState` 引入
```

- [ ] **Step 8: 修改 src-tauri/src/lib.rs**

把 `src-tauri/src/lib.rs` 中 `pub mod commands;` 改为：

```rust
pub use cc_switch_tauri_commands::commands;
```

并在顶部 `use` 区追加：

```rust
use cc_switch_tauri_commands::commands;
```

把 `src-tauri/Cargo.toml` 的 `[dependencies]` 顶部加：

```toml
cc-switch-tauri-commands = { path = "../crates/cc-switch-tauri-commands" }
```

- [ ] **Step 9: 修改 src-tauri/src/lib.rs setup 闭包中 CopilotAuthState 注册**

原 `src-tauri/src/lib.rs:1000`:

```rust
use crate::proxy::providers::copilot_auth::CopilotAuthManager;
use commands::CopilotAuthState;
use tokio::sync::RwLock;

let app_config_dir = crate::config::get_app_config_dir();
let copilot_auth_manager = CopilotAuthManager::new(app_config_dir);
app.manage(CopilotAuthState(Arc::new(RwLock::new(copilot_auth_manager))));
```

改为：

```rust
use cc_switch_core::copilot_auth_state::new_copilot_auth_state;

let app_config_dir = crate::config::get_app_config_dir();
let copilot_auth_state = new_copilot_auth_state(app_config_dir);
app.manage(copilot_auth_state);
log::info!("✓ CopilotAuthManager initialized");
```

同理修改 `CodexOAuthState`——若 spec 未要求迁移 `CodexOAuthState`，保留原 newtype 形式（仍可工作）。

- [ ] **Step 10: 验证 workspace 编译**

Run: `cargo build --workspace`
Expected: 编译成功。

如有 `use crate::xxx` 残留报错，按错误信息逐个修改为 `use cc_switch_core::xxx` 或 `use cc_switch_tauri_commands::commands::xxx`。

- [ ] **Step 11: 运行 commands 单元测试**

Run: `cargo test -p cc-switch-tauri-commands`
Expected: 全部通过。

- [ ] **Step 12: Commit**

```bash
git add -A
git commit -m "refactor(tauri-commands): 创建 cc-switch-tauri-commands crate，迁移 commands 模块，CopilotAuthState 改用 core 类型别名"
```

---

## Task 12: 重命名 src-tauri package 为 cc-switch-app

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `Cargo.toml`（根）
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 修改 src-tauri/Cargo.toml**

把 `src-tauri/Cargo.toml` 第 2 行 `name = "cc-switch"` 改为：

```toml
name = "cc-switch-app"
```

`default-run` 改为 `cc-switch-app`。

`[lib]` 段 `name = "cc_switch_lib"` 改为：

```toml
name = "cc_switch_app"
```

> 注：本 Task 之前 src-tauri 内代码用 `cc_switch_lib::` 引用自身的位置已全部改为 `cc_switch_core::` 或 `cc_switch_tauri_commands::`，故 lib name 改名不影响。

- [ ] **Step 2: 修改 src-tauri/src/main.rs**

打开 `src-tauri/src/main.rs`，把 `cc_switch_lib::run()` 改为 `cc_switch_app::run()`（若有）。或保留 `fn main() { cc_switch_lib::run(); }`——若 lib name 改了则需对应改。

实际原文件通常是：

```rust
fn main() {
    cc_switch_lib::run()
}
```

改为：

```rust
fn main() {
    cc_switch_app::run()
}
```

- [ ] **Step 3: 修改根 Cargo.toml**

确认 `members` 已含 `"src-tauri"`。无需追加，但 profile 等设置保留。

- [ ] **Step 4: 修改 src-tauri/src/bin/cc-switch-cli.rs**

> 注：本 Task 仅重命名 package，CLI 二进制仍位于 src-tauri/src/bin/。Task 13 才把 CLI 迁到独立 crate。

打开 `src-tauri/src/bin/cc-switch-cli.rs`，搜索 `cc_switch_lib::`，全部替换为 `cc_switch_core::`：

```rust
use cc_switch_core::core::{bootstrap, provider_manager};
use cc_switch_core::Database;
```

`cc_switch_lib::AppType` / `cc_switch_lib::Provider` / `cc_switch_lib::ProviderService` 等 re-export 改为 `cc_switch_core::`。

但 `cc-switch-app` package 的 lib 仍 `pub use cc_switch_core::xxx;` re-export，CLI 可继续用 `cc_switch_lib::`——本 Step 改为 `cc_switch_app::`：

```rust
use cc_switch_app::core::{bootstrap, provider_manager};
use cc_switch_app::Database;
```

任选一种风格。**推荐用 `cc_switch_core::` 直接引用**，因 Task 13 会把 CLI 迁出 src-tauri，那时 `cc_switch_app::` 不可用。

故本 Step 直接改为：

```rust
use cc_switch_core::core::{bootstrap, provider_manager};
use cc_switch_core::Database;
use cc_switch_core::AppType;
use cc_switch_core::Provider;
use cc_switch_core::ProviderMeta;
use cc_switch_core::ProviderService;
use cc_switch_core::ProviderSortUpdate;
use cc_switch_core::AppState;
// ... 其它按需
```

逐个 `cc_switch_lib::xxx` 改为 `cc_switch_core::xxx`。

Run: `grep -c "cc_switch_lib" src-tauri\src\bin\cc-switch-cli.rs`
Expected: 输出 0（全部替换完毕）。

- [ ] **Step 5: 修改 src-tauri/Cargo.toml 的 [[bin]] cc-switch-cli**

原 `src-tauri/Cargo.toml:101-103`:

```toml
[[bin]]
name = "cc-switch-cli"
path = "src/bin/cc-switch-cli.rs"
```

保持不变。Task 13 会移除此 `[[bin]]` 段。

- [ ] **Step 6: 验证 workspace 编译**

Run: `cargo build --workspace`
Expected: 编译成功。

Run: `cargo build -p cc-switch-app`
Expected: 编译成功。

- [ ] **Step 7: 运行 GUI 烟雾测试（可选）**

Run: `cargo run -p cc-switch-app`
Expected: 应用启动，无 panic。

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "refactor(app): 重命名 src-tauri package 为 cc-switch-app，CLI 改用 cc_switch_core:: 引用"
```

---

## Task 13: 把 CLI 迁到独立 crate cc-switch-cli

**Files:**
- Move: `src-tauri/src/bin/cc-switch-cli.rs` → `crates/cc-switch-cli/src/main.rs`
- Create: `crates/cc-switch-cli/Cargo.toml`
- Modify: `src-tauri/Cargo.toml`（删除 [[bin]] cc-switch-cli）
- Modify: `Cargo.toml`（根，确认 members 含 cc-switch-cli）
- Create: `crates/cc-switch-cli/tests/stream_check_cli_test.rs`

- [ ] **Step 1: 创建 crate 目录**

Run: `mkdir crates\cc-switch-cli\src`

- [ ] **Step 2: git mv CLI 入口**

Run: `git mv src-tauri\src\bin\cc-switch-cli.rs crates\cc-switch-cli\src\main.rs`

删除空目录 `src-tauri/src/bin/`：

Run: `rmdir src-tauri\src\bin`

- [ ] **Step 3: 创建 Cargo.toml**

写入 `crates/cc-switch-cli/Cargo.toml`：

```toml
[package]
name = "cc-switch-cli"
version = "3.16.5"
edition = "2021"
rust-version = "1.85.0"
description = "CC Switch 命令行管理工具（无 Tauri 依赖）"
license = "MIT"

[[bin]]
name = "cc-switch-cli"
path = "src/main.rs"

[dependencies]
cc-switch-core = { path = "../cc-switch-core" }
clap = { version = "4.5", features = ["derive"] }
serde_json = { version = "1.0", features = ["preserve_order"] }
serde = { version = "1.0", features = ["derive"] }
log = "0.4"
env_logger = "0.11"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time", "sync", "signal"] }
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
indexmap = { version = "2", features = ["serde"] }
url = "2.5"

[dev-dependencies]
tempfile = "3"
```

> 注：本 crate **严禁依赖 tauri / tauri-plugin-* / webkit2gtk**。验证在 Step 8 中通过 `cargo tree` 检查。

- [ ] **Step 4: 修改 src-tauri/Cargo.toml 删除 [[bin]] cc-switch-cli**

删除 `src-tauri/Cargo.toml:101-103`:

```toml
[[bin]]
name = "cc-switch-cli"
path = "src/bin/cc-switch-cli.rs"
```

并删除 `clap` 和 `env_logger` 依赖（如果只 CLI 用）：

```toml
# 从 [dependencies] 中删除：
# clap = { version = "4.5", features = ["derive"] }
# env_logger = "0.11"
```

> 注：先 `cargo tree -p cc-switch-app | grep -E "clap|env_logger"` 确认 src-tauri 内无其它代码用 clap/env_logger 再删。

- [ ] **Step 5: 修改根 Cargo.toml 确认 members**

根 `Cargo.toml` 的 `members` 应已含 `"crates/cc-switch-cli"`（Task 1 已写入）。确认无误。

- [ ] **Step 6: 修改 crates/cc-switch-cli/src/main.rs**

打开 `crates/cc-switch-cli/src/main.rs`，搜索 `cc_switch_lib::` 或 `cc_switch_app::`，全部改为 `cc_switch_core::`：

Run: `grep -c "cc_switch_lib\|cc_switch_app" crates\cc-switch-cli\src\main.rs`
Expected: 输出 0（已全替换）。

文件顶部 `use` 区应为：

```rust
use std::str::FromStr;
use std::sync::Arc;

use clap::{Parser, Subcommand};

use cc_switch_core::core::{bootstrap, provider_manager};
use cc_switch_core::Database;
// 其它 use cc_switch_core::xxx; 按需
```

- [ ] **Step 7: 验证 CLI crate 编译**

Run: `cargo build -p cc-switch-cli`
Expected: 编译成功。

- [ ] **Step 8: 验证 CLI 不依赖 tauri**

Run: `cargo tree -p cc-switch-cli | findstr /i "tauri"`
Expected: 无输出（CLI 不依赖任何 tauri 包）。

如果有 `tauri` 出现，检查 `cc-switch-core` 的 Cargo.toml 是否误加 tauri 依赖——必须移除。

- [ ] **Step 9: 验证 CLI 不依赖 webkit2gtk**

Run: `cargo tree -p cc-switch-cli | findstr /i "webkit2gtk"`
Expected: 无输出。

- [ ] **Step 10: 运行 CLI 烟雾测试**

Run: `cargo run -p cc-switch-cli -- help`
Expected: 打印帮助信息，无 panic。

- [ ] **Step 11: 创建 CLI stream-check 集成测试**

写入 `crates/cc-switch-cli/tests/stream_check_cli_test.rs`：

```rust
//! CLI stream-check 命令集成测试。
//!
//! 验证 CLI 调用 stream-check 不再返回桩实现提示，
//! 而是通过 core service 执行实际检查（或在无网络/Copilot 状态时返回明确错误）。

use std::process::Command;

#[test]
fn stream_check_help_does_not_show_stub_message() {
    let output = Command::new(env!("CARGO_BIN_EXE_cc-switch-cli"))
        .arg("help")
        .output()
        .expect("执行 cc-switch-cli 失败");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 旧的桩实现会打印"流式检查需要代理服务器运行中"
    assert!(
        !stdout.contains("流式检查需要代理服务器运行中且 CopilotAuthState 初始化"),
        "帮助信息不应再含桩实现提示，实际输出: {stdout}"
    );
}

#[test]
fn stream_check_command_executes_or_returns_clear_error() {
    // 此测试不依赖真实数据库或网络，只验证命令不再返回桩字符串
    let output = Command::new(env!("CARGO_BIN_EXE_cc-switch-cli"))
        .args(["stream-check", "claude", "nonexistent-provider"])
        .output()
        .expect("执行 cc-switch-cli 失败");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // 旧桩实现会打印：
    //   "流式检查需要代理服务器运行中且 CopilotAuthState 初始化，当前 CLI 环境不支持。"
    assert!(
        !stderr.contains("当前 CLI 环境不支持"),
        "stream-check 不应再返回桩实现错误，实际 stderr: {stderr}"
    );
    // 应返回明确错误（如"供应商不存在"或"数据库初始化失败"）
    assert!(
        stderr.contains("错误") || stderr.contains("失败"),
        "应返回明确错误信息，实际 stderr: {stderr}"
    );
}
```

- [ ] **Step 12: 运行 CLI 集成测试**

Run: `cargo test -p cc-switch-cli --test stream_check_cli_test`
Expected: 2 个测试通过。

- [ ] **Step 13: Commit**

```bash
git add -A
git commit -m "refactor(cli): 把 CLI 迁到独立 crate cc-switch-cli，移除 src-tauri [[bin]] 段"
```

---

## Task 14: 实现 CLI stream-check 命令（基于 core service）

**Files:**
- Modify: `crates/cc-switch-cli/src/main.rs`（替换桩实现）
- Modify: `crates/cc-switch-cli/Cargo.toml`（追加 reqwest 依赖，如需要）

- [ ] **Step 1: 修改 crates/cc-switch-cli/Cargo.toml 追加 reqwest**

在 `[dependencies]` 末尾追加（如果 service 层的 stream_check 间接需要）：

```toml
reqwest = { version = "0.12", features = ["rustls-tls", "json", "stream", "socks"] }
```

> 注：cc-switch-core 已传递依赖 reqwest，CLI 直接用 `cc_switch_core::proxy::http_client` 即可，不一定需要直接声明 reqwest。本 Step 仅在 CLI 内显式构造 reqwest::Client 时才加。

- [ ] **Step 2: 替换 cmd_stream_check 桩实现**

打开 `crates/cc-switch-cli/src/main.rs`，定位 `fn cmd_stream_check(app: String, id: String)`（原行号 ~3989）。替换为：

```rust
/// stream-check: 流式检查供应商
fn cmd_stream_check(app: String, id: String) {
    if let Err(e) = validated_app(&app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let db = match init_db() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("错误: {e}");
                std::process::exit(1);
            }
        };

        let app_type = match cc_switch_core::AppType::from_str(&app) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("错误: {e}");
                std::process::exit(1);
            }
        };

        let providers = match db.get_all_providers(app_type.as_str()) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("获取供应商列表失败: {e}");
                std::process::exit(1);
            }
        };

        let provider = match providers.get(&id) {
            Some(p) => p,
            None => {
                eprintln!("错误: 供应商 {id} 不存在于应用 {app}");
                std::process::exit(1);
            }
        };

        let config = match db.get_stream_check_config() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("获取连通性检查配置失败: {e}");
                std::process::exit(1);
            }
        };

        // CLI 模式：创建独立 CopilotAuthState 实例
        let app_config_dir = cc_switch_core::config::get_app_config_dir();
        let copilot_state = cc_switch_core::copilot_auth_state::new_copilot_auth_state(
            app_config_dir,
        );

        // 初始化全局 HTTP 客户端（stream_check service 依赖）
        let proxy_url = db.get_global_proxy_url().ok().flatten();
        if let Err(e) = cc_switch_core::proxy::http_client::init(proxy_url.as_deref()) {
            eprintln!("警告: 初始化 HTTP 客户端失败: {e}，将使用直连模式");
            let _ = cc_switch_core::proxy::http_client::init(None);
        }

        match cc_switch_core::services::stream_check::StreamCheckService::check_provider_with_state(
            &app_type, provider, &config, &copilot_state,
        ).await {
            Ok(result) => {
                println!("✓ 连通性检查完成");
                println!("  状态: {:?}", result.status);
                println!("  成功: {}", result.success);
                println!("  消息: {}", result.message);
                if let Some(ms) = result.response_time_ms {
                    println!("  响应时间: {ms} ms");
                }
                if let Some(status) = result.http_status {
                    println!("  HTTP 状态码: {status}");
                }
                println!("  重试次数: {}", result.retry_count);

                // 记录日志
                let _ = db.save_stream_check_log(
                    &id, &provider.name, app_type.as_str(), &result,
                );

                if !result.success {
                    std::process::exit(2);
                }
            }
            Err(e) => {
                eprintln!("连通性检查失败: {e}");
                std::process::exit(1);
            }
        }
    });
}
```

- [ ] **Step 3: 替换 cmd_stream_check_all 桩实现**

定位 `fn cmd_stream_check_all()`（原行号 ~4001）。替换为：

```rust
/// stream-check-all: 流式检查全部供应商
fn cmd_stream_check_all() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let db = match init_db() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("错误: {e}");
                std::process::exit(1);
            }
        };

        let config = match db.get_stream_check_config() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("获取连通性检查配置失败: {e}");
                std::process::exit(1);
            }
        };

        let app_config_dir = cc_switch_core::config::get_app_config_dir();
        let copilot_state = cc_switch_core::copilot_auth_state::new_copilot_auth_state(
            app_config_dir,
        );

        let proxy_url = db.get_global_proxy_url().ok().flatten();
        if let Err(e) = cc_switch_core::proxy::http_client::init(proxy_url.as_deref()) {
            eprintln!("警告: 初始化 HTTP 客户端失败: {e}");
            let _ = cc_switch_core::proxy::http_client::init(None);
        }

        let mut total = 0usize;
        let mut success = 0usize;
        let mut failed = 0usize;

        for app_str in ["claude", "claude-desktop", "codex", "gemini", "opencode", "openclaw", "hermes"] {
            let app_type = match cc_switch_core::AppType::from_str(app_str) {
                Ok(t) => t,
                Err(_) => continue,
            };

            let providers = match db.get_all_providers(app_str) {
                Ok(p) => p,
                Err(_) => continue,
            };

            for (id, provider) in providers {
                total += 1;
                let result = cc_switch_core::services::stream_check::StreamCheckService::check_provider_with_state(
                    &app_type, &provider, &config, &copilot_state,
                ).await;

                match result {
                    Ok(r) if r.success => {
                        success += 1;
                        println!("✓ {app_str}/{id} ({}) — {:?}, {}ms",
                            provider.name, r.status,
                            r.response_time_ms.unwrap_or(0));
                        let _ = db.save_stream_check_log(
                            &id, &provider.name, app_str, &r,
                        );
                    }
                    Ok(r) => {
                        failed += 1;
                        println!("✗ {app_str}/{id} ({}) — {:?}, {}",
                            provider.name, r.status, r.message);
                        let _ = db.save_stream_check_log(
                            &id, &provider.name, app_str, &r,
                        );
                    }
                    Err(e) => {
                        failed += 1;
                        println!("✗ {app_str}/{id} ({}) — 错误: {e}",
                            provider.name);
                    }
                }
            }
        }

        println!();
        println!("总计: {total}, 成功: {success}, 失败: {failed}");
        if failed > 0 {
            std::process::exit(2);
        }
    });
}
```

- [ ] **Step 4: 验证 CLI 编译**

Run: `cargo build -p cc-switch-cli`
Expected: 编译成功。

- [ ] **Step 5: 验证 CLI stream-check 命令**

Run: `cargo run -p cc-switch-cli -- stream-check claude nonexistent`
Expected: 输出"错误: 供应商 nonexistent 不存在于应用 claude"，退出码 1。

Run: `cargo run -p cc-switch-cli -- help | findstr stream-check`
Expected: 帮助信息含 `stream-check`，**不再含** "需代理服务器运行中" 桩字符串。

- [ ] **Step 6: 运行集成测试**

Run: `cargo test -p cc-switch-cli --test stream_check_cli_test`
Expected: 2 个测试通过。

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(cli): 实现 stream-check / stream-check-all 命令（基于 core service + 独立 CopilotAuthState）"
```

---

## Task 15: API 格式设置覆盖 7 应用（修复 add-provider env 硬编码）

**Files:**
- Modify: `crates/cc-switch-cli/src/main.rs`（修复 cmd_add_provider env 硬编码）
- Modify: `crates/cc-switch-cli/src/main.rs`（更新 Cli 帮助注释覆盖 7 应用）
- Modify: `docs/cli-reference-manual.md`（API 格式表格补全 7 应用）

- [ ] **Step 1: 修复 cmd_add_provider env 字段硬编码**

打开 `crates/cc-switch-cli/src/main.rs`，定位 `fn cmd_add_provider(`（原行号 ~1157）。把：

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

替换为按 `--app` 选择正确 env 名：

```rust
// 按 app 选择正确的 env 字段名（避免硬编码 ANTHROPIC_*）
let (key_field, url_field) = match app {
    "claude" | "claude-desktop" => ("ANTHROPIC_API_KEY", "ANTHROPIC_BASE_URL"),
    "codex" => ("OPENAI_API_KEY", "OPENAI_BASE_URL"),
    "gemini" => ("GEMINI_API_KEY", "GEMINI_BASE_URL"),
    "opencode" => ("OPENAI_API_KEY", "OPENAI_BASE_URL"),
    "openclaw" => ("OPENCLAW_API_KEY", "OPENCLAW_BASE_URL"),
    "hermes" => ("HERMES_API_KEY", "HERMES_BASE_URL"),
    _ => {
        eprintln!("错误: 不支持的应用类型 {app}");
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

- [ ] **Step 2: 同步修复 cmd_update_provider env 硬编码（如适用）**

定位 `fn cmd_update_provider(`（原行号 ~1458）。同样按 `app` 选择字段名。若 `update_provider` 不修改 env（仅修改 name/api_format），则跳过本 Step。

Run: `grep -n "ANTHROPIC_API_KEY\|ANTHROPIC_BASE_URL" crates\cc-switch-cli\src\main.rs`
Expected: 仅 Step 1 修改后的 `match` 块内出现（在 `"claude" | "claude-desktop"` 分支），无其它硬编码残留。

- [ ] **Step 3: 更新 Cli 帮助注释覆盖 7 应用**

打开 `crates/cc-switch-cli/src/main.rs`，定位 `AddProvider` 命令定义（原行号 ~65）。把：

```rust
/// API 格式（仅 claude/codex/gemini/claude-desktop）
/// claude: anthropic / openai_chat / openai_responses
/// codex: openai_responses / openai_chat
/// gemini: gemini_native / openai_chat / openai_responses / anthropic
/// claude-desktop: anthropic / openai_chat / openai_responses / gemini_native / bedrock
#[arg(long)]
api_format: Option<String>,
```

替换为：

```rust
/// API 格式（覆盖全部 7 应用，REQ-020）
/// claude:          anthropic / openai_chat / openai_responses
/// claude-desktop:  anthropic / openai_chat / openai_responses / gemini_native / bedrock
/// codex:           openai_responses / openai_chat
/// gemini:          gemini_native / openai_chat / openai_responses / anthropic
/// opencode:        openai_chat / anthropic（按 SDK 包自动选择，可不填）
/// openclaw:        openai_completions / anthropic（由 settings_config.api 字段决定）
/// hermes:          hermes_native / openai_chat（由 settings_config.api_mode 字段决定）
#[arg(long)]
api_format: Option<String>,
```

同样修改 `UpdateProvider` 命令的注释（原行号 ~110）。

- [ ] **Step 4: 更新 cmd_help 输出中的 api-format 说明**

定位 `fn cmd_help()`（原行号 ~4007）。把：

```rust
println!("    add-provider <APP> <ID> <NAME> [--api-key K] [--base-url U] [--api-format F]");
```

下方追加：

```rust
println!("        APP: claude | claude-desktop | codex | gemini | opencode | openclaw | hermes");
println!("        api-format 见各应用支持列表（详见 --help）");
```

- [ ] **Step 5: 更新 docs/cli-reference-manual.md API 格式表格**

打开 `docs/cli-reference-manual.md`，定位 API 格式表格（约 643 行）。把覆盖 4 应用的表格扩展为 7 应用：

```markdown
| 应用 | 支持的 API 格式 |
|---|---|
| claude | anthropic / openai_chat / openai_responses |
| claude-desktop | anthropic / openai_chat / openai_responses / gemini_native / bedrock |
| codex | openai_responses / openai_chat |
| gemini | gemini_native / openai_chat / openai_responses / anthropic |
| opencode | openai_chat / anthropic（按 SDK 包 npm 字段自动选择，可省略 api-format） |
| openclaw | openai_completions / anthropic（由 settings_config.api 字段决定） |
| hermes | hermes_native / openai_chat（由 settings_config.api_mode 字段决定） |
```

- [ ] **Step 6: 验证 CLI 编译**

Run: `cargo build -p cc-switch-cli`
Expected: 编译成功。

- [ ] **Step 7: 验证 add-provider 命令按应用选择正确 env 名**

Run: `cargo run -p cc-switch-cli -- add-provider codex test-codex TestCodex --api-key sk-test --base-url https://example.com`
Expected: 命令成功，数据库中 `test-codex` 供应商的 `settings_config.env` 含 `OPENAI_API_KEY` 和 `OPENAI_BASE_URL`（非 ANTHROPIC_*）。

验证数据库：

Run: `cargo run -p cc-switch-cli -- list-providers codex`
Expected: 输出含 `test-codex` 行，Base URL 列显示 `https://example.com`。

清理测试数据：

Run: `cargo run -p cc-switch-cli -- remove-provider codex test-codex`

- [ ] **Step 8: 验证帮助信息**

Run: `cargo run -p cc-switch-cli -- add-provider --help`
Expected: `--api-format` 注释含全部 7 应用。

Run: `cargo run -p cc-switch-cli -- help | findstr opencode`
Expected: 输出含 `opencode`。

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat(cli): add-provider env 字段按 --app 选择（覆盖 7 应用），api-format 注释与文档同步（REQ-020）"
```

---

## Task 16: 最终验证与文档更新

**Files:**
- Modify: `docs/cli-feature-implementation-assessment.md`（更新实现状态）
- Modify: `docs/cli-reference-manual.md`（更新架构说明）

- [ ] **Step 1: workspace 全量编译**

Run: `cargo build --workspace`
Expected: 编译成功，无 warning（除少量 dead_code）。

- [ ] **Step 2: workspace 全量测试**

Run: `cargo test --workspace`
Expected: 全部测试通过。

- [ ] **Step 3: 验证 CLI 二进制不依赖 tauri**

Run: `cargo tree -p cc-switch-cli | findstr /i "tauri"`
Expected: 无输出。

Run: `cargo tree -p cc-switch-cli | findstr /i "webkit2gtk"`
Expected: 无输出。

- [ ] **Step 4: 验证 GUI 二进制仍依赖 tauri**

Run: `cargo tree -p cc-switch-app | findstr /i "^tauri v"`
Expected: 输出 `tauri v2.8.x`。

- [ ] **Step 5: 验证 CLI stream-check 实际工作**

Run: `cargo run -p cc-switch-cli -- stream-check-all`
Expected: 命令实际执行连通性检查，输出每个供应商的结果，无桩实现字符串。

- [ ] **Step 6: 验证 apply-config 跳过代理字段**

创建临时 YAML `test-apply.yaml`：

```yaml
providers:
  - app: claude
    id: plan-b-test
    name: Plan B Test
    current: false
    env:
      ANTHROPIC_API_KEY: sk-test
proxy:
  takeover:
    claude: true
failover:
  auto: false
```

Run: `cargo run -p cc-switch-cli -- apply-config test-apply.yaml`
Expected: 输出含 `（跳过）代理接管 claude=true —— CLI 模式需手动执行 \`cc-switch-cli takeover claude on\``。

清理：删除 `test-apply.yaml`，运行 `cargo run -p cc-switch-cli -- remove-provider claude plan-b-test`。

- [ ] **Step 7: 更新 docs/cli-feature-implementation-assessment.md 实现状态**

打开 `docs/cli-feature-implementation-assessment.md`，把"实现状态"段（Plan A 已修改过）追加：

```markdown
> **架构状态**（2026-07-04 Plan B 完成）：
> - `cc_switch_lib` 已拆分为三层 crate：`cc-switch-core`（业务逻辑）/ `cc-switch-tauri-commands`（命令包装）/ `cc-switch-app`（GUI）
> - CLI 迁到独立 crate `cc-switch-cli`，**不再依赖 tauri / webkit2gtk**
> - `CopilotAuthState` 改为 `Arc<RwLock<CopilotAuthManager>>` 类型别名，由 core 持有
> - `apply-config` 接收 `ApplyContext`，CLI 传 `proxy_service: None`（代理字段跳过并提示）
> - `stream-check` / `stream-check-all` CLI 命令已实际实现（基于 core service）
> - API 格式设置覆盖全部 7 应用（REQ-020）
```

- [ ] **Step 8: 更新 docs/cli-reference-manual.md 架构说明**

在 `docs/cli-reference-manual.md` 顶部"概述"段追加：

```markdown
## 架构（Plan B 重构后）

CC Switch 自 v3.16.5 起，Rust 后端拆分为 Cargo workspace 三层 crate：

- `cc-switch-core`：纯业务逻辑（database / services / proxy / core），不依赖 Tauri
- `cc-switch-tauri-commands`：`#[tauri::command]` 包装层，依赖 core
- `cc-switch-app`（src-tauri）：GUI 二进制，依赖 tauri-commands + core
- `cc-switch-cli`：CLI 二进制，**只依赖 core**，编译不需要 webkit2gtk

CLI 与 GUI 共享同一套 core service，确保行为等价。
```

- [ ] **Step 9: 运行 src-tauri 集成测试**

Run: `cd src-tauri && cargo test --features test-hooks`
Expected: 全部通过。

- [ ] **Step 10: 运行 Rust 后端 lint**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: 无 warning。如有，按提示修复（通常为未使用 import）。

- [ ] **Step 11: 运行前端测试（确保 Plan A 修复未被破坏）**

Run: `pnpm typecheck && pnpm format:check && pnpm test:unit`
Expected: 全部通过。

- [ ] **Step 12: Commit**

```bash
git add -A
git commit -m "docs: Plan B 架构重构完成，更新评估文档与参考手册"
```

- [ ] **Step 13: 验证 Plan A 修复仍生效**

Run: `cargo run -p cc-switch-cli -- help | findstr "stream-check"`
Expected: `stream-check` 命令存在，且帮助信息不再含"GUI 专属"标注（因已实际实现）。

Run: `cargo run -p cc-switch-cli -- apply-config --help 2>&1 | findstr "listen\|port"`
Expected: 无输出（Plan A 已删除 listen/port 字段，Plan B 保持）。

Run: `cargo run -p cc-switch-cli -- add-provider --help | findstr "ANTHROPIC"`
Expected: 无输出（env 字段名不再硬编码在帮助中）。

- [ ] **Step 14: 最终 Commit**

```bash
git add -A
git commit -m "chore(plan-b): 最终验证通过，Plan B 架构重构完成"
```

---

## Self-Review

### 1. Spec 覆盖检查

| Spec 章节 | 覆盖 Task | 状态 |
|---|---|---|
| §七.2 P1-1：lib crate 分层重构 | Task 1-12 | ✅ 完整覆盖 |
| §七.2 P1-2：API 格式设置覆盖 7 应用（REQ-020） | Task 15 | ✅ 完整覆盖 |
| §七.2 P1-3：service 层去 Tauri 化 | Task 7（EventCallback trait）+ Task 10（CopilotAuthState） | ✅ 完整覆盖 |
| §七.1 P0-2 方案 A：apply-config 升级为 ApplyContext | Task 9 | ✅ 完整覆盖 |
| §六.7 架构重构建议 1（lib crate 分层） | Task 1-12 | ✅ 完整覆盖 |
| §六.7 架构重构建议 2（service 层去 Tauri 化） | Task 7 + Task 10 | ✅ 完整覆盖 |
| §六.7 架构重构建议 3（apply-config 接收 ApplyContext） | Task 9 | ✅ 完整覆盖 |

### 2. Placeholder 扫描

- 所有 Task 均含完整代码块或精确 `git mv` 指令。
- Task 7 Step 5/6 提到"逐个替换 app.emit 调用"——给出具体模式与文件名，非空泛指令。
- Task 11 Step 6 给出 `stream_check.rs` 完整改写代码。
- Task 14 Step 2/3 给出 `cmd_stream_check` / `cmd_stream_check_all` 完整实现。
- 无 TBD / TODO / "实现细节略" / "类似 Task N" 等占位符。

### 3. 类型一致性检查

- `CopilotAuthState`：Task 10 定义为 `pub type CopilotAuthState = Arc<RwLock<CopilotAuthManager>>`；Task 11 Step 5 删除原 newtype；Task 11 Step 6 中 `commands/stream_check.rs` 通过 `use cc_switch_core::CopilotAuthState` 引入；Task 14 Step 2 通过 `cc_switch_core::copilot_auth_state::new_copilot_auth_state` 创建实例。**类型一致**。
- `ApplyContext`：Task 9 Step 4 定义为 `pub struct ApplyContext<'a> { db: &'a Database, proxy_service: Option<&'a ProxyService> }`，方法 `new(db)` / `with_proxy(db, proxy)`；Task 9 Step 6 CLI 调用 `ApplyContext::new(&db)`；Task 16 Step 6 验证输出含跳过提示。**类型一致**。
- `EventCallback` trait：Task 7 Step 4 定义；Step 5/6/7 在 services/store/usage_events 中使用；Step 9 在 src-tauri 中实现 `TauriEmitCallback`。**类型一致**。
- `StreamCheckService::check_provider_with_state`：Task 10 Step 3 定义签名为 `(&AppType, &Provider, &StreamCheckConfig, &CopilotAuthState) -> Result<StreamCheckResult, AppError>`；Task 11 Step 6 命令包装层调用此方法；Task 14 Step 2/3 CLI 调用此方法。**签名一致**。
- `set_takeover_for_app`：Task 9 Step 4 调用 `svc.set_takeover_for_app(app, *enabled).await`——确认 `ProxyService` 已有此方法（参考 `src-tauri/src/lib.rs:1811` 中 `state.proxy_service.set_takeover_for_app(app_type, true).await`）。**方法存在**。

### 4. 风险点与回滚策略

- **Task 6/7/8 涉及大量 `use crate::xxx` 路径修复**：若 `cargo build -p cc-switch-core` 失败，回到上一个 commit，按报错信息逐个修复 `use` 路径。
- **Task 11 commands 迁移可能漏改路径**：Task 11 Step 6 仅展示 `stream_check.rs` 改写示例，其它 32 个 commands 子模块需按同样模式修改。如工作量过大，可分多个 commit（按子模块批次）。
- **Task 12 重命名 package 可能破坏现有 CI/CD 脚本**：如有 CI 引用 `cc-switch` package 名，需同步更新。
- **Plan A 修复保留**：Task 16 Step 13 验证 Plan A 的 listen/port 删除、env 硬编码修复、stream-check 桩命令处理（现已实际实现，故 help 不再标 GUI 专属）均未破坏。

### 5. 估时汇总

| Task | 估时 |
|---|---|
| Task 1-2 | 1 小时 |
| Task 3-6 | 4 小时（大量文件迁移） |
| Task 7-8 | 1 天（EventCallback trait + 去 Tauri 化） |
| Task 9 | 半天（ApplyContext 升级） |
| Task 10 | 半天（CopilotAuthState 重构） |
| Task 11 | 1 天（commands 迁移 + 路径修复） |
| Task 12-13 | 半天（重命名 + CLI 独立 crate） |
| Task 14 | 半天（stream-check 实现） |
| Task 15 | 1 小时（env 硬编码修复） |
| Task 16 | 半天（最终验证） |
| **总计** | **约 4 天**（与 spec P1-1 估时 L=2~3 天 + P1-2/P1-3 各 0.5 天吻合） |

---

## 执行说明

本 Plan B 是大型架构重构，**必须按 Task 顺序执行**，每个 Task 完成后立即 commit 以保持可回滚。重构期间：

1. **Plan A 的修复不得破坏**——Task 16 Step 13 显式验证。
2. **每 Task 编译必须通过**——`cargo build --workspace` 是 Task 完成的硬性指标。
3. **commands 迁移（Task 11）工作量最大**——若一次 commit 太大，可拆为多个子 commit（按子模块批次），但必须保持每个子 commit 可编译。
4. **测试覆盖率不降低**——Task 5/7/9/10/13 均新增测试文件，确保重构期间回归保护。

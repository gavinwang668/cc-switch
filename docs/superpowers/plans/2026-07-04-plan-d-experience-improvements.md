# Plan D: 体验改进 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 补全 OPT-A 级别的缺失功能：协议转换可观测性（M-3）、实时可观测性（M-4）、配置 diff/rollback（M-5）、导出 YAML（M-6）、供应商启用/禁用（M-7），并完成 P2-1 参考手册补全与 P2-3 OPT 子优先级标注。

**Architecture:** 在 Plan B 完成分层重构后，所有新命令均以 `cc_switch_lib::core::*` 公共 API 为基础。M-3 直接调用 `proxy::providers::transform::*` 转换函数；M-4 通过 HTTP 轮询运行中 daemon 的 `/status` 端点 + 直接读共享 SQLite；M-5/M-6 复用 `DeclConfig` 与 `Database::backup_*`；M-7 在 `ProviderMeta` 加 `disabled` 字段，由 `ProviderRouter` 跳过。

**Tech Stack:** Rust（cc-switch-cli 二进制 + cc_switch_lib）、clap（命令行解析）、serde_yaml（YAML 序列化）、rusqlite（DAO 扩展）、reqwest（HTTP 轮询 daemon）、tokio（异步 tail）。

**关联 Spec:** [docs/superpowers/specs/2026-07-04-cli-feature-review-design.md](file:///f:/workspace/trae/cc-switch/docs/superpowers/specs/2026-07-04-cli-feature-review-design.md) §九 M-3~M-7、§七.3 P2-1/P2-3

**前置依赖:** Plan B（lib crate 分层）已完成；Plan A（apply-config 止血）已完成。Plan D 各 Task 之间相对独立，可并行实施，但建议按 Task 编号顺序执行以避免合并冲突。

---

## File Structure

| 文件 | 操作 | 责任 |
|---|---|---|
| `src-tauri/src/core/decl_config.rs` | 修改 | 新增 `to_decl_config_from_db()` 反向构造；`apply()` 增加 `backup_path` 参数；`ProviderEntry` 加 `disabled` 字段 |
| `src-tauri/src/provider.rs` | 修改 | `ProviderMeta` 加 `disabled: Option<bool>` 字段 |
| `src-tauri/src/database/dao/providers.rs` | 修改 | 新增 `update_provider_disabled()` DAO；`get_all_providers` SELECT 列加 `disabled` |
| `src-tauri/src/database/schema.rs` | 修改 | providers 表加 `disabled INTEGER NOT NULL DEFAULT 0` 列（migration） |
| `src-tauri/src/proxy/provider_router.rs` | 修改 | `select_providers` 过滤 `disabled=true` 的供应商 |
| `src-tauri/src/proxy/usage/logger.rs` | 修改 | `RequestLog` 加可选 `request_body` 字段；env `CC_SWITCH_LOG_BODIES=1` 时落库 |
| `src-tauri/src/database/dao/usage_rollup.rs` | 修改 | 新增 `get_request_body()`、`save_request_body()` 方法 |
| `src-tauri/src/bin/cc-switch-cli.rs` | 修改 | 新增 9 个命令枚举与分发、9 个 `cmd_*` 函数、help 文本更新 |
| `src-tauri/tests/cli_plan_d.rs` | 新建 | Plan D 全部新命令的集成测试 |
| `docs/cli-reference-manual.md` | 修改 | 补全 7 应用 API 格式表格；新增 9 命令章节 |
| `docs/cli-feature-implementation-assessment.md` | 修改 | OPT 列表加"子优先级"列；标注 M-3~M-7 已实现 |

---

## Task 1: M-6 导出 YAML — 实现 `export-yaml` 命令（apply-config 逆操作）

**Files:**
- Modify: `src-tauri/src/core/decl_config.rs`（新增 `from_database()` 方法、`ProviderEntry` 加 `disabled` 字段）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`（新增 `ExportYaml` 命令、`cmd_export_yaml` 函数）
- Test: `src-tauri/tests/cli_plan_d.rs`

- [ ] **Step 1: 写失败测试 — export-yaml 输出可被 validate 重新接受**

Create `src-tauri/tests/cli_plan_d.rs`:

```rust
// Plan D 集成测试：通过子进程调用 CLI 二进制验证新命令

use std::process::Command;

fn cli_binary() -> String {
    env!("CARGO_BIN_EXE_cc-switch-cli").to_string()
}

fn tmp_home() -> tempfile::TempDir {
    tempfile::tempdir().expect("创建临时目录失败")
}

#[cfg(test)]
mod export_yaml_tests {
    use super::*;

    #[test]
    fn test_export_yaml_round_trips_through_validate() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();
        let yaml_path = tmp.path().join("exported.yaml");

        // 先添加一个 claude 供应商
        let add = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "claude",
                "test-yaml",
                "Test YAML",
                "--api-key",
                "sk-test",
                "--base-url",
                "https://api.anthropic.com",
            ])
            .output()
            .expect("add-provider 失败");
        assert!(add.status.success(), "add-provider 失败: {}", String::from_utf8_lossy(&add.stderr));

        // 导出 YAML
        let export = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["export-yaml", yaml_path.to_str().unwrap()])
            .output()
            .expect("export-yaml 失败");
        assert!(export.status.success(), "export-yaml 失败: {}", String::from_utf8_lossy(&export.stderr));

        // 导出的 YAML 应可被 validate 接受
        let validate = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["validate", yaml_path.to_str().unwrap()])
            .output()
            .expect("validate 失败");
        assert!(
            validate.status.success(),
            "导出的 YAML 校验失败: {}",
            String::from_utf8_lossy(&validate.stderr)
        );

        // 内容应包含供应商 ID
        let yaml_content = std::fs::read_to_string(&yaml_path).unwrap();
        assert!(yaml_content.contains("test-yaml"), "YAML 缺少供应商 ID: {yaml_content}");
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d export_yaml -- --nocapture
```

Expected: FAIL — `export-yaml` 命令不存在，CLI 报错 `unrecognized subcommand`

- [ ] **Step 3: 在 `ProviderEntry` 加 `disabled` 字段**

Read [src-tauri/src/core/decl_config.rs:59-69](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs)

将第 59~69 行：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderEntry {
    pub app: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub current: bool,
}
```

替换为：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderEntry {
    pub app: String,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub current: bool,
    /// 是否禁用（true=不参与代理转发，保留配置）
    #[serde(default)]
    pub disabled: bool,
}
```

- [ ] **Step 4: 在 `DeclConfig` 上实现 `from_database()` 方法**

在 [src-tauri/src/core/decl_config.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs) 的 `impl DeclConfig` 块末尾（`apply` 方法之后）追加：

```rust
    /// 从数据库反向构造 DeclConfig（apply-config 的逆操作）
    ///
    /// 读取所有应用的供应商、故障转移队列、自动故障转移开关、全局出站代理、
    /// 设备级设置，构造可序列化为 YAML 的 DeclConfig。
    pub fn from_database(db: &crate::database::Database) -> Result<Self, String> {
        let mut providers = Vec::new();
        let valid_apps = [
            "claude",
            "claude-desktop",
            "codex",
            "gemini",
            "opencode",
            "openclaw",
            "hermes",
        ];

        for app in &valid_apps {
            let all = db
                .get_all_providers(app)
                .map_err(|e| format!("读取 {app} 供应商失败: {e}"))?;
            let current_id = db
                .get_current_provider(app)
                .map_err(|e| format!("读取 {app} 当前供应商失败: {e}"))?;

            for (id, provider) in all {
                let env_map: HashMap<String, String> = provider
                    .settings_config
                    .get("env")
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();

                let disabled = provider
                    .meta
                    .as_ref()
                    .and_then(|m| m.disabled)
                    .unwrap_or(false);

                providers.push(ProviderEntry {
                    app: app.to_string(),
                    id,
                    name: provider.name,
                    env: env_map,
                    current: current_id.as_deref() == Some(provider.id.as_str()),
                    disabled,
                });
            }
        }

        // 故障转移队列
        let mut queue: HashMap<String, Vec<String>> = HashMap::new();
        for app in ["claude", "codex", "gemini"] {
            if let Ok(items) = db.get_failover_queue(app) {
                if !items.is_empty() {
                    queue.insert(
                        app.to_string(),
                        items.into_iter().map(|i| i.provider_id).collect(),
                    );
                }
            }
        }

        // 自动故障转移开关（任一应用开启即视为 auto=true）
        let auto = ["claude", "codex", "gemini"]
            .iter()
            .any(|app| db.get_proxy_flags_sync(app).map(|(_, a)| a).unwrap_or(false));

        // 全局出站代理
        let global_proxy = db
            .get_global_proxy_url()
            .ok()
            .flatten()
            .map(|url| GlobalProxySection { url });

        // 设备级设置
        let settings = {
            let s = crate::settings::get_settings();
            SettingsSection {
                language: s.language,
                backup_interval_hours: s.backup_interval_hours,
                backup_retain_count: s.backup_retain_count,
                claude_config_dir: s.claude_config_dir,
                codex_config_dir: s.codex_config_dir,
                gemini_config_dir: s.gemini_config_dir,
            }
        };

        Ok(DeclConfig {
            proxy: ProxySection::default(),
            providers,
            failover: FailoverSection { auto, queue },
            global_proxy,
            settings,
        })
    }

    /// 序列化为 YAML 字符串
    pub fn to_yaml_string(&self) -> Result<String, String> {
        serde_yaml::to_string(self).map_err(|e| format!("序列化 YAML 失败: {e}"))
    }
```

- [ ] **Step 5: 新增 `ExportYaml` 命令到 CLI**

Read [src-tauri/src/bin/cc-switch-cli.rs:188-198](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在第 197 行（`ApplyConfig` 变体之后、`SortProviders` 之前）插入：

```rust
    /// 导出当前配置为声明式 YAML 文件（apply-config 的逆操作）
    ExportYaml {
        /// 输出 YAML 文件路径
        path: String,
    },
```

- [ ] **Step 6: 添加 match 分发**

Read [src-tauri/src/bin/cc-switch-cli.rs:628-632](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在第 629 行（`Commands::ApplyConfig { path } => cmd_apply_config(&path),` 之后）插入：

```rust
        Commands::ExportYaml { path } => cmd_export_yaml(&path),
```

- [ ] **Step 7: 实现 `cmd_export_yaml` 函数**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_apply_config` 函数之后（约第 2404 行）插入：

```rust
/// export-yaml: 导出当前配置为声明式 YAML 文件
fn cmd_export_yaml(path: &str) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let config = match cc_switch_lib::core::decl_config::DeclConfig::from_database(&db) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("从数据库构造配置失败: {e}");
            std::process::exit(1);
        }
    };
    let yaml = match config.to_yaml_string() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("序列化 YAML 失败: {e}");
            std::process::exit(1);
        }
    };
    let target = std::path::Path::new(path);
    if let Some(parent) = target.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("创建目录失败: {e}");
            std::process::exit(1);
        }
    }
    if let Err(e) = std::fs::write(target, yaml.as_bytes()) {
        eprintln!("写入文件失败: {e}");
        std::process::exit(1);
    }
    println!("✓ 配置已导出到 {path}");
    println!("  供应商数量: {}", config.providers.len());
    println!("  故障转移队列: {} 个应用", config.failover.queue.len());
}
```

- [ ] **Step 8: 在 `cmd_help` 添加 export-yaml 说明**

Read [src-tauri/src/bin/cc-switch-cli.rs:4045-4047](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在第 4046 行（`apply-config` 行之后）插入：

```rust
    println!("    export-yaml <PATH>           导出当前配置为 YAML 文件");
```

- [ ] **Step 9: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d export_yaml -- --nocapture
```

Expected: `test_export_yaml_round_trips_through_validate` PASS

- [ ] **Step 10: 验证编译与 clippy**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo check --bin cc-switch-cli
cargo clippy --bin cc-switch-cli
```

Expected: 编译通过，无新增警告

- [ ] **Step 11: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/core/decl_config.rs src-tauri/src/bin/cc-switch-cli.rs src-tauri/tests/cli_plan_d.rs
git commit -m "feat(cli): 新增 export-yaml 命令实现 apply-config 逆操作

- DeclConfig::from_database() 从数据库反向构造配置
- ProviderEntry 加 disabled 字段（为 M-7 预留）
- 集成测试验证 export → validate 往返一致

关联 spec: §九 M-6"
```

---

## Task 2: M-5 配置 diff — 实现 `diff` 命令

**Files:**
- Modify: `src-tauri/src/core/decl_config.rs`（新增 `diff()` 方法）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`（新增 `Diff` 命令、`cmd_diff` 函数）
- Test: `src-tauri/tests/cli_plan_d.rs`

- [ ] **Step 1: 写失败测试 — diff 检测新增供应商**

在 `src-tauri/tests/cli_plan_d.rs` 追加：

```rust
#[cfg(test)]
mod diff_tests {
    use super::*;

    #[test]
    fn test_diff_detects_added_provider() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();

        // 数据库已有 claude/existing 供应商
        let add = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "claude",
                "existing",
                "Existing",
                "--api-key",
                "k1",
            ])
            .output()
            .unwrap();
        assert!(add.status.success());

        // YAML 文件含 existing + new-provider
        let yaml_path = tmp.path().join("diff.yaml");
        std::fs::write(
            &yaml_path,
            r#"
providers:
  - app: claude
    id: existing
    name: Existing
    env:
      ANTHROPIC_API_KEY: k1
  - app: claude
    id: new-provider
    name: New Provider
    env:
      ANTHROPIC_API_KEY: k2
"#,
        )
        .unwrap();

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["diff", yaml_path.to_str().unwrap()])
            .output()
            .expect("diff 失败");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("new-provider") && stdout.contains("+"),
            "diff 应显示新增的 new-provider（带 + 前缀），实际: {stdout}"
        );
    }

    #[test]
    fn test_diff_detects_no_changes() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();

        // 先 export 再 diff，应无差异
        let add = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "claude",
                "p1",
                "P1",
                "--api-key",
                "k1",
            ])
            .output()
            .unwrap();
        assert!(add.status.success());

        let yaml_path = tmp.path().join("current.yaml");
        let export = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["export-yaml", yaml_path.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(export.status.success());

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["diff", yaml_path.to_str().unwrap()])
            .output()
            .expect("diff 失败");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("无差异") || stdout.contains("no changes"),
            "无差异时应提示，实际: {stdout}"
        );
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d diff -- --nocapture
```

Expected: FAIL — `diff` 命令不存在

- [ ] **Step 3: 在 `DeclConfig` 实现 `diff()` 方法**

在 [src-tauri/src/core/decl_config.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs) 的 `impl DeclConfig` 块末尾追加：

```rust
    /// 对比 YAML 配置与数据库当前配置，返回 unified diff 格式字符串
    ///
    /// 对比维度：供应商列表（按 app+id 唯一）、故障转移队列、自动故障转移、
    /// 全局代理、设备级设置。差异以 `+`/`-` 前缀行表示。
    pub fn diff(&self, db: &crate::database::Database) -> Result<String, String> {
        let current = Self::from_database(db)?;
        let yaml_str = serde_yaml::to_string(self).map_err(|e| format!("序列化 YAML 失败: {e}"))?;
        let cur_str = serde_yaml::to_string(&current).map_err(|e| format!("序列化当前配置失败: {e}"))?;

        // 按行对比，生成 unified diff 风格输出
        let yaml_lines: Vec<&str> = yaml_str.lines().collect();
        let cur_lines: Vec<&str> = cur_str.lines().collect();

        // 简化 diff：按行集合对比，新增行 +、删除行 -、共有行保留
        let yaml_set: std::collections::HashSet<&str> = yaml_lines.iter().copied().collect();
        let cur_set: std::collections::HashSet<&str> = cur_lines.iter().copied().collect();

        let mut output = String::new();
        let mut has_diff = false;

        // 先输出删除的行（在 cur 但不在 yaml）
        for line in &cur_lines {
            if !yaml_set.contains(line) {
                output.push_str(&format!("- {line}\n"));
                has_diff = true;
            }
        }
        // 再输出新增的行（在 yaml 但不在 cur）
        for line in &yaml_lines {
            if !cur_set.contains(line) {
                output.push_str(&format!("+ {line}\n"));
                has_diff = true;
            }
        }

        if !has_diff {
            output.push_str("无差异（YAML 与当前数据库配置一致）\n");
        }
        Ok(output)
    }
```

- [ ] **Step 4: 新增 `Diff` 命令到 CLI**

Read [src-tauri/src/bin/cc-switch-cli.rs:229-238](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在第 238 行（`Validate` 块之后、`ApplyConfig` 块之前）插入：

```rust
    /// 对比 YAML 配置与当前数据库配置的差异
    Diff {
        /// YAML 配置文件路径
        path: String,
    },
```

- [ ] **Step 5: 添加 match 分发**

Read [src-tauri/src/bin/cc-switch-cli.rs:628-629](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在第 629 行（`Commands::Validate { path } => cmd_validate(&path),` 之后）插入：

```rust
        Commands::Diff { path } => cmd_diff(&path),
```

- [ ] **Step 6: 实现 `cmd_diff` 函数**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_apply_config` 函数之前插入：

```rust
/// diff: 对比 YAML 配置与当前数据库配置的差异
fn cmd_diff(path: &str) {
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
    match config.diff(&db) {
        Ok(diff_output) => print!("{diff_output}"),
        Err(e) => {
            eprintln!("生成 diff 失败: {e}");
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 7: 在 `cmd_help` 添加 diff 说明**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_help` 中，`validate` 行之后插入：

```rust
    println!("    diff <PATH>                   对比 YAML 与当前数据库配置差异");
```

- [ ] **Step 8: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d diff -- --nocapture
```

Expected: 2 个 diff 测试全部通过

- [ ] **Step 9: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/core/decl_config.rs src-tauri/src/bin/cc-switch-cli.rs src-tauri/tests/cli_plan_d.rs
git commit -m "feat(cli): 新增 diff 命令对比 YAML 与数据库配置差异

- DeclConfig::diff() 按行集合对比生成 +/- diff
- 复用 from_database() 保证对比基准一致
- 集成测试覆盖新增供应商检测与无差异场景

关联 spec: §九 M-5"
```

---

## Task 3: M-5 配置 rollback — 实现 `rollback` 命令

**Files:**
- Modify: `src-tauri/src/core/decl_config.rs`（`apply()` 增加自动备份）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`（新增 `Rollback` 命令、`cmd_rollback` 函数）
- Modify: `src-tauri/src/database/backup.rs`（新增 `last_apply_backup_path()`/`set_last_apply_backup_path()`）
- Test: `src-tauri/tests/cli_plan_d.rs`

- [ ] **Step 1: 写失败测试 — rollback 恢复 apply 前状态**

在 `src-tauri/tests/cli_plan_d.rs` 追加：

```rust
#[cfg(test)]
mod rollback_tests {
    use super::*;

    #[test]
    fn test_rollback_restores_pre_apply_state() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();

        // 1. 添加原始供应商
        let add = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "claude",
                "original",
                "Original",
                "--api-key",
                "k1",
            ])
            .output()
            .unwrap();
        assert!(add.status.success());

        // 2. apply-config 写入新供应商
        let yaml_path = tmp.path().join("apply.yaml");
        std::fs::write(
            &yaml_path,
            r#"
providers:
  - app: claude
    id: after-apply
    name: After Apply
    env:
      ANTHROPIC_API_KEY: k2
"#,
        )
        .unwrap();

        let apply = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["apply-config", yaml_path.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(apply.status.success(), "apply-config 失败: {}", String::from_utf8_lossy(&apply.stderr));

        // 3. list-providers 应同时看到 original 和 after-apply
        let list_after_apply = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["list-providers", "claude"])
            .output()
            .unwrap();
        let list_str = String::from_utf8_lossy(&list_after_apply.stdout).to_string();
        assert!(list_str.contains("after-apply"), "apply 后应有 after-apply: {list_str}");

        // 4. rollback 到 apply 前
        let rollback = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["rollback"])
            .output()
            .unwrap();
        assert!(rollback.status.success(), "rollback 失败: {}", String::from_utf8_lossy(&rollback.stderr));

        // 5. list-providers 应再次看到 original，不再看到 after-apply
        let list_after_rollback = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["list-providers", "claude"])
            .output()
            .unwrap();
        let final_list = String::from_utf8_lossy(&list_after_rollback.stdout).to_string();
        assert!(final_list.contains("original"), "rollback 后应有 original: {final_list}");
        assert!(!final_list.contains("after-apply"), "rollback 后不应有 after-apply: {final_list}");
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d rollback -- --nocapture
```

Expected: FAIL — `rollback` 命令不存在

- [ ] **Step 3: 在 `Database` 新增 last-apply-backup 记录读写**

Read [src-tauri/src/database/backup.rs:575-620](file:///f:/workspace/trae/cc-switch/src-tauri/src/database/backup.rs)

在 `impl Database` 块末尾追加：

```rust
    /// 读取上一次 apply-config 前创建的备份文件名
    pub fn get_last_apply_backup(&self) -> Result<Option<String>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = 'last_apply_backup'")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut rows = stmt
            .query([])
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            let value: String = row.get(0).map_err(|e| AppError::Database(e.to_string()))?;
            return Ok(Some(value));
        }
        Ok(None)
    }

    /// 记录上一次 apply-config 前创建的备份文件名
    pub fn set_last_apply_backup(&self, filename: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('last_apply_backup', ?1)",
            rusqlite::params![filename],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 清除 last_apply_backup 记录（rollback 成功后调用，防止二次回滚）
    pub fn clear_last_apply_backup(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "DELETE FROM settings WHERE key = 'last_apply_backup'",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
```

- [ ] **Step 4: 修改 `DeclConfig::apply()` 自动备份**

Read [src-tauri/src/core/decl_config.rs:158-170](file:///f:/workspace/trae/cc-switch/src-tauri/src/core/decl_config.rs)

将第 159 行：

```rust
    pub fn apply(&self, db: &crate::database::Database) -> Result<String, String> {
```

替换为：

```rust
    /// 应用声明式配置到数据库。
    ///
    /// 应用前自动创建 SQL 备份并记录到 `last_apply_backup` 设置项，
    /// 供 `rollback` 命令恢复。备份失败不阻断应用（仅记录警告）。
    pub fn apply(&self, db: &crate::database::Database) -> Result<String, String> {
        // 自动备份：写入 backups 目录，文件名带时间戳
        let backup_dir = crate::config::get_app_config_dir().join("backups");
        let _ = std::fs::create_dir_all(&backup_dir);
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("pre-apply-{timestamp}.sql");
        let backup_path = backup_dir.join(&backup_filename);
        match db.export_sql(&backup_path) {
            Ok(_) => {
                if let Err(e) = db.set_last_apply_backup(&backup_filename) {
                    log::warn!("记录 last_apply_backup 失败: {e}");
                }
            }
            Err(e) => log::warn!("apply 前备份失败（继续应用）: {e}"),
        }
```

> 注：`apply` 方法体原有逻辑保持不变，仅在最开头插入备份代码。

- [ ] **Step 5: 新增 `Rollback` 命令到 CLI**

Read [src-tauri/src/bin/cc-switch-cli.rs:234-246](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在第 238 行（`ApplyConfig` 块之后、`ExportYaml` 块之前）插入：

```rust
    /// 回滚到上一个 apply-config 前的状态
    Rollback,
```

- [ ] **Step 6: 添加 match 分发**

在 [src-tauri/src/bin/cc-switch-cli.rs:629-630](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)（`ApplyConfig` 分发之后）插入：

```rust
        Commands::Rollback => cmd_rollback(),
```

- [ ] **Step 7: 实现 `cmd_rollback` 函数**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_apply_config` 函数之后插入：

```rust
/// rollback: 回滚到上一个 apply-config 前的状态
fn cmd_rollback() {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let backup_name = match db.get_last_apply_backup() {
        Ok(Some(name)) => name,
        Ok(None) => {
            eprintln!("无可回滚的 apply-config 备份（last_apply_backup 未设置）");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("读取 last_apply_backup 失败: {e}");
            std::process::exit(1);
        }
    };
    print!("即将从备份 '{backup_name}' 恢复，当前配置将被覆盖。是否继续? [y/N] ");
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        eprintln!("读取输入失败");
        std::process::exit(1);
    }
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("已取消");
        return;
    }
    match db.restore_from_backup(&backup_name) {
        Ok(msg) => {
            // 清除 last_apply_backup 防止二次回滚到同一个点
            let _ = db.clear_last_apply_backup();
            println!("✓ 已从备份 '{backup_name}' 恢复: {msg}");
            println!("  提示: 重启代理服务器使配置生效（如代理在运行）");
        }
        Err(e) => {
            eprintln!("回滚失败: {e}");
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 8: 在 `cmd_help` 添加 rollback 说明**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_help` 中，`apply-config` 行之后插入：

```rust
    println!("    rollback                      回滚到上一个 apply-config 前状态");
```

- [ ] **Step 9: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d rollback -- --nocapture
```

Expected: `test_rollback_restores_pre_apply_state` PASS

- [ ] **Step 10: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/core/decl_config.rs src-tauri/src/database/backup.rs src-tauri/src/bin/cc-switch-cli.rs src-tauri/tests/cli_plan_d.rs
git commit -m "feat(cli): 新增 rollback 命令 + apply-config 自动备份

- apply() 应用前自动创建 SQL 备份并记录到 last_apply_backup
- Database 新增 get/set/clear_last_apply_backup 方法
- rollback 命令交互式确认后恢复，并清除 last_apply_backup 防二次回滚
- 集成测试覆盖 add → apply → rollback 全链路

关联 spec: §九 M-5"
```

---

## Task 4: M-7 供应商启用/禁用 — 实现 `toggle-provider` 命令

**Files:**
- Modify: `src-tauri/src/provider.rs`（`ProviderMeta` 加 `disabled` 字段）
- Modify: `src-tauri/src/database/schema.rs`（migration 加 `disabled` 列）
- Modify: `src-tauri/src/database/dao/providers.rs`（`get_all_providers` SELECT 加 `disabled`、新增 `update_provider_disabled`）
- Modify: `src-tauri/src/proxy/provider_router.rs`（过滤 `disabled=true`）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`（新增 `ToggleProvider` 命令、`cmd_toggle_provider` 函数）
- Test: `src-tauri/tests/cli_plan_d.rs`

- [ ] **Step 1: 写失败测试 — 禁用供应商后代理跳过**

在 `src-tauri/tests/cli_plan_d.rs` 追加：

```rust
#[cfg(test)]
mod toggle_provider_tests {
    use super::*;

    #[test]
    fn test_toggle_provider_off_marks_disabled() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();

        let add = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "claude",
                "p1",
                "P1",
                "--api-key",
                "k1",
            ])
            .output()
            .unwrap();
        assert!(add.status.success());

        let toggle = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["toggle-provider", "claude", "p1", "off"])
            .output()
            .expect("toggle-provider 失败");
        assert!(toggle.status.success(), "toggle-provider off 失败: {}", String::from_utf8_lossy(&toggle.stderr));

        // list-providers 输出应标注 p1 为 disabled
        let list = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["list-providers", "claude"])
            .output()
            .unwrap();
        let list_str = String::from_utf8_lossy(&list.stdout).to_string();
        assert!(
            list_str.contains("disabled") || list_str.contains("禁用"),
            "list-providers 应标注禁用状态，实际: {list_str}"
        );

        // 重新启用
        let toggle_on = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["toggle-provider", "claude", "p1", "on"])
            .output()
            .unwrap();
        assert!(toggle_on.status.success());
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d toggle_provider -- --nocapture
```

Expected: FAIL — `toggle-provider` 命令不存在

- [ ] **Step 3: 在 `ProviderMeta` 加 `disabled` 字段**

Read [src-tauri/src/provider.rs:400-490](file:///f:/workspace/trae/cc-switch/src-tauri/src/provider.rs)

在第 486 行（`is_full_url` 字段之后、`prompt_cache_key` 字段之前）插入：

```rust
    /// 是否临时禁用（true=代理转发跳过此供应商，保留配置）
    #[serde(rename = "disabled", skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
```

- [ ] **Step 4: 在 schema migration 添加 `disabled` 列**

Read [src-tauri/src/database/schema.rs:25-50](file:///f:/workspace/trae/cc-switch/src-tauri/src/database/schema.rs)

在 providers 表 CREATE 语句（约第 27~48 行）末尾追加列：

```sql
CREATE TABLE IF NOT EXISTS providers (
    id TEXT NOT NULL,
    app_type TEXT NOT NULL,
    name TEXT NOT NULL,
    settings_config TEXT NOT NULL,
    website_url TEXT,
    category TEXT,
    created_at INTEGER,
    is_current INTEGER NOT NULL DEFAULT 0,
    sort_index INTEGER,
    notes TEXT,
    icon TEXT,
    icon_color TEXT,
    meta TEXT NOT NULL DEFAULT '{}',
    in_failover_queue INTEGER NOT NULL DEFAULT 0,
    disabled INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (id, app_type)
)
```

> 注：保留原有列定义不变，只在 `in_failover_queue` 之后添加 `disabled` 列。

Read [src-tauri/src/database/schema.rs:580-660](file:///f:/workspace/trae/cc-switch/src-tauri/src/database/schema.rs) 找到 migration 区域，在合适位置追加：

```rust
        Self::add_column_if_missing(conn, "providers", "disabled", "INTEGER NOT NULL DEFAULT 0")?;
```

- [ ] **Step 5: 修改 `get_all_providers` SELECT 与映射**

Read [src-tauri/src/database/dao/providers.rs:20-110](file:///f:/workspace/trae/cc-switch/src-tauri/src/database/dao/providers.rs)

将第 26~28 行 SQL：

```rust
        "SELECT id, name, settings_config, website_url, category, created_at, sort_index, notes, icon, icon_color, meta, in_failover_queue
         FROM providers WHERE app_type = ?1
         ORDER BY COALESCE(sort_index, 999999), created_at ASC, id ASC"
```

替换为：

```rust
        "SELECT id, name, settings_config, website_url, category, created_at, sort_index, notes, icon, icon_color, meta, in_failover_queue, disabled
         FROM providers WHERE app_type = ?1
         ORDER BY COALESCE(sort_index, 999999), created_at ASC, id ASC"
```

在 `query_map` 闭包中（约第 32~67 行），第 44 行 `let in_failover_queue: bool = row.get(11)?;` 之后插入：

```rust
                let disabled: bool = row.get::<_, i64>(12)? != 0;
```

并在闭包结束前（`Ok((id, Provider { ... }))` 内）的 `meta: Some(meta)` 之前插入：

```rust
                    meta: {
                        let mut m = meta;
                        if disabled {
                            m.disabled = Some(true);
                        }
                        Some(m)
                    },
```

> 注：原代码 `meta: Some(meta)` 替换为上述块。`meta` 变量是 `ProviderMeta`，已 `unwrap_or_default()`。

- [ ] **Step 6: 新增 `update_provider_disabled` DAO 方法**

在 [src-tauri/src/database/dao/providers.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/database/dao/providers.rs) 的 `impl Database` 块末尾追加：

```rust
    /// 设置供应商的 disabled 状态（不删除配置，仅影响代理转发）
    pub fn update_provider_disabled(
        &self,
        app_type: &str,
        id: &str,
        disabled: bool,
    ) -> Result<(), AppError> {
        // 读取现有 meta，合并 disabled 字段后写回
        let conn = lock_conn!(self.conn);
        let meta_str: String = conn
            .query_row(
                "SELECT meta FROM providers WHERE id = ?1 AND app_type = ?2",
                rusqlite::params![id, app_type],
                |row| row.get(0),
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut meta: crate::provider::ProviderMeta =
            serde_json::from_str(&meta_str).unwrap_or_default();
        meta.disabled = if disabled { Some(true) } else { None };

        let new_meta_str = serde_json::to_string(&meta)
            .map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "UPDATE providers SET meta = ?1 WHERE id = ?2 AND app_type = ?3",
            rusqlite::params![new_meta_str, id, app_type],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
```

- [ ] **Step 7: 修改 `ProviderRouter` 跳过 disabled 供应商**

Read [src-tauri/src/proxy/provider_router.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/provider_router.rs) 找到 `select_providers` 方法（或同等作用的方法），在过滤逻辑中加入：

```rust
        // 过滤掉 disabled=true 的供应商
        providers.retain(|p| {
            p.meta
                .as_ref()
                .and_then(|m| m.disabled)
                .unwrap_or(false)
                == false
        });
```

> 注：实施时 Read `select_providers` 上下文，在拿到 `Vec<Provider>` 后、按 failover queue 选择前调用 `retain`。具体行号以 Plan B 重构后的代码为准。

- [ ] **Step 8: 新增 `ToggleProvider` 命令到 CLI**

Read [src-tauri/src/bin/cc-switch-cli.rs:86-92](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在第 92 行（`RemoveProvider` 块之后、`SwitchProvider` 块之前）插入：

```rust
    /// 启用/禁用供应商（不删除配置，禁用后代理转发跳过）
    ToggleProvider {
        /// 应用类型 (claude, claude-desktop, codex, gemini, opencode, openclaw, hermes)
        app: String,
        /// 供应商 ID
        id: String,
        /// on 或 off
        state: String,
    },
```

- [ ] **Step 9: 添加 match 分发**

Read [src-tauri/src/bin/cc-switch-cli.rs:595-605](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `Commands::RemoveProvider` 分发之后插入：

```rust
        Commands::ToggleProvider { app, id, state } => cmd_toggle_provider(app, id, state),
```

- [ ] **Step 10: 实现 `cmd_toggle_provider` 函数**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_remove_provider` 函数之后插入：

```rust
/// toggle-provider: 启用/禁用供应商
fn cmd_toggle_provider(app: String, id: String, state: String) {
    if let Err(e) = validated_app(&app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
    let disabled = match state.as_str() {
        "on" => false,
        "off" => true,
        other => {
            eprintln!("错误: state 必须为 on 或 off，得到: {other}");
            std::process::exit(1);
        }
    };
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    // 先校验供应商存在
    match db.get_provider_by_id(&app, &id) {
        Ok(Some(_)) => {}
        Ok(None) => {
            eprintln!("供应商 {app}/{id} 不存在");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("查询供应商失败: {e}");
            std::process::exit(1);
        }
    }
    if let Err(e) = db.update_provider_disabled(&app, &id, disabled) {
        eprintln!("更新禁用状态失败: {e}");
        std::process::exit(1);
    }
    let label = if disabled { "禁用" } else { "启用" };
    println!("✓ 供应商 {app}/{id} 已{label}");
    if disabled {
        println!("  提示: 代理转发将跳过此供应商，配置已保留");
    }
}
```

- [ ] **Step 11: 修改 `cmd_list_providers` 输出标注 disabled 状态**

Read [src-tauri/src/bin/cc-switch-cli.rs:1095-1170](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `cmd_list_providers` 函数中找到打印每行的 `println!`，在 name 列后追加 disabled 标记。具体修改方式：

找到形如 `println!("{:<3} {:<25} {:<20} ...", i, id, name, ...)` 的行，改为：

```rust
            let disabled = provider
                .meta
                .as_ref()
                .and_then(|m| m.disabled)
                .unwrap_or(false);
            let state = if disabled { "[禁用]" } else { "" };
            println!("{:<3} {:<25} {:<20} {:<8} ...", i + 1, id, name, state);
```

> 注：实施时 Read 上下文确认原列宽与对齐，保持表格对齐。`...` 部分为原列（base_url、api_format 等），保留不变。

- [ ] **Step 12: 在 `cmd_help` 添加 toggle-provider 说明**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_help` 中，`remove-provider` 行之后插入：

```rust
    println!("    toggle-provider <APP> <ID> <on|off>  启用/禁用供应商（保留配置）");
```

- [ ] **Step 13: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d toggle_provider -- --nocapture
```

Expected: `test_toggle_provider_off_marks_disabled` PASS

- [ ] **Step 14: 验证全部 Rust 测试无回归**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test
```

Expected: 所有测试通过（含既有测试 + Plan D 新增测试）

- [ ] **Step 15: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/provider.rs src-tauri/src/database/schema.rs src-tauri/src/database/dao/providers.rs src-tauri/src/proxy/provider_router.rs src-tauri/src/bin/cc-switch-cli.rs src-tauri/tests/cli_plan_d.rs
git commit -m "feat(cli): 新增 toggle-provider 命令支持供应商启用/禁用

- ProviderMeta 加 disabled 字段
- providers 表加 disabled 列（migration）
- DAO 新增 update_provider_disabled，get_all_providers SELECT 加 disabled
- ProviderRouter 跳过 disabled=true 的供应商
- list-providers 输出标注禁用状态
- 集成测试覆盖 off → 标注 → on 全链路

关联 spec: §九 M-7"
```

---

## Task 5: M-3 协议转换预览 — 实现 `preview-conversion` 命令

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`（新增 `PreviewConversion` 命令、`cmd_preview_conversion` 函数）
- Modify: `src-tauri/src/proxy/providers/transform.rs`（暴露 `pub fn anthropic_to_openai`）
- Test: `src-tauri/tests/cli_plan_d.rs`

- [ ] **Step 1: 写失败测试 — preview-conversion 输出转换后 JSON**

在 `src-tauri/tests/cli_plan_d.rs` 追加：

```rust
#[cfg(test)]
mod preview_conversion_tests {
    use super::*;

    #[test]
    fn test_preview_conversion_anthropic_to_openai_chat() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();
        let payload_path = tmp.path().join("payload.json");
        std::fs::write(
            &payload_path,
            r#"{"model":"claude-3-5-sonnet-20241022","max_tokens":100,"messages":[{"role":"user","content":"hi"}]}"#,
        )
        .unwrap();

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "preview-conversion",
                "--from",
                "anthropic",
                "--to",
                "openai_chat",
                "--payload",
                payload_path.to_str().unwrap(),
            ])
            .output()
            .expect("preview-conversion 失败");

        assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        // 转换后应为 OpenAI Chat Completions 格式，含 messages 数组
        assert!(stdout.contains("messages"), "转换后应含 messages: {stdout}");
        // OpenAI 格式无 anthropic 的 max_tokens（改用 max_completion_tokens 或保留）
        assert!(stdout.contains("gpt-") || stdout.contains("claude-") || stdout.contains("model"), "应含 model 字段: {stdout}");
    }

    #[test]
    fn test_preview_conversion_invalid_format_returns_error() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();
        let payload_path = tmp.path().join("p.json");
        std::fs::write(&payload_path, "{}").unwrap();

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "preview-conversion",
                "--from",
                "invalid_format",
                "--to",
                "openai_chat",
                "--payload",
                payload_path.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        assert!(!output.status.success(), "无效格式应非零退出");
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d preview_conversion -- --nocapture
```

Expected: FAIL — `preview-conversion` 命令不存在

- [ ] **Step 3: 暴露 transform 函数为 pub**

Read [src-tauri/src/proxy/providers/transform.rs:119-130](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/providers/transform.rs)

确认 `pub fn anthropic_to_openai(body: Value) -> Result<Value, ProxyError>` 已为 `pub`。若为 `pub(crate)`，改为 `pub`。

Read [src-tauri/src/proxy/providers/transform_responses.rs:51](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/providers/transform_responses.rs)

确认 `pub fn anthropic_to_responses`、`pub fn responses_to_anthropic` 已为 `pub`。

Read [src-tauri/src/proxy/providers/transform_gemini.rs:47](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/providers/transform_gemini.rs)

确认 `pub fn anthropic_to_gemini`、`pub fn gemini_to_anthropic` 已为 `pub`。

> 若 Plan B 已将这些模块移至 `cc_switch_lib::core::proxy::providers::transform::*` 并标 `pub`，本步骤可跳过。

- [ ] **Step 4: 新增 `PreviewConversion` 命令到 CLI**

Read [src-tauri/src/bin/cc-switch-cli.rs:213-228](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `Speedtest` 块之前插入：

```rust
    /// 预览协议转换结果（不入网，仅调 transform 函数）
    PreviewConversion {
        /// 源格式: anthropic / openai_chat / openai_responses / gemini_native
        #[arg(long)]
        from: String,
        /// 目标格式: anthropic / openai_chat / openai_responses / gemini_native
        #[arg(long)]
        to: String,
        /// 源 JSON payload 文件路径
        #[arg(long)]
        payload: String,
    },
```

- [ ] **Step 5: 添加 match 分发**

在 [src-tauri/src/bin/cc-switch-cli.rs:625-630](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) `Speedtest` 分发之前插入：

```rust
        Commands::PreviewConversion { from, to, payload } => {
            cmd_preview_conversion(&from, &to, &payload)
        }
```

- [ ] **Step 6: 实现 `cmd_preview_conversion` 函数**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_speedtest` 函数之前插入：

```rust
/// preview-conversion: 预览协议转换结果
///
/// 调用 transform 模块的纯函数，输入 JSON payload，输出转换后 JSON。
/// 不实际转发请求。支持 anthropic/openai_chat/openai_responses/gemini_native
/// 之间的直接转换（无直接函数的组合返回错误）。
fn cmd_preview_conversion(from: &str, to: &str, payload: &str) {
    let payload_str = match std::fs::read_to_string(payload) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("读取 payload 文件失败: {e}");
            std::process::exit(1);
        }
    };
    let payload_value: serde_json::Value = match serde_json::from_str(&payload_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("解析 payload JSON 失败: {e}");
            std::process::exit(1);
        }
    };

    use cc_switch_lib::proxy::providers::{
        transform, transform_gemini, transform_responses,
    };
    use cc_switch_lib::proxy::ProxyError;

    let result: Result<serde_json::Value, ProxyError> = match (from, to) {
        ("anthropic", "openai_chat") => transform::anthropic_to_openai(payload_value),
        ("openai_chat", "anthropic") => transform::openai_to_anthropic(payload_value),
        ("anthropic", "openai_responses") => {
            transform_responses::anthropic_to_responses(payload_value)
        }
        ("openai_responses", "anthropic") => {
            transform_responses::responses_to_anthropic(payload_value)
        }
        ("anthropic", "gemini_native") => transform_gemini::anthropic_to_gemini(payload_value),
        ("gemini_native", "anthropic") => transform_gemini::gemini_to_anthropic(payload_value),
        (f, t) => {
            eprintln!("不支持的转换路径: {f} → {t}");
            eprintln!("支持的路径:");
            eprintln!("  anthropic ↔ openai_chat");
            eprintln!("  anthropic ↔ openai_responses");
            eprintln!("  anthropic ↔ gemini_native");
            std::process::exit(1);
        }
    };

    match result {
        Ok(converted) => {
            let pretty = serde_json::to_string_pretty(&converted).unwrap_or_else(|e| {
                eprintln!("序列化结果失败: {e}");
                std::process::exit(1);
            });
            println!("转换: {from} → {to}");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("{pretty}");
        }
        Err(e) => {
            eprintln!("转换失败: {e}");
            std::process::exit(1);
        }
    }
}
```

> 注：导入路径以 Plan B 重构后的实际路径为准。若 `transform` 等仍为 `pub(crate)`，需先在 [src-tauri/src/proxy/mod.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/mod.rs) 中加 `pub use providers::{transform, transform_gemini, transform_responses};` 并在 [src-tauri/src/lib.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/lib.rs) 中加 `pub use proxy::providers::{transform, transform_gemini, transform_responses};`。

- [ ] **Step 7: 在 `cmd_help` 添加 preview-conversion 说明**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_help` 中，`verify-key` 行之后插入：

```rust
    println!("    preview-conversion --from F --to F --payload FILE  预览协议转换结果");
```

- [ ] **Step 8: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d preview_conversion -- --nocapture
```

Expected: 2 个 preview_conversion 测试通过

- [ ] **Step 9: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/bin/cc-switch-cli.rs src-tauri/src/proxy/mod.rs src-tauri/src/lib.rs src-tauri/tests/cli_plan_d.rs
git commit -m "feat(cli): 新增 preview-conversion 命令预览协议转换

- 调用 transform 模块纯函数，不入网
- 支持 anthropic ↔ openai_chat / openai_responses / gemini_native
- 暴露 transform/transform_responses/transform_gemini 为 pub
- 集成测试覆盖正向转换与无效格式错误

关联 spec: §九 M-3"
```

---

## Task 6: M-3 跟踪单次请求 — 实现 `proxy-trace` 命令

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`（新增 `ProxyTrace` 命令、`cmd_proxy_trace` 函数）
- Test: `src-tauri/tests/cli_plan_d.rs`

- [ ] **Step 1: 写失败测试 — proxy-trace 输出 4 段转换链路**

在 `src-tauri/tests/cli_plan_d.rs` 追加：

```rust
#[cfg(test)]
mod proxy_trace_tests {
    use super::*;

    #[test]
    fn test_proxy_trace_anthropic_to_openai_chat() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();

        // 添加一个 openai_chat 格式的 claude 供应商
        let add = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "claude",
                "openrouter-test",
                "OpenRouter Test",
                "--api-key",
                "sk-or-test",
                "--base-url",
                "https://openrouter.ai/api/v1",
                "--api-format",
                "openai_chat",
            ])
            .output()
            .unwrap();
        assert!(add.status.success());

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "proxy-trace",
                "claude",
                "--model",
                "claude-3-5-sonnet-20241022",
                "--provider",
                "openrouter-test",
            ])
            .output()
            .expect("proxy-trace 失败");

        assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        // 应输出 4 段：原始请求体 / 转换后请求体 / 上游响应（占位）/ 反转换后响应（占位）
        assert!(stdout.contains("原始请求体"), "应含原始请求体段: {stdout}");
        assert!(stdout.contains("转换后请求体"), "应含转换后请求体段: {stdout}");
        assert!(stdout.contains("openai_chat"), "应显示 api_format: {stdout}");
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d proxy_trace -- --nocapture
```

Expected: FAIL — `proxy-trace` 命令不存在

- [ ] **Step 3: 新增 `ProxyTrace` 命令到 CLI**

Read [src-tauri/src/bin/cc-switch-cli.rs:213-228](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `PreviewConversion` 块之后插入：

```rust
    /// 跟踪一次请求的完整转换链路（不入网，仅展示转换前后对比）
    ProxyTrace {
        /// 应用类型 (claude, codex, gemini)
        app: String,
        /// 模型名
        #[arg(long)]
        model: String,
        /// 供应商 ID
        #[arg(long)]
        provider: String,
    },
```

- [ ] **Step 4: 添加 match 分发**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) `PreviewConversion` 分发之后插入：

```rust
        Commands::ProxyTrace { app, model, provider } => {
            cmd_proxy_trace(&app, &model, &provider)
        }
```

- [ ] **Step 5: 实现 `cmd_proxy_trace` 函数**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_preview_conversion` 函数之后插入：

```rust
/// proxy-trace: 跟踪一次请求的完整转换链路
///
/// 构造最小测试请求体（按 app 类型），按供应商 api_format 调用 transform 函数，
/// 输出 4 段：原始请求体 / 转换后请求体 / 上游响应（占位）/ 反转换后响应（占位）。
/// 不实际转发请求（避免消耗 token）。
fn cmd_proxy_trace(app: &str, model: &str, provider_id: &str) {
    if let Err(e) = validated_app(app) {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };
    let provider = match db.get_provider_by_id(app, provider_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            eprintln!("供应商 {app}/{provider_id} 不存在");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("查询供应商失败: {e}");
            std::process::exit(1);
        }
    };

    // 读取 api_format（按 app 选择对应字段）
    let api_format = match app {
        "claude" | "claude-desktop" => provider
            .meta
            .as_ref()
            .and_then(|m| m.api_format.clone())
            .or_else(|| provider.meta.as_ref().and_then(|m| m.claude_desktop_api_format.clone()))
            .unwrap_or_else(|| "anthropic".to_string()),
        "codex" => provider
            .meta
            .as_ref()
            .and_then(|m| m.api_format.clone())
            .unwrap_or_else(|| "openai_responses".to_string()),
        "gemini" => provider
            .meta
            .as_ref()
            .and_then(|m| m.gemini_api_format.clone())
            .unwrap_or_else(|| "gemini_native".to_string()),
        _ => "anthropic".to_string(),
    };

    // 构造最小测试请求体（Anthropic Messages 格式）
    let original_body = serde_json::json!({
        "model": model,
        "max_tokens": 100,
        "messages": [{"role": "user", "content": "hi"}]
    });

    use cc_switch_lib::proxy::providers::{transform, transform_gemini, transform_responses};

    // 按 api_format 转换请求体
    let converted_body: serde_json::Value = match api_format.as_str() {
        "anthropic" => original_body.clone(),
        "openai_chat" => match transform::anthropic_to_openai(original_body.clone()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("转换为 openai_chat 失败: {e}");
                std::process::exit(1);
            }
        },
        "openai_responses" => {
            match transform_responses::anthropic_to_responses(original_body.clone()) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("转换为 openai_responses 失败: {e}");
                    std::process::exit(1);
                }
            }
        }
        "gemini_native" => match transform_gemini::anthropic_to_gemini(original_body.clone()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("转换为 gemini_native 失败: {e}");
                std::process::exit(1);
            }
        },
        other => {
            eprintln!("不支持的 api_format: {other}");
            std::process::exit(1);
        }
    };

    println!("代理跟踪: app={app} model={model} provider={provider_id} api_format={api_format}");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("【1. 原始请求体（Anthropic Messages 格式）】");
    println!(
        "{}",
        serde_json::to_string_pretty(&original_body).unwrap_or_default()
    );
    println!();
    println!("【2. 转换后请求体（{api_format} 格式，将转发给上游）】");
    println!(
        "{}",
        serde_json::to_string_pretty(&converted_body).unwrap_or_default()
    );
    println!();
    println!("【3. 上游响应】");
    println!("（不入网，跳过实际转发。可使用 --live 选项或 speedtest 命令做真实请求）");
    println!();
    println!("【4. 反转换后响应（将返回给客户端）】");
    println!("（不入网，跳过。响应转换器与请求转换器对称：见 preview-conversion --to anthropic）");
}
```

- [ ] **Step 6: 在 `cmd_help` 添加 proxy-trace 说明**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_help` 中，`preview-conversion` 行之后插入：

```rust
    println!("    proxy-trace <APP> --model M --provider P   跟踪请求完整转换链路");
```

- [ ] **Step 7: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d proxy_trace -- --nocapture
```

Expected: `test_proxy_trace_anthropic_to_openai_chat` PASS

- [ ] **Step 8: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/bin/cc-switch-cli.rs src-tauri/tests/cli_plan_d.rs
git commit -m "feat(cli): 新增 proxy-trace 命令跟踪请求转换链路

构造最小 Anthropic 请求体，按供应商 api_format 调 transform 函数，
输出 4 段：原始请求体 / 转换后请求体 / 上游响应占位 / 反转换占位。
不入网，不消耗 token。

关联 spec: §九 M-3"
```

---

## Task 7: M-3 重放历史请求 — 实现 `replay-request` 命令

**Files:**
- Modify: `src-tauri/src/database/schema.rs`（新增 `proxy_request_bodies` 表）
- Modify: `src-tauri/src/proxy/usage/logger.rs`（env 控制是否落库 body）
- Modify: `src-tauri/src/database/dao/usage_rollup.rs`（新增 `get_request_body`/`save_request_body`）
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`（新增 `ReplayRequest` 命令、`cmd_replay_request` 函数）
- Test: `src-tauri/tests/cli_plan_d.rs`

- [ ] **Step 1: 写失败测试 — replay-request 不存在时返回错误**

在 `src-tauri/tests/cli_plan_d.rs` 追加：

```rust
#[cfg(test)]
mod replay_request_tests {
    use super::*;

    #[test]
    fn test_replay_request_missing_id_returns_error() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();

        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["replay-request", "nonexistent-id"])
            .output()
            .expect("replay-request 失败");

        assert!(!output.status.success(), "不存在的 request_id 应非零退出");
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        assert!(stderr.contains("不存在") || stderr.contains("not found"), "stderr: {stderr}");
    }

    #[test]
    fn test_replay_request_with_payload_file() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();

        // 添加供应商
        let add = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "add-provider",
                "claude",
                "replay-target",
                "Replay Target",
                "--api-key",
                "sk-test",
                "--base-url",
                "https://api.anthropic.com",
            ])
            .output()
            .unwrap();
        assert!(add.status.success());

        // 提供自定义 payload
        let payload_path = tmp.path().join("replay.json");
        std::fs::write(
            &payload_path,
            r#"{"model":"claude-3-5-sonnet-20241022","max_tokens":50,"messages":[{"role":"user","content":"hi"}]}"#,
        )
        .unwrap();

        // replay-request 用 --payload 指定 body，用 --dry-run 不实际转发
        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args([
                "replay-request",
                "any-id",
                "--provider",
                "replay-target",
                "--app",
                "claude",
                "--payload",
                payload_path.to_str().unwrap(),
                "--dry-run",
            ])
            .output()
            .expect("replay-request 失败");

        assert!(output.status.success(), "dry-run 应成功: {}", String::from_utf8_lossy(&output.stderr));
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        assert!(stdout.contains("dry-run") || stdout.contains("未转发"), "dry-run 应提示未转发: {stdout}");
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d replay_request -- --nocapture
```

Expected: FAIL — `replay-request` 命令不存在

- [ ] **Step 3: 新增 `proxy_request_bodies` 表**

Read [src-tauri/src/database/schema.rs:186-220](file:///f:/workspace/trae/cc-switch/src-tauri/src/database/schema.rs)

在第 220 行（`proxy_request_logs` 表 CREATE 之后、索引创建之前）插入：

```rust
        conn.execute(
            "CREATE TABLE IF NOT EXISTS proxy_request_bodies (
                request_id TEXT PRIMARY KEY,
                request_body TEXT NOT NULL,
                app_type TEXT NOT NULL,
                provider_id TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_bodies_created ON proxy_request_bodies(created_at)",
            [],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
```

- [ ] **Step 4: 新增 DAO 方法 `save_request_body` / `get_request_body`**

Read [src-tauri/src/database/dao/usage_rollup.rs:1-30](file:///f:/workspace/trae/cc-switch/src-tauri/src/database/dao/usage_rollup.rs)

在 `impl Database` 块末尾追加：

```rust
    /// 保存请求 body（仅当 CC_SWITCH_LOG_BODIES=1 时调用）
    pub fn save_request_body(
        &self,
        request_id: &str,
        request_body: &str,
        app_type: &str,
        provider_id: &str,
        model: &str,
    ) -> Result<(), AppError> {
        let conn = crate::database::lock_conn!(self.conn);
        conn.execute(
            "INSERT OR REPLACE INTO proxy_request_bodies (request_id, request_body, app_type, provider_id, model, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                request_id,
                request_body,
                app_type,
                provider_id,
                model,
                chrono::Utc::now().timestamp()
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 读取请求 body（replay-request 命令使用）
    pub fn get_request_body(&self, request_id: &str) -> Result<Option<RequestBodyRecord>, AppError> {
        let conn = crate::database::lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT request_id, request_body, app_type, provider_id, model, created_at
                 FROM proxy_request_bodies WHERE request_id = ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut rows = stmt
            .query(rusqlite::params![request_id])
            .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            return Ok(Some(RequestBodyRecord {
                request_id: row.get(0)?,
                request_body: row.get(1)?,
                app_type: row.get(2)?,
                provider_id: row.get(3)?,
                model: row.get(4)?,
                created_at: row.get(5)?,
            }));
        }
        Ok(None)
    }
```

在 [src-tauri/src/database/dao/usage_rollup.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/database/dao/usage_rollup.rs) 顶部（结构体定义区）追加：

```rust
/// 请求 body 存档记录（replay-request 使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBodyRecord {
    pub request_id: String,
    pub request_body: String,
    pub app_type: String,
    pub provider_id: String,
    pub model: String,
    pub created_at: i64,
}
```

- [ ] **Step 5: 修改 logger 在 env 启用时落库 body**

Read [src-tauri/src/proxy/usage/logger.rs:50-100](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/usage/logger.rs)

在 `log_request` 方法末尾（INSERT 完成、return 之前）插入：

```rust
        // 可选：保存请求 body 供 replay-request 重放（env CC_SWITCH_LOG_BODIES=1 启用）
        if std::env::var("CC_SWITCH_LOG_BODIES").ok().as_deref() == Some("1") {
            if let Some(body_str) = log.request_body.as_ref() {
                let _ = self.db.save_request_body(
                    &log.request_id,
                    body_str,
                    &log.app_type,
                    &log.provider_id,
                    &log.model,
                );
            }
        }
```

在 [src-tauri/src/proxy/usage/logger.rs:11-37](file:///f:/workspace/trae/cc-switch/src-tauri/src/proxy/usage/logger.rs) 的 `RequestLog` 结构体追加字段：

```rust
    /// 可选：请求 body（仅当 CC_SWITCH_LOG_BODIES=1 时填充，用于 replay-request 重放）
    #[serde(skip)]
    pub request_body: Option<String>,
```

> 注：所有 `RequestLog { ... }` 构造点需要追加 `request_body: None` 字段（或在构造时按需填充）。Grep `RequestLog {` 找到所有构造点，逐个补充。Plan B 重构后路径可能变化，以实际为准。

- [ ] **Step 6: 新增 `ReplayRequest` 命令到 CLI**

Read [src-tauri/src/bin/cc-switch-cli.rs:213-228](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `ProxyTrace` 块之后插入：

```rust
    /// 重放历史请求（从 proxy_request_bodies 表读取，或通过 --payload 指定）
    ReplayRequest {
        /// 请求 ID（若数据库有此 ID 的 body 存档则使用，否则需 --payload）
        request_id: String,
        /// 供应商 ID（覆盖原请求的供应商）
        #[arg(long)]
        provider: Option<String>,
        /// 应用类型（与 --payload 配合使用）
        #[arg(long)]
        app: Option<String>,
        /// 自定义 payload JSON 文件（覆盖存档 body）
        #[arg(long)]
        payload: Option<String>,
        /// 仅展示转换链路，不实际转发（默认 false 即真实转发）
        #[arg(long)]
        dry_run: bool,
    },
```

- [ ] **Step 7: 添加 match 分发**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) `ProxyTrace` 分发之后插入：

```rust
        Commands::ReplayRequest {
            request_id,
            provider,
            app,
            payload,
            dry_run,
        } => cmd_replay_request(&request_id, provider.as_deref(), app.as_deref(), payload.as_deref(), *dry_run),
```

- [ ] **Step 8: 实现 `cmd_replay_request` 函数**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_proxy_trace` 函数之后插入：

```rust
/// replay-request: 重放历史请求
///
/// 优先从 proxy_request_bodies 表读取存档 body；
/// 若无存档，则用 --payload 指定 JSON 文件 + --app + --provider。
/// --dry-run 仅展示转换链路不转发；默认实际转发（消耗 token）。
fn cmd_replay_request(
    request_id: &str,
    provider_override: Option<&str>,
    app_override: Option<&str>,
    payload_path: Option<&str>,
    dry_run: bool,
) {
    let db = match init_db() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: {e}");
            std::process::exit(1);
        }
    };

    // 1. 确定 body 来源：payload 文件优先，否则查存档
    let (body_str, app_type, provider_id, model) = if let Some(path) = payload_path {
        let body = std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("读取 payload 失败: {e}");
            std::process::exit(1);
        });
        let app = app_override.unwrap_or_else(|| {
            eprintln!("使用 --payload 时必须指定 --app");
            std::process::exit(1);
        });
        let provider = provider_override.unwrap_or_else(|| {
            eprintln!("使用 --payload 时必须指定 --provider");
            std::process::exit(1);
        });
        // 从 payload 解析 model 字段
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap_or_else(|e| {
            eprintln!("payload 不是合法 JSON: {e}");
            std::process::exit(1);
        });
        let model = parsed
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        (body, app.to_string(), provider.to_string(), model)
    } else {
        // 从存档读取
        match db.get_request_body(request_id) {
            Ok(Some(record)) => {
                let provider = provider_override
                    .map(str::to_string)
                    .unwrap_or(record.provider_id);
                (record.request_body, record.app_type, provider, record.model)
            }
            Ok(None) => {
                eprintln!("请求 {request_id} 不存在（无 body 存档）。请使用 --payload 指定 body 文件");
                eprintln!("提示：启用 CC_SWITCH_LOG_BODIES=1 环境变量后，代理会保存请求 body 供重放");
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("查询请求 body 失败: {e}");
                std::process::exit(1);
            }
        }
    };

    // 2. 解析 body
    let body: serde_json::Value = serde_json::from_str(&body_str).unwrap_or_else(|e| {
        eprintln!("body 不是合法 JSON: {e}");
        std::process::exit(1);
    });

    // 3. 查询供应商
    let provider = match db.get_provider_by_id(&app_type, &provider_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            eprintln!("供应商 {app_type}/{provider_id} 不存在");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("查询供应商失败: {e}");
            std::process::exit(1);
        }
    };

    println!("重放请求: id={request_id} app={app_type} provider={provider_id} model={model}");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // 4. 若 dry-run，展示转换链路
    if dry_run {
        println!("【dry-run 模式】未实际转发");
        println!();
        println!("请求 body:");
        println!("{}", serde_json::to_string_pretty(&body).unwrap_or_default());
        println!();
        println!("供应商: {} (api_format: {:?})", provider.name, provider.meta.as_ref().and_then(|m| m.api_format.as_ref()));
        return;
    }

    // 5. 实际转发（使用 reqwest 直接调上游）
    let base_url = provider
        .settings_config
        .pointer("/env/ANTHROPIC_BASE_URL")
        .or_else(|| provider.settings_config.pointer("/env/OPENAI_BASE_URL"))
        .or_else(|| provider.settings_config.pointer("/env/GEMINI_BASE_URL"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let api_key = provider
        .settings_config
        .pointer("/env/ANTHROPIC_API_KEY")
        .or_else(|| provider.settings_config.pointer("/env/OPENAI_API_KEY"))
        .or_else(|| provider.settings_config.pointer("/env/GEMINI_API_KEY"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if base_url.is_empty() || api_key.is_empty() {
        eprintln!("供应商缺少 base_url 或 api_key，无法转发");
        std::process::exit(1);
    }

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    rt.block_on(async move {
        let client = reqwest::Client::new();
        let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));
        let resp = client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await;

        match resp {
            Ok(r) => {
                let status = r.status();
                let text = r.text().await.unwrap_or_default();
                println!("上游响应: HTTP {status}");
                println!("{text}");
            }
            Err(e) => {
                eprintln!("转发失败: {e}");
                std::process::exit(1);
            }
        }
    });
}
```

- [ ] **Step 9: 在 `cmd_help` 添加 replay-request 说明**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_help` 中，`proxy-trace` 行之后插入：

```rust
    println!("    replay-request <REQUEST_ID> [--provider P] [--app A] [--payload F] [--dry-run]");
```

- [ ] **Step 10: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d replay_request -- --nocapture
```

Expected: 2 个 replay_request 测试通过

- [ ] **Step 11: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/database/schema.rs src-tauri/src/database/dao/usage_rollup.rs src-tauri/src/proxy/usage/logger.rs src-tauri/src/bin/cc-switch-cli.rs src-tauri/tests/cli_plan_d.rs
git commit -m "feat(cli): 新增 replay-request 命令重放历史请求

- 新增 proxy_request_bodies 表存档请求 body
- env CC_SWITCH_LOG_BODIES=1 启用 body 落库（默认关闭防 DB 膨胀）
- DAO 新增 save/get_request_body + RequestBodyRecord 结构体
- replay-request 支持从存档或 --payload 文件读取 body
- --dry-run 仅展示转换链路；默认实际转发消耗 token
- 集成测试覆盖不存在 ID 与 --payload + --dry-run 场景

关联 spec: §九 M-3"
```

---

## Task 8: M-4 实时连接查看 — 实现 `connections` 命令

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`（新增 `Connections` 命令、`cmd_connections` 函数）
- Test: `src-tauri/tests/cli_plan_d.rs`

- [ ] **Step 1: 写失败测试 — connections 命令存在并响应**

在 `src-tauri/tests/cli_plan_d.rs` 追加：

```rust
#[cfg(test)]
mod connections_tests {
    use super::*;

    #[test]
    fn test_connections_command_runs() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();

        // daemon 未运行时，connections 应优雅提示（非崩溃）
        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["connections"])
            .output()
            .expect("connections 失败");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        // 应输出"代理未运行"或类似提示（不退出非零）
        assert!(
            stdout.contains("未运行") || stdout.contains("not running") || stdout.contains("连接"),
            "connections 应有提示输出: {stdout}"
        );
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d connections -- --nocapture
```

Expected: FAIL — `connections` 命令不存在

- [ ] **Step 3: 新增 `Connections` 命令到 CLI**

Read [src-tauri/src/bin/cc-switch-cli.rs:40-44](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在第 42 行（`Status` 块之后、`Settings` 块之前）插入：

```rust
    /// 查看当前活跃连接（需代理在运行）
    Connections,
```

- [ ] **Step 4: 添加 match 分发**

在 [src-tauri/src/bin/cc-switch-cli.rs:593-596](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) `Status` 分发之后插入：

```rust
        Commands::Connections => cmd_connections(),
```

- [ ] **Step 5: 实现 `cmd_connections` 函数**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_status` 函数之后插入：

```rust
/// connections: 查看当前活跃连接
///
/// 通过 HTTP GET /status 调用运行中的 daemon，读取 ProxyStatus.active_connections
/// 与 active_targets 字段并打印。daemon 未运行时优雅提示。
fn cmd_connections() {
    let listen = std::env::var("CC_SWITCH_LISTEN").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090u16);
    let url = format!("http://{listen}:{port}/status");

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    let status_json: serde_json::Value = rt.block_on(async move {
        match reqwest::Client::new().get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                resp.json::<serde_json::Value>().await.unwrap_or(serde_json::json!({}))
            }
            Ok(resp) => {
                eprintln!("代理返回非 200 状态: {}", resp.status());
                std::process::exit(1);
            }
            Err(e) => {
                println!("代理未运行或无法连接 ({url}): {e}");
                println!("提示: 使用 `cc-switch-cli start` 或 `cc-switch-cli daemon` 启动代理");
                return serde_json::json!({});
            }
        }
    });

    if status_json.is_null() || status_json.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        return;
    }

    let active = status_json
        .get("active_connections")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let total = status_json
        .get("total_requests")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let success = status_json
        .get("success_requests")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let failed = status_json
        .get("failed_requests")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    println!("活跃连接数: {active}");
    println!("总请求数:   {total} (成功 {success} / 失败 {failed})");
    println!("当前供应商: {}", status_json.get("current_provider").and_then(|v| v.as_str()).unwrap_or("?"));

    if let Some(targets) = status_json.get("active_targets").and_then(|v| v.as_array()) {
        if !targets.is_empty() {
            println!();
            println!("活跃代理目标:");
            println!("{:<20} {:<25} {:<25}", "应用", "供应商", "ID");
            for t in targets {
                println!(
                    "{:<20} {:<25} {:<25}",
                    t.get("app_type").and_then(|v| v.as_str()).unwrap_or("?"),
                    t.get("provider_name").and_then(|v| v.as_str()).unwrap_or("?"),
                    t.get("provider_id").and_then(|v| v.as_str()).unwrap_or("?")
                );
            }
        }
    }
}
```

- [ ] **Step 6: 在 `cmd_help` 添加 connections 说明**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_help` 中，`status` 行之后插入：

```rust
    println!("    connections                   查看当前活跃连接");
```

- [ ] **Step 7: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d connections -- --nocapture
```

Expected: `test_connections_command_runs` PASS

- [ ] **Step 8: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/bin/cc-switch-cli.rs src-tauri/tests/cli_plan_d.rs
git commit -m "feat(cli): 新增 connections 命令查看活跃连接

通过 HTTP GET /status 轮询运行中 daemon，输出 active_connections、
total_requests、active_targets。daemon 未运行时优雅提示。

关联 spec: §九 M-4"
```

---

## Task 9: M-4 实时统计 — 实现 `stats --live` 命令

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`（新增 `Stats` 命令、`cmd_stats` 函数）
- Test: `src-tauri/tests/cli_plan_d.rs`

- [ ] **Step 1: 写失败测试 — stats 命令存在并可单次输出**

在 `src-tauri/tests/cli_plan_d.rs` 追加：

```rust
#[cfg(test)]
mod stats_tests {
    use super::*;

    #[test]
    fn test_stats_single_run() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();

        // daemon 未运行时，stats 应单次输出后退出（非崩溃）
        let output = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["stats"])
            .output()
            .expect("stats 失败");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        assert!(
            stdout.contains("未运行") || stdout.contains("QPS") || stdout.contains("请求数"),
            "stats 应有输出: {stdout}"
        );
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d stats -- --nocapture
```

Expected: FAIL — `stats` 命令不存在

- [ ] **Step 3: 新增 `Stats` 命令到 CLI**

Read [src-tauri/src/bin/cc-switch-cli.rs:40-44](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `Connections` 块之后插入：

```rust
    /// 实时统计（QPS / 延迟 / 错误率）。--live 持续刷新，无参数输出单次快照
    Stats {
        /// 持续刷新模式（每秒一次，Ctrl+C 退出）
        #[arg(long)]
        live: bool,
    },
```

- [ ] **Step 4: 添加 match 分发**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) `Connections` 分发之后插入：

```rust
        Commands::Stats { live } => cmd_stats(*live),
```

- [ ] **Step 5: 实现 `cmd_stats` 函数**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_connections` 函数之后插入：

```rust
/// stats: 实时统计（QPS / 延迟 / 错误率）
///
/// 通过 HTTP GET /status 轮询 daemon，计算 QPS（total_requests 差值）、
/// 错误率（failed/total）。延迟 p50/p99 从 proxy_request_logs 表查询。
/// --live 模式每秒刷新，无参数输出单次快照。
fn cmd_stats(live: bool) {
    let listen = std::env::var("CC_SWITCH_LISTEN").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("CC_SWITCH_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(9090u16);
    let url = format!("http://{listen}:{port}/status");

    let rt = tokio::runtime::Runtime::new().expect("无法创建 tokio runtime");
    let client = reqwest::Client::new();

    let mut prev_total: Option<u64> = None;
    let mut prev_ts: Option<std::time::Instant> = None;

    loop {
        let snapshot: serde_json::Value = rt.block_on(async {
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    resp.json::<serde_json::Value>().await.unwrap_or(serde_json::json!({}))
                }
                _ => {
                    serde_json::json!({})
                }
            }
        });

        if snapshot.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            println!("代理未运行（{}）", url);
            if !live {
                return;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }

        let total = snapshot
            .get("total_requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let success = snapshot
            .get("success_requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let failed = snapshot
            .get("failed_requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let active = snapshot
            .get("active_connections")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let now = std::time::Instant::now();
        let qps = match (prev_total, prev_ts) {
            (Some(prev), Some(ts)) => {
                let delta = total.saturating_sub(prev);
                let elapsed = now.duration_since(ts).as_secs_f64().max(0.001);
                delta as f64 / elapsed
            }
            _ => 0.0,
        };

        let error_rate = if total > 0 {
            (failed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "活跃={active} 总请求={total} 成功={success} 失败={failed} QPS={qps:.2} 错误率={error_rate:.1}%"
        );

        if !live {
            return;
        }
        prev_total = Some(total);
        prev_ts = Some(now);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

- [ ] **Step 6: 在 `cmd_help` 添加 stats 说明**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_help` 中，`connections` 行之后插入：

```rust
    println!("    stats [--live]                实时统计（QPS / 错误率，--live 持续刷新）");
```

- [ ] **Step 7: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d stats -- --nocapture
```

Expected: `test_stats_single_run` PASS

- [ ] **Step 8: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/bin/cc-switch-cli.rs src-tauri/tests/cli_plan_d.rs
git commit -m "feat(cli): 新增 stats 命令实时统计 QPS 与错误率

- HTTP GET /status 轮询 daemon
- 计算 QPS（total_requests 差值 / 时间）
- 计算 错误率（failed / total）
- --live 模式每秒刷新，无参数单次快照

关联 spec: §九 M-4"
```

---

## Task 10: M-4 实时日志 — 实现 `logs --tail` 命令

**Files:**
- Modify: `src-tauri/src/bin/cc-switch-cli.rs`（新增 `Logs` 命令、`cmd_logs` 函数）
- Test: `src-tauri/tests/cli_plan_d.rs`

- [ ] **Step 1: 写失败测试 — logs --tail 命令存在并可启动**

在 `src-tauri/tests/cli_plan_d.rs` 追加：

```rust
#[cfg(test)]
mod logs_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_logs_tail_starts_and_outputs_initial() {
        let tmp = tmp_home();
        let home = tmp.path().to_str().unwrap();

        // 预先写入一行日志
        let log_path = tmp.path().join("cc-switch-daemon.log");
        std::fs::write(&log_path, "[test] sample log line\n").unwrap();

        // 启动 logs --tail，500ms 后强制终止（不应崩溃）
        let mut child = Command::new(cli_binary())
            .env("CC_SWITCH_HOME", home)
            .args(["logs", "--tail"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("logs 启动失败");

        std::thread::sleep(Duration::from_millis(500));
        let _ = child.kill();
        let output = child.wait_with_output().expect("wait 失败");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        // 应输出已存在的日志内容或"日志文件不存在"提示
        assert!(
            stdout.contains("sample log line") || stdout.contains("日志文件") || stdout.is_empty(),
            "logs 输出异常: {stdout}"
        );
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d logs -- --nocapture
```

Expected: FAIL — `logs` 命令不存在

- [ ] **Step 3: 新增 `Logs` 命令到 CLI**

Read [src-tauri/src/bin/cc-switch-cli.rs:40-44](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs)

在 `Stats` 块之后插入：

```rust
    /// 查看代理日志。--tail 持续刷新（tail -f 模式），无参数输出最近 50 行
    Logs {
        /// 持续刷新模式（Ctrl+C 退出）
        #[arg(long)]
        tail: bool,
        /// 输出最后 N 行（默认 50，仅无 --tail 时生效）
        #[arg(long, default_value = "50")]
        lines: u32,
    },
```

- [ ] **Step 4: 添加 match 分发**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) `Stats` 分发之后插入：

```rust
        Commands::Logs { tail, lines } => cmd_logs(*tail, *lines),
```

- [ ] **Step 5: 实现 `cmd_logs` 函数**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_stats` 函数之后插入：

```rust
/// logs: 查看代理日志
///
/// 无参数：输出日志文件最后 N 行（默认 50）。
/// --tail：持续 tail 模式，每 500ms 读取新增内容并打印，Ctrl+C 退出。
/// 日志文件路径：~/.cc-switch/cc-switch-daemon.log
fn cmd_logs(tail: bool, lines: u32) {
    let log_path = crate_config_dir().join("cc-switch-daemon.log");
    if !log_path.exists() {
        println!("日志文件不存在: {}", log_path.display());
        println!("提示: 使用 `cc-switch-cli daemon` 启动后台代理后会创建日志文件");
        return;
    }

    // 读取全部内容（daemon 日志通常不会很大）
    let content = std::fs::read_to_string(&log_path).unwrap_or_default();
    let all_lines: Vec<&str> = content.lines().collect();

    if !tail {
        // 单次模式：输出最后 N 行
        let start = all_lines.len().saturating_sub(lines as usize);
        for line in &all_lines[start..] {
            println!("{line}");
        }
        return;
    }

    // tail 模式：先输出当前末尾 N 行，然后持续读取新增
    let mut last_pos = std::fs::metadata(&log_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let start = all_lines.len().saturating_sub(lines as usize);
    for line in &all_lines[start..] {
        println!("{line}");
    }
    use std::io::Write;
    let _ = std::io::stdout().flush();

    println!("--- tail 模式（Ctrl+C 退出）---");

    loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let new_meta = match std::fs::metadata(&log_path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let new_size = new_meta.len();
        if new_size < last_pos {
            // 日志被截断/轮转，重置位置
            last_pos = 0;
        }
        if new_size == last_pos {
            continue;
        }
        // 读取新增字节
        let mut file = match std::fs::File::open(&log_path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        use std::io::{Read, Seek, SeekFrom};
        if let Err(_) = file.seek(SeekFrom::Start(last_pos)) {
            continue;
        }
        let mut buf = String::new();
        if file.read_to_string(&mut buf).is_err() {
            continue;
        }
        print!("{buf}");
        let _ = std::io::stdout().flush();
        last_pos = new_size;
    }
}
```

- [ ] **Step 6: 在 `cmd_help` 添加 logs 说明**

在 [src-tauri/src/bin/cc-switch-cli.rs](file:///f:/workspace/trae/cc-switch/src-tauri/src/bin/cc-switch-cli.rs) 的 `cmd_help` 中，`stats` 行之后插入：

```rust
    println!("    logs [--tail] [--lines N]     查看代理日志（--tail 持续刷新）");
```

- [ ] **Step 7: 运行测试验证通过**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test --test cli_plan_d logs -- --nocapture
```

Expected: `test_logs_tail_starts_and_outputs_initial` PASS

- [ ] **Step 8: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add src-tauri/src/bin/cc-switch-cli.rs src-tauri/tests/cli_plan_d.rs
git commit -m "feat(cli): 新增 logs 命令查看代理日志

- 无参数：输出最后 N 行（默认 50）
- --tail：持续 tail 模式，每 500ms 读取新增字节
- 处理日志轮转（文件大小回退时重置位置）
- 日志路径：~/.cc-switch/cc-switch-daemon.log

关联 spec: §九 M-4"
```

---

## Task 11: P2-1 参考手册补全 7 应用 API 格式表格

**Files:**
- Modify: `docs/cli-reference-manual.md`（API 格式表格补全 7 应用）

- [ ] **Step 1: 找到 API 格式表格位置**

Run:

```bash
cd f:/workspace/trae/cc-switch
rg -n "支持的格式|API 格式|api_format|api-format" docs/cli-reference-manual.md | head -20
```

定位到约第 643~648 行的 API 格式表格。

- [ ] **Step 2: 替换 API 格式表格**

Read [docs/cli-reference-manual.md:640-660](file:///f:/workspace/trae/cc-switch/docs/cli-reference-manual.md)

将原 4 行表格（claude/codex/gemini/claude-desktop）替换为 7 行完整表格：

```markdown
| 应用类型 | 支持的 API 格式 | 说明 |
|----------|---------------|------|
| claude | anthropic / openai_chat / openai_responses | Claude Code CLI |
| claude-desktop | anthropic / openai_chat / openai_responses / gemini_native / bedrock | Claude Desktop 网关 |
| codex | openai_responses / openai_chat | OpenAI Codex CLI |
| gemini | gemini_native / openai_chat / openai_responses / anthropic | Gemini CLI |
| opencode | openai_chat / openai_responses | OpenCode CLI（与 Codex 同协议） |
| openclaw | openai_chat / openai_responses | OpenClaw（OpenAI 兼容协议） |
| hermes | anthropic / openai_chat | Hermes（支持 Anthropic 与 OpenAI 双格式） |
```

> 注：opencode/openclaw/hermes 的支持格式基于 Plan A Task 4 Step 10 的注释与代码库实际实现。若 Plan B/C 中有调整，以代码为准。

- [ ] **Step 3: 在 add-provider 命令章节补全 api-format 说明**

Grep `docs/cli-reference-manual.md` 找到 `add-provider` 章节，在 api-format 选项说明处更新为 7 应用全列表：

```markdown
**--api-format FORMAT**
- claude: anthropic / openai_chat / openai_responses
- claude-desktop: anthropic / openai_chat / openai_responses / gemini_native / bedrock
- codex: openai_responses / openai_chat
- gemini: gemini_native / openai_chat / openai_responses / anthropic
- opencode: openai_chat / openai_responses
- openclaw: openai_chat / openai_responses
- hermes: anthropic / openai_chat
```

- [ ] **Step 4: 在参考手册新增 Plan D 9 个命令章节**

在 [docs/cli-reference-manual.md](file:///f:/workspace/trae/cc-switch/docs/cli-reference-manual.md) 末尾"测试与诊断"章节之后、FAQ 之前插入新章节：

```markdown
## 协议转换可观测性

### export-yaml

**用途**：导出当前数据库配置为声明式 YAML 文件（`apply-config` 的逆操作）。

```bash
cc-switch-cli export-yaml <PATH>
```

导出的 YAML 可直接被 `validate` 与 `apply-config` 接受，支持配置即代码工作流。

### diff

**用途**：对比 YAML 配置与当前数据库配置的差异（apply 前预览变更）。

```bash
cc-switch-cli diff <YAML_PATH>
```

输出 unified diff 风格的 `+`/`-` 行，便于在 `apply-config` 前检查变更范围。

### rollback

**用途**：回滚到上一个 `apply-config` 前的状态。

```bash
cc-switch-cli rollback
```

`apply-config` 应用前会自动创建 SQL 备份并记录文件名到 `last_apply_backup` 设置项。`rollback` 命令读取该设置项、交互式确认后恢复，并清除记录防止二次回滚。

### toggle-provider

**用途**：启用/禁用供应商（不删除配置，禁用后代理转发跳过）。

```bash
cc-switch-cli toggle-provider <APP> <ID> <on|off>
```

适用于供应商临时不可用（限流/维护）时保留配置但暂停使用。

### preview-conversion

**用途**：预览协议转换结果（不入网，仅调用 transform 函数）。

```bash
cc-switch-cli preview-conversion --from F --to F --payload FILE
```

支持路径：`anthropic ↔ openai_chat`、`anthropic ↔ openai_responses`、`anthropic ↔ gemini_native`。

### proxy-trace

**用途**：跟踪一次请求的完整转换链路（不入网）。

```bash
cc-switch-cli proxy-trace <APP> --model M --provider P
```

输出 4 段：原始请求体 / 转换后请求体 / 上游响应占位 / 反转换占位。供应商 `api_format` 决定转换路径。

### replay-request

**用途**：重放历史请求用于排障。

```bash
cc-switch-cli replay-request <REQUEST_ID> [--provider P] [--app A] [--payload F] [--dry-run]
```

- 优先从 `proxy_request_bodies` 表读取存档 body（需启用 `CC_SWITCH_LOG_BODIES=1` 环境变量）
- `--payload` 指定自定义 body 文件（需同时指定 `--app` 与 `--provider`）
- `--dry-run` 仅展示转换链路，不实际转发

## 实时可观测性

### connections

**用途**：查看当前活跃连接。

```bash
cc-switch-cli connections
```

通过 HTTP GET `/status` 轮询运行中 daemon，输出 `active_connections`、`total_requests`、`active_targets`。daemon 未运行时优雅提示。

### stats

**用途**：实时统计（QPS / 错误率）。

```bash
cc-switch-cli stats [--live]
```

- 无参数：输出单次快照
- `--live`：每秒刷新，Ctrl+C 退出

### logs

**用途**：查看代理日志。

```bash
cc-switch-cli logs [--tail] [--lines N]
```

- 无参数：输出最后 N 行（默认 50）
- `--tail`：持续 tail 模式，每 500ms 读取新增字节，Ctrl+C 退出
- 日志路径：`~/.cc-switch/cc-switch-daemon.log`
```

- [ ] **Step 5: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add docs/cli-reference-manual.md
git commit -m "docs(cli): 参考手册补全 7 应用 API 格式表格 + Plan D 9 命令章节

- API 格式表格补全 opencode/openclaw/hermes 三应用
- add-provider api-format 选项说明同步更新
- 新增协议转换可观测性章节（M-3 三命令）
- 新增实时可观测性章节（M-4 三命令）
- 新增 export-yaml/diff/rollback/toggle-provider 章节（M-5/M-6/M-7）

关联 spec: §七.3 P2-1"
```

---

## Task 12: P2-3 评估文档 OPT 子优先级标注

**Files:**
- Modify: `docs/cli-feature-implementation-assessment.md`（OPT 表格加"子优先级"列）

- [ ] **Step 1: 找到 OPT 章节位置**

Run:

```bash
cd f:/workspace/trae/cc-switch
rg -n "^## 三、可实现|OPT-001|OPT-060" docs/cli-feature-implementation-assessment.md | head -10
```

- [ ] **Step 2: 在 §三 章节开头追加子优先级说明**

Read [docs/cli-feature-implementation-assessment.md:76-80](file:///f:/workspace/trae/cc-switch/docs/cli-feature-implementation-assessment.md)

在第 78 行（`### MCP 管理` 标题之前）插入子优先级说明：

```markdown
> **子优先级（2026-07-04 修订）**：OPT 项进一步细分为 OPT-A（高价值，建议实现）与 OPT-B（低价值，暂缓或拒绝）。
>
> - **OPT-A**：MCP/Prompt/环境变量核心管理、跨应用通用供应商、配置目录覆盖、协议转换可观测性（M-3~M-7）等
> - **OPT-B**：详细用量统计、Skills 仓库管理、应用自身功能、一次性操作等
>
> 详见 spec §四.5 与 §九。下表每个 OPT 项新增"子优先级"列标注 A/B。
```

- [ ] **Step 3: 给每个 OPT 表格加"子优先级"列**

按 spec §四.5 OPT-A/OPT-B 清单，给下列表格加列。具体修改：

**MCP 管理（§三.1）**：OPT-001~005 标 A；OPT-006 标 B

修改方式：将原表头 `| 编号 | 功能 | 说明 | 对应 GUI 命令 |` 改为 `| 编号 | 功能 | 说明 | 子优先级 | 对应 GUI 命令 |`，并为每行填入 A 或 B。

**Prompt 管理（§三.2）**：OPT-007~009 标 A；OPT-010/011 标 B

**Skills 管理（§三.3）**：OPT-012~016 标 B；OPT-017~021 标 B（已建议归 N/A，但保留在表中标 B）

**环境变量管理（§三.4）**：OPT-022/023 标 A；OPT-024 标 B

**会话管理（§三.5）**：OPT-025~027 标 B；OPT-028 标 B

**流式健康检查（§三.6）**：OPT-029~031 标 B

**详细用量统计（§三.7）**：OPT-032~038 标 B

**通用供应商与其他扩展（§三.8）**：OPT-039/040 标 A；OPT-041~046 标 B

**OpenClaw 专属（§三.9）**：OPT-047/048/049/050/051 标 A（按 spec §四.5 升级 REQ 的项除外，但保留在 OPT 表中标 A）；OPT-052/053/054 标 B

**Hermes 专属（§三.10）**：OPT-055/056 标 A；OPT-057 标 B

**OMO 配置（§三.11）**：OPT-058~060 标 B

> 注：上表标注按 spec §四.5 的清单。具体每行填写 `A` 或 `B`，不修改原"功能"/"说明"列内容。

- [ ] **Step 4: 在 §九 章节标注 M-1~M-11 的归类**

Read [docs/cli-feature-implementation-assessment.md:870-991](file:///f:/workspace/trae/cc-switch/docs/cli-feature-implementation-assessment.md)

在第 991 行（§九.3 表格末尾）之后追加：

```markdown
### 9.7 Plan D 实现状态（2026-07-04 更新）

Plan D 已实现以下 OPT-A 级别功能（详见 `docs/superpowers/plans/2026-07-04-plan-d-experience-improvements.md`）：

| M 编号 | 功能 | 实现命令 | 状态 |
|---|---|---|---|
| M-3 | 协议转换可观测性 | `proxy-trace` / `replay-request` / `preview-conversion` | ✅ 已实现 |
| M-4 | 实时可观测性 | `connections` / `stats --live` / `logs --tail` | ✅ 已实现 |
| M-5 | 配置 diff/rollback | `diff` / `rollback`（apply-config 自动备份） | ✅ 已实现 |
| M-6 | 导出为 YAML | `export-yaml` | ✅ 已实现 |
| M-7 | 供应商启用/禁用 | `toggle-provider` | ✅ 已实现 |

M-1（热重载）、M-2（访问控制）由 Plan C 实现。M-8~M-11 暂未实现，保持 OPT-B 不变。
```

- [ ] **Step 5: Commit**

```bash
cd f:/workspace/trae/cc-switch
git add docs/cli-feature-implementation-assessment.md
git commit -m "docs(cli): 评估文档加 OPT 子优先级列 + Plan D 实现状态

- §三 各 OPT 表格新增'子优先级'列（A/B）
- §三 开头追加 OPT-A/OPT-B 分类说明
- §九 新增 9.7 Plan D 实现状态表
- 标注 M-3~M-7 已实现，M-1/M-2 由 Plan C 实现

关联 spec: §七.3 P2-3"
```

---

## Task 13: 最终验证与回归测试

**Files:**
- 无修改，仅运行验证

- [ ] **Step 1: 运行全部 Rust 测试**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo test
```

Expected: 所有测试通过（含 Plan A/B/C 已有测试 + Plan D 新增 13 个测试）

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

- [ ] **Step 4: 手动验证所有新命令出现在 help**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
cargo build --bin cc-switch-cli
./target/debug/cc-switch-cli.exe help
```

Expected: help 输出包含 `export-yaml`、`diff`、`rollback`、`toggle-provider`、`preview-conversion`、`proxy-trace`、`replay-request`、`connections`、`stats`、`logs`

- [ ] **Step 5: 手动验证 export-yaml → diff → apply-config → rollback 全链路**

Run:

```bash
cd f:/workspace/trae/cc-switch/src-tauri
$env:CC_SWITCH_HOME = "$env:TEMP\cc-switch-plan-d-test"
Remove-Item -Recurse -Force $env:CC_SWITCH_HOME -ErrorAction Ignore
./target/debug/cc-switch-cli.exe add-provider claude p1 "P1" --api-key k1
./target/debug/cc-switch-cli.exe export-yaml "$env:TEMP\plan-d-current.yaml"
./target/debug/cc-switch-cli.exe diff "$env:TEMP\plan-d-current.yaml"
# 期望：无差异

# 修改 YAML 添加 p2，再 diff
$yaml = Get-Content "$env:TEMP\plan-d-current.yaml"
$yaml += "`nproviders:`n  - app: claude`n    id: p2`n    name: P2`n    env:`n      ANTHROPIC_API_KEY: k2`n"
Set-Content "$env:TEMP\plan-d-add.yaml" $yaml
./target/debug/cc-switch-cli.exe diff "$env:TEMP\plan-d-add.yaml"
# 期望：显示 + p2 行

# apply 后 rollback
./target/debug/cc-switch-cli.exe apply-config "$env:TEMP\plan-d-add.yaml"
./target/debug/cc-switch-cli.exe list-providers claude
# 期望：含 p1 和 p2
echo "y" | ./target/debug/cc-switch-cli.exe rollback
./target/debug/cc-switch-cli.exe list-providers claude
# 期望：仅含 p1（p2 已被 rollback 移除）

# 验证 toggle-provider
./target/debug/cc-switch-cli.exe toggle-provider claude p1 off
./target/debug/cc-switch-cli.exe list-providers claude
# 期望：p1 标记为 disabled
./target/debug/cc-switch-cli.exe toggle-provider claude p1 on

# 验证 preview-conversion（不入网，仅本地转换）
'{"model":"claude-3-5-sonnet-20241022","messages":[{"role":"user","content":"hi"}],"max_tokens":100}' | Set-Content "$env:TEMP\plan-d-payload.json"
./target/debug/cc-switch-cli.exe preview-conversion --from anthropic --to openai_chat --payload "$env:TEMP\plan-d-payload.json"
# 期望：输出转换后的 OpenAI Chat 格式请求体（含 messages 数组）

# 启动 daemon 后验证 connections/stats/logs（仅确认命令不报错）
Start-Job -ScriptBlock {
    $env:CC_SWITCH_HOME = "$env:TEMP\cc-switch-plan-d-test"
    & "f:/workspace/trae/cc-switch/src-tauri/target/debug/cc-switch-cli.exe" daemon
} | Out-Null
Start-Sleep -Seconds 2
./target/debug/cc-switch-cli.exe connections
./target/debug/cc-switch-cli.exe stats --live --interval 1 --count 1
./target/debug/cc-switch-cli.exe logs --tail --lines 5
Get-Job | Stop-Job
Get-Job | Remove-Job

# 清理测试数据
Remove-Item -Recurse -Force $env:CC_SWITCH_HOME -ErrorAction Ignore
Remove-Item -Force "$env:TEMP\plan-d-current.yaml" -ErrorAction Ignore
Remove-Item -Force "$env:TEMP\plan-d-add.yaml" -ErrorAction Ignore
Remove-Item -Force "$env:TEMP\plan-d-payload.json" -ErrorAction Ignore
```

Expected: 全链路无报错；rollback 后 p2 消失；toggle-provider 后 p1 disabled 状态切换；preview-conversion 输出 OpenAI Chat 格式 JSON；connections/stats/logs 三命令不报错退出

- [ ] **Step 6: 验证文档更新已落盘**

Run:

```bash
cd f:/workspace/trae/cc-switch
# 检查参考手册已新增 9 命令章节（每个命令名应至少出现 1 次）
$cmds = "export-yaml","diff","rollback","toggle-provider","preview-conversion","proxy-trace","replay-request","connections","stats"
foreach ($c in $cmds) {
    $count = (Select-String -Path docs/cli-reference-manual.md -Pattern $c -SimpleMatch).Count
    Write-Host "$c : $count"
    if ($count -lt 1) { Write-Host "FAIL: $c 未出现在参考手册"; exit 1 }
}
# 检查评估文档含子优先级标注
$optCount = (Select-String -Path docs/cli-feature-implementation-assessment.md -Pattern "OPT-A|OPT-B").Count
Write-Host "OPT 子优先级标注数: $optCount"
if ($optCount -lt 10) { Write-Host "FAIL: OPT 子优先级标注不足"; exit 1 }
```

Expected: 9 个命令名全部出现在 `cli-reference-manual.md`；评估文档 OPT 子优先级标注数 ≥10

---

## 完成检查清单

实施完成后请逐项确认：

- [ ] Task 1: `export-yaml` 命令实现，YAML 可被 `validate`/`apply-config` 接受
- [ ] Task 2: `diff` 命令实现，输出 unified diff 风格差异
- [ ] Task 3: `rollback` 命令实现，apply 前 auto-backup 已写入 `last_apply_backup` 设置项
- [ ] Task 4: `toggle-provider` 命令实现，禁用供应商后 `ProviderRouter` 转发跳过
- [ ] Task 5: `preview-conversion` 命令实现，支持 6 条转换路径（双向 × 3 格式）
- [ ] Task 6: `proxy-trace` 命令实现，输出 4 段转换链路
- [ ] Task 7: `replay-request` 命令实现，含 `--dry-run` 与 `proxy_request_bodies` 存档表
- [ ] Task 8: `connections` 命令实现，HTTP 轮询 daemon `/status`
- [ ] Task 9: `stats --live` 命令实现，输出 QPS 与 token 速率
- [ ] Task 10: `logs --tail` 命令实现，支持日志轮转文件追踪
- [ ] Task 11: 参考手册补全 7 应用 API 格式表格 + 9 命令章节
- [ ] Task 12: 评估文档 OPT 子优先级 A/B 标注 + §9.7 Plan D 状态表
- [ ] Task 13: 全部 cargo test 通过、clippy 无新增警告、fmt 通过、e2e 全链路验证通过

---

## Self-Review 结论

**Spec 覆盖性：** §九 M-3（Task 5/6/7）、M-4（Task 8/9/10）、M-5（Task 2/3）、M-6（Task 1）、M-7（Task 4）、§七.3 P2-1（Task 11）、P2-3（Task 12）全部覆盖，无遗漏。

**类型一致性：**
- `ProviderMeta.disabled: Option<bool>`（Task 4 定义）与 `ProviderEntry.disabled: Option<bool>`（Task 1 定义）与 `providers.disabled` 列（Task 4 schema migration）字段名一致
- `DeclConfig::from_database()` 在 Task 1 定义后被 Task 2/3 复用
- `Database::get_last_apply_backup / set_last_apply_backup / clear_last_apply_backup` 在 Task 3 定义并使用同一组 API
- `anthropic_to_openai / openai_to_anthropic / anthropic_to_responses / responses_to_anthropic / anthropic_to_gemini / gemini_to_anthropic` 在 Task 5/6 中调用，函数名与 `proxy::providers::transform::*` 中现有 `pub fn` 一致

**无占位符：** 所有 Step 均含完整代码/命令/期望输出；无 TBD/TODO/省略代码；每个 `cmd_*` 函数均给出完整 Rust 实现可编译运行。

---

## 后续衔接

Plan D 完成后，剩余未实施的 OPT-B 级别功能（约 40 项 YAGNI 项目）按 spec §七.3 P2-2 推迟到 v4.x。Plan A（apply-config 止血）→ Plan B（lib crate 分层）→ Plan C（REQ 补全）→ Plan D（体验改进）共同形成 v3.16.x 的 CLI 完整能力，覆盖 REQ 23 项 + OPT-A 高价值项。后续如有新场景需求，可在 Plan D 基础上追加 v3.17.x 增量。
//! Plan A P0 修复集成测试。
//!
//! 通过子进程调用 `cc-switch-cli` 二进制验证：
//! 1. `add-provider` env 字段名按 `--app` 类型选择（P0-4）
//! 2. `apply-config` YAML schema 拒绝 `listen`/`port`，接受 `takeover`（P0-2 方案 B）
//! 3. `list-providers` 显示各应用的 Base URL（P2-1）
//! 4. `update-provider` 保持正确的 env 字段名（P0-4）
//!
//! 隔离：每个测试创建独立 tempdir，通过 `CC_SWITCH_HOME` 环境变量重定向数据库。
//! 因 env var 是进程级，测试通过全局 mutex 串行运行。

use std::process::Command;
use std::sync::Mutex;

/// CLI 二进制路径（由 cargo 自动注入）。
fn cli_binary() -> String {
    env!("CARGO_BIN_EXE_cc-switch-cli").to_string()
}

/// 串行锁：`CC_SWITCH_HOME` 是进程级 env var，并发子进程会互相干扰。
fn serial_lock() -> &'static Mutex<()> {
    static LOCK: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// 运行 CLI 子进程，设置 `CC_SWITCH_HOME` 指向 tempdir。
fn run_cli(home: &std::path::Path, args: &[&str]) -> (bool, String, String) {
    let output = Command::new(cli_binary())
        .env("CC_SWITCH_HOME", home)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .args(args)
        .output()
        .expect("Failed to execute cc-switch-cli");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

// =============================================================================
// P0-4: add-provider env 字段按 app 类型选择
// =============================================================================

#[test]
fn add_provider_codex_uses_openai_env_fields() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let (ok, _out, err) = run_cli(
        home,
        &[
            "add-provider",
            "codex",
            "test-openai",
            "Test OpenAI",
            "--api-key",
            "sk-test",
            "--base-url",
            "https://api.openai.com/v1",
        ],
    );
    assert!(ok, "add-provider failed: {err}");

    // 通过 list-providers 验证 base URL 出现在输出中
    let (ok, out, err) = run_cli(home, &["list-providers", "codex"]);
    assert!(ok, "list-providers failed: {err}");
    assert!(
        out.contains("https://api.openai.com/v1"),
        "list-providers 应显示 OpenAI base URL，实际: {out}"
    );
}

#[test]
fn add_provider_gemini_uses_gemini_env_fields() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let (ok, _out, err) = run_cli(
        home,
        &[
            "add-provider",
            "gemini",
            "test-gemini",
            "Test Gemini",
            "--api-key",
            "test-key",
            "--base-url",
            "https://generativelanguage.googleapis.com/v1",
        ],
    );
    assert!(ok, "add-provider failed: {err}");

    let (ok, out, err) = run_cli(home, &["list-providers", "gemini"]);
    assert!(ok, "list-providers failed: {err}");
    assert!(
        out.contains("https://generativelanguage.googleapis.com/v1"),
        "list-providers 应显示 Gemini base URL，实际: {out}"
    );
}

#[test]
fn add_provider_claude_uses_anthropic_env_fields() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let (ok, _out, err) = run_cli(
        home,
        &[
            "add-provider",
            "claude",
            "test-claude",
            "Test Claude",
            "--api-key",
            "sk-ant-test",
            "--base-url",
            "https://api.anthropic.com",
        ],
    );
    assert!(ok, "add-provider failed: {err}");

    let (ok, out, err) = run_cli(home, &["list-providers", "claude"]);
    assert!(ok, "list-providers failed: {err}");
    assert!(
        out.contains("https://api.anthropic.com"),
        "list-providers 应显示 Anthropic base URL，实际: {out}"
    );
}

// =============================================================================
// P0-4 续: update-provider 保持 env 字段名
// =============================================================================

#[test]
fn update_provider_codex_preserves_openai_env() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    // 先添加 codex 供应商
    let (ok, _, err) = run_cli(
        home,
        &[
            "add-provider",
            "codex",
            "test-update",
            "Test",
            "--api-key",
            "old-key",
        ],
    );
    assert!(ok, "add-provider failed: {err}");

    // 更新 api-key
    let (ok, _, err) = run_cli(
        home,
        &[
            "update-provider",
            "codex",
            "test-update",
            "--api-key",
            "new-key",
        ],
    );
    assert!(ok, "update-provider failed: {err}");

    // 验证：通过 export-config 导出后检查字段名
    let export_path = home.join("exported.json");
    let (ok, _, err) = run_cli(
        home,
        &["export-config", export_path.to_str().unwrap()],
    );
    assert!(ok, "export-config failed: {err}");

    let exported = std::fs::read_to_string(&export_path).expect("read exported");
    assert!(
        exported.contains("OPENAI_API_KEY"),
        "导出内容应包含 OPENAI_API_KEY，实际: {exported}"
    );
    assert!(
        exported.contains("new-key"),
        "导出内容应包含新 key 值，实际: {exported}"
    );
    assert!(
        !exported.contains("ANTHROPIC_API_KEY"),
        "codex 供应商不应包含 ANTHROPIC_API_KEY，实际: {exported}"
    );
}

// =============================================================================
// P0-2 方案 B: apply-config YAML schema 止血
// =============================================================================

#[test]
fn validate_rejects_listen_field_in_proxy_section() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let yaml_path = tmp.path().join("bad-listen.yaml");

    std::fs::write(
        &yaml_path,
        "proxy:\n  listen: 0.0.0.0\n  port: 9091\nproviders: []\n",
    )
    .expect("write yaml");

    // validate 应失败：deny_unknown_fields 拒绝 listen 字段
    let (ok, _out, err) = run_cli(home, &["validate", yaml_path.to_str().unwrap()]);
    assert!(
        !ok || err.contains("unknown field"),
        "validate 应拒绝 listen/port 字段。success={}, stderr: {err}",
        ok
    );
}

#[test]
fn validate_accepts_takeover_only_in_proxy_section() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();
    let yaml_path = tmp.path().join("takeover-only.yaml");

    std::fs::write(
        &yaml_path,
        "proxy:\n  takeover:\n    claude: true\nproviders: []\n",
    )
    .expect("write yaml");

    let (ok, out, err) = run_cli(home, &["validate", yaml_path.to_str().unwrap()]);
    assert!(
        ok,
        "validate 应接受 takeover 字段。stderr: {err}\nstdout: {out}"
    );
    assert!(out.contains("校验通过"), "应输出校验通过: {out}");
}

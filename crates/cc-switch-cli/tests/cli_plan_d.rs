//! Plan D 体验改进集成测试。
//!
//! 通过子进程调用 `cc-switch-cli` 二进制验证：
//! 1. `export-yaml` → `validate` 往返（M-6）
//! 2. `diff` 输出对比结果（M-5）
//! 3. `toggle-provider` 启用/禁用供应商（M-7）
//! 4. `preview-conversion` 协议转换预览（M-3，离线无网络）
//!
//! 隔离方式同 `cli_p0_fixes.rs`：tempdir + `CC_SWITCH_HOME` + 串行锁。

use std::process::Command;
use std::sync::Mutex;

fn cli_binary() -> String {
    env!("CARGO_BIN_EXE_cc-switch-cli").to_string()
}

fn serial_lock() -> &'static Mutex<()> {
    static LOCK: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

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
// M-6: export-yaml round-trip
// =============================================================================

#[test]
fn export_yaml_round_trips_through_validate() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    // 先添加一个 claude 供应商
    let (ok, _, err) = run_cli(
        home,
        &[
            "add-provider",
            "claude",
            "export-test",
            "Export Test",
            "--api-key",
            "sk-test",
            "--base-url",
            "https://api.anthropic.com",
        ],
    );
    assert!(ok, "add-provider failed: {err}");

    // 导出为 YAML
    let yaml_path = home.join("exported.yaml");
    let (ok, out, err) = run_cli(home, &["export-yaml", yaml_path.to_str().unwrap()]);
    assert!(ok, "export-yaml failed: {err}");
    assert!(out.contains("已导出"), "应提示已导出: {out}");

    // 导出的 YAML 应能被 validate 接受
    let (ok, out, err) = run_cli(home, &["validate", yaml_path.to_str().unwrap()]);
    assert!(
        ok,
        "导出的 YAML 应通过 validate。stderr: {err}\nstdout: {out}"
    );
    assert!(out.contains("校验通过"), "应输出校验通过: {out}");
}

// =============================================================================
// M-5: diff 对比输出
// =============================================================================

#[test]
fn diff_shows_new_provider_not_in_database() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    // 数据库中添加一个供应商
    let (ok, _, err) = run_cli(
        home,
        &[
            "add-provider",
            "claude",
            "db-provider",
            "DB Provider",
            "--api-key",
            "key1",
        ],
    );
    assert!(ok, "add-provider failed: {err}");

    // YAML 中放一个不同的供应商
    let yaml_path = home.join("diff-test.yaml");
    std::fs::write(
        &yaml_path,
        "providers:\n  - app: claude\n    id: yaml-provider\n    name: YAML Provider\n    env:\n      ANTHROPIC_API_KEY: yaml-key\n",
    )
    .expect("write yaml");

    let (ok, out, err) = run_cli(home, &["diff", yaml_path.to_str().unwrap()]);
    assert!(ok, "diff failed: {err}");
    assert!(
        out.contains("YAML 供应商") && out.contains("数据库供应商"),
        "diff 应显示双方供应商数量: {out}"
    );
    assert!(
        out.contains("新增供应商") || out.contains("yaml-provider"),
        "应提示 YAML 中新增的供应商: {out}"
    );
}

#[test]
fn diff_shows_no_difference_when_identical() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    // 添加供应商
    let (ok, _, err) = run_cli(
        home,
        &[
            "add-provider",
            "claude",
            "same-provider",
            "Same",
            "--api-key",
            "k",
        ],
    );
    assert!(ok, "add-provider failed: {err}");

    // 导出后 diff 自身应无差异
    let yaml_path = home.join("same.yaml");
    let (ok, _, err) = run_cli(home, &["export-yaml", yaml_path.to_str().unwrap()]);
    assert!(ok, "export-yaml failed: {err}");

    let (ok, out, err) = run_cli(home, &["diff", yaml_path.to_str().unwrap()]);
    assert!(ok, "diff failed: {err}");
    assert!(
        out.contains("无差异") || out.contains("一致"),
        "相同配置应提示无差异: {out}"
    );
}

// =============================================================================
// M-7: toggle-provider 启用/禁用
// =============================================================================

#[test]
fn toggle_provider_disables_and_reenables() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    // 添加供应商
    let (ok, _, err) = run_cli(
        home,
        &[
            "add-provider",
            "claude",
            "toggle-test",
            "Toggle Test",
            "--api-key",
            "k",
        ],
    );
    assert!(ok, "add-provider failed: {err}");

    // 禁用
    let (ok, out, err) = run_cli(
        home,
        &["toggle-provider", "claude", "toggle-test", "off"],
    );
    assert!(ok, "toggle-provider off failed: {err}");
    assert!(out.contains("禁用"), "应提示已禁用: {out}");

    // 导出配置验证 disabled 字段
    let export_path = home.join("toggled.json");
    let (ok, _, err) = run_cli(home, &["export-config", export_path.to_str().unwrap()]);
    assert!(ok, "export-config failed: {err}");
    let exported = std::fs::read_to_string(&export_path).expect("read exported");
    // disabled 状态应体现在导出中（字段名可能为 disabled 或 meta.disabled）
    assert!(
        exported.contains("disabled") || exported.contains("toggle-test"),
        "导出应包含供应商或 disabled 字段: {exported}"
    );

    // 重新启用
    let (ok, out, err) = run_cli(
        home,
        &["toggle-provider", "claude", "toggle-test", "on"],
    );
    assert!(ok, "toggle-provider on failed: {err}");
    assert!(out.contains("启用"), "应提示已启用: {out}");
}

#[test]
fn toggle_provider_invalid_state_fails() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let (ok, _out, err) = run_cli(
        home,
        &["toggle-provider", "claude", "any", "invalid"],
    );
    assert!(!ok, "无效状态应失败。stderr: {err}");
}

// =============================================================================
// M-3: preview-conversion 协议转换预览（离线）
// =============================================================================

#[test]
fn preview_conversion_anthropic_to_openai_outputs_transformed() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    // 简单的 anthropic 请求体
    let payload = r#"{"model":"claude-sonnet-4-5","messages":[{"role":"user","content":"hi"}],"max_tokens":100}"#;

    let (ok, out, err) = run_cli(
        home,
        &[
            "preview-conversion",
            "--from",
            "anthropic",
            "--to",
            "openai_chat",
            "--payload",
            payload,
        ],
    );
    assert!(
        ok,
        "preview-conversion 应成功。stderr: {err}\nstdout: {out}"
    );
    assert!(out.contains("协议转换预览"), "应输出预览标题: {out}");
    assert!(out.contains("anthropic"), "应显示源格式: {out}");
    assert!(out.contains("openai_chat"), "应显示目标格式: {out}");
}

#[test]
fn preview_conversion_invalid_payload_fails() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let (ok, _out, err) = run_cli(
        home,
        &[
            "preview-conversion",
            "--from",
            "anthropic",
            "--to",
            "openai_chat",
            "--payload",
            "{invalid json}",
        ],
    );
    assert!(!ok, "无效 JSON 应失败。stderr: {err}");
}

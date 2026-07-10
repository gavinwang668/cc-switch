//! Plan C 新功能集成测试。
//!
//! 通过子进程调用 `cc-switch-cli` 二进制验证：
//! 1. `auth-token set/clear` — 代理访问令牌存取（REQ-023 数据库层）
//! 2. `acl add/list/remove` — IP CIDR 白名单管理（REQ-023 数据库层）
//! 3. `smoke-test` — 协议转换烟雾测试退出码（REQ-021，离线无网络）
//!
//! 注：`reload` 命令需要运行中的代理服务器，不在子进程测试覆盖范围。
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
// auth-token（REQ-023 数据库层）
// =============================================================================

#[test]
fn auth_token_set_and_view() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    // 设置令牌
    let (ok, out, err) = run_cli(
        home,
        &["auth-token", "set", "--token", "secret-token-123"],
    );
    assert!(ok, "auth-token set failed: {err}");
    assert!(out.contains("已设置"), "应提示已设置: {out}");

    // 查看（无 action）应显示令牌已设置
    let (ok, out, err) = run_cli(home, &["auth-token"]);
    assert!(ok, "auth-token view failed: {err}");
    assert!(
        out.contains("令牌已设置"),
        "应显示令牌已设置: {out}"
    );
    // 不泄露完整令牌（只显示前缀）
    assert!(
        !out.contains("secret-token-123"),
        "查看时不应泄露完整令牌: {out}"
    );
}

#[test]
fn auth_token_clear_returns_to_open() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    // 先设置
    let (ok, _, err) = run_cli(
        home,
        &["auth-token", "set", "--token", "temp-token"],
    );
    assert!(ok, "auth-token set failed: {err}");

    // 清除
    let (ok, out, err) = run_cli(home, &["auth-token", "clear"]);
    assert!(ok, "auth-token clear failed: {err}");
    assert!(out.contains("已清除"), "应提示已清除: {out}");

    // 查看应显示开放状态
    let (ok, out, err) = run_cli(home, &["auth-token"]);
    assert!(ok, "auth-token view failed: {err}");
    assert!(
        out.contains("未设置") || out.contains("完全开放"),
        "清除后应显示未设置/开放: {out}"
    );
}

// =============================================================================
// acl（REQ-023 数据库层）
// =============================================================================

#[test]
fn acl_add_list_and_remove() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    // 初始 list 应为空
    let (ok, out, err) = run_cli(home, &["acl", "list"]);
    assert!(ok, "acl list failed: {err}");
    assert!(
        out.contains("未设置") || out.contains("不限制"),
        "初始应无 ACL: {out}"
    );

    // 添加 CIDR
    let (ok, out, err) = run_cli(
        home,
        &["acl", "add", "--cidr", "192.168.1.0/24"],
    );
    assert!(ok, "acl add failed: {err}");
    assert!(out.contains("已添加"), "应提示已添加: {out}");

    // 添加第二个 CIDR
    let (ok, _, err) = run_cli(
        home,
        &["acl", "add", "--cidr", "10.0.0.0/8"],
    );
    assert!(ok, "acl add 2nd failed: {err}");

    // list 应显示 2 条
    let (ok, out, err) = run_cli(home, &["acl", "list"]);
    assert!(ok, "acl list failed: {err}");
    assert!(out.contains("192.168.1.0/24"), "应包含第一条: {out}");
    assert!(out.contains("10.0.0.0/8"), "应包含第二条: {out}");
    assert!(out.contains("2 条"), "应显示 2 条: {out}");

    // 移除一条
    let (ok, out, err) = run_cli(
        home,
        &["acl", "remove", "--cidr", "192.168.1.0/24"],
    );
    assert!(ok, "acl remove failed: {err}");
    assert!(out.contains("已移除"), "应提示已移除: {out}");

    // list 应只剩 1 条
    let (ok, out, err) = run_cli(home, &["acl", "list"]);
    assert!(ok, "acl list after remove failed: {err}");
    assert!(!out.contains("192.168.1.0/24"), "不应再包含已移除的: {out}");
    assert!(out.contains("10.0.0.0/8"), "应保留第二条: {out}");
}

#[test]
fn acl_remove_nonexistent_fails() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let (ok, _out, err) = run_cli(
        home,
        &["acl", "remove", "--cidr", "172.16.0.0/12"],
    );
    assert!(
        !ok,
        "移除不存在的 CIDR 应失败。stderr: {err}"
    );
}

// =============================================================================
// smoke-test（REQ-021 离线协议转换）
// =============================================================================

#[test]
fn smoke_test_single_app_exits_successfully() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    // smoke-test 不需要数据库中的供应商，离线运行协议转换
    let (ok, out, err) = run_cli(home, &["smoke-test", "claude"]);
    assert!(
        ok,
        "smoke-test claude 应成功退出。stderr: {err}\nstdout: {out}"
    );
    assert!(
        out.contains("烟雾测试") || out.contains("协议转换"),
        "应输出烟雾测试结果: {out}"
    );
}

#[test]
fn smoke_test_invalid_app_fails() {
    let _guard = serial_lock().lock().expect("serial lock");
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = tmp.path();

    let (ok, _out, err) = run_cli(home, &["smoke-test", "invalid-app"]);
    assert!(!ok, "无效应用应失败退出。stderr: {err}");
}

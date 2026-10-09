//! CLI 入口集成测试（整体 help）。

mod common;

use common::cli;

#[test]
fn test_cli_help_shows_all_commands() {
    let output = cli().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("req"));
    assert!(stdout.contains("intent"));
    assert!(stdout.contains("spec"));
    assert!(stdout.contains("impl"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("trace"));
    assert!(stdout.contains("refute"));
    assert!(stdout.contains("status"));
}

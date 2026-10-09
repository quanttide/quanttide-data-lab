//! 工作空间 I/O 集成测试：`new` 建盘、`update` 写入、`show` 读回、`status` 汇总，
//! 以及 `--root` 隔离。

mod common;

use std::fs;

use common::{run, run_with_stdin, stdout, temp_root};

/// 合法路径的字符串借用，供 `run(&[&str])` 使用。
fn as_str(path: &std::path::Path) -> &str {
    path.to_str().expect("临时根目录路径")
}

#[test]
fn new_writes_four_docs_and_manifest() {
    let root = temp_root("new");
    let out = run(&["--root", as_str(&root), "--case", "case-a", "req", "new"]);
    assert!(
        out.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let case_dir = root.join("case-a");
    assert!(
        case_dir.join("manifest.json").is_file(),
        "缺少 manifest.json"
    );
    for name in ["requirement", "intent", "specification", "implementation"] {
        let doc = case_dir.join("docs").join(format!("{name}.md"));
        assert!(doc.is_file(), "缺少 {}", doc.display());
        assert_eq!(
            fs::read_to_string(&doc).unwrap(),
            "",
            "{} 应为空正文",
            doc.display()
        );
    }

    let raw = fs::read_to_string(case_dir.join("manifest.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(manifest["case"], "case-a");
    for name in ["requirement", "intent", "specification", "implementation"] {
        assert_eq!(manifest[name]["version"], 1, "{name} 版本应为 1");
        assert_eq!(manifest[name]["status"], "draft", "{name} 状态应为 draft");
    }
}

#[test]
fn update_then_show_roundtrips_body_and_bumps_version() {
    let root = temp_root("roundtrip");
    let root = as_str(&root);
    assert!(
        run(&["--root", root, "--case", "case-b", "req", "new"])
            .status
            .success()
    );

    let update = run_with_stdin(
        &["--root", root, "--case", "case-b", "req", "update"],
        "搜索相关失败率下降\n",
    );
    assert!(
        update.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&update.stderr)
    );

    let show = run(&["--root", root, "--case", "case-b", "req", "show"]);
    assert!(show.status.success());
    assert_eq!(stdout(&show), "搜索相关失败率下降\n");

    let status = run(&["--root", root, "--case", "case-b", "status"]);
    assert!(status.status.success());
    let text = stdout(&status);
    assert!(text.contains("case-b"), "{text}");
    assert!(text.contains("v2"), "版本应升到 2：{text}");
    assert!(text.contains("draft"), "{text}");
}

#[test]
fn show_missing_case_fails_with_readable_error() {
    let root = temp_root("missing");
    let out = run(&["--root", as_str(&root), "--case", "nope", "req", "show"]);
    assert!(!out.status.success(), "缺失案例应失败");
    assert_eq!(out.status.code(), Some(1), "退出码应为 1");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("nope"), "错误信息应可读：{err}");
}

#[test]
fn new_on_existing_case_fails() {
    let root = temp_root("existing");
    let root = as_str(&root);
    assert!(
        run(&["--root", root, "--case", "dup", "req", "new"])
            .status
            .success()
    );
    let again = run(&["--root", root, "--case", "dup", "req", "new"]);
    assert!(!again.status.success(), "重复建盘应失败");
    assert_eq!(again.status.code(), Some(1), "退出码应为 1");
    assert!(
        String::from_utf8_lossy(&again.stderr).contains("dup"),
        "错误信息应含案例名"
    );
}

#[test]
fn status_reports_each_layer() {
    let root = temp_root("status");
    let root = as_str(&root);
    assert!(
        run(&["--root", root, "--case", "case-c", "req", "new"])
            .status
            .success()
    );
    let out = run(&["--root", root, "--case", "case-c", "status"]);
    assert!(out.status.success());
    let text = stdout(&out);
    for name in ["requirement", "intent", "specification", "implementation"] {
        assert!(text.contains(name), "status 应列出 {name}：{text}");
    }
    assert!(text.contains("draft"), "{text}");
}

#[test]
fn root_isolates_cases() {
    let root_a = temp_root("iso-a");
    let root_b = temp_root("iso-b");
    let a = as_str(&root_a);
    let b = as_str(&root_b);
    assert!(
        run(&["--root", a, "--case", "shared", "req", "new"])
            .status
            .success()
    );
    // 同名案例在另一个根下并不存在。
    assert!(
        !run(&["--root", b, "--case", "shared", "req", "show"])
            .status
            .success(),
        "--root 应隔离案例"
    );
    // 在另一个根下可独立建同名案例。
    assert!(
        run(&["--root", b, "--case", "shared", "req", "new"])
            .status
            .success()
    );
}

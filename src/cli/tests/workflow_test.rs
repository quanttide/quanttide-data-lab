//! 流程命令测试：`run` / `trace` / `refute` / `status`。
//!
//! 对应案例《量潮搜索工程》的数据规格五步、追溯矩阵（§5.2 追溯 ID）、
//! 反向挑战（§5.4 挑战与回退）与层状态。

mod common;

use std::path::Path;

use common::{case_with_fixtures, fixture, run, run_with_stdin, stdout, temp_root};

/// 一条合格挑战记录正文：三行字段各含非空内容。
fn refutation_body() -> &'static str {
    "触发点：规格缺验收\n结论：定义不可计算\n改动：退回数据意图层修正\n"
}

/// 挑战记录目录：`<root>/<case>/refutations`。
fn refutations_dir(root: &str, case: &str) -> std::path::PathBuf {
    Path::new(root).join(case).join("refutations")
}

/// 目录内记录文件数（目录不存在为 0）。
fn record_count(root: &str, case: &str) -> usize {
    match std::fs::read_dir(refutations_dir(root, case)) {
        Ok(entries) => entries.filter_map(Result::ok).count(),
        Err(_) => 0,
    }
}

/// `status` 输出中某层的状态行。
fn status_row(root: &str, case: &str, stem: &str) -> String {
    let out = run(&["--root", root, "--case", case, "status"]);
    assert!(
        out.status.success(),
        "status 应成功：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = stdout(&out);
    text.lines()
        .find(|line| line.split_whitespace().next() == Some(stem))
        .unwrap_or_else(|| panic!("status 缺 {stem} 行：{text}"))
        .to_string()
}

/// 层短名 → 状态行前缀（正文文件名）。
fn stem_of(layer: &str) -> &'static str {
    match layer {
        "req" => "requirement",
        "intent" => "intent",
        "spec" => "specification",
        "impl" => "implementation",
        other => panic!("未知层名：{other}"),
    }
}

#[test]
fn run_accepts_spec_defined_steps() {
    // 步骤名由数据规格给出，不写死。
    let out = run(&["run", "load", "clean", "analyze"]);
    assert!(
        out.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!stdout(&out).trim().is_empty());
}

#[test]
fn run_without_steps_defaults_to_the_whole_chain() {
    // 缺省跑全链。
    let out = run(&["run"]);
    assert!(
        out.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!stdout(&out).trim().is_empty());
}

#[test]
fn trace_runs_without_a_claim() {
    let (root, case) = case_with_fixtures("workflow-trace-forward");
    let out = run(&["--root", &root, "--case", &case, "trace"]);
    assert!(
        out.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!stdout(&out).trim().is_empty());
}

#[test]
fn trace_accepts_a_case_traceability_id() {
    // 追溯 ID 形如 `INT-001-H2`（案例里命题 H2 的编号）。
    let (root, case) = case_with_fixtures("workflow-trace-claim");
    let out = run(&[
        "--root",
        &root,
        "--case",
        &case,
        "trace",
        "--claim",
        "INT-001-H2",
    ]);
    assert!(
        out.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout(&out).contains("INT-001-H2"), "{}", stdout(&out));
}

#[test]
fn refute_accepts_a_downstream_to_upstream_pair() {
    // 案例 §5.4：数据规格挑战数据意图。
    let (root, case) = case_with_fixtures("workflow-refute-smoke");
    let out = run_with_stdin(
        &["--root", &root, "--case", &case, "refute", "spec", "intent"],
        refutation_body(),
    );
    assert!(
        out.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = stdout(&out);
    assert!(text.contains("spec"), "{text}");
    assert!(text.contains("intent"), "{text}");
}

#[test]
fn refute_records_each_legal_challenge_and_stales_the_target() {
    // 不变量：三个相邻的「下游 → 上游」各落一条记录，被挑战层（to）置 stale。
    let (root, case) = case_with_fixtures("refute-legal");
    let challenges = [("impl", "spec"), ("spec", "intent"), ("intent", "req")];
    let files = [
        "0001-impl-spec.md",
        "0002-spec-intent.md",
        "0003-intent-req.md",
    ];
    for ((from, to), file) in challenges.into_iter().zip(files) {
        let out = run_with_stdin(
            &["--root", &root, "--case", &case, "refute", from, to],
            refutation_body(),
        );
        assert!(
            out.status.success(),
            "{from}→{to} 应成功：{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let path = refutations_dir(&root, &case).join(file);
        assert!(path.is_file(), "应落盘 {file}");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            refutation_body(),
            "{file} 正文应等于 stdin"
        );
        let row = status_row(&root, &case, stem_of(to));
        assert!(row.contains("stale"), "被挑战层 {to} 应置 stale：{row}");
    }
    assert_eq!(record_count(&root, &case), 3, "三合法挑战应恰好三条记录");
}

#[test]
fn refute_accepts_full_layer_names() {
    // 不变量：短名或全名都收，落盘文件名用短名。
    let (root, case) = case_with_fixtures("refute-fullnames");
    let out = run_with_stdin(
        &[
            "--root",
            &root,
            "--case",
            &case,
            "refute",
            "implementation",
            "specification",
        ],
        refutation_body(),
    );
    assert!(
        out.status.success(),
        "全名应被接受：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        refutations_dir(&root, &case)
            .join("0001-impl-spec.md")
            .is_file(),
        "落盘文件名应用短名"
    );
}

#[test]
fn refute_increments_the_sequence_for_the_same_challenge() {
    // 不变量：同一挑战再落一次，序号递增而不覆盖。
    let (root, case) = case_with_fixtures("refute-seq");
    for _ in 0..2 {
        let out = run_with_stdin(
            &["--root", &root, "--case", &case, "refute", "impl", "spec"],
            refutation_body(),
        );
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let dir = refutations_dir(&root, &case);
    assert!(dir.join("0001-impl-spec.md").is_file(), "首条应为 0001");
    assert!(dir.join("0002-impl-spec.md").is_file(), "再落应为 0002");
    assert_eq!(record_count(&root, &case), 2, "两次应两条记录");
}

#[test]
fn refute_rejects_illegal_pairs_without_writing() {
    // 不变量：同层、方向颠倒、跨两层都退 1 且不落盘。
    let (root, case) = case_with_fixtures("refute-illegal");
    let illegal = [
        ("spec", "spec"),   // 同层
        ("req", "intent"),  // 方向颠倒
        ("impl", "intent"), // 跨两层
        ("intent", "spec"), // 方向颠倒
    ];
    for (from, to) in illegal {
        let out = run_with_stdin(
            &["--root", &root, "--case", &case, "refute", from, to],
            refutation_body(),
        );
        assert_eq!(
            out.status.code(),
            Some(1),
            "{from}→{to} 非法对应退 1：{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert_eq!(record_count(&root, &case), 0, "非法对不应落盘");
}

#[test]
fn refute_rejects_a_body_missing_a_field_without_writing() {
    // 不变量：三行字段缺一即拒、退 1、不落盘。
    let (root, case) = case_with_fixtures("refute-fields");
    let bodies = [
        "",
        "触发点：只有一行\n",
        "触发点：a\n结论：b\n",
        "触发点：a\n结论：\n改动：c\n",
    ];
    for body in bodies {
        let out = run_with_stdin(
            &["--root", &root, "--case", &case, "refute", "spec", "intent"],
            body,
        );
        assert_eq!(
            out.status.code(),
            Some(1),
            "正文 `{body}` 应退 1：{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert_eq!(record_count(&root, &case), 0, "缺字段不应落盘");
}

#[test]
fn upstream_update_leaves_empty_downstream_draft() {
    // 不变量：下游空正文保持 draft，不因上游更新而 stale。
    let root = temp_root("refute-empty-downstream");
    let root = root.to_str().expect("临时根目录路径").to_string();
    let case = "case-empty-downstream".to_string();
    assert!(
        run(&["--root", &root, "--case", &case, "req", "new"])
            .status
            .success()
    );
    let out = run_with_stdin(
        &["--root", &root, "--case", &case, "req", "update"],
        &fixture("requirement.md"),
    );
    assert!(
        out.status.success(),
        "req 更新应过门禁：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(status_row(&root, &case, "requirement").contains("validated"));
    for layer in ["intent", "spec", "impl"] {
        let row = status_row(&root, &case, stem_of(layer));
        assert!(row.contains("draft"), "空正文下游应保持 draft：{row}");
    }
}

#[test]
fn upstream_update_stales_only_downstream_with_a_body() {
    // 不变量：四层按序写完不留 stale；再改 req，已有正文的下游三层 stale，req 仍 validated。
    let (root, case) = case_with_fixtures("refute-cascade");
    for layer in ["req", "intent", "spec", "impl"] {
        let row = status_row(&root, &case, stem_of(layer));
        assert!(row.contains("validated"), "四层写完应无 stale：{row}");
    }

    let out = run_with_stdin(
        &["--root", &root, "--case", &case, "req", "update"],
        &fixture("requirement.md"),
    );
    assert!(
        out.status.success(),
        "req 更新应过门禁：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        status_row(&root, &case, "requirement").contains("validated"),
        "被更新层应重新 validated"
    );
    for layer in ["intent", "spec", "impl"] {
        let row = status_row(&root, &case, stem_of(layer));
        assert!(row.contains("stale"), "上游变更后 {layer} 应 stale：{row}");
    }
    // `status` 状态列能出现 `stale`。
    let text = stdout(&run(&["--root", &root, "--case", &case, "status"]));
    assert!(text.contains("stale"), "{text}");
}

#[test]
fn status_on_missing_case_fails_with_readable_error() {
    let out = run(&["status", "--case", "quanttide-search"]);
    assert!(!out.status.success(), "缺失案例应失败");
    assert_eq!(out.status.code(), Some(1), "退出码应为 1");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("quanttide-search"),
        "错误信息应含案例名：{err}"
    );
}

#[test]
fn status_without_a_case_resolves_the_default_case() {
    // `--case` 为可选参数，缺省是 `default`；该案例不存在时报错（与非零退出码）。
    let root = temp_root("status-default");
    let out = run(&["--root", root.to_str().unwrap(), "status"]);
    assert!(!out.status.success(), "工作空间里没有 default 案例应失败");
    assert_eq!(out.status.code(), Some(1), "退出码应为 1");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("default"), "错误信息应含默认案例名：{err}");
}

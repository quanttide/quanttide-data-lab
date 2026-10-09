//! 流程命令测试：`run` / `trace` / `refute` / `status`。
//!
//! 对应案例《量潮搜索工程》的数据规格五步、追溯矩阵（§5.2 追溯 ID）、
//! 反向挑战（§5.4 挑战与回退）与层状态。

mod common;

use common::{run, stdout};

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
    let out = run(&["trace"]);
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
    let out = run(&["trace", "--claim", "INT-001-H2"]);
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
    let out = run(&["refute", "spec", "intent"]);
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
fn status_accepts_the_search_case_name() {
    let out = run(&["status", "--case", "quanttide-search"]);
    assert!(
        out.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout(&out).contains("quanttide-search"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn status_runs_without_a_case() {
    // `--case` 为可选参数。
    let out = run(&["status"]);
    assert!(
        out.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

//! 参数校验测试：缺参或非法参数按 clap 规则失败（非零退出）。

mod common;

use common::run;

#[test]
fn top_level_rejects_unknown_command() {
    assert!(!run(&["bogus"]).status.success());
}

#[test]
fn each_layer_rejects_unknown_action() {
    for layer in ["req", "intent", "spec", "impl"] {
        assert!(
            !run(&[layer, "bogus"]).status.success(),
            "{layer} bogus 应失败"
        );
    }
}

#[test]
fn each_layer_requires_an_action() {
    for layer in ["req", "intent", "spec", "impl"] {
        assert!(!run(&[layer]).status.success(), "{layer} 缺 action 应失败");
    }
}

#[test]
fn refute_requires_both_layers() {
    assert!(!run(&["refute"]).status.success());
    assert!(!run(&["refute", "spec"]).status.success());
}

#[test]
fn flags_are_rejected_where_not_supported() {
    assert!(!run(&["status", "--bogus"]).status.success());
    assert!(
        !run(&["refute", "spec", "intent", "--bogus"])
            .status
            .success()
    );
}

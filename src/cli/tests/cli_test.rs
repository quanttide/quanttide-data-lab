//! CLI 命令面集成测试。
//!
//! 命令面按《量潮搜索工程》（`docs/gallery/quanttide-search/index.md`）的四层链路设计：
//! 数据需求 → 数据意图 → 数据规格 → 数据实现；数据规格的流程步骤交给 `run` 按规格执行，
//! 追溯矩阵与反向挑战分别对应 `trace`、`refute`，层状态对应 `status`。

mod common;

use common::{run, stdout};

#[test]
fn help_lists_every_command_of_the_case() {
    let out = run(&["--help"]);
    assert!(out.status.success(), "--help 应成功退出");
    let text = stdout(&out);
    for cmd in [
        "req", "intent", "spec", "impl", "run", "trace", "refute", "status",
    ] {
        assert!(text.contains(cmd), "help 缺少命令 `{cmd}`：\n{text}");
    }
}

#[test]
fn help_states_the_four_contract_types() {
    // 案例把四层定义为四种合同：价值 / 形式化验证 / 约束 / 工程。
    let text = stdout(&run(&["--help"]));
    for contract in ["价值合同", "形式化验证合同", "约束合同", "工程合同"] {
        assert!(text.contains(contract), "help 应含「{contract}」：\n{text}");
    }
}

#[test]
fn help_states_the_reverse_challenge_semantics() {
    // 案例 §5.4：下游挑战上游，触发回退与再生成。
    let out = run(&["refute", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("反向挑战"), "{text}");
    assert!(text.contains("证伪上游"), "{text}");
}

#[test]
fn help_states_that_steps_are_defined_by_the_specification() {
    // 案例 §「流程步骤」：五步由数据规格定义，命令不写死步骤。
    let out = run(&["run", "--help"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("数据规格定义的步骤"), "{text}");
}

//! 四层结构化门禁集成测试。
//!
//! 正样本为主仓「量潮搜索工程」案例按四个二级标题切出的四份原文
//! （`examples/fixtures/quanttide-search/`）：逐层过门禁并置 `validated`。
//! 反样本在夹具上各改坏一处：被门禁拒绝、退出码 1、正文照落盘、状态留 `draft`、
//! 理由指名规则。

mod common;

use common::{run, run_with_stdin, stdout, temp_root};

/// 四层命令：层命令、文件名、状态行前缀。
const LAYERS: [(&str, &str, &str); 4] = [
    ("req", "requirement.md", "requirement"),
    ("intent", "intent.md", "intent"),
    ("spec", "specification.md", "specification"),
    ("impl", "implementation.md", "implementation"),
];

/// 读取某层夹具原文。
fn fixture(file: &str) -> String {
    std::fs::read_to_string(format!("../../examples/fixtures/quanttide-search/{file}"))
        .expect("读取门禁夹具")
}

/// 取 `status` 输出中某层的状态行。
fn status_row(text: &str, stem: &str) -> String {
    text.lines()
        .find(|line| line.split_whitespace().next() == Some(stem))
        .unwrap_or_else(|| panic!("status 缺 {stem} 行：{text}"))
        .to_string()
}

#[test]
fn every_fixture_passes_its_layer_gate_and_validates() {
    for (layer, file, stem) in LAYERS {
        let root = temp_root(&format!("gate-pass-{layer}"));
        let root = root.to_str().expect("临时根目录路径");
        let case = format!("case-{layer}");

        let new = run(&["--root", root, "--case", &case, layer, "new"]);
        assert!(
            new.status.success(),
            "{layer} new 应成功：{}",
            String::from_utf8_lossy(&new.stderr)
        );

        let update = run_with_stdin(
            &["--root", root, "--case", &case, layer, "update"],
            &fixture(file),
        );
        assert!(
            update.status.success(),
            "{layer} 夹具应过门禁，stderr：{}",
            String::from_utf8_lossy(&update.stderr)
        );

        let status = stdout(&run(&["--root", root, "--case", &case, "status"]));
        for (_, _, other) in LAYERS {
            let row = status_row(&status, other);
            if other == stem {
                assert!(
                    row.contains("validated"),
                    "{layer} 过门禁后本层应 validated：{row}"
                );
            } else {
                assert!(
                    row.contains("draft"),
                    "{layer} 更新不应改动 {other} 层：{row}"
                );
            }
        }
    }
}

#[test]
fn broken_fixture_is_rejected_stays_draft_and_names_the_rule() {
    // 每层反样本：把该层「改坏一处」，并给出应命中的规则号。
    let cases = [
        ("req", "REQ-1"),
        ("intent", "INT-2"),
        ("spec", "SPEC-2"),
        ("impl", "IMP-3"),
    ];
    for (layer, rule) in cases {
        let (_, file, stem) = *LAYERS
            .iter()
            .find(|(name, _, _)| *name == layer)
            .expect("层已登记");
        let root = temp_root(&format!("gate-reject-{layer}"));
        let root = root.to_str().expect("临时根目录路径");
        let case = format!("case-{layer}");
        assert!(
            run(&["--root", root, "--case", &case, layer, "new"])
                .status
                .success()
        );

        let broken = break_fixture(layer, &fixture(file));
        let update = run_with_stdin(&["--root", root, "--case", &case, layer, "update"], &broken);

        assert_eq!(
            update.status.code(),
            Some(1),
            "{layer} 反样本应被门禁拒绝并退出码 1，stderr：{}",
            String::from_utf8_lossy(&update.stderr)
        );
        let stderr = String::from_utf8_lossy(&update.stderr);
        assert!(
            stderr.contains(rule),
            "{layer} 拒绝理由应指名 {rule}：{stderr}"
        );

        // 正文照落盘。
        let show = stdout(&run(&["--root", root, "--case", &case, layer, "show"]));
        assert_eq!(show, broken, "{layer} 未过门禁也应落盘原文");

        // 状态留 draft。
        let status = stdout(&run(&["--root", root, "--case", &case, "status"]));
        let row = status_row(&status, stem);
        assert!(row.contains("draft"), "{layer} 未过门禁应留 draft：{row}");
    }
}

/// 在夹具上改坏一处，且必须确实改动（否则反样本失效）。
fn break_fixture(layer: &str, source: &str) -> String {
    let broken = match layer {
        // REQ：把成功指标写成交付物。
        "req" => source.replace(
            "| 搜索相关失败率 | 含搜索会话中，因搜索失败导致任务失败的占比 | 相对基线下降 |",
            "| 产出报告 | 交付一份诊断报告 | 完成 |",
        ),
        // INT：把命题表的「可证伪条件」列改没，命题失去可证伪条件。
        "intent" => source.replace("| 假设 | 内容 | 可证伪条件 |", "| 假设 | 内容 | 备注 |"),
        // SPEC：把流程步骤表的「验收」列改没，步骤失去验收标准。
        "spec" => source.replace(
            "| 步骤 | 输入 | 处理规则 | 产物 | 验收 |",
            "| 步骤 | 输入 | 处理规则 | 产物 | 说明 |",
        ),
        // IMP：另立一个上游层的「操作定义」小节。
        "impl" => format!("{source}\n### 操作定义\n\n- 搜索判定：本层另立的定义。\n"),
        other => panic!("未知层命令：{other}"),
    };
    assert_ne!(broken, source, "{layer} 反样本必须确实改坏了夹具");
    broken
}

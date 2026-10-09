//! 四层命令测试：每层的 `new|update|show` 动作与层级自述。
//!
//! 对应案例四层：数据需求（价值合同）、数据意图（形式化验证合同）、
//! 数据规格（约束合同）、数据实现（工程合同）。

mod common;

use common::{run, run_with_stdin, stdout, temp_root};

/// 层命令 → 案例中该层的一句话定位。
const LAYERS: [(&str, &str); 4] = [
    ("req", "数据需求（价值合同）"),
    ("intent", "数据意图（形式化验证合同）"),
    ("spec", "数据规格（约束合同）"),
    ("impl", "数据实现（工程合同）"),
];

/// 每层共有的动作：新建、更新、查看。
const ACTIONS: [&str; 3] = ["new", "update", "show"];

/// 该层过门禁的夹具正文（主仓量潮搜索工程案例逐层切分）。
fn fixture_for(layer: &str) -> String {
    let file = match layer {
        "req" => "requirement.md",
        "intent" => "intent.md",
        "spec" => "specification.md",
        "impl" => "implementation.md",
        other => panic!("未知层命令：{other}"),
    };
    std::fs::read_to_string(format!("../../examples/fixtures/quanttide-search/{file}"))
        .expect("读取门禁夹具")
}

#[test]
fn each_layer_help_states_its_name_and_contract() {
    for (layer, about) in LAYERS {
        let out = run(&[layer, "--help"]);
        assert!(out.status.success(), "{layer} --help 应成功退出");
        assert!(
            stdout(&out).contains(about),
            "{layer} --help 应包含「{about}」，实际：\n{}",
            stdout(&out)
        );
    }
}

#[test]
fn each_layer_lists_the_three_actions() {
    for (layer, _) in LAYERS {
        let text = stdout(&run(&[layer, "--help"]));
        for action in ACTIONS {
            assert!(
                text.contains(action),
                "{layer} --help 应列出 `{action}`：\n{text}"
            );
        }
    }
}

#[test]
fn each_layer_dispatches_every_action() {
    for (index, (layer, _)) in LAYERS.iter().enumerate() {
        let layer = *layer;
        let root_dir = temp_root(&format!("layer-{index}"));
        let root = root_dir.to_str().expect("临时根目录路径");
        let case = format!("layer-{index}");
        let case = case.as_str();
        for action in ACTIONS {
            let out = if action == "update" {
                run_with_stdin(
                    &["--root", root, "--case", case, layer, action],
                    &fixture_for(layer),
                )
            } else {
                run(&["--root", root, "--case", case, layer, action])
            };
            assert!(
                out.status.success(),
                "{layer} {action} 应成功退出，stderr：{}",
                String::from_utf8_lossy(&out.stderr)
            );
            assert!(!stdout(&out).trim().is_empty(), "{layer} {action} 应有输出");
        }
    }
}

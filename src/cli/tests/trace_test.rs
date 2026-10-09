//! 追溯矩阵命令测试：`trace` 正查 / 反查 / 断链。
//!
//! 夹具为主仓「量潮搜索工程」案例按四个二级标题切出的四份原文
//! （`examples/fixtures/quanttide-search/`），四份都过门禁；追溯矩阵 5 行，
//! 末行为整层通配（`全部命题`）。

mod common;

use common::{LAYERS, case_with_fixtures, fixture, run, run_with_stdin, stdout, temp_root};

/// 「追溯矩阵」小节的数据行数（表头与分隔行不算）。
fn matrix_rows() -> usize {
    let spec = fixture("specification.md");
    let matrix = &spec[spec.find("### 追溯矩阵").expect("夹具含追溯矩阵小节")..];
    matrix
        .lines()
        .filter(|line| {
            let line = line.trim();
            line.starts_with('|') && !line.contains("--") && !line.contains("报告章节")
        })
        .count()
}

/// 非空输出行。
fn lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .collect()
}

/// 建一个案例，四份夹具写进去，但规格层的追溯矩阵把第 1 行命题 ID 改成
/// 数据意图层不存在的值（断链反样本）。
fn case_with_broken_claim(tag: &str) -> (String, String) {
    let root = temp_root(tag);
    let root = root.to_str().expect("临时根目录路径").to_string();
    let case = format!("case-{tag}");
    assert!(
        run(&["--root", &root, "--case", &case, "req", "new"])
            .status
            .success(),
        "建案例应成功"
    );
    for (layer, file) in LAYERS {
        let mut body = fixture(file);
        if layer == "spec" {
            let broken = body.replace("| 结果质量 | INT-001-H1 |", "| 结果质量 | INT-001-H9 |");
            assert_ne!(broken, body, "反样本必须确实改坏命题格");
            body = broken;
        }
        let out = run_with_stdin(&["--root", &root, "--case", &case, layer, "update"], &body);
        assert!(
            out.status.success(),
            "{layer} update 应成功：{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    (root, case)
}

#[test]
fn trace_lists_one_line_per_matrix_row_with_four_columns() {
    // 不变量：正查链数等于矩阵数据行数，一条一行，每行四列齐全。
    let (root, case) = case_with_fixtures("trace-forward");
    let out = run(&["--root", &root, "--case", &case, "trace"]);
    assert!(
        out.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let text = stdout(&out);
    let rows = lines(&text);
    assert_eq!(
        rows.len(),
        matrix_rows(),
        "正查应一条链路一行，链数等于矩阵行数：\n{text}"
    );
    for row in &rows {
        assert_eq!(row.matches(" → ").count(), 3, "每行应为四列：{row}");
    }
    for value in ["结果质量", "INT-001-H1", "空结果率", "搜索相关失败率"] {
        assert!(text.contains(value), "输出缺 `{value}`：\n{text}");
    }
}

#[test]
fn trace_back_by_assertion_id_lists_only_matching_links() {
    // 不变量：按命题反查，只出命题格显式含该 ID 的链路，外加整层通配那一行。
    let (root, case) = case_with_fixtures("trace-back-claim");
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

    let text = stdout(&out);
    let rows = lines(&text);
    assert_eq!(
        rows.len(),
        3,
        "H2 应命中「行为模式」「搜索后首个动作分布」及整层通配：\n{text}"
    );
    for row in &rows {
        assert!(
            row.contains("INT-001-H2") || row.contains("全部"),
            "命中链路应含 H2 或整层通配：{row}"
        );
    }
    assert!(!text.contains("INT-001-H1"), "{text}");
    assert!(!text.contains("INT-001-H3"), "{text}");
}

#[test]
fn trace_back_by_report_section_hits_exactly_one_link() {
    // 不变量：按报告章节名反查，恰好命中矩阵里写该章节的那一行。
    let (root, case) = case_with_fixtures("trace-back-section");
    let out = run(&[
        "--root",
        &root,
        "--case",
        &case,
        "trace",
        "--claim",
        "结果体量",
    ]);
    assert!(
        out.status.success(),
        "stderr：{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let text = stdout(&out);
    let rows = lines(&text);
    assert_eq!(rows.len(), 1, "「结果体量」应恰好命中一条链路：\n{text}");
    assert!(
        rows[0].contains("结果体量") && rows[0].contains("INT-001-H3"),
        "{text}"
    );
}

#[test]
fn trace_back_with_an_unknown_assertion_id_misses_and_fails() {
    // 不变量：不存在的断言 ID 不得因整层通配而命中；未命中以退出码 1 结束。
    let (root, case) = case_with_fixtures("trace-back-miss");
    let out = run(&[
        "--root",
        &root,
        "--case",
        &case,
        "trace",
        "--claim",
        "INT-001-H9",
    ]);
    assert_eq!(out.status.code(), Some(1), "未命中应以退出码 1 结束");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("未命中"), "应报未命中：{stderr}");
    assert!(
        stderr.contains("INT-001-H9"),
        "未命中信息应含查询值：{stderr}"
    );
}

#[test]
fn trace_reports_a_chain_break_by_cell_and_still_lists_links() {
    // 反样本：矩阵第 1 行命题 ID 在数据意图层无对应行，应报断链并指名到格。
    let (root, case) = case_with_broken_claim("trace-break");
    let out = run(&["--root", &root, "--case", &case, "trace"]);
    assert_eq!(out.status.code(), Some(1), "断链应以退出码 1 结束");

    let text = stdout(&out);
    // 不变量：断链指名到格——行、列、值、原因。
    assert!(text.contains("断链"), "应报断链：\n{text}");
    assert!(
        text.contains("第 1 行") && text.contains("结果质量"),
        "应指名行与章节：\n{text}"
    );
    assert!(
        text.contains("命题") && text.contains("INT-001-H9"),
        "应指名列与值：\n{text}"
    );
    assert!(text.contains("数据意图层命题表"), "应给出原因：\n{text}");
    // 不变量：检出断链时仍打印链路（其余四行照出）。
    assert!(text.contains("INT-001-H3"), "断链时应仍打印链路：\n{text}");
}

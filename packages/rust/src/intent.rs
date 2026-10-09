//! 数据意图层（形式化验证合同）。
//!
//! 回答「意图如何被严格表达、怎么定义、怎么证、验了什么」：形式化对象、操作定义、指标、
//! 命题与可证伪条件、因果图、约束与不变量、验证方法、验证结论、验收清单。

use crate::{column, find_table, section, GateReport};

/// 数据意图层门禁。
///
/// 落实 ROADMAP「INT：每个概念有操作定义；每个命题有可证伪条件；相关与因果分列」。
pub fn gate(body: &str) -> GateReport {
    let mut reasons = Vec::new();

    // 落实「每个概念有操作定义」：须有「操作定义」小节，且至少给出一条
    // 「术语：定义」形式的条目。
    match section(body, "操作定义") {
        None => reasons.push("INT-1 每个概念须有操作定义：未见「操作定义」小节".into()),
        Some(text) => {
            let definitions = text
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    (trimmed.starts_with("- ") || trimmed.starts_with("* "))
                        && trimmed.contains('：')
                })
                .count();
            if definitions == 0 {
                reasons.push("INT-1 「操作定义」小节未给出任何「术语：定义」条目".into());
            }
        }
    }

    // 落实「每个命题有可证伪条件」：命题须以「假设 | 内容 | 可证伪条件」表格给出，
    // 且逐行都填了可证伪条件。
    match find_table(body, &["假设", "内容", "可证伪条件"]) {
        None => {
            reasons.push("INT-2 每个命题须有可证伪条件：未见「假设 | 内容 | 可证伪条件」表".into())
        }
        Some(table) => {
            if table.rows.is_empty() {
                reasons.push("INT-2 命题表为空：至少给出一个可证伪命题".into());
            }
            let condition_col = column(&table.header, "可证伪条件");
            for (index, row) in table.rows.iter().enumerate() {
                let condition = condition_col
                    .and_then(|col| row.get(col))
                    .map(String::as_str)
                    .unwrap_or("");
                if condition.trim().is_empty() {
                    reasons.push(format!("INT-2 命题第 {} 行缺可证伪条件", index + 1));
                }
            }
        }
    }

    // 落实「相关与因果分列」：须有一处表述同时点出「相关」与「因果」，明确二者分列。
    if !body
        .lines()
        .any(|line| line.contains("相关") && line.contains("因果"))
    {
        reasons.push("INT-3 相关与因果须分列：未见同时区分「相关」与「因果」的表述".into());
    }

    GateReport::from_reasons(reasons)
}

/// 命题标签：取「命题与假设」表的「假设」列，如 `H1`。
pub fn claims(body: &str) -> Vec<String> {
    crate::column_values(body, &["假设", "内容", "可证伪条件"], "假设")
}

/// 指标名：取「指标」表的「指标」列。
pub fn metrics(body: &str) -> Vec<String> {
    crate::column_values(body, &["指标", "定义"], "指标")
}

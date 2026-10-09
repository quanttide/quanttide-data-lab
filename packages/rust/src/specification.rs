//! 数据规格层（约束合同）。
//!
//! 数据意图的具象、计算机实现的约束：数据源与采集规则、清洗与归一化规则、表结构、
//! 指标计算口径、流程步骤、质量校验、追溯矩阵。不写具体语言与工具。

use std::collections::HashSet;

use crate::{column, find_table, section, GateReport};

/// 数据规格层门禁。
///
/// 落实 ROADMAP「SPEC：每个指标可计算；每个流程步骤有验收标准；命名与口径唯一」。
pub fn gate(body: &str) -> GateReport {
    let mut reasons = Vec::new();

    // 落实「每个指标可计算」：须有「指标计算」小节，且其中给出口径或定义。
    match section(body, "指标计算") {
        None => reasons.push("SPEC-1 每个指标须可计算：未见「指标计算口径」小节".into()),
        Some(text) => {
            let has_caliber = text.lines().any(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && !trimmed.starts_with('#')
                    && (trimmed.contains("口径") || trimmed.contains("定义"))
            });
            if !has_caliber {
                reasons.push("SPEC-1 「指标计算口径」未给出计算口径或定义".into());
            }
        }
    }

    // 落实「每个流程步骤有验收标准」：流程步骤表须有「验收」列，且逐行都填了验收标准。
    match find_table(body, &["步骤", "输入", "处理规则", "产物", "验收"]) {
        None => reasons.push(
            "SPEC-2 每个流程步骤须有验收标准：未见「步骤 | 输入 | 处理规则 | 产物 | 验收」表"
                .into(),
        ),
        Some(table) => {
            if table.rows.is_empty() {
                reasons.push("SPEC-2 流程步骤表为空：至少给出一个步骤".into());
            }
            let step_col = column(&table.header, "步骤");
            let accept_col = column(&table.header, "验收");
            for (index, row) in table.rows.iter().enumerate() {
                let step = step_col
                    .and_then(|col| row.get(col))
                    .map(String::as_str)
                    .unwrap_or("");
                let accept = accept_col
                    .and_then(|col| row.get(col))
                    .map(String::as_str)
                    .unwrap_or("");
                if accept.trim().is_empty() {
                    let label = if step.trim().is_empty() {
                        format!("第 {} 行", index + 1)
                    } else {
                        step.to_string()
                    };
                    reasons.push(format!("SPEC-2 流程步骤「{label}」缺验收标准"));
                }
            }
        }
    }

    // 落实「命名与口径唯一」：表结构里的表名不得重复，且口径须唯一回指「数据意图」。
    if let Some(table) = find_table(body, &["表", "内容"]) {
        let name_col = column(&table.header, "表");
        let mut seen = HashSet::new();
        for row in &table.rows {
            let name = name_col
                .and_then(|col| row.get(col))
                .map(|cell| cell.trim())
                .unwrap_or("");
            if !name.is_empty() && !seen.insert(name.to_string()) {
                reasons.push(format!("SPEC-3 命名不唯一：表名「{name}」重复定义"));
            }
        }
    }
    if !body.lines().any(|line| {
        (line.contains("数据意图") || line.contains("INT"))
            && (line.contains("指标定义") || line.contains("口径"))
    }) {
        reasons.push("SPEC-3 口径须唯一：未见口径回指「数据意图」的唯一来源声明".into());
    }

    GateReport::from_reasons(reasons)
}

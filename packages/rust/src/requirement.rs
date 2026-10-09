//! 数据需求层（价值合同）。
//!
//! 回答「为什么做、为谁做、什么算成功」：痛点、目标与非目标、业务口径成功指标、范围、
//! 利益相关者、约束。不写形式化对象、指标公式、表结构与代码。

use crate::{column, find_table, has_bullet_section, GateReport};

/// 数据需求层门禁。
///
/// 落实 ROADMAP「REQ：成功指标是业务后果而非交付物；范围可界定；约束齐全」。
pub fn gate(body: &str) -> GateReport {
    let mut reasons = Vec::new();

    // 落实「成功指标是业务后果而非交付物」：成功指标须以
    // 「指标 | 口径 | 目标方向」三列表格给出；每行给出业务口径与目标方向，
    // 且指标名或口径不得是交付物（报告、文档、产物等）。
    match find_table(body, &["指标", "口径", "目标方向"]) {
        None => reasons.push(
            "REQ-1 成功指标须为「指标 | 口径 | 目标方向」三列表格（业务后果）：未见该表".into(),
        ),
        Some(table) => {
            if table.rows.is_empty() {
                reasons.push("REQ-1 成功指标表为空：至少给出一个业务指标".into());
            }
            let metric_col = column(&table.header, "指标");
            let caliber_col = column(&table.header, "口径");
            let direction_col = column(&table.header, "目标方向");
            const DELIVERABLES: [&str; 6] = ["报告", "文档", "交付物", "产物", "白皮书", "幻灯片"];
            for (index, row) in table.rows.iter().enumerate() {
                let metric = metric_col
                    .and_then(|col| row.get(col))
                    .map(String::as_str)
                    .unwrap_or("");
                let caliber = caliber_col
                    .and_then(|col| row.get(col))
                    .map(String::as_str)
                    .unwrap_or("");
                for cell in [metric, caliber] {
                    if let Some(word) = DELIVERABLES.iter().find(|word| cell.contains(**word)) {
                        reasons.push(format!(
                            "REQ-1 成功指标第 {} 行像交付物（含「{word}」）：{cell}；成功指标须是业务后果",
                            index + 1
                        ));
                    }
                }
                let direction = direction_col
                    .and_then(|col| row.get(col))
                    .map(String::as_str)
                    .unwrap_or("");
                if direction.trim().is_empty() {
                    reasons.push(format!("REQ-1 成功指标第 {} 行缺「目标方向」", index + 1));
                }
            }
        }
    }

    // 落实「范围可界定」：须有「范围」小节，且其中至少列出一条边界。
    if !has_bullet_section(body, "范围") {
        reasons.push("REQ-2 范围须可界定：未见「范围」小节或其中无可界定的条目".into());
    }

    // 落实「约束齐全」：须有「约束」小节，且其中至少列出一条约束。
    if !has_bullet_section(body, "约束") {
        reasons.push("REQ-3 约束须齐全：未见「约束」小节或其中无约束条目".into());
    }

    GateReport::from_reasons(reasons)
}

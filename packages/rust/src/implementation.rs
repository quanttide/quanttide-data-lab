//! 数据实现层（工程合同）。
//!
//! 只写具体工程：技术栈、数据管道、命令行入口、质量校验脚本、版本管理、执行验证、报告生成。
//! 不重新定义概念、不重新解释指标、不另立流程。

use crate::{section, GateReport};

/// 数据实现层门禁。
///
/// 落实 ROADMAP「IMP：定义来自 `INT`、约束来自 `SPEC`，不另立定义与流程」。
pub fn gate(body: &str) -> GateReport {
    let mut reasons = Vec::new();

    // 落实「定义来自 INT」：正文须显式引用「数据意图」/`INT`。
    if !body.contains("数据意图") && !body.contains("INT") {
        reasons.push("IMP-1 定义须来自 INT：未见对「数据意图」的引用".into());
    }

    // 落实「约束来自 SPEC」：正文须显式引用「数据规格」/`SPEC`。
    if !body.contains("数据规格") && !body.contains("SPEC") {
        reasons.push("IMP-2 约束须来自 SPEC：未见对「数据规格」的引用".into());
    }

    // 落实「不另立定义与流程」：不得出现属于上游层的定义或流程小节标题。
    const FORBIDDEN: [&str; 5] = ["操作定义", "命题与假设", "表结构", "指标计算", "流程步骤"];
    for name in FORBIDDEN {
        if section(body, name).is_some() {
            reasons.push(format!(
                "IMP-3 不得另立定义与流程：出现上游层小节「{name}」"
            ));
        }
    }

    GateReport::from_reasons(reasons)
}

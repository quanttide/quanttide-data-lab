//! 数据规格层（约束合同）。
//!
//! 数据意图的具象、计算机实现的约束：数据源与采集规则、清洗与归一化规则、表结构、
//! 指标计算口径、流程步骤、质量校验、追溯矩阵。不写具体语言与工具。

use std::collections::HashSet;

use crate::{column, find_table, section, GateReport, Layer};

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

/// 追溯 ID 的层前缀：`REQ` / `INT` / `SPEC` / `IMP`。
fn layer_from_prefix(prefix: &str) -> Option<Layer> {
    Layer::ALL
        .into_iter()
        .find(|layer| layer.prefix() == prefix)
}

/// 解析三位层级编号，如 `001`。
fn parse_number(text: &str) -> Option<u32> {
    if text.len() == 3 && text.chars().all(|c| c.is_ascii_digit()) {
        text.parse().ok()
    } else {
        None
    }
}

/// 层级 ID：`<前缀>-<三位编号>`，如 `INT-001`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerId {
    /// 所属层。
    pub layer: Layer,
    /// 层级编号（三位）。
    pub number: u32,
}

impl std::fmt::Display for LayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = self.layer.prefix();
        let number = self.number;
        write!(f, "{prefix}-{number:03}")
    }
}

/// 断言 ID：挂在层级 ID 下的具体断言，如 `INT-001-H2`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionId {
    /// 所属层。
    pub layer: Layer,
    /// 层级编号（三位）。
    pub number: u32,
    /// 断言标签，如 `H2`。
    pub label: String,
}

impl std::fmt::Display for AssertionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = self.layer.prefix();
        let number = self.number;
        let label = &self.label;
        write!(f, "{prefix}-{number:03}-{label}")
    }
}

/// 层级 ID 或挂在层级下的断言 ID。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceId {
    /// `<前缀>-<三位编号>`，如 `REQ-001`。
    Layer(LayerId),
    /// `<前缀>-<三位编号>-<标签>`，如 `INT-001-H2`。
    Assertion(AssertionId),
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceId::Layer(id) => write!(f, "{id}"),
            TraceId::Assertion(id) => write!(f, "{id}"),
        }
    }
}

/// 解析层级 ID，拒绝多出的段。
pub fn parse_layer_id(text: &str) -> Result<LayerId, String> {
    let text = text.trim();
    let mut parts = text.split('-');
    let prefix = parts.next().unwrap_or("");
    let number = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return Err(format!("`{text}` 不是层级 ID（形如 `INT-001`）"));
    }
    let layer = layer_from_prefix(prefix)
        .ok_or_else(|| format!("`{text}` 的层前缀未知（应为 REQ/INT/SPEC/IMP）"))?;
    let number = parse_number(number).ok_or_else(|| format!("`{text}` 的层级编号须为三位数字"))?;
    Ok(LayerId { layer, number })
}

/// 解析断言 ID，要求恰有三段。
pub fn parse_assertion_id(text: &str) -> Result<AssertionId, String> {
    let text = text.trim();
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 3 {
        return Err(format!("`{text}` 不是断言 ID（形如 `INT-001-H2`）"));
    }
    let layer = layer_from_prefix(parts[0])
        .ok_or_else(|| format!("`{text}` 的层前缀未知（应为 REQ/INT/SPEC/IMP）"))?;
    let number =
        parse_number(parts[1]).ok_or_else(|| format!("`{text}` 的层级编号须为三位数字"))?;
    let label = parts[2].trim();
    if label.is_empty() {
        return Err(format!("`{text}` 的断言标签为空"));
    }
    Ok(AssertionId {
        layer,
        number,
        label: label.to_string(),
    })
}

/// 解析追溯 ID：先试断言 ID，再试层级 ID。
pub fn parse_trace_id(text: &str) -> Result<TraceId, String> {
    if let Ok(id) = parse_assertion_id(text) {
        return Ok(TraceId::Assertion(id));
    }
    parse_layer_id(text).map(TraceId::Layer)
}

/// 把标签拆成字母与数字，如 `H2` → (`H`, 2)。
fn label_parts(label: &str) -> Option<(&str, u32)> {
    let split = label.find(|c: char| c.is_ascii_digit())?;
    let (letters, digits) = label.split_at(split);
    if letters.is_empty() || digits.is_empty() {
        return None;
    }
    let number: u32 = digits.parse().ok()?;
    Some((letters, number))
}

/// 展开断言区间，如 `INT-001-H1–H3` → `INT-001-H1`、`INT-001-H2`、`INT-001-H3`。
fn expand_range(start: &str, end: &str) -> Result<Vec<AssertionId>, String> {
    let start = parse_assertion_id(start)?;
    let end_label = end.rsplit('-').next().unwrap_or("");
    let (start_letters, start_number) = label_parts(&start.label)
        .ok_or_else(|| format!("断言区间起点 `{start}` 的标签无法解析"))?;
    let (end_letters, end_number) =
        label_parts(end_label).ok_or_else(|| format!("断言区间终点 `{end}` 的标签无法解析"))?;
    if start_letters != end_letters || start_number > end_number {
        return Err(format!("断言区间 `{start}–{end}` 两端不一致"));
    }
    Ok((start_number..=end_number)
        .map(|n| AssertionId {
            layer: start.layer,
            number: start.number,
            label: format!("{start_letters}{n}"),
        })
        .collect())
}

/// 解析「命题」格：`、` 分隔的断言 ID，支持 `INT-001-H1–H3` 区间。
pub fn parse_assertion_cell(cell: &str) -> Result<Vec<AssertionId>, String> {
    let mut out = Vec::new();
    for token in cell.split('、') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        if let Some((start, end)) = token.split_once('–') {
            out.extend(expand_range(start.trim(), end.trim())?);
        } else {
            out.push(parse_assertion_id(token)?);
        }
    }
    Ok(out)
}

/// 追溯矩阵的一格：若干具体值，或 `全部…` 整层通配。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatrixCell<T> {
    /// 具体值。
    Items(Vec<T>),
    /// `全部…`：覆盖该列整层。
    All,
}

impl<T> MatrixCell<T> {
    /// 是否为整层通配。
    pub fn is_all(&self) -> bool {
        matches!(self, MatrixCell::All)
    }

    /// 具体值切片；整层通配时为空。
    pub fn items(&self) -> &[T] {
        match self {
            MatrixCell::Items(items) => items,
            MatrixCell::All => &[],
        }
    }
}

/// 追溯矩阵的一行：一条「报告章节 → 命题 → 数据意图指标 → 业务成功指标」链路。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceLink {
    /// 报告章节。
    pub section: MatrixCell<String>,
    /// 命题（挂在数据意图层的断言 ID）。
    pub claims: MatrixCell<AssertionId>,
    /// 数据意图指标名。
    pub intent_metrics: MatrixCell<String>,
    /// 业务成功指标名。
    pub success_metrics: MatrixCell<String>,
}

/// 一格断链：所在行、所在格与原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainBreak {
    /// 行号，自 1 起（表头不算）。
    pub row: usize,
    /// 该行的报告章节，用于定位。
    pub section: String,
    /// 列名。
    pub column: &'static str,
    /// 出问题的值。
    pub value: String,
    /// 断链原因。
    pub reason: String,
}

impl std::fmt::Display for ChainBreak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let row = self.row;
        let section = &self.section;
        let column = self.column;
        let value = &self.value;
        let reason = &self.reason;
        write!(
            f,
            "追溯矩阵第 {row} 行「{section}」的「{column}」格：{value}——{reason}"
        )
    }
}

/// 解析名称格：`、` 分隔的名称；以「全部」开头视为整层通配。
fn parse_names_cell(cell: &str) -> MatrixCell<String> {
    if cell.trim().starts_with("全部") {
        return MatrixCell::All;
    }
    MatrixCell::Items(
        cell.split('、')
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .collect(),
    )
}

/// 解析命题格：先判整层通配，否则按断言 ID 解析。
fn parse_claims_cell(cell: &str) -> Result<MatrixCell<AssertionId>, String> {
    if cell.trim().starts_with("全部") {
        return Ok(MatrixCell::All);
    }
    Ok(MatrixCell::Items(parse_assertion_cell(cell)?))
}

/// 从数据规格层正文的「追溯矩阵」小节解析四列表。
pub fn traceability(body: &str) -> Result<Vec<TraceLink>, String> {
    let text =
        section(body, "追溯矩阵").ok_or_else(|| "数据规格层未见「追溯矩阵」小节".to_string())?;
    let table = find_table(&text, &["报告章节", "命题", "数据意图指标", "业务成功指标"])
        .ok_or_else(|| {
            "「追溯矩阵」须为「报告章节 | 命题 | 数据意图指标 | 业务成功指标」四列表".to_string()
        })?;
    let col = |name: &str| column(&table.header, name).expect("追溯矩阵表头已校验");
    let (section_col, claim_col, intent_col, success_col) = (
        col("报告章节"),
        col("命题"),
        col("数据意图指标"),
        col("业务成功指标"),
    );
    let mut links = Vec::new();
    for (index, row) in table.rows.iter().enumerate() {
        let line = index + 1;
        let cell = |position: usize| row.get(position).map(String::as_str).unwrap_or("").trim();
        let claims = parse_claims_cell(cell(claim_col))
            .map_err(|err| format!("追溯矩阵第 {line} 行「命题」格：{err}"))?;
        links.push(TraceLink {
            section: parse_names_cell(cell(section_col)),
            claims,
            intent_metrics: parse_names_cell(cell(intent_col)),
            success_metrics: parse_names_cell(cell(success_col)),
        });
    }
    Ok(links)
}

/// 该行报告章节的可读标签，用于定位断链。
fn cell_label(cell: &MatrixCell<String>, all: &str) -> String {
    match cell {
        MatrixCell::Items(items) => items.join("、"),
        MatrixCell::All => all.to_string(),
    }
}

/// 检测矩阵断链：命题须在意图层命题表有行；指标名须在意图层指标表/需求层成功指标表存在；
/// 报告章节须在实现层出现。返回逐格理由。
pub fn chain_breaks(
    links: &[TraceLink],
    intent_claims: &[String],
    intent_metrics: &[String],
    success_metrics: &[String],
    implementation: &str,
) -> Vec<ChainBreak> {
    let mut breaks = Vec::new();
    for (index, link) in links.iter().enumerate() {
        let row = index + 1;
        let section = cell_label(&link.section, "全部章节");
        if let MatrixCell::Items(sections) = &link.section {
            for name in sections {
                if !implementation.contains(name.as_str()) {
                    breaks.push(ChainBreak {
                        row,
                        section: section.clone(),
                        column: "报告章节",
                        value: name.clone(),
                        reason: "不在数据实现层「报告生成」章节".to_string(),
                    });
                }
            }
        }
        if let MatrixCell::Items(claims) = &link.claims {
            for claim in claims {
                let reason = if claim.layer != Layer::Intent {
                    Some("命题须挂在数据意图层".to_string())
                } else if !intent_claims.iter().any(|label| label == &claim.label) {
                    Some("在数据意图层命题表无对应行".to_string())
                } else {
                    None
                };
                if let Some(reason) = reason {
                    breaks.push(ChainBreak {
                        row,
                        section: section.clone(),
                        column: "命题",
                        value: claim.to_string(),
                        reason,
                    });
                }
            }
        }
        if let MatrixCell::Items(metrics) = &link.intent_metrics {
            for name in metrics {
                if !intent_metrics.iter().any(|metric| metric == name) {
                    breaks.push(ChainBreak {
                        row,
                        section: section.clone(),
                        column: "数据意图指标",
                        value: name.clone(),
                        reason: "不在数据意图层指标表".to_string(),
                    });
                }
            }
        }
        if let MatrixCell::Items(metrics) = &link.success_metrics {
            for name in metrics {
                if !success_metrics.iter().any(|metric| metric == name) {
                    breaks.push(ChainBreak {
                        row,
                        section: section.clone(),
                        column: "业务成功指标",
                        value: name.clone(),
                        reason: "不在数据需求层成功指标表".to_string(),
                    });
                }
            }
        }
        if link.claims.is_all() && intent_claims.is_empty() {
            breaks.push(ChainBreak {
                row,
                section: section.clone(),
                column: "命题",
                value: "全部".to_string(),
                reason: "整层通配但数据意图层命题表为空".to_string(),
            });
        }
        if link.intent_metrics.is_all() && intent_metrics.is_empty() {
            breaks.push(ChainBreak {
                row,
                section: section.clone(),
                column: "数据意图指标",
                value: "全部".to_string(),
                reason: "整层通配但数据意图层指标表为空".to_string(),
            });
        }
        if link.success_metrics.is_all() && success_metrics.is_empty() {
            breaks.push(ChainBreak {
                row,
                section: section.clone(),
                column: "业务成功指标",
                value: "全部".to_string(),
                reason: "整层通配但数据需求层成功指标表为空".to_string(),
            });
        }
    }
    breaks
}

/// 某条链路是否命中查询：`claim` 取报告章节名或断言 ID。
///
/// 整层通配 `全部命题` 只覆盖数据意图层真实存在的命题——`known_claims` 传该层命题表的标签，
/// 不存在的断言 ID 不得因通配而命中。
pub fn link_matches(link: &TraceLink, claim: &str, known_claims: &[String]) -> bool {
    let claim = claim.trim();
    if let Ok(id) = parse_assertion_id(claim) {
        match &link.claims {
            MatrixCell::Items(ids) => ids.iter().any(|candidate| candidate == &id),
            MatrixCell::All => known_claims.iter().any(|label| label == &id.label),
        }
    } else {
        match &link.section {
            MatrixCell::Items(names) => names.iter().any(|name| name == claim),
            MatrixCell::All => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{intent, requirement, Workspace};
    use std::path::PathBuf;

    /// 读案例原文切出的某层夹具。
    fn fixture(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/fixtures/quanttide-search")
            .join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("读取夹具 {} 失败：{err}", path.display()))
    }

    /// 建一个空的临时工作空间根目录。
    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "quanttide-data-lab-trace-{}-{tag}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("建临时根目录");
        dir
    }

    #[test]
    fn id_parsing_separates_layer_and_assertion_and_expands_ranges() {
        // 不变量：区间展开为逐条断言 ID，且层与层级编号一致。
        let expanded = parse_assertion_cell("INT-001-H1–H3").expect("区间可解析");
        let labels: Vec<&str> = expanded.iter().map(|id| id.label.as_str()).collect();
        assert_eq!(labels, ["H1", "H2", "H3"]);
        assert!(expanded
            .iter()
            .all(|id| id.layer == Layer::Intent && id.number == 1));

        // 不变量：多个断言 ID 用「、」分隔。
        let multi = parse_assertion_cell("INT-001-H1、SPEC-001-P2").expect("多 ID 可解析");
        assert_eq!(multi.len(), 2);
        assert_eq!(multi[1].to_string(), "SPEC-001-P2");

        // 不变量：层级 ID 与断言 ID 是两种形态，互不被对方接受。
        assert!(parse_layer_id("INT-001").is_ok());
        assert!(parse_assertion_id("INT-001").is_err());
        assert!(parse_layer_id("INT-001-H1").is_err());
        assert!(matches!(parse_trace_id("REQ-001"), Ok(TraceId::Layer(_))));
    }

    #[test]
    fn matrix_yields_one_link_per_row_with_intent_claims() {
        let links = traceability(&fixture("specification.md")).expect("矩阵可解析");
        assert!(!links.is_empty(), "矩阵至少给出一行链路");

        // 不变量：命题都是挂在数据意图层的断言 ID。
        let explicit: Vec<String> = links
            .iter()
            .flat_map(|link| link.claims.items().iter().map(|id| id.to_string()))
            .collect();
        assert!(!explicit.is_empty());
        assert!(
            explicit.iter().all(|id| id.starts_with("INT-001-")),
            "命题须为数据意图层断言 ID：{explicit:?}"
        );

        // 不变量：矩阵写了一处整层通配（区间与多 ID 的写法在 id_parsing 用例里逐条验）。
        assert!(links.iter().any(|link| link.claims.is_all()));
    }

    #[test]
    fn valid_matrix_has_no_chain_break() {
        let links = traceability(&fixture("specification.md")).expect("矩阵可解析");
        let breaks = chain_breaks(
            &links,
            &intent::claims(&fixture("intent.md")),
            &intent::metrics(&fixture("intent.md")),
            &requirement::success_metrics(&fixture("requirement.md")),
            &fixture("implementation.md"),
        );
        assert!(breaks.is_empty(), "案例矩阵不应有断链：{breaks:?}");
    }

    #[test]
    fn forward_lists_all_links_and_back_matches_section_or_claim() {
        let root = temp_root("query");
        let workspace = Workspace::new(&root);
        let case = "trace-case";
        workspace.create(case).expect("建案例");
        workspace
            .write_doc(case, Layer::Requirement, &fixture("requirement.md"))
            .expect("写需求层");
        workspace
            .write_doc(case, Layer::Intent, &fixture("intent.md"))
            .expect("写意图层");
        workspace
            .write_doc(case, Layer::Specification, &fixture("specification.md"))
            .expect("写规格层");
        workspace
            .write_doc(case, Layer::Implementation, &fixture("implementation.md"))
            .expect("写实现层");

        // 不变量：正查链数等于「追溯矩阵」小节的数据行数，且案例矩阵无断链。
        let forward = workspace.trace_forward(case).expect("正查");
        let spec = fixture("specification.md");
        let matrix = &spec[spec.find("### 追溯矩阵").expect("夹具含追溯矩阵小节")..];
        let data_rows = matrix
            .lines()
            .filter(|line| {
                let line = line.trim();
                line.starts_with('|') && !line.contains("--") && !line.contains("报告章节")
            })
            .count();
        assert_eq!(forward.links.len(), data_rows, "正查应覆盖矩阵每一行");
        assert!(forward.breaks.is_empty(), "案例矩阵不应有断链");

        // 不变量：按命题反查，命中链路的命题格要么含该命题，要么是整层通配。
        let by_claim = workspace
            .trace_back(case, "INT-001-H1")
            .expect("按命题反查");
        assert!(!by_claim.links.is_empty());
        assert!(by_claim.links.iter().all(|link| {
            link.claims.is_all() || link.claims.items().iter().any(|id| id.label == "H1")
        }));

        // 不变量：按报告章节反查，命中链路的章节格必含该章节名。
        let by_section = workspace.trace_back(case, "结果体量").expect("按章节反查");
        assert!(!by_section.links.is_empty());
        assert!(by_section.links.iter().all(|link| {
            link.section.is_all() || link.section.items().iter().any(|name| name == "结果体量")
        }));

        // 不变量：不存在的命题不得因整层通配而命中（通配只覆盖数据意图层真实命题）。
        let missing = workspace
            .trace_back(case, "INT-001-H9")
            .expect("查不存在命题");
        assert!(
            missing.links.is_empty(),
            "不存在的命题不应命中任何链路：{:?}",
            missing.links
        );
    }

    #[test]
    fn broken_claim_reports_the_exact_cell() {
        let spec = fixture("specification.md");
        let broken = spec.replace("| 结果质量 | INT-001-H1 |", "| 结果质量 | INT-001-H9 |");
        assert_ne!(broken, spec, "反样本必须确实改坏命题格");
        let links = traceability(&broken).expect("矩阵仍可解析");
        let breaks = chain_breaks(
            &links,
            &intent::claims(&fixture("intent.md")),
            &intent::metrics(&fixture("intent.md")),
            &requirement::success_metrics(&fixture("requirement.md")),
            &fixture("implementation.md"),
        );
        assert_eq!(breaks.len(), 1, "应恰好报一处断链：{breaks:?}");
        let text = breaks[0].to_string();
        assert!(
            text.contains("结果质量") && text.contains("命题") && text.contains("INT-001-H9"),
            "断链须指名所在格：{text}"
        );
    }
}

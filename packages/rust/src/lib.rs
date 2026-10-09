//! quanttide-data-lab 库入口：四层形式化数据智能体的领域模型。
//!
//! 四层合同各占一个模块：
//!
//! | 模块 | 层级 | 合同 |
//! |------|------|------|
//! | [`requirement`] | 数据需求 | 价值合同 |
//! | [`intent`] | 数据意图 | 形式化验证合同 |
//! | [`specification`] | 数据规格 | 约束合同 |
//! | [`implementation`] | 数据实现 | 工程合同 |

pub mod error;
pub mod implementation;
pub mod intent;
pub mod requirement;
pub mod specification;

pub use error::LabError;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 四层合同的层级标识，用于追溯 ID（`REQ → INT → SPEC → IMP`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// 数据需求（价值合同）。
    Requirement,
    /// 数据意图（形式化验证合同）。
    Intent,
    /// 数据规格（约束合同）。
    Specification,
    /// 数据实现（工程合同）。
    Implementation,
}

impl Layer {
    /// 四层的固定顺序：需求 → 意图 → 规格 → 实现。
    pub const ALL: [Layer; 4] = [
        Layer::Requirement,
        Layer::Intent,
        Layer::Specification,
        Layer::Implementation,
    ];

    /// 层级前缀，如 `REQ`。
    pub fn prefix(self) -> &'static str {
        match self {
            Layer::Requirement => "REQ",
            Layer::Intent => "INT",
            Layer::Specification => "SPEC",
            Layer::Implementation => "IMP",
        }
    }

    /// 层正文的文件名（不含扩展名），如 `requirement`。
    pub fn file_stem(self) -> &'static str {
        match self {
            Layer::Requirement => "requirement",
            Layer::Intent => "intent",
            Layer::Specification => "specification",
            Layer::Implementation => "implementation",
        }
    }

    /// 对该层正文跑结构化门禁，返回「通过 / 不通过 + 逐条理由」。
    pub fn gate(self, body: &str) -> GateReport {
        match self {
            Layer::Requirement => requirement::gate(body),
            Layer::Intent => intent::gate(body),
            Layer::Specification => specification::gate(body),
            Layer::Implementation => implementation::gate(body),
        }
    }
}

/// 结构化门禁的结论：是否通过，以及不通过时的逐条理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateReport {
    /// 是否通过门禁。
    pub passed: bool,
    /// 不通过的理由，每条对应一次机械判定。
    pub reasons: Vec<String>,
}

impl GateReport {
    /// 由逐条理由构造：无理由即通过。
    pub fn from_reasons(reasons: Vec<String>) -> Self {
        Self {
            passed: reasons.is_empty(),
            reasons,
        }
    }
}

/// 产物状态机的状态：`draft → validated → executed`，另有 `failed` 与 `stale`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// 起草：产物已落盘，未过门禁。
    Draft,
    /// 已过门禁。
    Validated,
    /// 已执行。
    Executed,
    /// 验证未过。
    Failed,
    /// 上游变更后待再生成。
    Stale,
}

impl Status {
    /// 状态的小写稳定名，用于展示与序列化。
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Draft => "draft",
            Status::Validated => "validated",
            Status::Executed => "executed",
            Status::Failed => "failed",
            Status::Stale => "stale",
        }
    }
}

/// 单层产物在 manifest 中的登记项：层、版本、状态、更新时间。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    /// 所属层。
    pub layer: Layer,
    /// 版本，自 1 起。
    pub version: u32,
    /// 当前状态。
    pub status: Status,
    /// 更新时间，RFC3339 字符串。
    pub updated_at: String,
}

/// 一个案例的清单：case 名 + 四层各一条 [`Artifact`]。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// 案例名。
    pub case: String,
    /// 数据需求。
    pub requirement: Artifact,
    /// 数据意图。
    pub intent: Artifact,
    /// 数据规格。
    pub specification: Artifact,
    /// 数据实现。
    pub implementation: Artifact,
}

impl Manifest {
    /// 新建清单：四层均为版本 1、状态 `draft`，更新时间取当前时刻。
    pub fn new(case: impl Into<String>) -> Self {
        let updated_at = now_rfc3339();
        let artifact = |layer| Artifact {
            layer,
            version: 1,
            status: Status::Draft,
            updated_at: updated_at.clone(),
        };
        Self {
            case: case.into(),
            requirement: artifact(Layer::Requirement),
            intent: artifact(Layer::Intent),
            specification: artifact(Layer::Specification),
            implementation: artifact(Layer::Implementation),
        }
    }

    /// 按层取登记项。
    pub fn artifact(&self, layer: Layer) -> &Artifact {
        match layer {
            Layer::Requirement => &self.requirement,
            Layer::Intent => &self.intent,
            Layer::Specification => &self.specification,
            Layer::Implementation => &self.implementation,
        }
    }

    /// 按层取可写登记项。
    pub fn artifact_mut(&mut self, layer: Layer) -> &mut Artifact {
        match layer {
            Layer::Requirement => &mut self.requirement,
            Layer::Intent => &mut self.intent,
            Layer::Specification => &mut self.specification,
            Layer::Implementation => &mut self.implementation,
        }
    }
}

/// 追溯查询的中间产物：链路、断链，以及数据意图层的命题标签（供通配命中判定用）。
type TraceParts = (
    Vec<specification::TraceLink>,
    Vec<specification::ChainBreak>,
    Vec<String>,
);

/// 工作空间：以根目录为界的案例产物读写。
///
/// 布局：
///
/// ```text
/// <root>/<case>/manifest.json
/// <root>/<case>/docs/{requirement,intent,specification,implementation}.md
/// ```
///
/// 法源是 `manifest.json`，层正文不带状态、不写 frontmatter。
#[derive(Debug, Clone)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// 以给定根目录打开工作空间。
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// 案例目录。
    fn case_dir(&self, case: &str) -> PathBuf {
        self.root.join(case)
    }

    /// 四层正文目录。
    fn docs_dir(&self, case: &str) -> PathBuf {
        self.case_dir(case).join("docs")
    }

    /// 清单文件。
    fn manifest_path(&self, case: &str) -> PathBuf {
        self.case_dir(case).join("manifest.json")
    }

    /// 某层正文文件。
    fn doc_path(&self, case: &str, layer: Layer) -> PathBuf {
        self.docs_dir(case)
            .join(format!("{}.md", layer.file_stem()))
    }

    /// 案例是否已建盘（以清单是否存在为准）。
    pub fn exists(&self, case: &str) -> bool {
        self.manifest_path(case).is_file()
    }

    /// 新建案例：建目录、四份空正文、清单（四层版本 1、状态 `draft`）。
    ///
    /// 案例目录已存在时报错。
    pub fn create(&self, case: &str) -> Result<Manifest, LabError> {
        let case_dir = self.case_dir(case);
        if case_dir.exists() {
            return Err(LabError::new(format!(
                "案例 `{case}` 已存在：{}",
                case_dir.display()
            )));
        }
        fs::create_dir_all(self.docs_dir(case))?;
        for layer in Layer::ALL {
            fs::write(self.doc_path(case, layer), "")?;
        }
        let manifest = Manifest::new(case);
        self.write_manifest(case, &manifest)?;
        Ok(manifest)
    }

    /// 读取案例清单，缺失或损坏时报错。
    pub fn read_manifest(&self, case: &str) -> Result<Manifest, LabError> {
        let path = self.manifest_path(case);
        let text = fs::read_to_string(&path).map_err(|err| {
            LabError::new(format!(
                "案例 `{case}` 的清单不可读：{}（{err}）",
                path.display()
            ))
        })?;
        serde_json::from_str(&text)
            .map_err(|err| LabError::new(format!("案例 `{case}` 的清单无法解析：{err}")))
    }

    /// 读取某层正文，缺失时报错。
    pub fn read_doc(&self, case: &str, layer: Layer) -> Result<String, LabError> {
        let path = self.doc_path(case, layer);
        fs::read_to_string(&path).map_err(|err| {
            LabError::new(format!(
                "案例 `{case}` 的{}正文缺失：{}（{err}）",
                layer_name(layer),
                path.display()
            ))
        })
    }

    /// 写入某层正文：版本 +1、状态回 `draft`、更新时间刷新。
    pub fn write_doc(&self, case: &str, layer: Layer, body: &str) -> Result<Manifest, LabError> {
        let mut manifest = self.read_manifest(case)?;
        fs::write(self.doc_path(case, layer), body)?;
        let artifact = manifest.artifact_mut(layer);
        artifact.version += 1;
        artifact.status = Status::Draft;
        artifact.updated_at = now_rfc3339();
        self.write_manifest(case, &manifest)?;
        Ok(manifest)
    }

    /// 写回清单。
    fn write_manifest(&self, case: &str, manifest: &Manifest) -> Result<(), LabError> {
        let text = serde_json::to_string_pretty(manifest)
            .map_err(|err| LabError::new(format!("案例 `{case}` 的清单无法序列化：{err}")))?;
        fs::write(self.manifest_path(case), text)?;
        Ok(())
    }
}

impl Workspace {
    /// 对某层已落盘的正文跑门禁；通过则把状态置 `validated`，不通过保持 `draft`。
    ///
    /// 门禁只看正文结构，不写盘、不接网络。返回逐条理由供调用方展示。
    pub fn gate_doc(&self, case: &str, layer: Layer) -> Result<GateReport, LabError> {
        let body = self.read_doc(case, layer)?;
        let report = layer.gate(&body);
        if report.passed {
            let mut manifest = self.read_manifest(case)?;
            manifest.artifact_mut(layer).status = Status::Validated;
            self.write_manifest(case, &manifest)?;
        }
        Ok(report)
    }
}

/// 追溯查询结果：命中的链路，以及全矩阵的断链。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceReport {
    /// 命中的链路。
    pub links: Vec<specification::TraceLink>,
    /// 全矩阵的断链检测结果。
    pub breaks: Vec<specification::ChainBreak>,
}

impl Workspace {
    /// 正查：案例追溯矩阵的全部链路。
    pub fn trace_forward(&self, case: &str) -> Result<TraceReport, LabError> {
        let (links, breaks, _) = self.trace_parts(case)?;
        Ok(TraceReport { links, breaks })
    }

    /// 反查：`claim` 取报告章节名或断言 ID，返回命中的上游链路。
    pub fn trace_back(&self, case: &str, claim: &str) -> Result<TraceReport, LabError> {
        let (links, breaks, known_claims) = self.trace_parts(case)?;
        let links = links
            .into_iter()
            .filter(|link| specification::link_matches(link, claim, &known_claims))
            .collect();
        Ok(TraceReport { links, breaks })
    }

    /// 读四层正文，解析追溯矩阵、检测断链，并带出数据意图层的命题标签。
    fn trace_parts(&self, case: &str) -> Result<TraceParts, LabError> {
        let requirement_body = self.read_doc(case, Layer::Requirement)?;
        let intent_body = self.read_doc(case, Layer::Intent)?;
        let spec_body = self.read_doc(case, Layer::Specification)?;
        let implementation_body = self.read_doc(case, Layer::Implementation)?;
        let links = specification::traceability(&spec_body).map_err(LabError::new)?;
        let known_claims = intent::claims(&intent_body);
        let breaks = specification::chain_breaks(
            &links,
            &known_claims,
            &intent::metrics(&intent_body),
            &requirement::success_metrics(&requirement_body),
            &implementation_body,
        );
        Ok((links, breaks, known_claims))
    }
}

/// Markdown 表格：表头与数据行。
pub(crate) struct Table {
    /// 表头单元格（已去空白）。
    pub header: Vec<String>,
    /// 数据行。
    pub rows: Vec<Vec<String>>,
}

/// 标题层级：以若干 `#` 开头且后接空白者为标题。
pub(crate) fn heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = &trimmed[hashes..];
    if rest.is_empty() || rest.starts_with(' ') {
        Some(hashes)
    } else {
        None
    }
}

/// 标题文本（去掉前导 `#` 与空白）。
pub(crate) fn heading_text(line: &str) -> &str {
    line.trim_start().trim_start_matches('#').trim()
}

/// 取标题文本包含 `name` 的小节正文（含标题行），到下一个同级或更高级标题为止。
pub(crate) fn section(body: &str, name: &str) -> Option<String> {
    let lines: Vec<&str> = body.lines().collect();
    let start = lines
        .iter()
        .position(|line| heading_level(line).is_some() && heading_text(line).contains(name))?;
    let level = heading_level(lines[start])?;
    let mut end = lines.len();
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        if let Some(inner) = heading_level(line) {
            if inner <= level {
                end = index;
                break;
            }
        }
    }
    Some(lines[start..end].join("\n"))
}

/// 小节是否存在且至少含一个非空列表项。
pub(crate) fn has_bullet_section(body: &str, name: &str) -> bool {
    section(body, name).is_some_and(|text| {
        text.lines().any(|line| {
            let trimmed = line.trim();
            (trimmed.starts_with("- ") || trimmed.starts_with("* "))
                && !trimmed[2..].trim().is_empty()
        })
    })
}

/// 是否为表格行（含 `|`）。
fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.matches('|').count() >= 2
}

/// 是否为表格分隔行（各单元格形如 `:--:`）。
fn is_separator_row(line: &str) -> bool {
    if !is_table_row(line) {
        return false;
    }
    let cells = parse_cells(line.trim());
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let dashes = cell.trim_matches(':');
            !dashes.is_empty() && dashes.chars().all(|c| c == '-')
        })
}

/// 拆一行表格为单元格文本。
fn parse_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

/// 取正文中全部 Markdown 表格。
pub(crate) fn tables(body: &str) -> Vec<Table> {
    let lines: Vec<&str> = body.lines().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if is_table_row(lines[index])
            && index + 1 < lines.len()
            && is_separator_row(lines[index + 1])
        {
            let header = parse_cells(lines[index].trim());
            let mut rows = Vec::new();
            let mut cursor = index + 2;
            while cursor < lines.len() && is_table_row(lines[cursor]) {
                rows.push(parse_cells(lines[cursor].trim()));
                cursor += 1;
            }
            out.push(Table { header, rows });
            index = cursor;
        } else {
            index += 1;
        }
    }
    out
}

/// 表头同时含全部 `required` 单元格的表格。
pub(crate) fn find_table(body: &str, required: &[&str]) -> Option<Table> {
    tables(body).into_iter().find(|table| {
        required
            .iter()
            .all(|name| table.header.iter().any(|cell| cell == name))
    })
}

/// 表头中某列的下标。
pub(crate) fn column(header: &[String], name: &str) -> Option<usize> {
    header.iter().position(|cell| cell == name)
}

/// 取「表头含全部 `required` 且含 `name` 列」的表在该列的非空取值。
pub(crate) fn column_values(body: &str, required: &[&str], name: &str) -> Vec<String> {
    find_table(body, required)
        .and_then(|table| {
            let col = column(&table.header, name)?;
            Some(
                table
                    .rows
                    .iter()
                    .filter_map(|row| row.get(col))
                    .map(|cell| cell.trim().to_string())
                    .filter(|cell| !cell.is_empty())
                    .collect(),
            )
        })
        .unwrap_or_default()
}

/// 层的中文名，用于可读错误信息。
fn layer_name(layer: Layer) -> &'static str {
    match layer {
        Layer::Requirement => "数据需求",
        Layer::Intent => "数据意图",
        Layer::Specification => "数据规格",
        Layer::Implementation => "数据实现",
    }
}

/// 当前时刻的 RFC3339 UTC 字符串，如 `2026-01-02T03:04:05Z`。
fn now_rfc3339() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0);
    format_rfc3339(seconds)
}

/// 把 Unix 秒数（可为负）格式化成 RFC3339 UTC 字符串。
fn format_rfc3339(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let remainder = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = remainder / 3_600;
    let minute = (remainder % 3_600) / 60;
    let second = remainder % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// 把 Unix 天数（1970-01-01 起）转成公历年月日（Howard Hinnant 的 `civil_from_days`）。
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_pos = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_pos + 2) / 5 + 1;
    let month = if month_pos < 10 {
        month_pos + 3
    } else {
        month_pos - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_rfc3339_matches_known_epochs() {
        assert_eq!(format_rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_rfc3339(-1), "1969-12-31T23:59:59Z");
        assert_eq!(format_rfc3339(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(format_rfc3339(1_700_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(format_rfc3339(1_791_504_000), "2026-10-09T00:00:00Z");
    }

    #[test]
    fn layer_prefixes_match_traceability_scheme() {
        assert_eq!(Layer::Requirement.prefix(), "REQ");
        assert_eq!(Layer::Intent.prefix(), "INT");
        assert_eq!(Layer::Specification.prefix(), "SPEC");
        assert_eq!(Layer::Implementation.prefix(), "IMP");
    }
}

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

/// 四层合同的层级标识，用于追溯 ID（`REQ → INT → SPEC → IMP`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// 层级前缀，如 `REQ`。
    pub fn prefix(self) -> &'static str {
        match self {
            Layer::Requirement => "REQ",
            Layer::Intent => "INT",
            Layer::Specification => "SPEC",
            Layer::Implementation => "IMP",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_prefixes_match_traceability_scheme() {
        assert_eq!(Layer::Requirement.prefix(), "REQ");
        assert_eq!(Layer::Intent.prefix(), "INT");
        assert_eq!(Layer::Specification.prefix(), "SPEC");
        assert_eq!(Layer::Implementation.prefix(), "IMP");
    }
}

//! crate 统一错误类型：公共接口返回 `Result<_, LabError>`，调用方只看它。

use std::fmt;
use std::io;

/// 实验室 crate 的统一错误，携带用户可读消息。
#[derive(Debug)]
pub struct LabError {
    message: String,
}

impl LabError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for LabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for LabError {}

impl From<io::Error> for LabError {
    fn from(err: io::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl From<String> for LabError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for LabError {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

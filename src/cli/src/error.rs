//! CLI 统一错误类型：命令入口返回 `Result<(), CliError>`，`main` 顶层统一格式化。

use std::fmt;
use std::io;

/// CLI 统一错误，携带用户可读消息。
#[derive(Debug)]
pub struct CliError {
    message: String,
}

impl CliError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        Self::new(err.to_string())
    }
}

impl From<String> for CliError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for CliError {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_error_roundtrips_through_display() {
        let err = CliError::new("pipeline failed");
        assert_eq!(err.to_string(), "pipeline failed");
    }

    #[test]
    fn cli_error_converts_from_string_and_str() {
        assert_eq!(CliError::from("abc").to_string(), "abc");
        assert_eq!(CliError::from("abc".to_string()).to_string(), "abc");
    }
}

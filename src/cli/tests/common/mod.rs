//! 命令级集成测试共享 helper（spawn CLI 二进制）。

use std::process::Command;

#[allow(dead_code)]
/// spawn 编译好的 CLI 二进制（先 `cargo build`）。
pub fn cli() -> Command {
    Command::new("./target/debug/qtcloud-data-lab")
}

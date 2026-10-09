//! 命令级集成测试共享 helper（spawn CLI 二进制）。

use std::process::{Command, Output};

#[allow(dead_code)]
/// spawn 编译好的 CLI 二进制（先 `cargo build`）。
pub fn cli() -> Command {
    Command::new("./target/debug/qtcloud-data-lab")
}

#[allow(dead_code)]
/// 以给定参数执行 CLI，返回完整输出（退出码、stdout、stderr）。
pub fn run(args: &[&str]) -> Output {
    cli().args(args).output().expect("运行 qtcloud-data-lab")
}

#[allow(dead_code)]
/// 取 stdout 文本。
pub fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

//! 命令级集成测试共享 helper（spawn CLI 二进制）。

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// 生成互不相同的临时根目录名，避免并行测试互相覆盖。
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

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
/// 以给定参数执行 CLI，并把 `stdin` 喂给子进程。
pub fn run_with_stdin(args: &[&str], stdin: &str) -> Output {
    let mut child = cli()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("启动 qtcloud-data-lab");
    child
        .stdin
        .as_mut()
        .expect("子进程 stdin")
        .write_all(stdin.as_bytes())
        .expect("写入子进程 stdin");
    child.wait_with_output().expect("等待 qtcloud-data-lab")
}

#[allow(dead_code)]
/// 取 stdout 文本。
pub fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[allow(dead_code)]
/// 建一个空的临时工作空间根目录（std，无额外依赖）。
pub fn temp_root(tag: &str) -> PathBuf {
    let seq = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "qtcloud-data-lab-{}-{tag}-{seq}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("创建临时根目录");
    dir
}

#[allow(dead_code)]
/// 四层命令与对应夹具文件名（主仓「量潮搜索工程」案例）。
pub const LAYERS: [(&str, &str); 4] = [
    ("req", "requirement.md"),
    ("intent", "intent.md"),
    ("spec", "specification.md"),
    ("impl", "implementation.md"),
];

#[allow(dead_code)]
/// 读取某层夹具原文（相对 CLI crate 根）。
pub fn fixture(file: &str) -> String {
    std::fs::read_to_string(format!("../../examples/fixtures/quanttide-search/{file}"))
        .unwrap_or_else(|err| panic!("读取夹具 {file} 失败：{err}"))
}

#[allow(dead_code)]
/// 建一个案例，把四份夹具经 `update` 写进去（顺走过门禁），返回（根目录, 案例名）。
pub fn case_with_fixtures(tag: &str) -> (String, String) {
    let root = temp_root(tag);
    let root = root.to_str().expect("临时根目录路径").to_string();
    let case = format!("case-{tag}");
    assert!(
        run(&["--root", &root, "--case", &case, "req", "new"])
            .status
            .success(),
        "建案例应成功"
    );
    for (layer, file) in LAYERS {
        let out = run_with_stdin(
            &["--root", &root, "--case", &case, layer, "update"],
            &fixture(file),
        );
        assert!(
            out.status.success(),
            "{layer} 夹具应过门禁：{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    (root, case)
}

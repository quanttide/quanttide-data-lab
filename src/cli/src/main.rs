//! 二进制入口：CLI 参数解析与命令分发（run_command）。

use std::io::Read;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use qtcloud_data_lab::error::CliError;
use quanttide_data_lab::{Layer, Workspace};

#[derive(Parser)]
#[command(
    name = "qtcloud-data-lab",
    about = "量潮数据工程实验室 CLI — 四层形式化数据智能体"
)]
struct Cli {
    /// 工作空间根目录
    #[arg(long, global = true, default_value = "workspace")]
    root: PathBuf,
    /// 案例名
    #[arg(long, global = true, default_value = "default")]
    case: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 数据需求（价值合同）
    Req {
        #[command(subcommand)]
        action: LayerAction,
    },
    /// 数据意图（形式化验证合同）
    Intent {
        #[command(subcommand)]
        action: LayerAction,
    },
    /// 数据规格（约束合同）
    Spec {
        #[command(subcommand)]
        action: LayerAction,
    },
    /// 数据实现（工程合同）
    Impl {
        #[command(subcommand)]
        action: LayerAction,
    },
    /// 执行数据规格定义的步骤（缺省全链）
    Run {
        /// 步骤名，由数据规格定义
        steps: Vec<String>,
    },
    /// 查追溯矩阵（可反向查结论）
    Trace {
        /// 按结论 ID 反向查，如 `INT-001-H2`
        #[arg(long)]
        claim: Option<String>,
    },
    /// 反向挑战：下游证伪上游，触发回退与再生成
    Refute {
        /// 发起层（下游）
        from_layer: String,
        /// 目标层（上游）
        to_layer: String,
    },
    /// 各层状态与版本
    Status,
}

/// 单层的 `new|update|show` 动作。
#[derive(Subcommand)]
enum LayerAction {
    /// 新建
    New,
    /// 更新
    Update,
    /// 查看
    Show,
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run_command(&cli) {
        eprintln!("错误: {err}");
        std::process::exit(1);
    }
}

/// 命令分发：只解析参数、调库、打印；领域与存储逻辑在 `quanttide-data-lab` 库里。
fn run_command(cli: &Cli) -> Result<(), CliError> {
    let workspace = Workspace::new(cli.root.clone());
    let case = cli.case.as_str();
    match &cli.command {
        Commands::Req { action } => layer(&workspace, case, Layer::Requirement, action),
        Commands::Intent { action } => layer(&workspace, case, Layer::Intent, action),
        Commands::Spec { action } => layer(&workspace, case, Layer::Specification, action),
        Commands::Impl { action } => layer(&workspace, case, Layer::Implementation, action),
        Commands::Run { steps } => {
            println!("run: {steps:?}（尚未实现）");
            Ok(())
        }
        Commands::Trace { claim } => {
            println!("trace: {claim:?}（尚未实现）");
            Ok(())
        }
        Commands::Refute {
            from_layer,
            to_layer,
        } => {
            println!("refute: {from_layer} → {to_layer}（尚未实现）");
            Ok(())
        }
        Commands::Status => status(&workspace, case),
    }
}

/// 单层的 `new|update|show` 分发。
fn layer(
    workspace: &Workspace,
    case: &str,
    layer: Layer,
    action: &LayerAction,
) -> Result<(), CliError> {
    match action {
        LayerAction::New => {
            workspace.create(case)?;
            println!("已新建案例 `{case}`：四层版本 1，状态 draft");
            Ok(())
        }
        LayerAction::Update => {
            let mut body = String::new();
            std::io::stdin().read_to_string(&mut body)?;
            let manifest = workspace.write_doc(case, layer, &body)?;
            let artifact = manifest.artifact(layer);
            println!(
                "已更新{}：版本 {}，状态 {}，更新于 {}",
                layer_label(layer),
                artifact.version,
                artifact.status.as_str(),
                artifact.updated_at
            );
            Ok(())
        }
        LayerAction::Show => {
            print!("{}", workspace.read_doc(case, layer)?);
            Ok(())
        }
    }
}

/// `status`：列出该案例各层的状态 / 版本 / 更新时间。
fn status(workspace: &Workspace, case: &str) -> Result<(), CliError> {
    if !workspace.exists(case) {
        println!("案例 `{case}`：未找到 manifest");
        return Ok(());
    }
    let manifest = workspace.read_manifest(case)?;
    println!("案例：{}", manifest.case);
    for layer in Layer::ALL {
        let artifact = manifest.artifact(layer);
        println!(
            "{:<14} {:<9} v{:<3} {}",
            layer.file_stem(),
            artifact.status.as_str(),
            artifact.version,
            artifact.updated_at
        );
    }
    Ok(())
}

/// 层的中文名，用于命令输出。
fn layer_label(layer: Layer) -> &'static str {
    match layer {
        Layer::Requirement => "数据需求",
        Layer::Intent => "数据意图",
        Layer::Specification => "数据规格",
        Layer::Implementation => "数据实现",
    }
}

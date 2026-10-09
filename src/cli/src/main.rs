//! 二进制入口：CLI 参数解析与命令分发（run_command）。

use clap::{Parser, Subcommand};
use qtcloud_data_lab::error::CliError;

#[derive(Parser)]
#[command(
    name = "qtcloud-data-lab",
    about = "量潮数据工程实验室 CLI — 四层形式化数据智能体"
)]
struct Cli {
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
    Status {
        /// 案例名
        #[arg(long)]
        case: Option<String>,
    },
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
    if let Err(err) = run_command(&cli.command) {
        eprintln!("错误: {err}");
        std::process::exit(1);
    }
}

/// 命令分发。当前为脚手架：各命令打印提示后返回 `Ok`。
fn run_command(command: &Commands) -> Result<(), CliError> {
    match command {
        Commands::Req { action } => layer("数据需求", action),
        Commands::Intent { action } => layer("数据意图", action),
        Commands::Spec { action } => layer("数据规格", action),
        Commands::Impl { action } => layer("数据实现", action),
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
        Commands::Status { case } => {
            println!("status: {case:?}（尚未实现）");
            Ok(())
        }
    }
}

/// 单层的 `new|update|show` 分发。
fn layer(name: &str, action: &LayerAction) -> Result<(), CliError> {
    let action = match action {
        LayerAction::New => "new",
        LayerAction::Update => "update",
        LayerAction::Show => "show",
    };
    println!("{name}: {action}（尚未实现）");
    Ok(())
}

# qtcloud-data-lab

量潮数据工程实验室 CLI —— 四层形式化数据智能体。

以量潮数据云 CLI（`qtcloud-data`）为样本初始化；设计见仓库根 `docs/index.md`。

## 命令

```text
qtcloud-data-lab req        new|update|show            # 数据需求
qtcloud-data-lab intent     new|update|show            # 数据意图
qtcloud-data-lab spec       new|update|show            # 数据规格
qtcloud-data-lab impl       new|update|show            # 数据实现
qtcloud-data-lab run        [<step>...]                # 执行数据规格定义的步骤（缺省全链）
qtcloud-data-lab trace      [--claim <id>]             # 查追溯矩阵（可反向查结论）
qtcloud-data-lab refute     <from-layer> <to-layer>    # 反向挑战（下游证伪上游）
qtcloud-data-lab status     [--case <name>]            # 各层状态与版本
```

> 当前为脚手架：各命令尚未实现，仅打印提示。

## 构建

```bash
cargo build
cargo test
```

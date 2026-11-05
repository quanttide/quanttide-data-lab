# ROADMAP — qtcloud-data-lab CLI

## 可能重写（2026-11-05 标注）

现有八命令骨架（req / intent / spec / impl / run / trace / refute / status）按搜索项目实践定型，且多为未实现脚手架。随着 quanttide-legal 项目的新实践（四层合同正本定于 profile 仓、实现层选型 Python、生产流程为「规格 → 采集 → 生产 → 研判 → 交付」五步），工作流程将重新改进——CLI **可能重写**，重写方案依新实践确定后再动代码。

## 当前已知状态

- `examples/fixtures/quanttide-search/` 四层合同已迁出至 profile 仓（正本归位），五个测试文件仍读取原路径，`cargo test` 处于损坏状态；重写评估时一并决定测试样本策略（合成最小样本或迁移读取路径）。
- `docs/index.md` 中的 fixtures 路径引用待同步。

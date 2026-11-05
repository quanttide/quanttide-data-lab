# quanttide-legal demo — 算法备案数据集四层管道

按 profile 仓 `data/profile/quanttide-legal/` 四层文档实现的可运行演示。零依赖（Python 3.12+ 标准库）。

## 运行

```bash
cd examples/quanttide_legal
python3 -m qtlegal demo        # 按序运行 spec → collect → generate → judge → verify → deliver
```

单步运行：

```bash
python3 -m qtlegal generate    # 生产 draft（入库即拦截）
python3 -m qtlegal verify      # 覆盖差集、量纲、溯源与指标计算
```

产物在 `work/`（已 gitignore）：`store/`（questions、keywords、runs、judgments 的 JSONL）、`indicators.json`、`deliver/<batch>/`（交付包、report.md、复算记录.txt）。

## 演示了什么

- 入库即拦截：C05 类的一道题模拟服务异常返回空输出，schema 校验在生成当天拦截并留拦截日志，不流到审核。
- 类别继承：同音变体关键词强制继承父条目类别，无 parent_id 入不了库。
- 研判双人流：标注人研判 → 双人盲评子集（demo 中指定一处分歧，一致率 5/6）→ 独立验证人复核（与研判人不同人），状态机 draft → judged → verified。
- 阈值先注册后跑：研判一致率阈值 0.6 写在 `fixtures/regulation/spec.sample.json`，verify 比对的是注册值。
- 交付门槛：覆盖差集非空、量纲不达、溯源不完整任一发生，`deliver` 拒绝打包。

## 与四层文档的对应

| 本 demo | 正本 |
|:--|:--|
| 判定条件（schema.py 镜像校验） | `schema/*.cue` 的 Python 镜像，条件来自 intent.md |
| 表结构与状态机 | specification.md |
| 命令划分与报告章节 | implementation.md |

## 相对规格的简化（demo 限定）

- JSONL 落盘，未切 Parquet（polars 在实现层引入）。
- 被测模型服务用 `fixtures/mock_service.json` 模拟应答；真实实现按 implementation.md 用 httpx 异步调度。
- 研判用 `--auto` 式确定性模拟（含一处预设分歧）；真实研判由人工通过研判工具产出 JSONL 导入。
- CUE schema 为正本，本 demo 无 cue CLI，校验由 `schema.py` 镜像执行——两处口径必须同步修改。

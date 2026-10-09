# ROADMAP

四层形式化数据智能体的路线图。设计见 [`docs/index.md`](docs/index.md)，方法论见主仓 `docs/essay/intro/index.md`，验收案例见主仓 `docs/gallery/quanttide-search/index.md`。

状态记号：`[ ]` 未开始 · `[~]` 进行中 · `[x]` 完成

---

## 已完成

- [x] **脚手架**：`packages/rust`（`quanttide-data-lab`，data 库）与 `src/cli`（`qtcloud-data-lab`，bin）分离，库不依赖 `quanttide-agent`，CLI 组装 data 与 agent 库。
- [x] **四层领域模型骨架**：`requirement` / `intent` / `specification` / `implementation` 四模块 + 统一 `Artifact` 与状态机（`draft → validated → executed` / `failed` / `stale`）。
- [x] **命令行入口**：`req|intent|spec|impl new|update|show`、`run [<step>...]`、`trace [--claim <id>]`、`refute <from> <to>`、`status [--case <name>]`。
- [x] **案例驱动测试**：以「量潮搜索工程」为样本，`src/cli/tests/` 4 个集成测试文件。

---

## M1 单层可生成

> 四层生成器各自产出**过门禁**的产物。

- [ ] **工作空间 I/O**：`workspace/<case>/docs/{requirement,intent,specification,implementation}.md` 与 `manifest.json` 的读写（版本、时间戳、状态）。
- [ ] **LLM 起草**：`new` 分支接 `quanttide-agent`（≥ 0.1.2，读 `MIMO_API_KEY`），由模糊判断起草 `REQ` 草稿，人裁决后落盘。
- [ ] **四层结构化门禁**：
  - `REQ`：成功指标是业务后果而非交付物；范围可界定；约束齐全。
  - `INT`：每个概念有操作定义；每个命题有可证伪条件；相关与因果分列。
  - `SPEC`：每个指标可计算；每个流程步骤有验收标准；命名与口径唯一。
  - `IMP`：定义来自 `INT`、约束来自 `SPEC`，不另立定义与流程。
- [ ] **门禁拒绝路径**：不达标产物停在 `draft`，不得标 `validated`，并给出可读的拒绝理由。
- [ ] **验收**：用「量潮搜索工程」的 `REQ`/`INT`/`SPEC`/`IMP` 四份原文作为样本，逐层通过门禁。

---

## M2 全链可追溯

> 四层贯通，追溯矩阵双向可查。

- [ ] **追溯 ID 解析**：`REQ-001 → INT-001 → SPEC-001 → IMP-001`，支持挂层级下的断言 ID（如 `INT-001-H2`）。
- [ ] **`trace`**：正查（需求 → 指标 → 命题 → 报告章节）与反查（`--claim <id>` 回溯结论的上游依据）。
- [ ] **`refute` 反向挑战**：`INT→REQ`（无法形式化）、`SPEC→INT`（不可计算）、`IMP→SPEC`（规格自相矛盾）；记「触发点 → 结论 → 改动」，并将被挑战层置 `stale`。
- [ ] **`status`**：列出各层状态、版本与时间戳；上游变更后下游标 `stale`。
- [ ] **验收**：`trace --claim` 能从案例报告结论一路回指到业务成功指标，无断链。

---

## M3 验证可执行

> `verify` 自动产出检验结论并回填 `INT`。

- [ ] **`run` 执行引擎**：按 `SPEC` 定义的步骤执行（不写死步骤名），逐步校验产物与验收标准。
- [ ] **统计检验**：两比例 z 检验、Spearman / 卡方，产出检验统计量、p 值、效应量、置信区间。
- [ ] **因果识别降级**：不可识别时报相关并显式标注，未识别不得冒称因果。
- [ ] **回填**：检验结论写回 `INT` 的「验证结论」；未过验收清单则该层置 `failed`，阻断报告。
- [ ] **验收**：案例的 `H1–H3` 跑出完整检验结论，数值与报告一致。

---

## M4 跨案例泛化

> 在第二个数据工程案例上复现。

- [ ] 以问卷数据清洗案例（`docs/essay/processing/cleaning.md`）为第二案例，从模糊判断走完四层。
- [ ] 抽出案例无关的通用部分，把「量潮搜索工程」专属口径移出库。

---

## 风险与未决

| 项 | 说明 |
|:--|:--|
| 门禁的判定方式 | 结构化规则（可测、无 LLM）先行，语义类判定（如「是否业务后果」）由 LLM 提议、人裁决 |
| 统计检验库 | 尚未选型，倾向零依赖自实现 + 对照 `scipy` 基准验证 |
| 密钥管理 | `MIMO_API_KEY` 由本机 `XIAOMI_API_KEY` 导出，凭据不得进入任何产物 |
| 测试覆盖 | 当前 `cargo llvm-cov` 未安装，覆盖率数字待补测（目标 ≥ 80%） |

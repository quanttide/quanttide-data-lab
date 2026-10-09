# ROADMAP

四层形式化数据智能体的路线图。设计见 [`docs/index.md`](docs/index.md)，方法论见主仓 `docs/essay/intro/index.md`，验收样本见主仓 `docs/gallery/quanttide-search/index.md`（242 行四层案例，已就绪）。

状态记号：`[ ]` 未开始 · `[~]` 进行中 · `[x]` 完成

---

## 已完成

- [x] **脚手架**：`packages/rust`（`quanttide-data-lab`，data 库）与 `src/cli`（`qtcloud-data-lab`，bin）分离：库不依赖 `quanttide-agent`；CLI 已声明对 data 库与 `quanttide-agent` 0.1.2 的依赖，agent 的调用待 M1b。
- [x] **四层分层约定**：`requirement` / `intent` / `specification` / `implementation` 四个模块占位，`lib.rs` 的 `Layer` 枚举给出 `REQ/INT/SPEC/IMP` 前缀。实体、`Artifact` 与状态机（`draft → validated → executed` / `failed` / `stale`）待 M1a。
- [x] **命令行入口**：`req|intent|spec|impl new|update|show`、`run [<step>...]`、`trace [--claim <id>]`、`refute <from> <to>`、`status`；另有全局参数 `--root <dir>`（工作空间根，默认 `./workspace/`）与 `--case <name>`（案例名，默认 `default`）。四层的 `new|update|show`、`status` 与 `trace` 已接真实读写；`run`/`refute` 仍为占位。
- [x] **案例驱动测试**：以「量潮搜索工程」为样本，`src/cli/tests/` 7 个集成测试文件（库 7 条 + CLI 41 条，离线全绿；覆盖率未测）。

---

## M1a 有 I/O、有门禁（不接模型）—— 已完成

> 四层产物能落盘、能读回、能被结构化门禁拦下——全程零 LLM。

- [x] **工作空间 I/O**：`workspace/<case>/docs/{requirement,intent,specification,implementation}.md` 与 `manifest.json` 的读写（版本、时间戳、状态）；四层的 `new|update|show` 与 `status` 接上真实读写。
- [x] **四层结构化门禁**：规则落在 `packages/rust/src/{requirement,intent,specification,implementation}.rs`，逐条编号（`REQ-1…3` / `INT-1…3` / `SPEC-1…3` / `IMP-1…3`），每条在代码注释里注明它落实下面哪一句：
  - `REQ`：成功指标是业务后果而非交付物；范围可界定；约束齐全。
  - `INT`：每个概念有操作定义；每个命题有可证伪条件；相关与因果分列。
  - `SPEC`：每个指标可计算；每个流程步骤有验收标准；命名与口径唯一。
  - `IMP`：定义来自 `INT`、约束来自 `SPEC`，不另立定义与流程。

  判定是**结构化近似**（表头、列、小节、编号与关键词），语义判定仍待「未决」里那一条拍板。
- [x] **门禁拒绝路径**：四层的 `new`/`update` 落盘后自动跑本层门禁——通过则置 `validated`；不通过则正文照落盘、状态保持或回退 `draft`、逐条打印拒绝理由、以退出码 1 结束（`new` 建的是空正文，不跑门禁）。
- [x] **验收**：夹具 `examples/fixtures/quanttide-search/`（主仓案例按四个二级标题切分，与原文逐字核对一致）逐层过门禁并置 `validated`；四层各有一个反样本（改坏一处）被拒——退出码 1、正文照落盘、状态留 `draft`、理由指名规则号。

## M1b 单层可生成（接模型）

> 四层生成器各自产出过门禁的产物。

- [ ] **LLM 起草**：`new` 分支接 `quanttide-agent`（≥ 0.1.2，读 `MIMO_API_KEY`），由模糊判断起草 `REQ` 草稿，人裁决后落盘。
- [ ] **验收**：从一个模糊判断（如「多个智能体的搜索行为不稳定」）起草出 `REQ` 草稿；人裁决后落盘并过门禁。

---

## M2 全链可追溯 —— 已完成

> 四层贯通，追溯矩阵双向可查。

- [x] **追溯 ID 解析（库内）**：`REQ-001 → INT-001 → SPEC-001 → IMP-001`，支持挂层级下的断言 ID（`INT-001-H2`、区间 `INT-001-H1–H3`、整层通配 `全部命题`）。回溯矩阵的解析、断链检测（指名所在格）与正查/反查两个查询都在 `packages/rust/src/specification.rs`，矩阵格式见 `docs/index.md` §5.2；`trace` 命令接线待下一步。
- [x] **`trace`**：正查一条链路一行（报告章节 → 命题 → 数据意图指标 → 业务成功指标），反查 `--claim <报告章节名|断言 ID>`；未命中与断链都以退出码 1 结束，断链逐条指名到格。「整层通配只覆盖数据意图层真实存在的命题」这条命中语义在库里（`specification::link_matches` 接收该层命题标签）。
- [x] **`refute` 反向挑战**：`refute <from> <to>` 从 stdin 读记录正文，须含 `触发点：`／`结论：`／`改动：` 三行字段（缺一即拒不落盘）；合法对只有相邻的下游挑战上游（`impl→spec`、`spec→intent`、`intent→req`），短名全名都收，非法对先于读正文报错；记录落 `workspace/<case>/refutations/<四位序号>-<from>-<to>.md`（序号递增），并把被挑战层（`to`）置 `stale`。格式见 `docs/index.md` §5.4。
- [x] **`status`**：列出各层状态、版本与时间戳；`stale` 会出现在状态列。上游变更的 `stale` 联动口径：`update` 写入某层后，把该层**下游已有正文（非空）**的层置 `stale`，下游空正文保持 `draft`（这就是 §5.3 那句的落地口径）。
- [x] **验收**：`trace --claim 结果体量` 从案例报告章节一路回指到业务成功指标（`结果体量 → INT-001-H3 → 结果体量、过载率 → 低效会话占比`），案例矩阵无断链。

---

## M3 验证可执行

> `verify` 自动产出检验结论并回填 `INT`。

- [ ] **`verify` 进命令面**：M3 起新增该子命令（现命令面没有，属新增而非改造）。
- [ ] **`run` 执行引擎**：按 `SPEC` 定义的步骤执行（不写死步骤名），逐步校验产物与验收标准。
- [ ] **统计检验**：两比例 z 检验、Spearman / 卡方，产出检验统计量、p 值、效应量、置信区间。
- [ ] **因果识别降级**：不可识别时报相关并显式标注，未识别不得冒称因果。
- [ ] **回填**：检验结论写回 `INT` 的「验证结论」；未过验收清单则该层置 `failed`，阻断报告。
- [ ] **验收**：案例的 `H1–H3` 跑出完整检验结论，数值与报告一致。

---

## M4 跨案例泛化

> 在第二个数据工程案例上复现。

- [ ] 第二案例从模糊判断走完四层；候选在主仓 `data/archive/gallery/` 的三篇 beta 案例里挑（化妆品检验报告、GHTorrent、SEC 信贷协议）——落点待定。
- [ ] 抽出案例无关的通用部分，把「量潮搜索工程」专属口径移出库。

---

## 待办（动作明确，逐条做完即勾）

- 覆盖率补测：`cargo llvm-cov` 未安装；目标 ≥ 80%。
- 统计检验库选型：倾向零依赖自实现 + 对照 `scipy` 基准验证。

## 未决（要拍板）

| 项 | 说明 |
|:--|:--|
| 语义类门禁的判定方式 | 结构化规则（可测、不接模型）先行；「是否业务后果」这类语义判定由 LLM 提议、人裁决——关口放在哪一步、比例多少，待定 |
| 密钥注入方式 | `MIMO_API_KEY` 由本机 `XIAOMI_API_KEY` 导出；凭据不得进入任何产物 |

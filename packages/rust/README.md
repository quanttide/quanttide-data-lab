# quanttide-data-lab

量潮数据工程实验室 —— 四层形式化数据智能体核心库。

以量潮数据工具箱（`quanttide-data`）为样本初始化，承载四层合同（数据需求 / 数据意图 / 数据规格 / 数据实现）的领域模型。

## 模块

| 模块 | 层级 | 合同 |
|------|------|------|
| `src/requirement.rs` | 数据需求 | 价值合同 |
| `src/intent.rs` | 数据意图 | 形式化验证合同 |
| `src/specification.rs` | 数据规格 | 约束合同 |
| `src/implementation.rs` | 数据实现 | 工程合同 |
| `src/error.rs` | — | 统一错误类型 |

## 构建

```bash
cargo build
cargo test
```

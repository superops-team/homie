# 发布就绪报告 — app-code-intelligence-module-split

## 变更概述

将 `homie/crates/homie-app/src/code_intelligence.rs`（1,426 行，轻量工作区智能 God Module）
机械拆分为目录化聚焦子模块：索引构建、引用解析、文本/评分辅助、测试各自下沉，
`mod.rs` 收尾为 facade。公共 API 与运行时行为完全不变，`code_viewer.rs` 零改动，
单文件 < 800 行。

- change_id：`app-code-intelligence-module-split`
- Beads：`homie-atu`
- 类型：task（机械拆分，行为不变）
- 上游：`architecture-audit-governance-2026-08`（模块降熵序列延续）

## 模块划分

```text
code_intelligence/
├── mod.rs       facade（公共常量/类型 + impl CodeIntelligence + mod 声明），546 行
├── index.rs     工作区发现 + 索引构建 + 符号提取 + WorkspaceIndex/IndexedFile/IndexedSymbol，344 行
├── reference.rs 引用解析 + ParsedReference，224 行
├── text.rs      文本/评分辅助，130 行
└── tests.rs     既有测试（原样下沉，#[cfg(test)]），205 行
```

依赖方向：公共类型/常量全部在 `mod.rs` 定义，子模块 `index`/`reference`/`text` 读取 facade
类型，`index.rs` 额外引用 `text::excerpt`；`mod.rs` 反向调用三个子模块的 `pub(super)` 函数。
`code_viewer.rs` 仅依赖 `crate::code_intelligence::{...}` 公共 API，路径不变。

## 交付切片 T1–T5

| 切片 | 内容 | 状态 |
|------|------|------|
| T1 | 抽取索引构建 index.rs | 完成 |
| T2 | 抽取引用解析 reference.rs | 完成 |
| T3 | 抽取文本/评分辅助 text.rs | 完成 |
| T4 | 抽取测试 tests.rs + mod.rs 收尾 facade | 完成 |
| T5 | 全量验证 + code review + release readiness | 完成 |

## 验证证据

| 验证项 | 命令 | 结果 |
|--------|------|------|
| 编译检查 | `cargo check -p homie-app --offline` | 通过，0 警告 |
| 格式检查 | `cargo fmt --all --check` | 通过 |
| 静态检查 | `cargo clippy -p homie-app --all-targets --offline` | 通过，0 警告 |
| 全量测试 | `cargo test -p homie-app --offline` | **303 passed / 0 failed / 1 ignored** |
| 单文件行数 | `wc -l` | mod 546 / index 344 / reference 224 / text 130 / tests 205，均 < 800 |

- `code_intelligence` 相关 7 个测试（`cargo test -p homie-app --offline code_intelligence`）
  全部原样通过，索引/引用解析/评分行为与拆分前等价。
- 公共 API 兼容性：`CodeIntelligence`、`CodeIntelligenceError`、`SearchHit`、`SearchHitKind`、
  `SourceSnapshot`、`SourceTarget`、`SourceLanguage`、`MAX_SOURCE_BYTES` 路径不变；
  `cargo check -p homie-app` 编译通过证明可达性不变。
- `code_viewer.rs` 零改动（`git diff` 确认无任何改动）。
- 可见性管控：仅跨模块调用函数（`discover_workspace_root`/`build_index`/`parse_reference_candidates`/
  `lexical_normalize`/`source_lines`/`clamp_target`/`path_score`/`fuzzy_score`/`excerpt`）与私有
  数据结构（`WorkspaceIndex`/`IndexedFile`/`IndexedSymbol`/`ParsedReference`）升为 `pub(super)`，
  无 `pub` 泄漏到 crate 外。

## 已知限制与后续

- 无已知限制。
- 机械重构未改变任何运行时行为、公共 API 不变；纯职责搬迁，不做向后兼容。
- 后续候选（`store/mod.rs` 2,434 行、`terminal_pane/mod.rs` 1,486 行、`git_review.rs` 1,427 行）
  依赖密、风险更高，建议作为更谨慎的独立切片处理。

## 结论

所有验收标准（C1–C6）均已满足，验证证据齐备，可发布。

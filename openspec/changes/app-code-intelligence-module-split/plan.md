# OpenSpec Plan — app-code-intelligence-module-split

## 概述

将 `homie/crates/homie-app/src/code_intelligence.rs`（1,426 行）机械拆分为目录化聚焦子模块：
索引构建、引用解析、文本/评分辅助、测试各自下沉，`mod.rs` 收尾为 facade。
公共 API 与运行时行为完全不变，`code_viewer.rs` 零改动，单文件 < 800 行。

## 模块划分与依赖

```text
code_intelligence/
├── mod.rs       facade（公共常量/类型 + impl CodeIntelligence + mod 声明），< 800
├── index.rs     工作区发现 + 索引构建 + 符号提取 + WorkspaceIndex/IndexedFile/IndexedSymbol，< 800
├── reference.rs 引用解析 + ParsedReference，< 800
├── text.rs      文本/评分辅助，< 800
└── tests.rs     既有测试（原样下沉，#[cfg(test)]），< 800
```

依赖方向：

- `mod → index`（`discover_workspace_root`/`build_index`）
- `mod → reference`（`parse_reference_candidates`）
- `mod → text`（`lexical_normalize`/`source_lines`/`clamp_target`/`path_score`/`fuzzy_score`/`excerpt`）
- `index → mod`（`SourceLanguage`/`CodeIntelligenceError`/`excerpt` 等公共类型）
- `reference → mod`（`SourceTarget`/`CodeIntelligenceError` 等公共类型）
- `text → mod`（`SourceLine`/`SourceTarget` 等公共类型）
- `tests → mod`（公共 API + 常量）

公共类型/常量全部在 `mod.rs` 定义，子模块通过 `use super::*;` 访问；私有类型/函数
在对应子模块内定义，必要时以 `pub(super)` 暴露给兄弟模块。

## 任务清单

| Task | 描述 | 验收 | 关联验证 Case |
|------|------|------|---------------|
| T1 | 抽取 index.rs（工作区发现 + 索引构建 + 符号提取） | cargo check 全绿 | C1 |
| T2 | 抽取 reference.rs（引用解析） | cargo check 全绿 | C2 |
| T3 | 抽取 text.rs（文本/评分辅助） | cargo check 全绿 | C3 |
| T4 | 抽取 tests.rs + mod.rs 收尾为 facade | 单文件 < 800 | C4 |
| T5 | 全量验证 + code review + release readiness | fmt/check/clippy/test 全绿 | C5/C6 |

## 验证口径

- `cargo fmt --check`
- `cargo check -p homie-app`
- `cargo clippy -p homie-app --all-targets`
- `cargo test -p homie-app`（code_intelligence 相关 7 个测试原样通过，行为等价）
- `mod.rs` / `index.rs` / `reference.rs` / `text.rs` / `tests.rs` 均 < 800 行
- `code_viewer.rs` 零改动

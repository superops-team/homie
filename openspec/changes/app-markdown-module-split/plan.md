# OpenSpec Plan — app-markdown-module-split

## 目标

将 `homie/crates/homie-app/src/markdown.rs`（1,105 行）机械拆分为目录化聚焦子模块：
`block.rs`、`inline.rs`、`tests.rs`，`mod.rs` 保留 facade（文档模型类型 + 常量 + 纯文本投影）。
公共 API 与运行时行为零变更，引用方零改动，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ `block.rs`/`inline.rs`（子）。子模块间通过 `use super::*` 访问
  父模块私有类型/常量/字段；`block.rs` 通过 `use super::inline::parse_inline;` 显式引用
  兄弟模块入口。
- 块级解析（`parse_blocks` 与块级辅助）下沉 `block.rs`；行内解析（`parse_inline` 与行内
  辅助）下沉 `inline.rs`；既有测试原样下沉 `tests.rs`。
- 文档模型类型（`MarkdownDocument`/`MarkdownBlock`/`MarkdownListItem`/`InlineText`/
  `MarkdownSpan`/`InlineStyle`）与常量、`bounded_source`、纯文本投影保留在 `mod.rs`。

## 交付切片

- T1：抽取块级解析 → `block.rs`。
- T2：抽取行内解析 → `inline.rs`。
- T3：测试原样下沉 → `tests.rs`。
- T4：全量验证 + code review + release readiness。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-markdown-module-split/`。

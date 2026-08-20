# PRD — app-markdown-module-split

## 背景

`homie/crates/homie-app/src/markdown.rs`（1,105 行）是有界、呈现无关的 Markdown 解析模块的
God Module：一个文件同时承载文档模型类型、块级解析、行内解析、纯文本投影、以及测试五个
职责，单文件超过 800 行阈值，阅读与变更成本高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `markdown.rs` 机械拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何 Markdown 解析语义、敌对输入限制、链接激活策略。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `MarkdownDocument`/`MarkdownBlock`/`InlineText` 等公共类型与 `parse`/`plain_text`
  方法签名。

## 用户场景

1. 开发者定位「块级解析」（标题/列表/引用/代码块）时，直接进入 `block.rs`。
2. 开发者定位「行内解析」（强调/代码/链接）时，直接进入 `inline.rs`。
3. 开发者定位「文档模型与纯文本投影」时，聚焦在 `mod.rs`。
4. 新成员理解 Markdown 解析器的职责边界，降低上手成本。

## 模块划分方案

```text
markdown/
├── mod.rs      facade：文档模型类型 + 常量 + 纯文本投影 + 模块声明，~186 行
├── block.rs    块级解析（parse_blocks + 标题/列表/引用/代码块/分隔线识别），~297 行
├── inline.rs   行内解析（parse_inline + 强调/代码/链接/自动链接/转义），~374 行
└── tests.rs    既有测试（原样下沉，#[cfg(test)]），~256 行
```

职责边界：
- `mod.rs` 保留：`MarkdownDocument`/`MarkdownBlock`/`MarkdownListItem`/`InlineText`/
  `MarkdownSpan`/`InlineStyle` 类型与常量；`impl MarkdownDocument`/`impl InlineText`；
  `ParsedBlocks`/`Fence`/`ListMarker` 内部类型；`bounded_source`；`blocks_plain_text`/
  `block_plain_text` 纯文本投影。
- `block.rs` 下沉：`parse_blocks` 与块级辅助（`starts_block`/`strip_block_indent`/
  `fence_start`/`is_fence_close`/`heading`/`is_thematic_break`/`quote_content`/`list_marker`/
  `task_marker`/`leading_indent`）。
- `inline.rs` 下沉：`parse_inline` 与行内辅助（`parse_inline_into`/`escaped_character`/
  `style_delimiter`/`find_style_close`/`find_unescaped`/`is_escaped`/`parse_link`/
  `matching_bracket`/`matching_parenthesis`/`link_destination`/`looks_like_autolink`/
  `sanitize_link`/`matches_ignore_ascii_case`/`bare_url`/`trim_url_punctuation`/`push_span`）。

## 可见性设计

- 公共 API（`MarkdownDocument`、`MarkdownBlock`、`MarkdownListItem`、`InlineText`、
  `MarkdownSpan`、`InlineStyle`）路径不变，仍通过 `crate::markdown::{...}` 可达。
- `parse_blocks` 升为 `pub(super)`（`mod.rs` 调用）；`parse_inline` 升为 `pub(super)`
  （`mod.rs` 与 `block.rs` 跨模块调用）；其余辅助函数仅模块内部调用，保持私有 `fn`。
- `block.rs` 通过 `use super::inline::parse_inline;` 显式引用兄弟模块入口；其余通过
  `use super::*` 访问父模块私有项。

## 影响面

- 引用方：`markdown_view`、`inspector/*` 等仅依赖公共 API，零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo test -p homie-app --offline` 全绿（markdown 相关测试原样通过）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件删除。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `markdown/` 目录）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：无可见性泄漏到 crate 外（仅 `parse_blocks`/`parse_inline` 升 `pub(super)`）。
- C6：release readiness 证据写入 `docs/verification/app-markdown-module-split/`。

## Beads

- `homie-vu7`

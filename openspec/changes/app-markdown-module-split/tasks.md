# OpenSpec Tasks — app-markdown-module-split

## T1 抽取块级解析 block.rs

- [x] `block.rs`：下沉 `parse_blocks` 与块级辅助（`starts_block`/`strip_block_indent`/
  `fence_start`/`is_fence_close`/`heading`/`is_thematic_break`/`quote_content`/`list_marker`/
  `task_marker`/`leading_indent`）。
- [x] `parse_blocks` 升 `pub(super)`（mod.rs 调用）；其余辅助保持私有。
- [x] 头：`use super::*;` + `use super::inline::parse_inline;`。
- [x] `mod.rs`：`mod block;` + `use block::parse_blocks;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T2 抽取行内解析 inline.rs

- [x] `inline.rs`：下沉 `parse_inline` 与行内辅助（`parse_inline_into`/`escaped_character`/
  `style_delimiter`/`find_style_close`/`find_unescaped`/`is_escaped`/`parse_link`/
  `matching_bracket`/`matching_parenthesis`/`link_destination`/`looks_like_autolink`/
  `sanitize_link`/`matches_ignore_ascii_case`/`bare_url`/`trim_url_punctuation`/`push_span`）。
- [x] `parse_inline` 升 `pub(super)`（mod.rs 与 block.rs 调用）；其余辅助保持私有。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod inline;` + `use inline::parse_inline;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T3 测试原样下沉 tests.rs

- [x] `tests.rs`：既有测试原样下沉为 `#[cfg(test)]` 文件，剥离 `mod tests {}` 外壳。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`#[cfg(test)] mod tests;`。
- 验收：`cargo test -p homie-app --offline markdown` 12/12 通过。关联 C2/C4。

## T4 全量验证 + code review + release readiness

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-app --offline`
- [x] `cargo clippy -p homie-app --all-targets --offline`
- [x] `cargo test -p homie-app --offline`
- [x] code review（拆分边界清晰、无行为变更、无可见性泄漏、引用方零改动）
- [x] release readiness 证据写入 `docs/verification/app-markdown-module-split/`
- 验收：全部通过。关联 C3/C4/C6。

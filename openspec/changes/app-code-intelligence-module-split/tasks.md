# OpenSpec Tasks — app-code-intelligence-module-split

## T1 抽取索引构建 index.rs

- [x] `code_intelligence.rs` → `code_intelligence/mod.rs`（目录化）。
- [x] `index.rs`：移动 `discover_workspace_root` / `build_index` / `git_paths` /
  `bytes_to_path`（unix/not-unix）/ `filesystem_paths` / `ignored_path` / `index_symbols` /
  `symbol_name` / `is_symbol_text_file`。
- [x] 私有结构 `WorkspaceIndex` / `IndexedFile` / `IndexedSymbol` 移入 `index.rs`。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod index;` + 相关 `use`。
- 验收：`cargo check -p homie-app` 全绿。关联 C1。

## T2 抽取引用解析 reference.rs

- [x] `reference.rs`：移动 `parse_reference_candidates` / `clean_wrappers` / `parse_reference_fragment` /
  `parse_file_uri` / `parse_line_fragment` / `parse_stack_reference` / `parse_colon_target` /
  `looks_like_path` / `percent_decode` / `hex_value`。
- [x] 私有结构 `ParsedReference` 移入 `reference.rs`。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod reference;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2。

## T3 抽取文本/评分辅助 text.rs

- [x] `text.rs`：移动 `lexical_normalize` / `source_lines` / `clamp_target` / `path_score` /
  `fuzzy_score` / `subsequence_score` / `excerpt`。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod text;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C3。

## T4 抽取测试 tests.rs + mod.rs 收尾 facade

- [x] `tests.rs`：移动 `#[cfg(test)] mod tests` 全部内容，头 `use super::*;`。
- [x] `mod.rs`：`#[cfg(test)] mod tests;`。
- [x] `mod.rs` 保留公共常量/类型 + `impl CodeIntelligence` + 子模块声明。
- 验收：每个文件 < 800 行。关联 C4。

## T5 全量验证 + code review + release readiness

- [x] `cargo fmt --check`
- [x] `cargo check -p homie-app`
- [x] `cargo clippy -p homie-app --all-targets`
- [x] `cargo test -p homie-app`
- [x] code review（拆分边界清晰、无行为变更、无可见性泄漏、code_viewer.rs 零改动）
- [x] release readiness 证据写入 `docs/verification/app-code-intelligence-module-split/`
- 验收：全部通过。关联 C5/C6。

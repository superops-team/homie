# OpenSpec Tasks — app-diff-mod-split

## T1 抽取 git 加载/比较逻辑 load.rs

- [x] `load.rs`：下沉 `discover_repository`/`load_diff_from_repository`/`append_cached_diff`/
  `append_working_diff`/`append_untracked_diffs` 与 `ComparisonResolution`/`resolve_comparison`/
  `resolve_default_branch_ref`/`git`/`git_command`/`append_output`/`append_bytes`/`git_failure`。
- [x] 头：`use super::{DiffError, DiffLayer, DiffRow, DiffRowKind, DiffSnapshot, LocalDiffSource,
  MAX_DIFF_BYTES, MAX_UNTRACKED_FILES}; use super::parse::parse_unified_diff_bytes;` 等。
- [x] `discover_repository`/`load_diff_from_repository`/`git_command` → `pub(super)`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T2 抽取 patch 状态机解析 parse.rs

- [x] `parse.rs`：下沉 `parse_unified_diff_bytes`/`trim_patch_line`/`push_row`/`finish_hunk`/
  `finish_file`/`fnv1a64`/`diff_path`/`parse_hunk_start`/`range_start`。
- [x] 头：`use super::{DiffFile, DiffHunk, DiffRow, DiffRowKind, DiffSnapshot};` 等。
- [x] `parse_unified_diff_bytes`/`fnv1a64`/`parse_hunk_start` → `pub(super)`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T3 测试原样下沉 tests.rs

- [x] `tests.rs`：既有测试原样下沉为 `#[cfg(test)]` 文件，剥离 `mod tests {}` 外壳。
- [x] 头：`use super::*; use super::load::git_command; use super::parse::{fnv1a64,
  parse_hunk_start}; use std::fs; use std::process::Command;`。
- [x] `mod.rs`：`mod load; mod parse; #[cfg(test)] mod tests;`。
- 验收：`cargo test -p homie-app --offline diff` 9/9 通过。关联 C2/C4。

## T4 全量验证 + code review + release readiness

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-app --offline`
- [x] `cargo clippy -p homie-app --all-targets --offline`
- [x] `cargo test -p homie-app --offline diff`
- [x] code review（拆分边界清晰、无行为变更、无 `pub` 可见性泄漏、引用方零改动）
- [x] release readiness 证据写入 `docs/verification/app-diff-mod-split/`
- 验收：全部通过。关联 C3/C4/C6。

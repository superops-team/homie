# OpenSpec Tasks — app-git-review-module-split

## T1 抽取子进程执行 process.rs

- [x] `git_review.rs` → `git_review/mod.rs`（目录化）。
- [x] `process.rs`：移动 `GitOutput` + impl、`ensure_success`、`run_git`、`read_bounded`、`join_reader`。
- [x] 常量 `GIT_TIMEOUT`/`POLL_INTERVAL`/`MAX_STDOUT_BYTES`/`MAX_STDERR_BYTES` 移入 `process.rs`。
- [x] `GitOutput` 结构体 + `status`/`stdout` 字段 + `stderr_message`/`failure` 方法升为 `pub(super)`；
  `ensure_success`/`run_git` 升为 `pub(super)`。
- [x] 头：`use super::*;` + 相关 std 导入。
- [x] `mod.rs`：`mod process;` + `use process::{ensure_success, run_git};`。
- 验收：`cargo check -p homie-app` 全绿。关联 C1。

## T2 抽取状态解析 status.rs

- [x] `status.rs`：移动 `parse_status`/`parse_branch_header`/`parse_prefixed_count`/`add_tracked_change`/
  `is_unmerged`/`change_kind`/`split_fields`/`require_fields`/`path_from_bytes`/
  `path_from_output_line`/`trim_line_ending`。
- [x] 常量 `MAX_STATUS_ENTRIES` 移入 `status.rs`。
- [x] `parse_status`/`path_from_output_line`/`trim_line_ending` 升为 `pub(super)`。
- [x] 头：`use super::*;` + 相关 std 导入。
- [x] `mod.rs`：`mod status;` + `use status::{parse_status, path_from_output_line, trim_line_ending};`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2。

## T3 抽取路径/补丁辅助 paths.rs

- [x] `paths.rs`：移动 `patch_creates_file`/`patch_rejected`/`literal_path_command`/`validate_paths`/
  `invalid_path`/`os_str_eq_ignore_ascii_case`/`os_str_contains_nul`。
- [x] `patch_creates_file`/`patch_rejected`/`literal_path_command`/`validate_paths` 升为 `pub(super)`。
- [x] 头：`use super::*;` + `use super::process::GitOutput;` + 相关 std 导入。
- [x] `mod.rs`：`mod paths;` + `use paths::{...};`。
- 验收：`cargo check -p homie-app` 全绿。关联 C3。

## T4 抽取测试 tests.rs + mod.rs 收尾 facade

- [x] `tests.rs`：移动 `#[cfg(test)] mod tests` 全部内容。
- [x] 头：`use super::*;` + `use super::status::parse_status;` + `use crate::diff::{...};`。
- [x] `mod.rs`：`#[cfg(test)] mod tests;`。
- [x] `mod.rs` 保留公共常量/类型 + `GitReviewError` + `impl GitRepository` + 子模块声明。
- 验收：每个文件 < 800 行。关联 C4。

## T5 全量验证 + code review + release readiness

- [x] `cargo fmt --check`
- [x] `cargo check -p homie-app`
- [x] `cargo clippy -p homie-app --all-targets`
- [x] `cargo test -p homie-app`
- [x] code review（拆分边界清晰、无行为变更、无可见性泄漏、inspector 零改动）
- [x] release readiness 证据写入 `docs/verification/app-git-review-module-split/`
- 验收：全部通过。关联 C5/C6。

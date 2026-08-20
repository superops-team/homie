# OpenSpec Tasks — app-surface-shell-mod-split

## T1 抽取历史面板逻辑 history.rs

- [x] `history.rs`：下沉 `open_history`/`finish_history_load`/`finish_history_resume`/
  `resume_history`/`visible_history`/`move_history`/`activate_history`。
- [x] 头：`use std::collections::HashSet; use std::sync::Arc; use std::time::Duration;
  use gpui::Context; use homie_proto::HistoryEntry; use super::{RESULT_LIMIT, Surface};`。
- [x] `finish_history_load`/`resume_history`/`visible_history`/`move_history`/`activate_history` →
  `pub(super)`；`finish_history_resume` 保持私有。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T2 抽取 worktree 面板逻辑 worktrees.rs

- [x] `worktrees.rs`：下沉 `open_worktrees`/`refresh_worktrees`/`finish_worktrees_refresh`/
  `confirm_cleanup`。
- [x] 头：`use std::sync::Arc; use std::time::Duration; use gpui::Context; use super::Surface;`。
- [x] `refresh_worktrees`/`finish_worktrees_refresh`/`confirm_cleanup` → `pub(super)`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T3 抽取主机管理逻辑 hosts.rs

- [x] `hosts.rs`：下沉 `reload_hosts`/`begin_adding_host`/`begin_editing_host`/`select_host_field`/
  `save_host`/`initialize_host`/`reinstall_host`/`prepare_host`/`retry_host_initialization`/
  `request_remove_host`/`persist_hosts`/`edit_host_field`/`handle_host_editor_key`。
- [x] 头：`use std::sync::Arc; use gpui::{Context, KeyDownEvent, Window};
  use homie_proto::{HostEntry, HostsConfig}; use super::host_editor::...;
  use super::host_init::...;`。
- [x] `reload_hosts`/`begin_adding_host`/`begin_editing_host`/`select_host_field`/`save_host`/
  `reinstall_host`/`retry_host_initialization`/`request_remove_host`/`handle_host_editor_key` →
  `pub(super)`；`initialize_host`/`prepare_host`/`persist_hosts`/`edit_host_field` 保持私有。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T4 清理跨模块 import + 全量验证 + code review + release readiness

- [x] `mod.rs` 删除不再使用的 `HashSet`、`HostsConfig`、`expire_completed_reinstall` import。
- [x] `history.rs` 删除未使用的 `Task` import；`hosts.rs` 删除未使用的 `Duration` import。
- [x] `tests.rs` 显式导入 `super::host_init::expire_completed_reinstall`。
- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-app --offline`
- [x] `cargo clippy -p homie-app --all-targets --offline`
- [x] `cargo build -p homie-app --offline`
- [x] `cargo test -p homie-app --offline`
- [x] code review（拆分边界清晰、无行为变更、无 `pub` 可见性泄漏、引用方零改动）
- [x] release readiness 证据写入 `docs/verification/app-surface-shell-mod-split/`
- 验收：全部通过。关联 C3/C4/C6。

# Release Readiness — app-surface-shell-mod-split

## 变更摘要

将 `homie/crates/homie-app/src/surface_shell/mod.rs`（904 行）机械拆分为 3 个聚焦子模块，
`mod.rs` 收尾 facade（355 行）。公共 API 与运行时行为零变更，引用方零改动。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 355 | 结构体定义 + new + 主题颜色（colors/settings_colors）+ 代际计数 + persist_prefs + close_surface/is_open/open_settings/open_add_remote_host/toggle_history/key_down + Focusable + 模块声明 |
| `history.rs` | 161 | 历史面板：open_history/finish_history_load/finish_history_resume/resume_history/visible_history/move_history/activate_history |
| `worktrees.rs` | 78 | worktree 面板：open_worktrees/refresh_worktrees/finish_worktrees_refresh/confirm_cleanup |
| `hosts.rs` | 351 | 主机管理：reload_hosts/begin_adding_host/begin_editing_host/select_host_field/save_host/initialize_host/reinstall_host/prepare_host/retry_host_initialization/request_remove_host/persist_hosts/edit_host_field/handle_host_editor_key |

全部单文件 < 800 行。

## 可见性管控

跨模块符号均采用 `pub(super)`（仅对父模块 `surface_shell` 可见），无 `pub` 可见性泄漏到 crate 外：
- history.rs：`pub(super) fn finish_history_load/resume_history/visible_history/move_history/activate_history`。
- worktrees.rs：`pub(super) fn refresh_worktrees/finish_worktrees_refresh/confirm_cleanup`。
- hosts.rs：`pub(super) fn reload_hosts/begin_adding_host/begin_editing_host/select_host_field/save_host/reinstall_host/retry_host_initialization/request_remove_host/handle_host_editor_key`。
- 同文件内部互调保持私有：`finish_history_resume`、`initialize_host`/`prepare_host`/`persist_hosts`/`edit_host_field`。
- `tests.rs` 显式导入 `super::host_init::expire_completed_reinstall`，消除对 `super::*` 私有 `use` 泄漏的隐式依赖。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-app --offline` | ✅ 通过 |
| `cargo test -p homie-app --offline` | ✅ 301 passed / 2 failed（沙箱内 2 个 `daemon_launch` socket bind EPERM 失败属预期） |
| 引用方零改动 | ✅ `git status` 仅改 `surface_shell/` 目录 + 文档 |

## 沙箱测试说明

沙箱内 `cargo test -p homie-app` 中 2 个 `daemon_launch` 测试因 fixture daemon 的 socket bind
返回 `PermissionDenied`（EPERM）而失败，属沙箱网络限制预期，与本次拆分无关。拆分涉及的
`surface_shell` 相关测试全部通过。

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`store/mod.rs`（2,434 行）、`store/tests.rs`（1,658 行）、
  `sidebar/popover.rs`（1,249 行）、`sidebar/sections.rs`（1,202 行）、`root/mod.rs`（1,190 行）、
  `terminal_pane/view.rs`（863 行）、`switcher.rs`（797 行）等。

# OpenSpec Plan — app-surface-shell-mod-split

## 目标

将 `homie/crates/homie-app/src/surface_shell/mod.rs`（904 行）机械拆分为目录化聚焦子模块：
`history.rs`（历史面板）、`worktrees.rs`（worktree 面板）、`hosts.rs`（主机管理），`mod.rs` 保留
facade（结构体定义 + new + 主题颜色 + 代际计数 + persist_prefs + 外壳开合 + Focusable）。
公共 API 与运行时行为零变更，引用方零改动，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ `history.rs`/`worktrees.rs`/`hosts.rs`（子，经 `impl super::UtilitySurfaces`
  跨模块实现）；`tests.rs` → `mod.rs`/`host_init`。
- 跨子模块互调采用 `pub(super)`：`finish_history_load`/`resume_history`/`visible_history`/
  `move_history`/`activate_history`（history.rs）、`refresh_worktrees`/`finish_worktrees_refresh`/
  `confirm_cleanup`（worktrees.rs）、`reload_hosts`/`begin_adding_host`/`begin_editing_host`/
  `select_host_field`/`save_host`/`reinstall_host`/`retry_host_initialization`/`request_remove_host`/
  `handle_host_editor_key`（hosts.rs）。全部仅对父模块 `surface_shell` 可见，无 `pub` 泄漏。
- 同文件内部互调保持私有：`finish_history_resume`、`initialize_host`/`prepare_host`/
  `persist_hosts`/`edit_host_field`。
- `tests.rs` 改为显式导入 `super::host_init::expire_completed_reinstall`，消除对 `super::*` 私有
  `use` 泄漏的隐式依赖。

## 交付切片

- T1：抽取历史面板逻辑 → `history.rs`。
- T2：抽取 worktree 面板逻辑 → `worktrees.rs`。
- T3：抽取主机管理逻辑 → `hosts.rs`。
- T4：清理跨模块 import 警告 + 全量验证 + code review + release readiness。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-surface-shell-mod-split/`。

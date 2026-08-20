# OpenSpec Tasks — app-store-mod-split

## T1 抽取构造 + 基础访问器 lifecycle.rs

- [x] `lifecycle.rs`：下沉 `headless`/`load`/`with_path`/`load_default`/`persist_preferences`/
  `remember_window_placement`/`daemon_state`/`sessions`/`auxiliary_terminal_for`/`projects`/
  `selected_session_id`/`sidebar_selection`/`pending_close`/`auto_resuming`/`migrating`/`syncing_prefs`。
- [x] 头：`use super::*;`。
- [x] `with_path` → `pub(super)`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T2 抽取主机/catalog/migration/sync/repo/directory/prefs hosts.rs

- [x] `hosts.rs`：下沉 `reload_hosts`/`set_agent_catalog`/`refresh_agent_catalog`/`agent_catalog`/
  `agent_descriptor`/`set_hosts`/`hosts`/`host`/`host_display_name`/`default_spawn_host`/
  `set_default_spawn_host`/`repair_default_spawn_host`/`migrate_session`/`finish_migration`/
  `sync_prefs`/`finish_prefs_sync`/`begin_repo_targeting`/`request_repo_target`/`repo_target`/
  `request_directory_listing`/`directory_listing`/`finish_directory_listing`/`set_repo_target`/
  `preferences`/`terminal_residency`/`app_is_active`/`last_action_error`/`switcher_state`/
  `overview_state`/`request_snapshot_publish`/`update_preferences`/`zoom_terminal`/`reset_terminal_zoom`。
- [x] 头：`use super::*;`。
- [x] `repair_default_spawn_host`/`finish_directory_listing` → `pub(super)`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T3 抽取 sidebar 排序/pin/collapse/projection ordering.rs

- [x] `ordering.rs`：下沉 `reconcile_sidebar_order`/`sync_sidebar_order`/`sidebar_session_order`/
  `sidebar_project_order`/`set_project_order`/`set_session_order`/`stage_project_order`/
  `stage_session_order`/`toggle_project_pin`/`toggle_session_pin`/`toggle_project_collapsed`/
  `toggle_session_collapsed`/`is_descendant_of`/`toggle_archive_expanded`/`governor_settings`/
  `sidebar_projection`/`ordered_sessions`/`selected_session`。
- [x] 头：`use super::*;`。
- [x] `reconcile_sidebar_order`/`sync_sidebar_order`/`is_descendant_of` → `pub(super)`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T4 抽取 hydrate/handle_event/upsert events.rs

- [x] `events.rs`：下沉 `hydrate`/`handle_event`/`handle_event_change`/`upsert_session`/
  `remove_session_record`/`select`/`apply_spawn_result`/`sidebar_click`/`clear_sidebar_selection`/
  `sidebar_selection_ordered`/`focus_neighbor`/`mru_sessions`。
- [x] 头：`use super::*;`。
- [x] `handle_event_change`/`apply_spawn_result` → `pub(super)`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T5 抽取 switcher/overview/snapshot switcher.rs

- [x] `switcher.rs`：下沉 `handle_switcher_key`/`handle_switcher_modifiers_changed`/
  `commit_switcher_index`/`cancel_switcher`/`toggle_overview`/`dismiss_overview`/`set_overview_mode`/
  `set_overview_filter`/`append_overview_query`/`overview_backspace`/`overview_escape`/
  `move_overview_focus`/`activate_overview_focus`/`activate_overview_session`/
  `toggle_overview_selection`/`clear_overview_selection`/`select_all_overview_sessions`/
  `close_overview_selection`/`close_overview_session`/`global_attention`/`needs_input_sessions`/
  `snapshot`。
- [x] 头：`use super::*;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T6 抽取会话生命周期 sessions.rs

- [x] `sessions.rs`：下沉 `request_close`/`confirm_pending_close`/`cancel_pending_close`/
  `remove_sessions`/`archive_sessions`/`revive_sessions`/`auto_resume_if_needed`/`finish_auto_resume`/
  `resume`/`rename`/`reopen_last`/`spawn_default`/`spawn_shell`/`spawn_auxiliary_terminal`/
  `spawn_kind`/`local_fallback_directory`/`default_new_agent_directory`/`active_directory`/`set_active`。
- [x] 头：`use super::*;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T7 抽取导航 reconcile navigation.rs

- [x] `navigation.rs`：下沉 `focus_session`/`set_selected_survivor`/`auto_select_if_needed`/
  `auto_resume_selected_if_needed`/`apply_switcher_outcome`/`apply_overview_outcome`/
  `reconcile_navigation`/`reveal`/`sidebar_visible_order`/`invalidate_projection`/`emit`。
- [x] 头：`use super::*;`。
- [x] `focus_session`/`set_selected_survivor`/`auto_select_if_needed`/`auto_resume_selected_if_needed`/
  `apply_switcher_outcome`/`apply_overview_outcome`/`reconcile_navigation`/`reveal`/
  `sidebar_visible_order`/`invalidate_projection`/`emit` → `pub(super)`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T8 抽取 StoreRuntime + run_effects runtime.rs

- [x] `runtime.rs`：下沉 `StoreRuntime` 结构体 + `impl StoreRuntime` + `impl Drop for StoreRuntime` +
  `run_effects`。
- [x] 头：`use super::*;`。
- [x] `tasks` 字段 → `pub(super)`（仅 `store::tests` 访问）。
- [x] `mod.rs` 经 `pub use runtime::StoreRuntime;` 再导出，路径不变。
- 验收：`cargo check -p homie-app` 全绿。关联 C1/C2/C5。

## T9 清理跨模块可见性 + 全量验证 + code review + release readiness

- [x] `mod.rs` 收尾 facade：prelude + 模块声明 + 自由函数 + `pub use runtime::StoreRuntime;` +
  `prefs_path_in_home` + `tests` 声明。
- [x] 19 个私有方法升级 `pub(super)`，`tasks` 字段升级 `pub(super)`。
- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-app --offline`
- [x] `cargo clippy -p homie-app --all-targets --offline`
- [x] `cargo build -p homie-app --offline`
- [x] `cargo test -p homie-app --offline`
- [x] code review（拆分边界清晰、无行为变更、无 `pub` 可见性泄漏、引用方零改动）
- [x] release readiness 证据写入 `docs/verification/app-store-mod-split/`
- 验收：全部通过。关联 C3/C4/C6。

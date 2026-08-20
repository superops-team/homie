# PRD — app-store-mod-split

## 背景

`homie/crates/homie-app/src/store/mod.rs`（2,434 行）是应用会话存储的 God Module：一个文件同时
承载 `SessionStore` 结构体定义与构造、daemon 事件 hydrate/handle_event/upsert、会话生命周期
（spawn/close/archive/resume/rename）、sidebar 排序/pin/collapse/projection、switcher/overview
交互、导航 reconcile/reveal、以及 `StoreRuntime` 运行时桥与 `run_effects` 效果循环，单文件远超
800 行阈值，阅读与变更成本极高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `store/mod.rs` 机械拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何 daemon 事件处理、会话生命周期、sidebar 排序、switcher/overview、导航 reconcile、
  LLM/效果循环行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `SessionStore`/`StoreRuntime` 结构体字段、`StoreEffect`/`DaemonState` 枚举、任何公开
  函数签名。

## 用户场景

1. 开发者定位「会话生命周期（spawn/close/archive/resume/rename）」时，直接进入 `sessions.rs`。
2. 开发者定位「daemon 事件 hydrate/handle_event/upsert」时，聚焦在 `events.rs`。
3. 开发者定位「sidebar 排序/pin/collapse/projection」时，聚焦在 `ordering.rs`。
4. 开发者定位「switcher/overview 交互」时，聚焦在 `switcher.rs`。
5. 开发者定位「运行时桥 + 效果循环」时，聚焦在 `runtime.rs`。

## 模块划分方案

```text
store/
├── mod.rs         facade：prelude（imports/常量/枚举/结构体）+ 自由函数 + StoreRuntime 再导出 +
│                  prefs_path_in_home + 模块声明 + tests 声明
├── lifecycle.rs   构造 + 基础访问器：headless/load/with_path/load_default/persist_preferences/
│                  remember_window_placement/daemon_state/sessions/auxiliary_terminal_for/projects/
│                  selected_session_id/sidebar_selection/pending_close/auto_resuming/migrating/
│                  syncing_prefs
├── hosts.rs       主机/catalog/migration/sync/repo/directory/prefs：reload_hosts/set_agent_catalog/
│                  refresh_agent_catalog/agent_catalog/agent_descriptor/set_hosts/hosts/host/
│                  host_display_name/default_spawn_host/set_default_spawn_host/repair_default_spawn_host/
│                  migrate_session/finish_migration/sync_prefs/finish_prefs_sync/begin_repo_targeting/
│                  request_repo_target/repo_target/request_directory_listing/directory_listing/
│                  finish_directory_listing/set_repo_target/preferences/terminal_residency/app_is_active/
│                  last_action_error/switcher_state/overview_state/request_snapshot_publish/
│                  update_preferences/zoom_terminal/reset_terminal_zoom
├── ordering.rs    sidebar 排序/pin/collapse/projection：reconcile_sidebar_order/sync_sidebar_order/
│                  sidebar_session_order/sidebar_project_order/set_project_order/set_session_order/
│                  stage_project_order/stage_session_order/toggle_project_pin/toggle_session_pin/
│                  toggle_project_collapsed/toggle_session_collapsed/is_descendant_of/
│                  toggle_archive_expanded/governor_settings/sidebar_projection/ordered_sessions/
│                  selected_session
├── events.rs      hydrate/handle_event/handle_event_change/upsert_session/remove_session_record/
│                  select/apply_spawn_result/sidebar_click/clear_sidebar_selection/
│                  sidebar_selection_ordered/focus_neighbor/mru_sessions
├── switcher.rs    switcher/overview/snapshot：handle_switcher_key/handle_switcher_modifiers_changed/
│                  commit_switcher_index/cancel_switcher/toggle_overview/dismiss_overview/
│                  set_overview_mode/set_overview_filter/append_overview_query/overview_backspace/
│                  overview_escape/move_overview_focus/activate_overview_focus/activate_overview_session/
│                  toggle_overview_selection/clear_overview_selection/select_all_overview_sessions/
│                  close_overview_selection/close_overview_session/global_attention/needs_input_sessions/
│                  snapshot
├── sessions.rs    会话生命周期：request_close/confirm_pending_close/cancel_pending_close/
│                  remove_sessions/archive_sessions/revive_sessions/auto_resume_if_needed/
│                  finish_auto_resume/resume/rename/reopen_last/spawn_default/spawn_shell/
│                  spawn_auxiliary_terminal/spawn_kind/local_fallback_directory/
│                  default_new_agent_directory/active_directory/set_active
├── navigation.rs  导航 reconcile：focus_session/set_selected_survivor/auto_select_if_needed/
│                  auto_resume_selected_if_needed/apply_switcher_outcome/apply_overview_outcome/
│                  reconcile_navigation/reveal/sidebar_visible_order/invalidate_projection/emit
└── runtime.rs     StoreRuntime 结构体 + impl + Drop + run_effects
```

## 可见性设计

- 公共 API（`SessionStore::new`/`load`/`load_default`/`hydrate`/`handle_event`/`spawn_default` 等、
  `StoreRuntime::start`/`start_default`/`inert`/`changes`/`snapshots` 等）路径不变。
- `StoreRuntime` 在 `runtime.rs` 中定义为 `pub struct`，经 `mod.rs` 中 `pub use runtime::StoreRuntime;`
  再导出，`crate::store::StoreRuntime` 路径不变。
- 跨子模块互调采用 `pub(super)`（仅对父模块 `store` 可见，不泄漏到 crate 外），共 19 个私有方法
  因跨文件调用升级为 `pub(super)`：
  - `lifecycle.rs`：`with_path`。
  - `hosts.rs`：`repair_default_spawn_host`、`finish_directory_listing`。
  - `ordering.rs`：`reconcile_sidebar_order`、`sync_sidebar_order`、`is_descendant_of`。
  - `events.rs`：`handle_event_change`、`apply_spawn_result`。
  - `navigation.rs`：`focus_session`、`set_selected_survivor`、`auto_select_if_needed`、
    `auto_resume_selected_if_needed`、`apply_switcher_outcome`、`apply_overview_outcome`、
    `reconcile_navigation`、`reveal`、`sidebar_visible_order`、`invalidate_projection`、`emit`。
  - `runtime.rs`：`tasks` 字段升级为 `pub(super)`（仅 `store::tests` 访问）。
- 无 `pub` 可见性泄漏到 crate 外，所有跨模块符号均为 `pub(super)` 或保持私有。

## 影响面

- 引用方：`crate::store::{SessionStore, StoreRuntime, StoreEffect, Prefs, ...}` 路径不变，零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo build -p homie-app --offline` 通过。
- `cargo test -p homie-app --offline` 全绿（沙箱内 2 个 `daemon_launch` EPERM 属预期）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（2,434 行）瘦身为 facade（352 行）。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `store/` 目录 + 文档）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：无 `pub` 可见性泄漏到 crate 外（跨模块符号均为 `pub(super)` 或私有）。
- C6：release readiness 证据写入 `docs/verification/app-store-mod-split/`。

## Beads

- `homie-yyy`

# Release Readiness — app-store-mod-split

## 变更摘要

将 `homie/crates/homie-app/src/store/mod.rs`（2,434 行）机械拆分为 8 个聚焦子模块 + facade，
`mod.rs` 收尾 facade（352 行）。公共 API 与运行时行为零变更，引用方零改动。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 352 | facade：prelude（imports/常量/枚举/结构体）+ 自由函数（attention_rank/reconcile_order/retain_live/toggle_vec_member/now_millis）+ `pub use runtime::StoreRuntime` + prefs_path_in_home + 模块声明 + tests 声明 |
| `lifecycle.rs` | 126 | 构造 + 基础访问器：headless/load/with_path/load_default/persist_preferences/remember_window_placement/daemon_state/sessions/auxiliary_terminal_for/projects/selected_session_id/sidebar_selection/pending_close/auto_resuming/migrating/syncing_prefs |
| `hosts.rs` | 237 | 主机/catalog/migration/sync/repo/directory/prefs：reload_hosts/set_agent_catalog/…/migrate_session/…/directory_listing/…/update_preferences/zoom_terminal/reset_terminal_zoom |
| `ordering.rs` | 224 | sidebar 排序/pin/collapse/projection：reconcile_sidebar_order/sync_sidebar_order/…/sidebar_projection/ordered_sessions/selected_session |
| `events.rs` | 345 | hydrate/handle_event/handle_event_change/upsert_session/…/focus_neighbor/mru_sessions |
| `switcher.rs` | 169 | switcher/overview/snapshot：handle_switcher_key/…/global_attention/needs_input_sessions/snapshot |
| `sessions.rs` | 315 | 会话生命周期：request_close/…/spawn_default/…/set_active |
| `navigation.rs` | 171 | 导航 reconcile：focus_session/…/reveal/…/emit |
| `runtime.rs` | 447 | StoreRuntime 结构体 + impl + Drop + run_effects |

全部单文件 < 800 行。

## 可见性管控

跨模块符号均采用 `pub(super)`（仅对父模块 `store` 可见），无 `pub` 可见性泄漏到 crate 外：

- 19 个私有方法因跨文件调用升级为 `pub(super)`：`with_path`（lifecycle）、`repair_default_spawn_host`/
  `finish_directory_listing`（hosts）、`reconcile_sidebar_order`/`sync_sidebar_order`/`is_descendant_of`
  （ordering）、`handle_event_change`/`apply_spawn_result`（events）、`focus_session`/`set_selected_survivor`/
  `auto_select_if_needed`/`auto_resume_selected_if_needed`/`apply_switcher_outcome`/`apply_overview_outcome`/
  `reconcile_navigation`/`reveal`/`sidebar_visible_order`/`invalidate_projection`/`emit`（navigation）。
- `runtime.rs` 的 `tasks` 字段升级为 `pub(super)`（仅 `store::tests` 访问）。
- `StoreRuntime` 经 `mod.rs` 中 `pub use runtime::StoreRuntime;` 再导出，`crate::store::StoreRuntime` 路径不变。
- 自由函数与常量保留在 `mod.rs`，各子模块经 `use super::*;` 访问。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-app --offline` | ✅ 通过 |
| `cargo test -p homie-app --offline` | ✅ 301 passed / 2 failed（沙箱内 2 个 `daemon_launch` socket bind EPERM 失败属预期） |
| `cargo test -p homie-app --offline store::` | ✅ 53 passed / 0 failed |
| 引用方零改动 | ✅ `git status` 仅改 `store/` 目录 + 文档 |

## 沙箱测试说明

沙箱内 `cargo test -p homie-app` 中 2 个 `daemon_launch` 测试因 fixture daemon 的 socket bind
返回 `PermissionDenied`（EPERM）而失败，属沙箱网络限制预期，与本次拆分无关。拆分涉及的
`store` 相关测试（53 个）全部通过。

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`store/tests.rs`（1,658 行）、`sidebar/popover.rs`（1,249 行）、
  `sidebar/sections.rs`（1,202 行）、`root/mod.rs`（1,190 行）、`terminal_pane/view.rs`（863 行）、
  `switcher.rs`（797 行）等。

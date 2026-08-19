# OpenSpec Tasks — app-surface-shell-view-split

## T1 抽取通用 UI 原语 widgets.rs

- [x] `widgets.rs`：移动 21 个自由渲染辅助（`surface_button` / `settings_primary_button` /
  `settings_danger_button` / `danger_button` / `toggle_row` / `setting_section` / `setting_row` /
  `setting_text_stack` / `wrappable_setting_copy` / `settings_note` / `settings_page` /
  `setting_divider` / `settings_select_button` / `settings_dropdown` / `settings_choice_row` /
  `theme_preview` / `chip` / `colored_badge` / `empty_label` / `host_field_value` /
  `text_offset_for_x`）。
- [x] 全部标记 `pub(super)`（供 view.rs / settings_view.rs / hosts_view.rs 调用）。
- [x] `mod.rs`：`mod widgets;`。
- [x] `tests.rs`：`use super::view::setting_row;` → `use super::widgets::setting_row;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C1。

## T2 抽取远端主机管理渲染 hosts_view.rs

- [x] `hosts_view.rs`：移动 `remote_settings` / `remote_hosts_section` /
  `host_initialization_card` / `host_editor_panel` / `host_text_field`。
- [x] `remote_settings` 标记 `pub(super)`（供 settings_view.rs 调用）；其余保持私有。
- [x] `hosts_view.rs` 头：`use super::*;` + `use super::widgets::*;`。
- [x] `mod.rs`：`mod hosts_view;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2。

## T3 抽取应用设置渲染 settings_view.rs

- [x] `settings_view.rs`：移动 `render_settings` / `general_settings` / `default_agent_dropdown` /
  `update_settings` / `terminal_settings` / `resource_settings` / `terminal_theme_dropdown` /
  `hibernate_dropdown` / `memory_dropdown`。
- [x] `render_settings` 标记 `pub(super)`（供 view.rs facade 调用）；其余保持私有。
- [x] `settings_view.rs` 头：`use super::*;` + `use super::widgets::*;`。
- [x] `mod.rs`：`mod settings_view;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C3。

## T4 view.rs 收尾为 facade

- [x] `view.rs`：保留 `render_history` / `render_worktrees` / `impl Render for UtilitySurfaces`。
- [x] `view.rs` 头：`use super::*;` + `use super::widgets::*;`。
- 验收：`view.rs` < 800 行。关联 C4。

## T5 全量验证 + code review + release readiness

- [x] `cargo fmt --check`
- [x] `cargo check -p homie-app`
- [x] `cargo test -p homie-app`
- [x] code review（拆分边界清晰、无行为变更、无可见性泄漏）
- [x] release readiness 证据写入 `docs/verification/app-surface-shell-view-split/`
- 验收：全部通过。关联 C5/C6。

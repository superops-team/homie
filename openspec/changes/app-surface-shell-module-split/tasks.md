# OpenSpec Tasks — app-surface-shell-module-split

## S1 抽取纯投影 projection.rs

- [x] `projection.rs`：移动 `ui_agent`、`ui_default_agent`、`folder_name`、`relative_parent`、`update_detail`、`relative_time`，`pub(crate)`。
- [x] `mod.rs`：`mod projection;` + `use projection::*;`。
- 验收：`cargo test -p homie-app` 全绿。关联 C1。

## S2 抽取 host 表单状态机 host_editor.rs

- [x] `host_editor.rs`：移动 `HostFormField` + `impl HostFormField`（`ALL`/`adjacent`/`debug_name`/`index`）、`HostEditor` + `impl HostEditor`（`adding`/`editing`/`from_draft`/`draft`/`field_mut`/`field`）、`text_editor`，`pub(crate)`。
- [x] `mod.rs`：`mod host_editor;` + `use host_editor::{HostEditor, HostFormField, text_editor};`。
- 验收：`cargo test -p homie-app` 全绿。关联 C2。

## S3 抽取 host 初始化生命周期 host_init.rs

- [x] `host_init.rs`：移动 `HostPreparationKind`、`HostInitialization` + `impl HostInitialization`（`id`/`operation`）、`HostInitializationCardModel`、`expire_completed_reinstall`，`pub(crate)`。
- [x] `mod.rs`：`mod host_init;` + `use host_init::{HostInitialization, HostInitializationCardModel, HostPreparationKind, expire_completed_reinstall};`。
- 验收：`cargo test -p homie-app` 全绿。关联 C3。

## S4 抽取渲染到 view.rs

- [x] `view.rs`：移动 `impl Render for UtilitySurfaces` + 16 个渲染方法（`render_history`/`render_worktrees`/`render_settings`/`general_settings`/`default_agent_dropdown`/`update_settings`/`terminal_settings`/`resource_settings`/`remote_settings`/`remote_hosts_section`/`host_initialization_card`/`host_editor_panel`/`host_text_field`/`terminal_theme_dropdown`/`hibernate_dropdown`/`memory_dropdown`）+ 23 个自由渲染辅助函数。
- [x] `mod.rs`：`mod view;`；从 `impl UtilitySurfaces` 删除渲染方法。
- 验收：`cargo test -p homie-app` 全绿。关联 C4。

## S5 迁移测试到 tests.rs + facade 收尾

- [x] `tests.rs`：移动 16 个测试 + `utility_surfaces_for_unit_tests`/`history_entry`/`worktree_entry` 辅助 + `use super::*;` + 测试专属 `use`。
- [x] `mod.rs`：删除内联 `mod tests`，加 `#[cfg(test)] mod tests;`。
- [x] 全量验证：`cargo check` + `cargo fmt --check` + `cargo test`。
- 验收：全部通过。关联 C5。

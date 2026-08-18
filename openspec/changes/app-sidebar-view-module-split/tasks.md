# OpenSpec Tasks — app-sidebar-view-module-split

## S1 抽取纯投影 projection.rs

- [x] `projection.rs`：移动 `count_label`/`display_title`/`status_state`/`agent_picker_options`/`ui_agent_kind`/`rollup_attention`/`attention_rank`/`retain_live_glyphs`/`shortcut_ranks`/`clamp_path`/`session_title_available_width`/`compact_duration`，`pub(crate)`。
- [x] `mod.rs`：`mod projection;` + `use projection::*;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C1。

## S2 抽取渲染辅助 render_helpers.rs

- [x] `render_helpers.rs`：移动 `icon_button`/`ENDED_TITLE`/`indent_rails`/`pin_mark`/`state_chip`/`alert_chip`/`project_badge`/`menu_row`/`directory_row`/`menu_divider`/`copy_session_id_row`/`section_label`/`usage_row`/`hover_detail`，`pub(crate)`。
- [x] `mod.rs`：`mod render_helpers;` + `use render_helpers::*;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2。

## S3 抽取 section 渲染 sections.rs

- [x] `sections.rs`：移动 `impl Sidebar` 的 `new_agent_row`/`top_bar`/`empty_state`/`project_section`/`session_row`/`disclosure`/`archived_bucket`/`archived_row`/`update_pill`/`account_footer`，`pub(crate)`。
- [x] `mod.rs`：`mod sections;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C3。

## S4 抽取 popover 方法 popover.rs

- [x] `popover.rs`：移动 `impl Sidebar` 的 popover 相关方法（10 个），`pub(crate)`。
- [x] `mod.rs`：`mod popover;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C4。

## S5 抽取 dispatch commands.rs + tests.rs + facade 收尾

- [x] `commands.rs`：移动 `impl Sidebar` 的 `surface_fill`/`scroll_fades`/`hover_card`/`status_glyph`/`shortcut_for`/`reorder_project`/`reorder_session`/`finish_drag`/`reorder_session_to_end`/`archive_sessions`/`close_sessions`/`close_sessions_immediately`，`pub(crate)`。
- [x] `tests.rs`：移动 16 个测试 + `SidebarPopoverHarness` + `use super::*;` + 测试专属 `use`。
- [x] `mod.rs`：删除内联 `mod tests`，加 `#[cfg(test)] mod tests;`；`pub use` 再导出保持公共 API。
- [x] 全量验证：`cargo check` + `cargo fmt --check` + `cargo test`。
- 验收：全部通过。关联 C5。

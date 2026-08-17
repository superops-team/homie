# OpenSpec Tasks — app-terminal-pane-module-split

## S1 抽取纯函数（keys / policy / projection）

- [x] `keys.rs`：移动 `terminal_key_event`，`pub(crate)`。
- [x] `policy.rs`：移动 `terminal_damage_should_repaint`、`ResizePlan`、`plan_resize`、`should_hold_reflow`、`estimated_grid_size`、`clipboard_image`，`pub(crate)`。
- [x] `projection.rs`：移动 `ui_agent_kind`、`status_state`、`exit_description`，`pub(crate)`。URL/PR 解析辅助（`pr_number`/`linear_key`/`url_host`/`url_port`/`pr_tint`/`pr_help`/`comments_help`）因仅被 chip 使用，随 S2 收拢到 `chip.rs`。
- [x] `mod.rs`：`mod keys; mod policy; mod projection;` + `use keys::*; use policy::*; use projection::*;`（或按需 `use`）。
- 验收：`cargo test -p homie-app` 全绿。关联 C1。

## S2 抽取 chip 投影

- [x] `chip.rs`：移动 `ChipTint`、`PaneChip` + `impl PaneChip`（`for_session`/`from_artifact`/`quiet_artifact`/`checks_chip`/`comments_chip`）+ URL/PR 解析辅助（`pr_number`/`linear_key`/`url_host`/`url_port`/`pr_tint`/`pr_help`/`comments_help`），`ChipTint`/`PaneChip` 保持 `pub`（原公共 API）。
- [x] `mod.rs`：`mod chip;` + `pub use chip::{ChipTint, PaneChip};`。
- 验收：`cargo test -p homie-app` 全绿。关联 C2。

## S3 抽取 attachment 生命周期

- [x] `attachment.rs`：移动 `AttachmentState`、`AttachmentCommand`、`AttachmentControl` + `impl`，`spawn_attachment`、`wait_for_retry`，`pub(crate)`。
- [x] `mod.rs`：`mod attachment;` + `use attachment::{AttachmentControl, AttachmentState, spawn_attachment};`；`ResidentTerminal` 字段类型保持。
- 验收：`cargo test -p homie-app` 全绿。关联 C3。

## S4 抽取渲染到 view.rs

- [x] `view.rs`：移动 `impl Render for TerminalPane` + 7 个 `render_*` 方法（`render_sidebar_reveal_control`/`render_header`/`render_grid_and_overlays`/`render_exit_pill`/`render_find_bar`/`render_exited_takeover`/`render_exited_card`/`render_archived_overlay`）+ 4 个自由渲染辅助函数。
- [x] `mod.rs`：`mod view;`；从 `impl TerminalPane` 删除 render 方法。
- 验收：`cargo test -p homie-app` 全绿。关联 C4。

## S5 迁移测试到 tests.rs + facade 收尾

- [x] `tests.rs`：移动 24 个测试 + `sorted_checks`（测试专属辅助）+ `use super::*;` + 测试专属 `use`。
- [x] `mod.rs`：删除内联 `mod tests`，加 `#[cfg(test)] mod tests;`。
- [x] 全量验证：`cargo check` + `cargo fmt --check` + `cargo test`。
- 验收：全部通过。关联 C5。

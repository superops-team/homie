# OpenSpec Tasks — app-terminal-pane-mod-split

## T1 抽取事件分发 + 网格重排 events.rs

- [x] `events.rs`：下沉 `handle_pane_event`/`apply_grid_updates`/`hold_reflow`/
  `release_reflow_hold`/`request_terminal_repaint`。
- [x] 跨模块调用的方法升 `pub(super)`；仅内部调用保持私有。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod events;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T2 抽取键盘输入 input.rs

- [x] `input.rs`：下沉 `handle_key_down`/`handle_key_up`/`handle_modifiers_changed`。
- [x] 跨模块调用的方法升 `pub(super)`；仅内部调用保持私有。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod input;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T3 抽取查找 find.rs

- [x] `find.rs`：下沉 `schedule_find`/`start_due_find`/`open_find`/`close_find`/
  `close_find_for_selected`/`find_next`/`find_previous`/`navigate_find`。
- [x] 跨模块调用的方法升 `pub(super)`；仅内部调用保持私有。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod find;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T4 抽取网格几何 + 缩放 geometry.rs

- [x] `geometry.rs`：下沉 `grid_cell_at`/`grid_row_overflow`/`grid_inner_height`/
  `zoom_in`/`zoom_out`/`reset_zoom`/`change_zoom`/`update_selected_geometry`。
- [x] 跨模块调用的方法升 `pub(super)`；仅内部调用保持私有。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod geometry;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T5 抽取剪贴板 clipboard.rs

- [x] `clipboard.rs`：下沉 `copy_selection`/`paste`。
- [x] 跨模块调用的方法升 `pub(super)`；仅内部调用保持私有。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod clipboard;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T6 抽取滚动回取 + 滚动 scroll.rs

- [x] `scroll.rs`：下沉 `pump_scrollback_fetch`/`handle_scroll`。
- [x] 跨模块调用的方法升 `pub(super)`；仅内部调用保持私有。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod scroll;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T7 全量验证 + code review + release readiness

- [x] `cargo fmt --check`
- [x] `cargo check -p homie-app`
- [x] `cargo clippy -p homie-app --all-targets`
- [x] `cargo test -p homie-app`
- [x] code review（拆分边界清晰、无行为变更、无可见性泄漏、引用方零改动）
- [x] release readiness 证据写入 `docs/verification/app-terminal-pane-mod-split/`
- 验收：全部通过。关联 C3/C4/C6。

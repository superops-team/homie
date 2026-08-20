# PRD — app-terminal-pane-mod-split

## 背景

`homie/crates/homie-app/src/terminal_pane/mod.rs`（1,486 行）是终端面板的 God Module：单个
`impl TerminalPane` 块高达约 1,229 行，叠加生命周期/驻留协调、事件分发、网格重排、查找、
缩放、网格几何、剪贴板、键盘输入、滚动回取/滚动、选中几何更新十余个职责。虽然渲染
（`view.rs`）、chip、policy、projection、attachment、keys 已下沉为子模块，`mod.rs` 本身仍超
800 行阈值，阅读与变更成本高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `terminal_pane/mod.rs` 机械拆分为聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何 GPUI 输入/查找/缩放/滚动/剪贴板/网格几何行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `TerminalPane` 的公共方法签名与字段语义，不新增 `pub` 可见性泄漏。

## 用户场景

1. 开发者定位「键盘输入处理」时，直接进入 `input.rs`，无需在 1486 行文件中翻找。
2. 开发者定位「查找/搜索」逻辑时，聚焦在 `find.rs`。
3. 开发者定位「网格几何/缩放」时，聚焦在 `geometry.rs`。
4. 开发者定位「剪贴板」时，聚焦在 `clipboard.rs`。
5. 开发者定位「滚动回取/滚动」时，聚焦在 `scroll.rs`。
6. 新成员理解终端面板的职责边界，降低上手成本。

## 模块划分方案

```text
terminal_pane/
├── attachment.rs  已有（不拆分）
├── chip.rs        已有（不拆分）
├── keys.rs        已有（不拆分）
├── mod.rs         facade：类型/常量 + 生命周期/驻留协调 + 视图接口 + 状态字形/主题 + 选择 + 事件分发 + 网格重排，~616 行
├── events.rs      事件分发 handle_pane_event + 网格更新 apply_grid_updates + 重排 hold/release_reflow + 重绘 request_terminal_repaint，~166 行
├── input.rs       键盘输入 handle_key_down/handle_key_up/handle_modifiers_changed，~204 行
├── find.rs        查找 schedule_find/start_due_find/open_find/close_find/close_find_for_selected/find_next/find_previous/navigate_find，~68 行
├── geometry.rs    网格几何 grid_cell_at/grid_row_overflow/grid_inner_height + 缩放 zoom_*/change_zoom + update_selected_geometry，~167 行
├── clipboard.rs   剪贴板 copy_selection/paste，~110 行
├── scroll.rs      滚动回取 pump_scrollback_fetch + 滚动 handle_scroll，~82 行
├── policy.rs      已有（不拆分）
├── projection.rs  已有（不拆分）
├── view.rs        已有（不拆分）
└── tests.rs       已有（不拆分）
```

职责边界：
- `mod.rs` 保留：类型/枚举/常量/`actions!`/`bind_terminal_keys`；`impl TerminalPane` 中的生命周期
  （new/new_fixed/new_with_source）、驻留协调（reconcile_residency/reconcile_store_change/
  resident_buffers）、视图接口（set_shell_entities/focus/set_viewport/set_shell_chrome/is_focused）、
  状态字形与主题（sync_status_glyphs/current_colors）、选择（selected_id/selected_session）。
- `events.rs` 下沉：handle_pane_event/apply_grid_updates/hold_reflow/release_reflow_hold/
  request_terminal_repaint。
- `input.rs` 下沉：handle_key_down/handle_key_up/handle_modifiers_changed。
- `find.rs` 下沉：schedule_find/start_due_find/open_find/close_find/close_find_for_selected/
  find_next/find_previous/navigate_find。
- `geometry.rs` 下沉：grid_cell_at/grid_row_overflow/grid_inner_height/zoom_in/zoom_out/reset_zoom/
  change_zoom/update_selected_geometry。
- `clipboard.rs` 下沉：copy_selection/paste。
- `scroll.rs` 下沉：pump_scrollback_fetch/handle_scroll。

## 可见性设计

- 公共 API（`TerminalPane`、`TerminalPaneEvent`、`TerminalViewport`、`bind_terminal_keys`、
  `ChipTint`、`PaneChip`）路径不变，仍通过 `crate::terminal_pane::{...}` 可达。
- 跨模块调用的方法升为 `pub(super)`（父 `mod.rs` 与兄弟子模块互相调用）；仅模块内部调用的
  方法保持私有 `fn`。
- 所有子模块通过 `use super::*` 访问父模块私有字段/类型/方法（子模块可访问父模块私有项）。

## 影响面

- 引用方：`crate::terminal_pane::{...}` 仅依赖公共 API，零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo test -p homie-app --offline` 全绿（terminal_pane 相关测试原样通过）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，`mod.rs` 收尾 facade。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `terminal_pane/` 目录）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：无可见性泄漏到 crate 外（仅跨模块调用的方法升 `pub(super)`）。
- C6：release readiness 证据写入 `docs/verification/app-terminal-pane-mod-split/`。

## Beads

- `homie-433`

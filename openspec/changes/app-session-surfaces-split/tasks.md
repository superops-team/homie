# OpenSpec Tasks — app-session-surfaces-split

## T1 抽取投影自由函数 projection.rs

- [x] `session_surfaces.rs` → `session_surfaces/mod.rs`（目录化）。
- [x] `projection.rs`：移动 `switcher_key` / `ui_agent_kind` / `ui_status_state` /
  `status_color` / `state_badge` / `state_badge_color` / `clamp_branch`。
- [x] `switcher_key` 保持 `pub(crate)`；`mod.rs` 加 `pub(crate) use projection::switcher_key;`。
- [x] 其余自由函数保持私有；`mod.rs` 加 `mod projection;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C1。

## T2 抽取 switcher 渲染 switcher.rs

- [x] `switcher.rs`：移动 `render_switcher`。
- [x] 头：`use super::*;` + `use super::projection::{ui_agent_kind, status_color};`。
- [x] `mod.rs`：`mod switcher;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2。

## T3 抽取 overview chrome 渲染 overview.rs

- [x] `overview.rs`：移动 `render_overview` / `mode_button` / `filter_chip` /
  `overview_empty_state` / `overview_board` / `overview_list`。
- [x] 头：`use super::*;`。
- [x] `mod.rs`：`mod overview;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C3。

## T4 抽取 overview 卡片/行渲染 overview_card.rs

- [x] `overview_card.rs`：移动 `overview_card` / `overview_list_row` / `bulk_close_bar`。
- [x] 头：`use super::*;` + `use super::projection::{status_color, state_badge, state_badge_color};`。
- [x] `mod.rs`：`mod overview_card;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C4。

## T5 抽取测试 tests.rs + mod.rs 收尾 facade

- [x] `tests.rs`：移动 `#[cfg(test)] mod tests` 全部内容，头 `use super::*;`。
- [x] `mod.rs`：`#[cfg(test)] mod tests;`。
- [x] `mod.rs` 保留结构体 + 非渲染 impl + 事件路由 + impl Render +
  `render_grid_or_logo` / `status_glyph` 共享辅助。
- 验收：每个文件 < 800 行。关联 C5。

## T6 全量验证 + code review + release readiness

- [x] `cargo fmt --check`
- [x] `cargo check -p homie-app`
- [x] `cargo clippy -p homie-app --all-targets`
- [x] `cargo test -p homie-app`
- [x] code review（拆分边界清晰、无行为变更、无可见性泄漏）
- [x] release readiness 证据写入 `docs/verification/app-session-surfaces-split/`
- 验收：全部通过。关联 C6/C7。

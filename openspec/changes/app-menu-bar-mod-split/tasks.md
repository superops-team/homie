# OpenSpec Tasks — app-menu-bar-mod-split

## T1 抽取数据/样式模型 model.rs

- [x] `model.rs`：下沉 `MenuBodyRow`/`panel_fingerprint`/`menu_rows`/`active_session_count`/
  `TrailingStatus`/`trailing_status`/`agent_symbol`/`session_color`/`display_title`/`FontStyle`/
  `system_font`/`label`/`symbol_image`/`symbol_view`/`separator`/`style_footer_button`/`rect`。
- [x] 头：`use super::{POPUP_WIDTH, ...}; use homie_proto::{...}; use objc2_*::{...};`。
- [x] `MenuBodyRow`/`FontStyle`/`TrailingStatus` → `pub(super)`；`menu_rows`/`active_session_count`/
  `panel_fingerprint`/`symbol_image`/`symbol_view`/`label`/`rect`/`system_font`/`session_color`/
  `agent_symbol`/`trailing_status`/`display_title` → `pub(super)`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T2 抽取 body 行渲染 render.rs

- [x] `render.rs`：下沉 `add_project_header`/`add_session_row`/`add_more_row`/`add_empty_state`
  到 `impl super::NativeMenuBar` 扩展块。
- [x] 头：`use super::{...}; use super::model::{...}; use super::target::MenuBarTarget;`。
- [x] 四个方法 → `pub(super)`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2/C5。

## T3 抽取 objc2 事件目标类 target.rs

- [x] `target.rs`：下沉 `MenuBarTargetIvars` + `define_class! MenuBarTarget` + `impl MenuBarTarget`。
- [x] 头：`use super::{POPUP_WIDTH}; use objc2::{define_class, ...};`。
- [x] `MenuBarTarget::new`/`set_session_ids`/`show_main_window` → `pub(super)`。
- 验收：`cargo build -p homie-app --offline` 通过（验证 `define_class!` 跨模块引用）。关联 C2/C5。

## T4 mod.rs facade 收尾

- [x] `mod.rs`：保留 doc + imports + 常量 + `NativeMenuBar` 结构 + `impl`（new/update/
  set_attention/update_activity/rebuild_body）+ 模块声明 `mod render; mod model; mod target;`。
- [x] 头：`use model::{...}; use target::MenuBarTarget;`。
- [x] 字段 `_target` 类型改为 `Retained<target::MenuBarTarget>`。
- 验收：`cargo check -p homie-app` 全绿。关联 C1/C2/C3。

## T5 全量验证 + code review + release readiness

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-app --offline`
- [x] `cargo clippy -p homie-app --all-targets --offline`
- [x] `cargo build -p homie-app --offline`
- [x] code review（拆分边界清晰、无行为变更、无 `pub` 可见性泄漏、引用方零改动）
- [x] release readiness 证据写入 `docs/verification/app-menu-bar-mod-split/`
- 验收：全部通过。关联 C3/C4/C6。

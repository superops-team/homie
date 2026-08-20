# Release Readiness — app-menu-bar-mod-split

## 变更摘要

将 `homie/crates/homie-app/src/macos/menu_bar.rs`（962 行）机械拆分为 4 个聚焦子模块，
`mod.rs` 收尾 facade（400 行）。公共 API 与运行时行为零变更，引用方零改动。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 400 | doc + imports + 常量 + `NativeMenuBar` 结构 + 生命周期（new/update/set_attention/update_activity/rebuild_body）+ 模块声明 |
| `render.rs` | 196 | body 行渲染：add_project_header/add_session_row/add_more_row/add_empty_state（`impl super::NativeMenuBar` 扩展块） |
| `model.rs` | 296 | 数据/样式模型：MenuBodyRow/panel_fingerprint/menu_rows/active_session_count/TrailingStatus/trailing_status/agent_symbol/session_color/display_title/FontStyle/system_font/label/symbol_image/symbol_view/separator/style_footer_button/rect |
| `target.rs` | 123 | objc2 事件目标：MenuBarTargetIvars + `define_class! MenuBarTarget` + `impl MenuBarTarget`（new/set_session_ids/show_main_window） |

全部单文件 < 800 行。

## 可见性管控

跨模块符号均采用 `pub(super)`（仅对父模块 `menu_bar` 可见），无 `pub` 可见性泄漏到 crate 外：
- `pub(super) enum MenuBodyRow / FontStyle`、`pub(super) struct TrailingStatus`（model.rs）。
- `pub(super) fn panel_fingerprint / menu_rows / active_session_count / trailing_status /
  agent_symbol / session_color / display_title / system_font / label / symbol_image / symbol_view /
  separator / style_footer_button / rect`（model.rs）。
- `pub(super) fn add_project_header / add_session_row / add_more_row / add_empty_state`（render.rs）。
- `pub(super) struct MenuBarTarget / MenuBarTargetIvars`、`pub(super) fn new / set_session_ids /
  show_main_window`（target.rs）。

`define_class! MenuBarTarget` 在 `target.rs` 子模块内定义，父模块 `mod.rs` 通过
`use target::MenuBarTarget;` 引用，objc2 类名与选择子（`toggleHomieMenu:`/`openHomie:`/
`selectSession:`/`quitHomie:`）不变，运行时注册行为不变。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-app --offline` | ✅ 通过（验证 `define_class!` 跨模块引用链接） |
| 引用方零改动 | ✅ `git status` 仅改 `macos/menu_bar/` 目录 + 文档（`root/mod.rs` 两处引用未改） |

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`store/mod.rs`（2,434 行）、`store/tests.rs`（1,658 行）、
  `sidebar/popover.rs`（1,249 行）、`sidebar/sections.rs`（1,202 行）、`root/mod.rs`（1,190 行）、
  `surface_shell/mod.rs`（904 行）、`terminal_pane/view.rs`（863 行）、`switcher.rs`（797 行）等。

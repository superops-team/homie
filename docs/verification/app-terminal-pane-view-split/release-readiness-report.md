# Release Readiness — app-terminal-pane-view-split

## 变更摘要

将 `homie/crates/homie-app/src/terminal_pane/view.rs`（863 行）的渲染逻辑按关注点拆分为 6 个
聚焦子模块。8 个渲染方法（render_sidebar_reveal_control/render_header/render_grid_and_overlays/
render_find_bar/render_exit_pill/render_exited_takeover/render_exited_card/render_archived_overlay）
与 4 个自由辅助函数（find_icon_button/primary_button/centered_message/centered_symbol_message）
逐字迁移，可见性统一提升为 `pub(crate) fn`（crate 内部，无外部泄漏）。`impl Render for
TerminalPane` 的 `render` 入口保留在 `view/mod.rs`。生产代码语义零变更。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 124 | facade：`use super::*;` + 子模块声明 + `impl Render` |
| `buttons.rs` | 103 | 自由辅助：find_icon_button/primary_button/centered_message/centered_symbol_message |
| `chrome.rs` | 176 | 渲染：render_sidebar_reveal_control/render_header |
| `grid.rs` | 200 | 渲染：render_grid_and_overlays |
| `find_bar.rs` | 98 | 渲染：render_find_bar |
| `status.rs` | 178 | 渲染：render_exit_pill/render_exited_takeover/render_exited_card/render_archived_overlay |

全部单文件 < 800 行（最大 `grid.rs` 200 行）。

## 方法逐字迁移

- 8 个渲染方法 + 4 个自由辅助函数全部逐字迁移，渲染逻辑、交互行为零变更。
- 8 个原 `pub(super) fn` 渲染方法因拆分到 `view/` 子目录后需跨子模块调用（`mod.rs` 的 `render`
  入口），统一提升为 `pub(crate) fn`。
- 4 个自由辅助函数提升为 `pub(crate) fn`，供 `chrome.rs`/`grid.rs`/`find_bar.rs`/`status.rs` 经
  `use super::buttons::*;` 调用。
- `pub(crate)` 仍为 crate 内部可见，无外部 API 泄漏，无生产代码语义变更。
- 各子模块以 `use super::*;` 引入 `TerminalPane` 与渲染依赖。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过（0 警告） |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-app --offline` | ✅ 通过 |
| `cargo test -p homie-app --offline` | ✅ 303 passed / 0 failed（1 ignored） |
| 引用方零改动 | ✅ `git status` 仅 `view.rs` → `view/` 目录 + 文档 |

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`switcher.rs`（797 行）、`surface_shell/settings_view.rs`（765 行）、
  `notifications.rs`（762 行）等。

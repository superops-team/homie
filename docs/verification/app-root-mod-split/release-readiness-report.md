# Release Readiness — app-root-mod-split

## 变更摘要

将 `homie/crates/homie-app/src/root/mod.rs`（1,190 行）的 `impl RootView` 块按关注点拆分为 5 个
聚焦子模块。25 个方法逐字迁移，24 个私有方法提升为 `pub(crate) fn`（crate 内部，无外部泄漏），
`new` 保持 `pub(crate) fn` 不变。`RootView` 结构体、`Focusable`/`Render` trait 实现与
`cached_window_overlay` 保留在 `mod.rs`。生产代码语义零变更。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 289 | facade：结构体 + Focusable/Render + cached_window_overlay |
| `new.rs` | 390 | 构造函数：new |
| `input.rs` | 241 | 键盘输入：colors/on_key_down/on_key_up/on_modifiers_changed/open_launcher |
| `sessions.rs` | 93 | 会话派生：spawn/spawn_default/arrow_surface_visible/close_selected_session/reopen_last_session |
| `layout.rs` | 146 | 布局/缩放：window_bounds_changed/settled_sidebar_seam/begin_sidebar_slide/settled_inspector_seam/begin_inspector_slide/drag_resize/drag_terminal_resize/finish_terminal_resize/finish_resize/drag_inspector_resize/finish_inspector_resize |
| `inspector.rs` | 39 | 检查器：set_inspector_open/toggle_inspector/reveal_inspector |

全部单文件 < 800 行（最大 `new.rs` 390 行）。

## 方法逐字迁移

- 25 个方法全部逐字迁移，方法体、渲染逻辑、交互行为零变更。
- `new` 保持 `pub(crate) fn`；24 个私有 `fn` 因跨子模块调用（`render` 调用输入/缩放方法，`view.rs`
  调用 `finish_resize` 等）提升为 `pub(crate) fn`。
- `pub(crate)` 仍为 crate 内部可见，无外部 API 泄漏，无生产代码语义变更。
- 各子模块以 `use super::*;` 引入 `RootView` 与渲染依赖，`impl RootView` 跨子模块实现。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo build -p homie-app --offline` | ✅ 通过 |
| `cargo test -p homie-app --offline` | ✅ 303 passed / 0 failed（1 ignored） |
| 引用方零改动 | ✅ `git status` 仅改 `root/mod.rs` + 新增 5 个 `root/*.rs` + 文档 |

## 已知限制 / 延期

- 无。后续候选（按行数排序）：`terminal_pane/view.rs`（863 行）、`switcher.rs`（797 行）、
  `surface_shell/settings_view.rs`（765 行）等。

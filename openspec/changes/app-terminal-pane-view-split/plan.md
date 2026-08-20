# OpenSpec Plan — app-terminal-pane-view-split

## 目标

将 `homie/crates/homie-app/src/terminal_pane/view.rs`（863 行）的渲染逻辑按关注点拆分为 6 个
聚焦子模块：`buttons.rs`（4 个自由渲染辅助函数）、`chrome.rs`（header/sidebar 揭示控件，2 方法）、
`grid.rs`（grid/overlay，1 方法）、`find_bar.rs`（find bar，1 方法）、`status.rs`（exit/archived
状态，4 方法）。`mod.rs` 保留 `impl Render for TerminalPane` 的 `render` 入口 + 子模块声明。
8 个渲染方法 + 4 个自由辅助函数逐字迁移，可见性统一为 `pub(crate)`，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（facade，`render` 入口）→ 各子模块（经 `use super::*;` +
  `use super::buttons::*;` 引入 `TerminalPane` 与渲染依赖）。
- 每个子模块以 `use super::*;` + `impl TerminalPane { ... }` 或自由 `pub(crate) fn` 实现，
  方法体逐字迁移。
- 8 个原 `pub(super) fn` 渲染方法提升为 `pub(crate) fn`，以便 `mod.rs` 的 `render` 跨子模块调用。
- 4 个自由辅助函数提升为 `pub(crate) fn`，供各渲染子模块调用。
- 无生产代码语义变更，无外部 API 泄漏（`pub(crate)` 仍为 crate 内部）。

## 交付切片

- T1：编写方法边界扫描器，精确定位 8 个渲染方法 + 4 个自由辅助函数的闭合括号。
- T2：生成 `buttons.rs`/`chrome.rs`/`grid.rs`/`find_bar.rs`/`status.rs` 子模块文件。
- T3：重建 `view/mod.rs`（保留 `impl Render` + 子模块声明），编译验证。
- T4：全量验证（fmt/check/clippy/build/test）。
- T5：code review + release readiness 证据。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-terminal-pane-view-split/`。

# PRD — app-terminal-pane-view-split

## 背景

`homie/crates/homie-app/src/terminal_pane/view.rs`（863 行）是终端面板的渲染 God Module：单个文件
同时承载 `impl Render for TerminalPane` 的 `render` 入口、8 个 `render_*` 渲染方法（header、
sidebar 揭示控件、grid/overlay、find bar、exit pill/exited takeover/exited card/archived overlay），
以及 4 个自由渲染辅助函数（find_icon_button/primary_button/centered_message/centered_symbol_message）。
单文件远超 800 行阈值，阅读与变更成本极高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `view.rs` 按关注点拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何渲染逻辑的运行时行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `TerminalPane` 结构体字段、任何方法签名语义。
- 不合并或重命名方法。

## 用户场景

1. 开发者定位「header / sidebar 揭示控件」渲染时，聚焦在 `chrome.rs`。
2. 开发者定位「grid / overlay」渲染时，聚焦在 `grid.rs`。
3. 开发者定位「find bar」渲染时，聚焦在 `find_bar.rs`。
4. 开发者定位「exit / archived 状态」渲染时，聚焦在 `status.rs`。
5. 开发者定位「通用渲染辅助函数」时，聚焦在 `buttons.rs`。

## 模块划分方案

```text
terminal_pane/view/
├── mod.rs       facade：doc 注释 + `use super::*;` + 子模块声明 + `impl Render`
├── buttons.rs   自由辅助：find_icon_button/primary_button/centered_message/centered_symbol_message
├── chrome.rs    渲染：render_sidebar_reveal_control/render_header
├── grid.rs      渲染：render_grid_and_overlays
├── find_bar.rs  渲染：render_find_bar
└── status.rs    渲染：render_exit_pill/render_exited_takeover/render_exited_card/render_archived_overlay
```

## 可见性设计

- 8 个原为 `pub(super) fn` 的渲染方法（render_sidebar_reveal_control/render_header/
  render_grid_and_overlays/render_find_bar/render_exit_pill/render_exited_takeover/
  render_exited_card/render_archived_overlay），因拆分到 `view` 子目录后需跨子模块（`mod.rs` 的
  `render` 入口调用）访问，统一提升为 `pub(crate) fn`。`pub(crate)` 仍为 crate 内部可见，无外部
  API 泄漏，无生产代码语义变更。
- 4 个自由辅助函数（find_icon_button/primary_button/centered_message/centered_symbol_message）
  逐字迁移为 `pub(crate) fn`，供 `chrome.rs`/`grid.rs`/`find_bar.rs`/`status.rs` 经
  `use super::buttons::*;` 调用。
- `impl Render for TerminalPane { fn render(...) }` 保留在 `view/mod.rs`。
- 各子模块以 `use super::*;`（+ `use super::buttons::*;`）引入 `TerminalPane` 与渲染依赖。

## 影响面

- 仅 `terminal_pane/view.rs` 的渲染方法 + 自由辅助函数拆分为 6 个聚焦子模块，生产代码与其它模块
  零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo build -p homie-app --offline` 通过。
- `cargo test -p homie-app --offline` 全绿（303 passed / 0 failed）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（863 行）拆为 6 子模块 + facade。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `terminal_pane/view.rs` → `view/` 目录 +
  文档）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：8 个渲染方法 + 4 个自由辅助函数逐字迁移，可见性统一为 `pub(crate)`。
- C6：release readiness 证据写入 `docs/verification/app-terminal-pane-view-split/`。

## Beads

- `homie-2mv`

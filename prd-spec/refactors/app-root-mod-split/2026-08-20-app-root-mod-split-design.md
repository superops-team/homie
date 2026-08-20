# PRD — app-root-mod-split

## 背景

`homie/crates/homie-app/src/root/mod.rs`（1,190 行）是 RootView 工作台的 God Module：单个文件同时
承载 `RootView` 结构体、`Focusable`/`Render` trait 实现、387 行的构造函数 `new`，以及 24 个私有方法
（键盘输入、会话派生、拖拽/缩放、检查器开关等），单文件远超 800 行阈值，阅读与变更成本极高，违背
仓库「组件模块化、关注点清晰」原则。

## 目标

将 `mod.rs` 按关注点拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何渲染、输入、派生、缩放或检查器逻辑的运行时行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `RootView` 结构体字段、任何方法签名语义。
- 不合并或重命名方法。

## 用户场景

1. 开发者定位「构造函数」时，直接进入 `new.rs`。
2. 开发者定位「键盘输入处理」时，聚焦在 `input.rs`。
3. 开发者定位「会话派生」时，聚焦在 `sessions.rs`。
4. 开发者定位「拖拽/缩放/布局」时，聚焦在 `layout.rs`。
5. 开发者定位「检查器开关」时，聚焦在 `inspector.rs`。

## 模块划分方案

```text
root/
├── mod.rs             facade：结构体 + Focusable/Render trait + 子模块声明
├── new.rs             构造函数：new
├── input.rs           键盘输入：colors/on_key_down/on_key_up/on_modifiers_changed/open_launcher
├── sessions.rs        会话派生：spawn/spawn_default/arrow_surface_visible/close_selected_session/reopen_last_session
├── layout.rs          布局/缩放：window_bounds_changed/settled_sidebar_seam/begin_sidebar_slide/
│                      settled_inspector_seam/begin_inspector_slide/drag_resize/drag_terminal_resize/
│                      finish_terminal_resize/finish_resize/drag_inspector_resize/finish_inspector_resize
└── inspector.rs       检查器：set_inspector_open/toggle_inspector/reveal_inspector
```

## 可见性设计

- `new` 原为 `pub(crate) fn`，迁移后保持 `pub(crate)` 不变。
- 其余 24 个方法原为私有 `fn`，因跨子模块调用（`render` 调用输入/缩放方法，`view.rs` 调用
  `finish_resize` 等），统一提升为 `pub(crate) fn`。`pub(crate)` 仍为 crate 内部可见，无外部 API
  泄漏，无生产代码语义变更。
- 各子模块以 `use super::*;` 引入 `RootView` 与渲染依赖，`impl RootView` 跨子模块实现。
- `RootView` 结构体、`impl Focusable`、`impl Render`、`cached_window_overlay` 保留在 `mod.rs`。

## 影响面

- 仅 `root/mod.rs` 的 `impl RootView` 块拆分为 5 个聚焦子模块，生产代码与其它模块零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo build -p homie-app --offline` 通过。
- `cargo test -p homie-app --offline` 全绿（303 passed / 0 failed）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（1,190 行）拆为 5 子模块 + facade。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `root/` 目录 + 文档）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：25 个方法逐字迁移，24 个私有方法提升为 `pub(crate)`，`new` 保持 `pub(crate)`。
- C6：release readiness 证据写入 `docs/verification/app-root-mod-split/`。

## Beads

- `homie-64q`

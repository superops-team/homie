# OpenSpec Plan — app-root-mod-split

## 目标

将 `homie/crates/homie-app/src/root/mod.rs`（1,190 行）的 `impl RootView` 块按关注点拆分为 5 个
聚焦子模块：`new.rs`（构造函数，1 方法）、`input.rs`（键盘输入，5 方法）、`sessions.rs`（会话派生，
5 方法）、`layout.rs`（布局/缩放，11 方法）、`inspector.rs`（检查器，3 方法）。`mod.rs` 保留
`RootView` 结构体、`Focusable`/`Render` trait 实现与 `cached_window_overlay`。25 个方法逐字迁移，
24 个私有方法提升为 `pub(crate)`，`new` 保持 `pub(crate)`，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ 各子模块（经 `use super::*;` 引入 `RootView` 与渲染依赖）。
- 每个子模块以 `use super::*;` + `impl RootView { ... }` 实现，方法体逐字迁移。
- 24 个私有 `fn` 提升为 `pub(crate) fn`，以便跨子模块调用（`render`/`view.rs`/构造函数等）。
- `new` 保持 `pub(crate) fn` 不变。
- 无生产代码语义变更，无外部 API 泄漏（`pub(crate)` 仍为 crate 内部）。

## 交付切片

- T1：编写方法边界扫描器，精确定位 25 个方法的闭合括号（含 doc 注释、多行签名、嵌套闭包）。
- T2：生成 `new.rs`/`input.rs`/`sessions.rs`/`layout.rs`/`inspector.rs` 子模块文件。
- T3：重建 `mod.rs`（移除旧 `impl RootView` 块，新增子模块声明），编译验证。
- T4：全量验证（fmt/check/clippy/build/test）。
- T5：code review + release readiness 证据。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-root-mod-split/`。

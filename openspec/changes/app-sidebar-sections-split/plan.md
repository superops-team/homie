# OpenSpec Plan — app-sidebar-sections-split

## 目标

将 `homie/crates/homie-app/src/sidebar/sections.rs`（1,202 行）机械拆分为目录化聚焦子模块：
`chrome.rs`（顶栏与空状态，3 方法）、`project.rs`（1 方法）、`session.rs`（2 方法）、`archive.rs`
（2 方法）、`footer.rs`（2 方法），`mod.rs` 保留 facade。10 个方法全部逐字迁移，`pub(crate)` 可见性
不变，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ 各子模块（经 `use super::*;` 引入 `Sidebar` 与渲染依赖）。
- 每个子模块以 `use super::*;` + `impl Sidebar { ... }` 实现，方法体逐字迁移。
- `pub(crate) fn` 可见性保持不变，crate 内调用方经 `Sidebar` 类型访问，路径零改动。
- 无生产代码语义变更，无 `pub` 可见性泄漏。

## 交付切片

- T1：编写 Rust 括号扫描器，逐字解析 10 个方法边界（含多行签名与嵌套闭包）。
- T2：生成 `sections/mod.rs` facade 与 5 个子模块文件。
- T3：删除旧 `sections.rs`，编译验证。
- T4：全量验证（fmt/check/clippy/build/test）。
- T5：code review + release readiness 证据。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-sidebar-sections-split/`。

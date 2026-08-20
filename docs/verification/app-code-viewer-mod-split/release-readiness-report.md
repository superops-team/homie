# Release Readiness — app-code-viewer-mod-split

## 变更摘要

将 `homie/crates/homie-app/src/code_viewer.rs`（1,050 行）机械拆分为 3 个聚焦子模块，
`mod.rs` 收尾 facade（330 行）。公共 API 与运行时行为零变更，引用方零改动。

## 拆分结果

| 文件 | 行数 | 职责 |
|------|------|------|
| `mod.rs` | 330 | 文档注释 + 常量 + ViewerState + CodeViewer + 生命周期/事件/导航 + Focusable |
| `render.rs` | 429 | 工具栏/源码列表/搜索面板/空态渲染 + Render impl |
| `highlight.rs` | 248 | 词法高亮 + 关键字表 |
| `tests.rs` | 48 | 既有测试（原样下沉） |

全部单文件 < 800 行。

## 可见性管控

仅跨模块调用的函数升为 `pub(super)`，无 `pub` 可见性泄漏到 crate 外：
- `highlight.rs`：`source_row`（render.rs 调用）、`lexical_highlights`（tests.rs 调用）。

其余函数保持私有。`render.rs` 作为 `impl super::CodeViewer` 扩展块 + `impl Render for
super::CodeViewer`，通过 `use super::*` 访问父模块私有字段/方法，通过
`use super::highlight::source_row;` 引用兄弟模块入口。

## 验证证据

| 项 | 结果 |
|----|------|
| `cargo fmt --all --check` | ✅ 通过 |
| `cargo check -p homie-app --offline` | ✅ 通过 |
| `cargo clippy -p homie-app --all-targets --offline` | ✅ 0 警告 |
| `cargo test -p homie-app --offline code_viewer` | ✅ 3/3 passed |
| `cargo test -p homie-app --offline` | ✅ 301 passed（沙箱内 2 个 `daemon_launch` socket bind EPERM 失败属预期） |
| `cargo test -p homie-app --offline daemon_launch::tests`（沙箱外） | ✅ 8/8 passed |
| 引用方零改动 | ✅ `git status` 仅改 `code_viewer/` 目录 + 文档 |

## 沙箱测试说明

沙箱内 `cargo test -p homie-app` 中 2 个 `daemon_launch` 测试因 fixture daemon 的 socket bind
返回 `PermissionDenied`（EPERM）而失败，属沙箱网络限制预期。沙箱外复测 `daemon_launch::tests`
8/8 全部通过，确认拆分未影响 daemon 启动逻辑。

## 已知限制 / 延期

- 无。`store/mod.rs`（2,434 行）为后续最重拆分目标，建议留到最后。

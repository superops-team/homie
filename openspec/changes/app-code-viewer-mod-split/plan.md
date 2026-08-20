# OpenSpec Plan — app-code-viewer-mod-split

## 目标

将 `homie/crates/homie-app/src/code_viewer.rs`（1,050 行）机械拆分为目录化聚焦子模块：
`render.rs`、`highlight.rs`、`tests.rs`，`mod.rs` 保留 facade（状态模型 + 生命周期/事件/导航 +
Focusable）。公共 API 与运行时行为零变更，引用方零改动，单文件 < 800 行。

## 拆分策略

- 依赖方向：`mod.rs`（父）→ `render.rs`/`highlight.rs`（子）；`render.rs` → `highlight.rs`
  （`source_row` 跨模块调用）；`tests.rs` → `highlight.rs`（`lexical_highlights` 跨模块调用）。
- 渲染方法 + `Render` impl 下沉 `render.rs`（`impl super::CodeViewer` + `impl Render for
  super::CodeViewer`），使渲染职责内聚；子模块通过 `use super::*` 访问父模块私有字段/方法。
- 词法高亮纯函数 + 关键字表下沉 `highlight.rs`。
- 既有测试原样下沉 `tests.rs`。

## 交付切片

- T1：抽取词法高亮 → `highlight.rs`。
- T2：抽取渲染方法 + Render impl → `render.rs`。
- T3：测试原样下沉 → `tests.rs`。
- T4：全量验证 + code review + release readiness。

## 验证

见 `tasks.md`，最终证据写入 `docs/verification/app-code-viewer-mod-split/`。

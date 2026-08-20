# PRD — app-code-viewer-mod-split

## 背景

`homie/crates/homie-app/src/code_viewer.rs`（1,050 行）是只读代码查看器的 God Module：一个文件
同时承载查看器状态模型、生命周期/事件/导航、四个渲染方法、词法高亮纯函数、关键字表、以及测试
六个职责，单文件超过 800 行阈值，阅读与变更成本高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `code_viewer.rs` 机械拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何高亮语义、查看器状态机、搜索/导航行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `CodeViewer`/`ViewerState` 公共类型与 `new`/`set_workspace`/`open_reference` 等公共
  方法签名。

## 用户场景

1. 开发者定位「词法高亮」（注释/字符串/关键字/标识符边界）时，直接进入 `highlight.rs`。
2. 开发者定位「工具栏/源码列表/搜索面板/空态渲染」时，直接进入 `render.rs`。
3. 开发者定位「状态模型与事件/导航」时，聚焦在 `mod.rs`。
4. 新成员理解查看器职责边界，降低上手成本。

## 模块划分方案

```text
code_viewer/
├── mod.rs       facade：文档注释 + 常量 + ViewerState + CodeViewer + 生命周期/事件/导航 + Focusable
├── render.rs    渲染方法（render_toolbar/render_source/render_picker/render_message）+ Render impl
├── highlight.rs 词法高亮（source_row/highlighted_source/lexical_highlights/is_ident + 关键字表）
└── tests.rs     既有测试（原样下沉，#[cfg(test)]）
```

职责边界：
- `mod.rs` 保留：`ViewerState`、`CodeViewer` 结构体、常量 `SOURCE_ROW_HEIGHT`/`SOURCE_GUTTER_WIDTH`；
  `impl CodeViewer` 的 `new`/`set_workspace`/`open_reference`/`open_reference_inner`/`navigate`/
  `handle_key_down`/`toggle_picker`/`schedule_search`/`open_highlighted`；`impl Focusable`。
- `render.rs` 下沉：`render_toolbar`/`render_source`/`render_picker`/`render_message`（作为
  `impl super::CodeViewer` 扩展块）与 `impl Render for super::CodeViewer`。
- `highlight.rs` 下沉：`source_row`/`highlighted_source`/`lexical_highlights`/`is_ident` 与
  `RUST_KEYWORDS`/`SWIFT_KEYWORDS`/`PYTHON_KEYWORDS`/`JS_KEYWORDS`/`COMMON_KEYWORDS` 常量表。

## 可见性设计

- 公共 API（`CodeViewer`、`ViewerState`、`new`/`set_workspace`/`open_reference`）路径不变，
  仍通过 `crate::code_viewer::{...}` 可达。
- `source_row` 升 `pub(super)`（`render.rs` 跨模块调用）；`lexical_highlights` 升 `pub(super)`
  （`tests.rs` 跨模块调用）；其余函数仅模块内部调用，保持私有。
- `render.rs` 通过 `use super::*` + `use super::highlight::source_row;` 访问父模块与兄弟模块；
  `highlight.rs`/`tests.rs` 通过 `use super::*` 访问父模块私有项；`tests.rs` 额外
  `use super::highlight::lexical_highlights;`。

## 影响面

- 引用方：仅依赖 `crate::code_viewer::CodeViewer` 等公共 API，零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo test -p homie-app --offline` 全绿（code_viewer 相关测试原样通过）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件删除。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `code_viewer/` 目录）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：无可见性泄漏到 crate 外（仅 `source_row`/`lexical_highlights` 升 `pub(super)`）。
- C6：release readiness 证据写入 `docs/verification/app-code-viewer-mod-split/`。

## Beads

- `homie-bm9`

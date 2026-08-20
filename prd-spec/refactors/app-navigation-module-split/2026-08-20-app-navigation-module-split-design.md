# PRD — app-navigation-module-split

## 背景

`homie/crates/homie-app/src/navigation.rs`（1,376 行）是 GPUI 导航覆盖层（命令面板 + Quick Open）
的 God Module：一个 `impl NavigationOverlay` 块高达 923 行，叠加生命周期、目录索引、排序、
命令执行、渲染五个职责，加上一组渲染辅助自由函数与测试。单文件超过 800 行阈值，阅读与
变更成本高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `navigation.rs` 机械拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何 GPUI 渲染行为、排序逻辑、命令分发语义、索引/扫描行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `NavigationOverlay` 的公共方法签名与字段语义。

## 用户场景

1. 开发者定位「快速打开」渲染逻辑时，直接进入 `render.rs`，无需在 1300+ 行文件中翻找。
2. 开发者定位目录索引/扫描逻辑时，聚焦在 `mod.rs` 的索引方法区。
3. 新成员理解导航覆盖层的职责边界，降低上手成本。

## 模块划分方案

```text
navigation/
├── mod.rs       facade：类型/常量 + 生命周期/排序/命令执行 + 焦点/渲染 trait + 共享查询字段工具，~646 行
├── index.rs     目录索引/扫描 + Quick Open 快照构建，~126 行
├── render.rs    覆盖层行/区块/浮层渲染 + 渲染辅助自由函数，~519 行
└── tests.rs     既有测试（原样下沉，#[cfg(test)]），~109 行
```

职责边界：
- `mod.rs` 保留：`OverlayLayout`/`Overlay`/`CommandSelection`/`NavigationEvent`/`NavigationOverlay`
  类型与布局常量；`impl NavigationOverlay` 中的生命周期（new/is_open/open/close/reset）、排序
  （schedule_rank/on_key_down/edit_query/query_changed/move_highlight/scroll_to_highlight/
  highlight_child/visible_count）、命令执行（run_highlighted/run_command_selection/
  run_palette_command/refresh_command_items/current_quick_item）；`EventEmitter`/`Focusable`/`Render`
  trait 实现；`row_child_index`/`CARET`/`query_label`/`highlighted_label`/`highlighted_label_styled`。
- `index.rs` 下沉：目录索引/扫描方法（index_roots/load_cached_index/refresh_directory_index/
  snapshot_inputs/project_roots）。
- `render.rs` 下沉：`impl NavigationOverlay` 中的渲染方法（render_overlay/render_search/
  render_command_palette/render_action_row/render_session_row/render_quick_open/render_quick_row）
  与渲染辅助自由函数（section_header/empty_label/palette_row/chip/attention_color/kind_label/
  relative_parent）。

## 可见性设计

- 公共 API（`query_label`、`CARET`、`NavigationOverlay`、`NavigationEvent`、`ToggleCommandPalette`、
  `ToggleQuickOpen`）路径不变，仍通过 `crate::navigation::{...}` 可达。
- `render_overlay`/`load_cached_index`/`refresh_directory_index` 升为 `pub(super)`（`Render` trait
  与 `mod.rs` 的生命周期/命令方法跨模块调用）；其余子模块方法仅模块内部调用，保持私有 `fn`。
- `relative_parent` 升为 `pub(super)`（`tests.rs` 跨模块断言）。
- `render.rs` 通过 `use super::*` 访问父模块私有字段/方法（子模块可访问父模块私有项）。

## 影响面

- 引用方：`terminal_pane/mod.rs`、`code_viewer.rs`、`inspector/review.rs`、`inspector/ask.rs`、
  `sidebar/*`、`surface_shell/*`、`launcher.rs` 仅依赖公共 API，零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo test -p homie-app --offline` 全绿（navigation 相关测试原样通过）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件删除。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅删旧文件+新增目录）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：无可见性泄漏到 crate 外（仅 `render_overlay` 升 `pub(super)`）。
- C6：release readiness 证据写入 `docs/verification/app-navigation-module-split/`。

## Beads

- `homie-itl`

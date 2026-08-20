# PRD — app-launcher-mod-split

## 背景

`homie/crates/homie-app/src/launcher.rs`（1,046 行）是 Command-N 新会话启动面板的 God Module：
一个文件同时承载布局常量、状态模型（`LauncherOverlay`/`Picker`/`HarnessChoice`）、生命周期/事件/
提交逻辑、四个渲染方法、三个纯函数辅助、以及测试六个职责，单文件超过 800 行阈值，阅读与变更
成本高，违背仓库「组件模块化、关注点清晰」原则。

## 目标

将 `launcher.rs` 机械拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何启动面板交互、提交逻辑、pick 选择、composer 排版行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `LauncherOverlay`/`LauncherEvent`/`Picker` 类型与 `new`/`open`/`is_open`/`focus`/
  `handle_key_down` 等公共方法签名。

## 用户场景

1. 开发者定位「harness/project picker 与面板/composer 渲染」时，直接进入 `render.rs`。
2. 开发者定位「状态模型、生命周期、提交、pick 选择、composer 编辑」时，聚焦在 `mod.rs`。
3. 开发者定位「纯函数辅助（初始目标/agent 映射/title case）」时，聚焦在 `mod.rs` 尾部的 free
   function 区。
4. 新成员理解启动面板职责边界，降低上手成本。

## 模块划分方案

```text
launcher/
├── mod.rs     facade：布局常量 + 状态模型 + 生命周期/事件/提交/编辑 + Focusable + 纯函数辅助
├── render.rs  渲染方法（render_harness_picker/render_project_picker/render_panel/floating）+ Render impl
└── tests.rs   既有测试（原样下沉，#[cfg(test)]）
```

职责边界：
- `mod.rs` 保留：布局常量（`PANEL_WIDTH`…`COMPOSER_CONTROLS_HEIGHT`）、`composer_text_height`、
  `HarnessChoice`/`LauncherEvent`/`LauncherOverlay`/`Picker` 类型；`impl LauncherOverlay` 的
  `new`/`is_open`/`open`/`focus`/`close`/`harness_choices`/`projects`/`selected_harness_label`/
  `selected_project_label`/`blocker`/`can_submit`/`submit`/`handle_key_down`/`handle_picker_key`/
  `commit_highlight`/`toggle_picker`/`cycle_harness`/`edit_prompt`/`choose_folder`；
  `impl Focusable`；`initial_target`/`ui_agent_kind`/`title_case_id`。
- `render.rs` 下沉：`render_harness_picker`/`render_project_picker`/`render_panel`/`floating`
  （作为 `impl super::LauncherOverlay` 扩展块）与 `impl Render for super::LauncherOverlay`。

## 可见性设计

- 公共 API（`LauncherOverlay`、`LauncherEvent`、`new`/`open`/`is_open`/`focus`/`handle_key_down`）
  路径不变，仍通过 `crate::launcher::{...}` 可达。
- 渲染方法之间同模块互调，无需跨模块可见性提升；`render.rs` 通过 `use super::*` 访问父模块
  私有字段/方法（`can_submit`/`blocker`/`selected_harness_label`/`choose_folder`/`edit_prompt` 等）
  与私有 free function（`ui_agent_kind`/`composer_text_height`）。
- 无任何 `pub(super)` 新增，无可见性泄漏到 crate 外。

## 影响面

- 引用方：仅依赖 `crate::launcher::LauncherOverlay` 等公共 API，零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo test -p homie-app --offline` 全绿（launcher 相关测试原样通过）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件删除。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `launcher/` 目录）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：无可见性泄漏到 crate 外（渲染方法同模块内聚，零 `pub(super)` 新增）。
- C6：release readiness 证据写入 `docs/verification/app-launcher-mod-split/`。

## Beads

- `homie-0fg`

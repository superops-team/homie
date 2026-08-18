# OpenSpec Tasks — app-root-store-deepening

## S1 抽取 root/shortcuts.rs 纯策略

- [x] `shortcuts.rs`：移动 `NewSessionShortcut` 枚举、`new_session_shortcut`、`session_navigation_delta`，`pub(crate)`。
- [x] `mod.rs`：`mod shortcuts;` + `use shortcuts::{...};`。
- 验收：`cargo check -p homie-app` 全绿。关联 C1。

## S2 抽取 root/seams.rs seam 动画 + 拖拽边缘 marker

- [x] `seams.rs`：移动 `DraggedSidebarEdge`/`DraggedTerminalEdge`/`DraggedInspectorEdge` marker + `advance_seam` 纯函数，`pub(crate)`。
- [x] `mod.rs`：`mod seams;` + `use seams::{...};`。
- 验收：`cargo check -p homie-app` 全绿。关联 C2。

## S3 抽取 root/auxiliary.rs auxiliary terminal 编排

- [x] `auxiliary.rs`：移动 `impl RootView` 的 `open_auxiliary_terminal`/`sync_auxiliary_terminal`，`pub(crate) fn`。
- [x] `mod.rs`：`mod auxiliary;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C3。

## S4 抽取 root/view.rs render 方法 + 渲染辅助

- [x] `view.rs`：移动 `impl RootView` render 方法 `resize_handle`/`terminal_resize_handle`/`inspector_resize_handle`/`resize_shield`/`terminal_card`/`preview_workbench`/`close_confirmation`/`status_banner` + 游离渲染辅助 `preview_control`/`preview_hint`，`pub(crate) fn`。
- [x] `mod.rs`：`mod view;`。
- 验收：`cargo check -p homie-app` 全绿。关联 C4。

## S5 抽取 root/tests.rs + facade 收尾

- [x] `tests.rs`：移动旧内联 `command_modifiers` 辅助 + 2 个测试（`command_t_launches_the_configured_default_agent`/`session_navigation_requires_command_option_arrows`），`use super::*;`。
- [x] `mod.rs`：删除内联 `#[cfg(test)] mod tests`，加 `#[cfg(test)] mod tests;`；`RootView` 字段统一 `pub(crate)` 供子模块访问。
- [x] 全量验证：`cargo check` + `cargo fmt --check` + `cargo test`。
- 验收：全部通过。关联 C5。

## S6 评估 store session/project 投影单点化（F8）

- [x] 核查 `store/projection.rs`（`SidebarProjection`/`SidebarProject`/`SidebarRow`/`build_projection`/`build_tree`）与 engine `registry.rs::session_project_id` 的关系。
- [x] 结论：engine `session_project_id` 负责稳定项目身份（FNV-1a 哈希 → `ProjectId`），app `store/projection.rs` 负责 UI 呈现投影；两者职责正交，无重复投影需消除。
- 验收：记录评估结论于验证报告。关联 C6。

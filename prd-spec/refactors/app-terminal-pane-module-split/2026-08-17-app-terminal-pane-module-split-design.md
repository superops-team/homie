# App TerminalPane 模块拆分设计文档

## 1. 背景与动机

### 1.1 现状

`homie/crates/homie-app/src/terminal_pane.rs` 约 3,495 行，是 `homie-app` 中最大的单文件。它同时承担了：

- GPUI 渲染（`impl Render` + 7 个 `render_*` 方法 + 4 个自由渲染辅助函数）
- Terminal 事件分发（`handle_pane_event`、`apply_grid_updates`、resize/reflow 调度）
- 输入/键位适配（`handle_key_down/up`、`terminal_key_event`、`bind_terminal_keys`）
- 剪贴板暂存（`clipboard_image`、`paste`）
- 状态 chip 投影（`ChipTint`、`PaneChip` + 5 个方法）
- attachment 生命周期（`AttachmentState/Command/Control`、`spawn_attachment`）
- 纯决策/投影辅助（resize debounce、reflow hold、URL/PR/退出文案投影）
- 24 个测试

该文件对应 2026-08 架构审计 finding **F6（Critical）**：单文件职责过载，纯逻辑与 GPUI 渲染耦合，无法独立单测，维护/评审成本高。

### 1.2 治理依据

本拆分是 `homie-ubu`（Homie 架构治理总纲，2026-08 审计）下 `homie-ubu.4` 子任务的落地，遵循已在前序 `app-inspector-module-split` 中验证通过的拆分模式：纯逻辑子域（投影 / 策略 / 状态）与 GPUI 渲染分离，facade 保留公共 API 与事件分发。

## 2. 目标

- 将纯逻辑子域（键位映射、resize debounce、reflow hold、剪贴板暂存、status chip 投影、URL/PR/退出文案投影、attachment 生命周期）抽取为可独立单测的 `pub(crate)` 模块，**无 `Window`/`Context`/`Entity`/渲染依赖**。
- 渲染逻辑（`impl Render` + `render_*` + 渲染辅助函数）收敛到 `view.rs`。
- `mod.rs` 作为 facade，保留结构/枚举定义、公共 API（`TerminalPane`、`TerminalPaneEvent`、`TerminalViewport`、`bind_terminal_keys`、`CopySelection`/`Paste`）与事件分发。
- 保持视觉与行为完全不变（纯机械重构）。

## 3. 非目标

- 不重做 terminal 渲染/视觉设计。
- 不改变 GPUI 层级、事件流或 PTY 协议行为。
- 不迁移全局 store、不改动 `homie-term`/`homie-client`/`homie-proto`。
- 不新增功能、不调整任何公开签名。

## 4. 目标模块拓扑

```text
homie/crates/homie-app/src/terminal_pane/
├── mod.rs          # facade：常量、actions!、TerminalPaneEvent、bind_terminal_keys、
│                   #        TerminalViewport、TerminalPane 结构、PaneEvent/ResidentTerminal/
│                   #        SessionSource/ReflowHold、impl EventEmitter、impl TerminalPane（事件分发）
├── chip.rs         # ChipTint、PaneChip + impl + URL/PR 解析辅助（pr_number/linear_key/
│                   # url_host/url_port/pr_tint/pr_help/comments_help）
├── attachment.rs   # AttachmentState、AttachmentCommand、AttachmentControl、spawn_attachment、wait_for_retry
├── projection.rs   # ui_agent_kind、status_state、exit_description
├── policy.rs       # terminal_damage_should_repaint、ResizePlan、plan_resize、should_hold_reflow、
│                   # estimated_grid_size、clipboard_image
├── keys.rs         # terminal_key_event
├── view.rs         # impl Render + render_* 方法 + find_icon_button/primary_button/
│                   # centered_message/centered_symbol_message
└── tests.rs        # 原 24 个测试原样迁移 + sorted_checks（测试专属辅助）
```

## 5. 公共 API 兼容性约束

以下公开名字必须保持可达，签名不变：

- `root.rs`：`use crate::terminal_pane::{TerminalPane, TerminalPaneEvent, TerminalViewport};`
- `main.rs`：`use terminal_pane::bind_terminal_keys;` 及 `terminal_pane::CopySelection`、`terminal_pane::Paste`（actions 宏生成的 action 类型）。

## 6. 实施切片（每片 `cargo test -p homie-app` 全绿）

- **S1 纯函数抽取**：`keys.rs`（`terminal_key_event`）、`policy.rs`（`ResizePlan`/`plan_resize`/`should_hold_reflow`/`terminal_damage_should_repaint`/`estimated_grid_size`/`clipboard_image`）、`projection.rs`（`ui_agent_kind`/`status_state`/`exit_description`）。URL/PR 解析辅助（`pr_number`/`linear_key`/`url_host`/`url_port`/`pr_tint`/`pr_help`/`comments_help`）因仅被 chip 使用，随 S2 收拢到 `chip.rs`；`sorted_checks` 为测试专属，随 S5 收拢到 `tests.rs`。
- **S2 chip 抽取**：`ChipTint`、`PaneChip` + 5 个方法 → `chip.rs`。
- **S3 attachment 抽取**：`AttachmentState`/`AttachmentCommand`/`AttachmentControl` + `spawn_attachment`/`wait_for_retry` → `attachment.rs`。
- **S4 渲染抽取**：`impl Render` + 7 个 `render_*` 方法 + 4 个渲染辅助函数 → `view.rs`。
- **S5 测试迁移**：24 个测试 → `tests.rs`，`mod.rs` 加 `#[cfg(test)] mod tests;`。

## 7. 验收标准

- `terminal_pane/` 下每个子模块职责单一；`view.rs` < 800 行；`mod.rs` 收敛为 facade。
- `projection.rs`/`policy.rs`/`keys.rs` 无 GPUI 渲染依赖（可纯单测）。
- `cargo check -p homie-app` 零警告；`cargo fmt --check` 干净。
- `cargo test -p homie-app` 全绿（基线 303 passed / 0 failed / 1 ignored，行为不变）。
- 公共 API 名（`TerminalPane`、`TerminalPaneEvent`、`TerminalViewport`、`bind_terminal_keys`、`CopySelection`、`Paste`）签名与可达性不变。

## 8. 测试计划

- 现有 24 个单测/`#[gpui::test]` 原样迁移，作为行为等价回归门禁。
- 新增无（纯机械重构，不改变行为；若 S 切分暴露纯函数可测性，可补纯函数单测但不强制）。

## 9. Beads 追踪

- change_id `app-terminal-pane-module-split`
- Beads：`homie-ubu.4`（IN_PROGRESS），parent `homie-ubu`
- 证据目录：`docs/verification/app-terminal-pane-module-split/`
- 版本 tag：`v0.1.11`（internal refactor patch）

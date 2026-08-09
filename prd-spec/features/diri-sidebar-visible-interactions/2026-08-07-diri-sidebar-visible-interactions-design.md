# Diri Sidebar Visible Interactions 设计文档

```yaml
change_id: diri-sidebar-visible-interactions
beads: homie-5r1
target_rows:
  - UI-002
```

## 1. 背景

`diri-sidebar-session-model` 已在 `homie-ui` 中提供 `SidebarSessionModel`，覆盖 select、multi-select、rename、pin、archive 和 reorder helper。但 `homie-app` 当前侧边栏仍主要是点击选择 session，用户可见界面没有 pin/archive/multi-select 控件。

这使 `UI-002` 继续停留在模型层 partial：功能存在于库中，但没有进入 Diri 风格侧边栏工作流。

## 2. 目标

- 在 `homie-app` 中维护 sidebar interaction state。
- 侧边栏 session row 显示 pin、archive、多选控件。
- 点击 pin/archive/multi-select 后更新 app 内 sidebar state，并保持 selected session 逻辑一致。
- 源码回归测试禁止这些能力退化为隐藏模型或 notice-only 文案。
- 本阶段保持 `UI-002` 为 `partial`，直到 hover card、drag UI 和真实鼠标 E2E 完成。

## 3. 非目标

- 不从 storage/runtime 删除 session。
- 不实现拖拽排序的 GPUI pointer E2E。
- 不实现 hover card 截图。
- 不把 `UI-002` 标为 implemented。

## 4. 用户场景

### 场景 1: 用户可以在侧边栏 pin session

**Given** Homie app 侧边栏显示多个 live sessions  
**When** 用户点击 session row 的 pin 控件  
**Then** 该 row 标记为 pinned，并排序到前面。

### 场景 2: 用户可以在侧边栏 archive session

**Given** 当前 session 被选中  
**When** 用户点击 archive 控件  
**Then** 该 row 从可见 active rows 中移除，选中态清空或转移。

### 场景 3: 用户可以多选 session

**Given** 侧边栏显示多个 sessions  
**When** 用户点击 multi-select 控件  
**Then** app 内 multi-select 计数更新，并在 row 上显示 selected marker。

## 5. 需求

| ID | 需求 | 优先级 |
|----|------|--------|
| FR-1 | `AppState` 必须包含 sidebar model/state，而不是每次只从 raw session list 直接渲染。 | P0 |
| FR-2 | `refresh_sessions_from_client` 必须同步 sidebar model rows，并保留 pin/archive/multi-select 状态。 | P0 |
| FR-3 | session row 必须提供可见 pin、archive、multi-select 控件。 | P0 |
| FR-4 | archive 不得误杀 runtime session；本阶段只隐藏 app sidebar row 并更新状态。 | P0 |
| FR-5 | 回归测试必须覆盖 app 源码里真实 helper 和控件 wiring。 | P1 |

## 6. 影响文件

- `crates/homie-app/src/main.rs`
- `crates/homie-app/tests/app_shell_copy_regression.rs`
- `docs/verification/diri-sidebar-visible-interactions/`
- `openspec/changes/diri-sidebar-visible-interactions/`

## 7. 功能验证

- `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture`
- `cargo test -p homie-ui --test workbench_state -- --nocapture`
- `cargo check -p homie-app`
- `cargo clippy -p homie-app --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

## 8. 验收标准

- app 源码包含 sidebar model refresh/helper，而不是只渲染 raw session rows。
- app 源码包含 visible pin/archive/multi-select controls，并由 click handler 更新 state。
- `cargo test -p homie-app --test app_shell_copy_regression` 通过。
- `make parity-lock` 保持 `UI-002` partial，不误报完成。

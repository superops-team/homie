# Diri 工作台运行时接线设计文档

```yaml
change_id: diri-workbench-runtime-wiring
beads: homie-3tz
parent_bead: homie-h7n.4
source_lock: docs/research/diri-parity-lock.md
target_rows:
  - UI-001
  - UI-003
```

## 1. 概述

### 1.1 问题/背景

`homie-app` 已经显示 Diri 风格的 sidebar + terminal + inspector，并在启动时通过 `HomieClient` 创建一个 live shell。但仍存在未闭合的 runtime UI 行为：

- command palette 的 `SpawnShell` 只修改本地 notice，没有创建真实 runtime session；
- sidebar 只显示当前 session，不展示 runtime session list，也不能选择其它 session；
- app 没有把 terminal pane 尺寸变化回写给 runtime holder；
- UI 回归测试仍主要检查源码字符串，不能证明 workbench action 走真实 client API。

这些缺口会让 app 仍然接近“静态工作台”，不满足用户要求的 Diri 功能复刻方向。

### 1.2 目标

- app 启动后维护 runtime-backed session projection。
- command palette `SpawnShell` 必须调用 `HomieClient::spawn_shell` 并选择新 session。
- sidebar 必须渲染 `HomieClient::list_sessions` 返回的真实 session rows。
- 选择 session 后，terminal refresh 必须 attach/read 选中 session snapshot。
- terminal pane resize 必须通过 `HomieClient::resize_session` 回写 runtime，避免只在 UI 本地改变。
- 增加可执行回归测试，证明 app 不再只改本地 notice。

## 2. 用户场景

### 场景 1: 从 command palette 新建真实 session

**Given** 用户在 Homie workbench 打开 command palette  
**When** 执行 Spawn Shell  
**Then** Homie 创建新的 holder-backed runtime session，sidebar session count 更新，terminal attach 到新 session。

### 场景 2: 在 sidebar 选择已有 session

**Given** runtime 中有多个 session  
**When** 用户点击 sidebar 的 session row  
**Then** 选中 session id 更新，terminal refresh 使用该 session 的 snapshot，inspector 展示该 session 状态。

### 场景 3: terminal pane resize

**Given** terminal pane 尺寸发生变化  
**When** app 计算出新的 cols/rows  
**Then** app 调用 `HomieClient::resize_session(session_id, cols, rows)`，runtime holder stat 返回新的几何尺寸。

## 3. 功能需求

### FR-1: Runtime session projection

`AppState` 必须包含 session rows 和 selected session，来源为 `HomieClient::list_sessions`，而不是静态单行。

### FR-2: SpawnShell 真实执行

`PaletteCommand::SpawnShell` 必须调用真实 `HomieClient::spawn_shell`，并在成功后选择新 session、刷新 terminal output。

### FR-3: Sidebar selection

sidebar session row 必须有 click handler，选择真实 session id，并触发 snapshot refresh。

### FR-4: Runtime resize

terminal pane resize 或显式 resize helper 必须调用 `HomieClient::resize_session`。重复尺寸不应刷屏式调用。

### FR-5: 测试门禁

必须新增 app 回归测试，检查源码中存在真实 action wiring，并禁止 `SpawnShell => self.state.terminal_notice = "spawned"` 这类本地-only 实现。

## 4. 实现方案

- 新增 `SessionRow` 和 `TerminalGeometry`。
- 把 session refresh 提取为 `refresh_sessions_from_client`。
- 把 spawn/select/resize 提取为独立 helper，便于测试用源码约束和后续 UI/E2E 调用。
- sidebar render 遍历 `state.sessions`，每行绑定 `on_mouse_down`。
- `SpawnShell` palette branch 调用 `spawn_runtime_shell`。
- 在 render terminal pane 前调用保守的 `sync_terminal_geometry(120, 40)`；后续 GPUI 精确尺寸计算可替换为元素 bounds 驱动，但当前必须先打通 runtime resize 路径。

## 5. 边界情况

| 场景 | 处理方式 |
|------|---------|
| client unavailable | 保持 degraded notice，不暴露假成功 |
| spawn 失败 | notice 显示 safe error，不新增 session row |
| selected session 被删除/不可读 | 回退到第一条 session 或 no session |
| resize 无 selected session | no-op |
| resize 尺寸未变化 | no-op |

## 6. 涉及文件

- `crates/homie-app/src/main.rs`
- `crates/homie-app/tests/app_shell_copy_regression.rs`
- `specs/desktop-shell/README.md`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-workbench-runtime-wiring/*`
- `openspec/changes/diri-workbench-runtime-wiring/*`

## 7. 测试计划

- `cargo test -p homie-app --tests -- --nocapture`
- `cargo test -p homie-client --tests -- --nocapture`
- `cargo clippy -p homie-app --all-targets -- -D warnings`
- `make parity-lock`

## 8. 验收标准

- `SpawnShell` 不再是本地 notice placeholder。
- sidebar 使用真实 session list 并支持选择。
- terminal refresh 绑定 selected session snapshot。
- app 存在 runtime resize 调用路径。
- `UI-001`/`UI-003` 证据更新，但仍保持 `partial`，直到完整 GPUI interaction screenshot/E2E 完成。


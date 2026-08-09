# Diri 通知中心对齐设计文档

```yaml
change_id: diri-notification-center
beads: homie-5pe
parent_bead: homie-h7n.4
target_rows:
  - UI-008
```

## 1. 概述

### 1.1 问题/背景

`UI-008` 仍为 `missing`。Homie 当前只有 CLI `hook/notify` JSON 输出，没有 app 内通知中心、状态 rollup、quick approve/deny 模型，也没有 macOS native notification 命令构建。Diri 的用户体验里，通知和状态汇总是工作台判断 agent 是否需要关注的核心入口。

### 1.2 目标

- 在 `homie-ui` 中新增通知中心模型。
- 将 session 状态、needs-input 详情、agent approve/deny 能力汇总成 notification items。
- 提供 quick approve/deny action model，不直接执行 keystroke，避免越过 runtime 权限边界。
- 提供 macOS notification command builder，生成可测试、脱敏的 `osascript` 参数。
- 在 `homie-app` inspector 中显示 notification rollup，避免 UI-008 继续为空。

## 2. 功能需求

### FR-1: Notification model

通知条目必须包含 severity、title、body、session id、status、quick actions 和 redacted native body。

### FR-2: Status rollup

通知中心必须根据 session rows 汇总 blocked/running/exited/total 数量，并提供 badge 文案。

### FR-3: Quick actions

当 agent manifest 提供 approve/deny 能力且 needs-input 是 approval 时，通知条目必须暴露 approve/deny action descriptor；未知 agent 不暴露 quick action。

### FR-4: macOS native notification builder

必须生成安全的 `/usr/bin/osascript -e display notification ...` 参数，不包含 raw token、authorization、cookie。

## 3. 非目标

- 不在本轮执行真实 approve/deny keystroke。
- 不实现菜单栏常驻 agent。
- 不把 `UI-008` 标为 implemented，直到 native notification E2E 和真实 action 验证完成。

## 4. 验收标准

- `cargo test -p homie-ui --tests -- --nocapture` 覆盖 rollup、quick actions、macOS command escaping。
- `cargo test -p homie-app --tests -- --nocapture` 覆盖 app 渲染 notification rollup。
- `cargo clippy -p homie-ui -p homie-app --all-targets -- -D warnings` 通过。
- parity lock 将 `UI-008` 从 `missing` 更新为 `partial`。


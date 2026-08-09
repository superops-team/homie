# Diri Worktree Sheet 对齐设计文档

```yaml
change_id: diri-worktree-sheet
beads: homie-hm1
target_rows:
  - UI-007
```

## 1. 概述

`UI-007` 仍缺少 app 内 worktree sheet。Runtime 已有 `WorktreeSheet` cleanup suggestion 模型，但未接入 app。目标是新增 worktree sheet surface，基于当前 session workspaces 生成 worktree overview，并显示 cleanup suggestion 状态。

## 2. 目标

- `HomieClient` 提供 worktree overview projection。
- `homie-app` 增加 Worktrees surface。
- command palette ToggleSidebar 暂改为打开 worktree sheet，避免 notice-only。
- 证据更新但保持 `UI-007` partial，直到 create/remove/cleanup E2E 完成。

## 3. 验收

- `cargo test -p homie-runtime --test worktree_safety -- --nocapture`
- `cargo test -p homie-client --tests -- --nocapture`
- `cargo test -p homie-app --tests -- --nocapture`
- `cargo clippy -p homie-client -p homie-app --all-targets -- -D warnings`


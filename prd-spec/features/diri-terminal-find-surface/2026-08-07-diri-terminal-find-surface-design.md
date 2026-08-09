# Diri Terminal Find Surface 对齐设计文档

```yaml
change_id: diri-terminal-find-surface
beads: homie-42v
target_rows:
  - UI-003
  - TERM-004
```

## 1. 概述

`homie-term` 已有 find/selection/key/paste 模型，但 `homie-app` 没有可见 find surface。目标是新增 app find bar 状态和可见 UI，接入 `TerminalFindModel` 的查询状态，避免 find 只停留在库层。

## 2. 验收

- `cargo test -p homie-term --test grid_input_find -- --nocapture`
- `cargo test -p homie-app --tests -- --nocapture`
- `cargo clippy -p homie-app --all-targets -- -D warnings`


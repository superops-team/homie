# Diri Sidebar 会话模型对齐设计文档

```yaml
change_id: diri-sidebar-session-model
beads: homie-f02
target_rows:
  - UI-002
```

## 1. 概述

`UI-002` 仍缺少 Diri sidebar 的核心状态模型：session status glyph、multi-select、rename、pin/archive、drag reorder。当前 Homie app 只渲染 session rows，缺少可测试模型。

## 2. 目标

- 在 `homie-ui` 增加 `SidebarSessionModel`。
- 支持 select/toggle multi-select、rename、pin/archive、move before/end。
- 提供 status glyph 名称映射。
- 用单元测试覆盖行为。

## 3. 非目标

- 不实现完整 hover card 截图。
- 不把 `UI-002` 标为 implemented，直到 screenshot/manual E2E 完成。

## 4. 验收标准

- `cargo test -p homie-ui --tests -- --nocapture`
- `cargo clippy -p homie-ui --all-targets -- -D warnings`
- `make parity-lock`


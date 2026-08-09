# Diri MCP wait_for_children 对齐设计文档

```yaml
change_id: diri-mcp-wait-children
beads: homie-ne8
target_rows:
  - API-005
  - API-004
feature_atoms:
  - M13-F002
```

## 1. 概述

Homie 已有 MCP direct parent/child linkage 和 `list_children`，但 `wait_for_children` 仍未实现。Diri 的该工具等待 caller 的 child sessions 到达 settled/done/exited 状态。

## 2. 目标

- 实现 direct children 的 `wait_for_children`。
- 支持 `until` 为 `settled`、`done`、`exited`。
- 支持 `timeout_s`。
- 只允许等待 caller 的 direct children。

## 3. 非目标

- 不实现 event-subscribe server-side wait。
- 不实现 recursive descendants。
- 不实现 release_agent permission guard。
- API-005 保持 partial。

## 4. 验收

- `cargo test -p homie-cli --test mcp_wait_children_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`


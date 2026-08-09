# Diri MCP Lineage Children 对齐设计文档

```yaml
change_id: diri-mcp-lineage-children
beads: homie-s82
target_rows:
  - API-005
  - API-004
feature_atoms:
  - M13-F002
```

## 1. 概述

Diri MCP lineage 从 session parent 字段推导 caller/children。Homie 目前 `mcp-stdio --session-id` 只在 `whoami` 中回显 identity，`spawn_agent` 不写 parent linkage，`list_children` 仍返回 unsupported。

## 2. 目标

- `spawn_agent` 在 MCP context 有 `--session-id` 时，把新 session 的 `parent_session_id` 写入 storage metadata。
- 实现 `list_children`，返回 caller 的 direct children。
- 保留完整 wait/release/permission enforcement 为后续 lane。

## 3. 非目标

- 不实现 recursive descendants。
- 不实现 wait_for_children。
- 不实现 release_agent permission guard。
- API-005 保持 partial。

## 4. 验收

- `cargo test -p homie-cli --test mcp_lineage_children_cli -- --nocapture`
- `cargo check -p homie-storage -p homie-client -p homie-cli`
- `cargo clippy -p homie-storage -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`


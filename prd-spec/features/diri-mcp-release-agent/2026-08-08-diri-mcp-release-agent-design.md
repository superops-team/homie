# Diri MCP release_agent Lineage Guard 设计文档

```yaml
change_id: diri-mcp-release-agent
beads: homie-3th
target_rows:
  - API-005
  - API-004
feature_atoms:
  - M13-F002
```

## 1. 概述

Diri 的 `release_agent` 允许结束被委派的 child session，但拒绝释放 caller/self 或 parent/ancestor。Homie MCP 目前只把 `release_agent` 暴露为 descriptor，调用仍 unsupported。

## 2. 目标

- 实现 MCP `release_agent`。
- 允许释放 direct child。
- 拒绝释放 caller/self。
- 后续再补 parent/ancestor 和 recursive permission。

## 3. 非目标

- 不实现 recursive lineage permission。
- 不实现 report_to_parent。
- API-005 保持 partial。

## 4. 验收

- `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`


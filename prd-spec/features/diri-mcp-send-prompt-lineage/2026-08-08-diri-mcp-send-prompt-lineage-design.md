# Diri MCP send_prompt Lineage Provenance 设计文档

```yaml
change_id: diri-mcp-send-prompt-lineage
beads: homie-ggi
target_rows:
  - API-005
  - API-004
feature_atoms:
  - M13-F002
```

## 1. 概述

Diri `send_prompt` 会根据 caller 与 target 的 lineage relation 决定是否添加 provenance header。Homie 当前 MCP `send_prompt` 直接写入目标 session，没有 self guard，也没有 sibling/unrelated attribution。

## 2. 目标

- 拒绝 `send_prompt` 发送给 calling session 自己。
- parent/direct child 保持原文投递。
- sibling/unrelated 增加 `[message from id:<caller> (...), channel: homie]` provenance header。
- 返回 `relation` 和 `attributed`。

## 3. 非目标

- 不实现完整 permission profile。
- 不实现 recursive descendant attribution。
- API-005 保持 partial。

## 4. 验收

- `cargo test -p homie-cli --test mcp_send_prompt_lineage_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`


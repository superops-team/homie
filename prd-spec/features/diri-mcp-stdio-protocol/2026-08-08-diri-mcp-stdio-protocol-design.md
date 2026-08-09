# Diri MCP Stdio Protocol 设计文档

```yaml
change_id: diri-mcp-stdio-protocol
beads: homie-0xb
target_rows:
  - API-004
feature_atoms:
  - M13-F001
```

## 1. 背景

`API-004` 仍为 `partial`。Homie CLI 目前只有 `mcp-tools` 和 `mcp-call` stub，没有 stdio JSON-RPC 协议入口。Diri 的 MCP server 通过 stdio 暴露 tools，让 agent 可以发现工具和调用编排能力。

## 2. 目标

- 增加 `homie mcp-stdio` 子命令。
- 实现 newline-delimited JSON-RPC 处理函数。
- 支持 `tools/list`。
- 支持 `tools/call` 的 `list_agents` 和 `whoami` 最小工具。
- 对未知工具返回 error envelope。

## 3. 非目标

- 不实现完整 MCP tool set。
- 不连接 runtime 执行 spawn/send/wait。
- 不把 `API-004` 标为 implemented；完整 MCP transcript E2E 仍待补。

## 4. 验收

- `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture`
- `cargo check -p homie-cli`
- `cargo clippy -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

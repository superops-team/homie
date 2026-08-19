# Plan — remote-capability-mcpstdio-removal

## 目标

删除 MCP stdio transport 移除后遗留的死 wire 协议枚举变体
`homie_proto::RemoteCapability::McpStdio`，使协议能力枚举与实现能力面一致。

## 影响范围

- 仅 `homie/crates/homie-proto/src/remote_pty.rs`：
  - 删除 `RemoteCapability` 枚举中的 `McpStdio,` 变体。
  - 删除 `RemoteCapability::wire_name` 中的 `Self::McpStdio => "mcp-stdio",` 分支。

## 兼容性

`#[serde(other)] Unknown` 保证前后向兼容；删除不改变语义，不 bump `PROTOCOL_MAJOR/MINOR`。

## 验证

- `cargo test -p homie-proto -p homie-remote -p homie-engine --offline`
- `cargo clippy -p homie-proto --all-targets --offline`

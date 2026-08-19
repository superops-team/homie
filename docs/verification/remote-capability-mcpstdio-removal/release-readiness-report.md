# Release Readiness Report — remote-capability-mcpstdio-removal

- Beads: `homie-cqm`
- change_id: `remote-capability-mcpstdio-removal`
- 日期: 2026-08-19

## 1. 交付范围

删除 MCP stdio transport 移除后遗留的死 wire 协议枚举变体
`homie_proto::RemoteCapability::McpStdio`，使协议能力枚举与现实能力面一致。

## 2. 落地内容

- 删除 `homie/crates/homie-proto/src/remote_pty.rs` 中 `RemoteCapability` 枚举的
  `McpStdio` 变体。
- 删除 `RemoteCapability::wire_name` 的 `Self::McpStdio => "mcp-stdio"` 分支。

## 3. 验证证据

### 3.1 零引用

```text
grep -rn "McpStdio\|mcp-stdio" homie/crates/   # 无源码引用（仅历史 release 报告文字）
```

### 3.2 单元测试

```text
cargo test -p homie-proto --offline
test result: ok. 16 passed; 0 failed; 0 ignored
```

关键：`unknown_optional_capability_is_forward_compatible` 仍通过，确认 `#[serde(other)]
Unknown` 前后向兼容路径不受影响。

### 3.3 clippy

```text
cargo clippy -p homie-proto --all-targets --offline
Finished (无告警)
```

### 3.4 集成

`cargo test -p homie-proto -p homie-remote -p homie-engine --offline`：
- `homie-proto` / `homie-engine` 全绿。
- `homie-remote` e2e 中 `environment_capture_frames_its_payload_and_scrubs_ssh_state`
  失败（`login environment capture timed out`），为**既有环境相关 flake**（登录 shell 超时），
  与本次枚举变体删除无关；其余 8 项 e2e 通过。

## 4. 兼容性结论

`RemoteCapability` 使用 `#[serde(other)] Unknown` 处理未知变体，删除死变体不改变语义，
前后向安全；不 bump `PROTOCOL_MAJOR/MINOR`。

## 5. 结论

窄幅内部重构完成，相关 crate 测试/clippy 全绿，唯一失败为既有环境 flake。tag `v0.8.1`（patch）。

# RemoteCapability::McpStdio 死 wire 协议变体移除设计文档

## 1. 背景

`mcp-http-transport-unified`（change_id，Beads `homie-gyj`，已关闭 v0.8.0）将 MCP 工具
transport 从 stdio 统一为 daemon 内嵌 `streamable-http`，删除了 `homie-mcp` crate、
Swift stdio MCP bridge（`Sources/HomieMCP`、`MCPBridge.swift`、`MCPLineage*.swift`）以及
`mcp-stdio`/`mcp-tools`/`mcp-call` 子命令。

但 `homie-proto::RemoteCapability` 枚举中的 `McpStdio` 变体（wire 名 `"mcp-stdio"`）被
保留了下来，成为**死代码**：它不再被任何 helper / engine / holder 引用，也未被纳入
`PHASE_ONE_HOLDER_CAPABILITIES` 或 `PHASE_ONE_HELPER_CAPABILITIES` 广告面。

release 报告 `docs/verification/mcp-http-transport-unified/release-readiness-report.md`
§3 已明确将其标注为「删除属 wire 协议变更，超出本 change 范围，留待独立 PRD」。

## 2. 目标

- 删除 `RemoteCapability::McpStdio` 枚举变体及其 `wire_name` 分支，使 wire 协议枚举与
  现实能力面一致。

## 3. 非目标

- 不删除其他「前瞻性」能力变体（`AgentEvents` / `ResourceInspect` / `PortForward` /
  `RebootRecovery` / `Migration`）：它们是协议版本化策略中已声明、尚未接线命令的前瞻能力，
  与本 stdio 删除无关。
- 不修改 `PROTOCOL_MAJOR` / `PROTOCOL_MINOR` 版本号（删除死变体不改变语义，且
  `#[serde(other)] Unknown` 已保证前后向兼容）。
- 不新增任何兼容层 / 迁移路径 / fallback。

## 4. 兼容性论证

`RemoteCapability` 使用 `#[serde(other)] Unknown` 处理未知变体：

- 旧 helper 仍广告 `"mcp-stdio"` → 新 daemon 反序列化为 `Unknown` 并忽略（本地不 require
  该能力），安全。
- 新 helper 不再广告 `"mcp-stdio"` → 旧 daemon 本就不 require 它，安全。

因此删除为**前后向安全**的窄幅内部重构（patch 级）。

## 5. 需求

### FR-1 删除死变体

- 移除 `homie/crates/homie-proto/src/remote_pty.rs` 中 `RemoteCapability` 枚举的
  `McpStdio` 变体。
- 移除 `RemoteCapability::wire_name` 中 `Self::McpStdio => "mcp-stdio"` 分支。

## 6. 受影响 Specs

- `specs/mcp-transport.md`：§移除资产 已记录 `mcp-stdio` 子命令删除；本次补充说明 wire
  协议枚举 `RemoteCapability::McpStdio` 已同步移除（若该 spec 提及该变体）。

## 7. 测试计划

- `cargo test -p homie-proto -p homie-remote -p homie-engine --offline` 全绿。
- `cargo clippy -p homie-proto --all-targets --offline` 无新增告警。
- 确认 `unknown_optional_capability_is_forward_compatible` 单测仍通过（验证 `Unknown`
  兼容路径不受影响）。

## 8. 验收标准

- `McpStdio` 在源码中零引用。
- 全量相关 crate 测试 / clippy 绿。
- 证据齐全：`docs/verification/remote-capability-mcpstdio-removal/`。

## 9. Beads 追踪

- change_id: `remote-capability-mcpstdio-removal`
- 类型: refactor
- 优先级: P2（窄幅内部重构）

# Diri MCP Runtime-backed Tool Surface 设计文档

```yaml
change_id: diri-mcp-tool-surface
beads: homie-0pd
target_rows:
  - API-004
  - API-005
  - API-003
feature_atoms:
  - M12-F002
  - M13-F001
  - M13-F002
```

## 1. 概述

### 1.1 问题/背景

Homie 已有 `homie mcp-stdio` 的最小 JSON-RPC shell，但它只返回静态 `list_agents` 和 `whoami` 结果。Diri 的 MCP server 是 agent 编排入口，至少需要通过 stdio tool 调真实 runtime session，而不能只返回占位数据。

### 1.2 目标

- 让 `homie mcp-stdio --data-dir <dir>` 使用真实 `HomieClient`。
- `tools/list` 暴露 Diri 编排工具目录中的基础 runtime-backed 工具。
- `tools/call` 支持 `list_agents`、`whoami`、`get_status`、`read_output`、`send_prompt`、`spawn_agent`。
- 工具返回 MCP text content，其中 text 是 JSON 字符串，方便 agent 解析。
- 未提供 `--data-dir` 时保持已有最小 no-runtime 模式，避免测试或 hook 无意创建真实用户 HOME 数据。

## 2. 用户场景

### 场景 1: agent 发现当前 Homie session

**Given** Homie runtime 已有一个 session。  
**When** MCP client 调用 `list_agents`。  
**Then** 返回真实 session 列表，包含 `id/title/status/workspace`。

### 场景 2: agent 读取另一个 session 输出

**Given** 某 session 有 live output。  
**When** MCP client 调用 `read_output`。  
**Then** 返回真实 output 文本，而不是静态占位。

### 场景 3: agent 发送 prompt 到 session

**Given** 某 session 正在运行。  
**When** MCP client 调用 `send_prompt`。  
**Then** Homie 通过 runtime 写入 text，并返回 safe JSON 结果。

### 场景 4: agent 通过 MCP spawn 新 session

**Given** MCP client 有 workspace/cwd。  
**When** 调用 `spawn_agent`。  
**Then** Homie 创建 runtime-backed shell session，并返回新 session summary。

## 3. 功能需求

### FR-1: Runtime-backed MCP context

`homie mcp-stdio` 必须接受可选 `--data-dir`。有 `--data-dir` 时工具调用使用 `HomieClient`；无 `--data-dir` 时保留 no-runtime 最小模式。

### FR-2: Tool descriptor 对齐

`tools/list` 必须包含 `spawn_agent`、`list_agents`、`get_status`、`send_prompt`、`read_output`、`whoami`，并保留 Diri 未来工具名描述。

### FR-3: Tool call 行为

- `list_agents` 返回真实 sessions。
- `get_status` 读取 session status report。
- `read_output` 返回 output text。
- `send_prompt` 调 `HomieClient::send_text`。
- `spawn_agent` 调 `HomieClient::spawn_shell`。
- `whoami` 返回当前 session identity，如果无 session 则返回 `unbound`。

### FR-4: 安全和错误

工具错误必须返回 JSON-RPC error 或 safe JSON text，不泄漏 raw provider key、Authorization、cookie 或完整敏感 tool args。

## 4. 实现方案

### 4.1 CLI 参数

把 `McpStdio` 从 unit subcommand 改为 `McpStdio(McpStdioArgs)`，新增 `--data-dir`、`--session-id`、`--parent-session-id`。

### 4.2 MCP handler

引入内部 `McpRuntimeContext`，在有 data-dir 时打开 `HomieClient`。`mcp_stdio_response` 增加 context 参数，旧测试继续走 no-runtime context。

### 4.3 测试

新增 CLI integration test：先用 `homie session create --data-dir` 创建真实 session，再通过 `mcp-stdio --data-dir` 调 `list_agents/get_status/read_output/send_prompt/spawn_agent`。

## 5. 边界情况

| 场景 | 处理方式 |
|------|---------|
| 无 `--data-dir` | no-runtime mode，`list_agents` 为空，`whoami` 为 unbound |
| runtime session 不存在 | 返回 JSON-RPC error，code `-32000` |
| `send_prompt` 缺少 session id/text | 返回 JSON-RPC invalid params |
| `spawn_agent` 缺少 cwd/workspace | 返回 JSON-RPC invalid params |
| 后续 lineage children 工具未完成 | tool descriptor 可保留，调用返回 unsupported，parity 仍 partial |

## 6. 涉及文件

- `crates/homie-cli/src/main.rs`
- `crates/homie-cli/tests/mcp_stdio_runtime_cli.rs`
- `specs/mcp-automation/README.md`
- `docs/research/diri-parity-lock.md`

## 7. 验收标准

- `cargo test -p homie-cli --test mcp_stdio_runtime_cli -- --nocapture`
- `cargo test -p homie-cli --test mcp_stdio_cli -- --nocapture`
- `cargo check -p homie-cli`
- `cargo clippy -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

## 8. Beads 跟踪

- Beads: `homie-0pd`
- 父级分组: `homie-h7n.1`
- 本 slice 完成后不关闭 `API-004/API-005` 全量 parity；仍需 lineage permission、children、wait/release、worktree/browser/test_run E2E。

# Diri MCP wait_for_agent Runtime 设计文档

```yaml
change_id: diri-mcp-wait-for-agent
beads: homie-trk
target_rows:
  - API-004
  - API-005
feature_atoms:
  - M13-F001
  - M13-F002
```

## 1. 概述

### 1.1 问题/背景

Diri 的 MCP orchestration flow 明确包含 `spawn_agent -> wait_for_agent(until:"done") -> read_output -> release_agent`。Homie 当前 `mcp-stdio` 已具备 runtime-backed `spawn_agent`、`read_output`、`release_agent` 和 `wait_for_children`，但 `wait_for_agent` 仍返回 unsupported，导致 agent 无法等待单个目标 session 进入 done/idle/exited 状态。

该缺口不依赖截图或 UI，可通过真实 `homie mcp-stdio --data-dir` 与 `homie notify --data-dir` 完成端到端验证。

### 1.2 目标

- 实现 MCP `wait_for_agent` runtime-backed tool。
- 支持 Diri 参数拼写：`session_id`，并兼容 Homie 已用 camelCase `sessionId`。
- 支持 `until`，默认 `done`；`done` 视为 `idle` 或 `exited`。
- 支持 `timeout_s`/`timeoutS`，超时返回 settled=false/timedOut=true，而不是 panic。
- 返回目标 session 的当前状态和等待目标。

## 2. 用户场景

### 场景 1：等待 agent 完成

**Given** 已有一个 Homie session。  
**When** Codex notify 将该 session 标记为 turn complete。  
**Then** MCP `wait_for_agent(session_id, until:"done")` 返回 settled=true，状态为 idle。

### 场景 2：等待运行中的 agent 超时

**Given** 已有一个仍 running 的 Homie session。  
**When** MCP `wait_for_agent(session_id, until:"done", timeout_s:0)` 被调用。  
**Then** 返回 settled=false、timedOut=true，并包含当前状态 running。

### 场景 3：等待 exited 状态

**Given** 目标 session 已通过 Homie runtime 终止。  
**When** MCP `wait_for_agent(session_id, until:"exited")` 被调用。  
**Then** 返回 settled=true，状态为 exited。

## 3. 功能需求

### FR-1：Runtime-backed wait_for_agent

`homie mcp-stdio --data-dir` 必须支持 `tools/call` name=`wait_for_agent` 并读取真实 runtime session status。

### FR-2：Diri 参数兼容

必须接受 `session_id` 和 `sessionId`；必须接受 `timeout_s` 和 `timeoutS`。

### FR-3：等待判定

`until:"done"` 通过条件为 `idle` 或 `exited`；`until:"exited"` 通过条件为 `exited`；其它目标使用现有安全状态集合，与 `wait_for_children` 保持一致。

### FR-4：超时可观测

超时必须返回结构化结果，不得返回 unsupported 或 panic。

## 4. 实现方案

### 4.1 CLI/MCP tool handler

在 `crates/homie-cli/src/main.rs` 中新增 `wait_for_agent_payload`：

- 打开 `runtime_client(context)`。
- 解析 `session_id`/`sessionId`。
- 解析 `until`，默认 `done`。
- 解析 `timeout_s`/`timeoutS`，默认 600 秒。
- 轮询 `HomieClient::status_report(session_id)`，直到满足 `child_has_reached(until, status)` 或超时。
- 返回 `settled`、`timedOut`、`sessionId`、`status`、`waitedFor`。

### 4.2 测试策略

新增 `crates/homie-cli/tests/mcp_wait_for_agent_cli.rs`：

- `waits_for_agent_until_done`：用真实 notify 持久化 idle，再调用 MCP 等待 done。
- `timeout_returns_current_status`：对 running session 使用 `timeout_s:0` 验证超时结构。
- `waits_for_exited_agent`：先 kill session，再等待 exited。

## 5. 边界情况

| 场景 | 处理方式 |
|------|---------|
| no `--data-dir` | 仍返回 runtime unavailable safe error |
| 缺少 session id | 返回 invalid params |
| target 不存在 | 透传 runtime safe error |
| timeout_s=0 | 至少检查一次当前状态，然后立即返回 settled 或 timedOut |
| unknown until | 沿用 `child_has_reached` 的安全集合 |

## 6. 涉及文件

- `crates/homie-cli/src/main.rs`
- `crates/homie-cli/tests/mcp_wait_for_agent_cli.rs`
- `specs/mcp-automation/README.md`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-mcp-wait-for-agent/`

## 7. 验收标准

- `cargo test -p homie-cli --test mcp_wait_for_agent_cli -- --nocapture`
- `cargo test -p homie-cli --test mcp_wait_children_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

## 8. 受影响长期规格

- `specs/mcp-automation/README.md`：将 `wait_for_agent` 从未实现工具列表移到 runtime-backed 已实现工具，并记录 status wait 合同。

## 9. Beads 跟踪

- Bead: `homie-trk`
- 完成验证后关闭。

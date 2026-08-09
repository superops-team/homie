# Diri MCP get_artifacts Runtime 设计文档

```yaml
change_id: diri-mcp-get-artifacts
beads: homie-pyt
target_rows:
  - API-004
  - ART-001
  - ART-002
feature_atoms:
  - M13-F001
  - M04-F002
```

## 1. 概述

### 1.1 问题/背景

Diri MCP `get_artifacts` 用于让 agent 从 session 输出中获取 PR 链接、预览 URL 和监听端口，是 `test_run`/browser preview 前置能力。Homie 已有 runtime artifact scanner 和 client API `scan_session_artifacts`，但 MCP `get_artifacts` 仍在 descriptor 中返回 unsupported，导致 agent 无法从 MCP orchestration flow 中发现 preview/ports。

该能力不依赖截图，可通过真实 session 输出、runtime scanner 和 `homie mcp-stdio --data-dir` 完成端到端验证。

### 1.2 目标

- 实现 MCP `get_artifacts` runtime-backed tool。
- 支持 Diri 参数 `session_id`，并兼容 Homie camelCase `sessionId`。
- 返回 `artifacts` 数组和 Diri 命名 `listeningPorts` 数组。
- 复用 `HomieClient::scan_session_artifacts`，不在 MCP 层重复解析 URL/端口。

## 2. 用户场景

### 场景 1：Agent 获取 preview URL 和端口

**Given** session 输出中出现 `http://localhost:5173`。  
**When** agent 调用 MCP `get_artifacts(session_id)`。  
**Then** Homie 返回 preview artifact 和 listeningPorts 中的 5173。

### 场景 2：Agent 获取 PR/link artifact

**Given** session 输出中出现 pull request URL 和普通文档 URL。  
**When** agent 调用 MCP `get_artifacts(session_id)`。  
**Then** Homie 返回 kind 区分的 artifact 列表。

### 场景 3：参数缺失

**Given** MCP 调用未传 `session_id/sessionId`。  
**When** agent 调用 `get_artifacts`。  
**Then** Homie 返回 JSON-RPC invalid params，不 panic。

## 3. 功能需求

### FR-1：Runtime-backed get_artifacts

`homie mcp-stdio --data-dir` 必须支持 `tools/call` name=`get_artifacts`，并通过 `HomieClient::scan_session_artifacts` 读取真实 session output。

### FR-2：Diri 输出命名

返回结构必须包含：

- `artifacts`
- `listeningPorts`

### FR-3：参数兼容

必须接受 `session_id` 和 `sessionId`。

### FR-4：范围诚实

本阶段不实现 Diri PR monitor 的 live GitHub stats，也不实现 browser/test_run。

## 4. 实现方案

### 4.1 MCP payload 分支

在 `crates/homie-cli/src/main.rs` 的 `mcp_tool_payload` 中新增 `get_artifacts` 分支：

- 解析 `session_id` 或 `sessionId`；
- 调用 `runtime_client(context)?.scan_session_artifacts(&session_id)`；
- 返回 `{ sessionId, artifacts, listeningPorts }`。

### 4.2 测试策略

新增 `crates/homie-cli/tests/mcp_get_artifacts_cli.rs`：

- 创建真实 session；
- 通过 `control-stdio` 发送含 PR、preview、普通链接的输出；
- 通过 MCP `get_artifacts` 获取结果；
- 轮询直到 scanner 看到 preview/port；
- 缺参返回 `-32602`。

## 5. 边界情况

| 场景 | 处理方式 |
|------|---------|
| session 不存在 | 透传 runtime safe error |
| 输出中无 artifact | 返回空数组 |
| no `--data-dir` | 返回 runtime unavailable safe error |
| PR live stats 不存在 | 不返回 `pr` 扩展字段，本阶段 scope note 说明 |

## 6. 涉及文件

- `crates/homie-cli/src/main.rs`
- `crates/homie-cli/tests/mcp_get_artifacts_cli.rs`
- `specs/mcp-automation/README.md`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-mcp-get-artifacts/`

## 7. 验收标准

- `cargo test -p homie-cli --test mcp_get_artifacts_cli -- --nocapture`
- `cargo test -p homie-runtime --test artifact_scanner`
- `cargo test -p homie-cli --test ports_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

## 8. 受影响长期规格

- `specs/mcp-automation/README.md`：将 `get_artifacts` 从 unsupported 列表移到 runtime-backed tools，并记录当前不含 PR live stats。

## 9. Beads 跟踪

- Bead: `homie-pyt`
- 验证完成后关闭。

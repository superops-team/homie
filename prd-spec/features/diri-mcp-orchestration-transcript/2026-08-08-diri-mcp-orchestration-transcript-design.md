# Diri MCP Orchestration Transcript E2E 设计文档

```yaml
change_id: diri-mcp-orchestration-transcript
beads: homie-3vh
target_rows:
  - API-004
  - API-005
  - AUTO-001
  - ART-001
feature_atoms:
  - M13-F001
  - M13-F002
  - M12-F002
```

## 1. 概述

### 1.1 问题/背景

Diri MCP server instructions 描述了典型 orchestration flow：`spawn_agent -> wait_for_agent(until:"done") -> read_output -> send_prompt -> get_artifacts -> release_agent`。Homie 已分片实现这些工具，但 parity lock 仍保留 “full transcript E2E pending”，因为尚未有一个测试把这些工具串成同一个真实 MCP transcript。

该切片不依赖截图，也不需要 browser sidecar；它只验证已实现 MCP tools 能在同一 runtime-backed data dir 下组成 Diri 风格编排闭环。

### 1.2 目标

- 新增真实 MCP transcript E2E 测试。
- 覆盖 `spawn_agent`、`send_prompt`、`wait_for_agent`、`read_output`、`get_artifacts`、`release_agent`。
- 验证 child session 完成后可读取输出、发现 artifact/port，并可由 parent release。
- 如测试暴露缺口，做最小修复；如无缺口，只补证据和 parity lock。

## 2. 用户场景

### 场景 1：父 agent 完成一次子 agent 编排

**Given** parent session 通过 Homie MCP 运行。  
**When** parent 调用 `spawn_agent` 创建 child，并通过 `send_prompt` 让 child 输出预览 URL，随后上报 turn complete。  
**Then** parent 可以 `wait_for_agent(until:"done")`、`read_output`、`get_artifacts`，最后 `release_agent` 清理 child。

## 3. 功能需求

### FR-1：Transcript E2E

测试必须使用真实 `homie mcp-stdio --data-dir --session-id`，不能直接调用内部函数或静态 payload。

### FR-2：工具链覆盖

同一测试必须覆盖 spawn、send、wait、read、artifact、release 至少六个工具。

### FR-3：释放后状态

`release_agent` 后 child session 必须进入 exited 状态或 runtime 不再认为它 running。

## 4. 实现方案

新增 `crates/homie-cli/tests/mcp_orchestration_transcript_cli.rs`：

1. 创建 parent session。
2. MCP `spawn_agent` 创建 child。
3. MCP `send_prompt` 向 child 写入 `echo http://localhost:6123`。
4. 等待 child output 出现 URL。
5. `homie notify --data-dir` 将 child 标记 turn complete。
6. MCP `wait_for_agent` 等待 done。
7. MCP `read_output` 断言 URL 可读。
8. MCP `get_artifacts` 断言 listeningPorts 包含 6123。
9. MCP `release_agent` 从 parent 释放 child。
10. `homie session snapshot` 断言 child status exited。

## 5. 非目标

- 不实现 browser/test_run。
- 不新增 MCP 工具。
- 不改 UI。
- 不改 runtime holder 生命周期策略，除非 E2E 暴露真实回归。

## 6. 涉及文件

- `crates/homie-cli/tests/mcp_orchestration_transcript_cli.rs`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-mcp-orchestration-transcript/`

## 7. 验收标准

- `cargo test -p homie-cli --test mcp_orchestration_transcript_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

## 8. Beads 跟踪

- Bead: `homie-3vh`
- 验证完成后关闭。

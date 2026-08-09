# Diri MCP release_agent Owned-child 权限设计文档

```yaml
change_id: diri-mcp-release-owned-child-guard
beads: homie-al5
target_rows:
  - API-005
  - API-004
feature_atoms:
  - M13-F002
```

## 1. 概述

### 1.1 问题/背景

Homie 已补齐 MCP `release_agent` 的 direct child 释放、自身释放拒绝、parent/ancestor 释放拒绝，但仍缺少 Diri lineage permission 中最小权限边界：调用方只能释放自己直接 spawn 的 child，不能释放 sibling 或 unrelated session。

如果保留当前行为，任意有 MCP runtime identity 的 session 仍可能通过 `release_agent` 终止同级或无关会话，破坏 Diri 中“父子任务边界隔离”的安全假设。

### 1.2 目标

- `release_agent` 只允许释放调用方的 direct child。
- `release_agent` 拒绝 sibling target。
- `release_agent` 拒绝 unrelated target。
- 保持 direct child release、self guard、parent/ancestor guard 的既有行为不回归。

## 2. 用户场景

### 场景 1：父会话释放自己 spawn 的子会话

**Given** Homie MCP caller 绑定到父 session，且 target 是该 caller spawn 的 direct child。  
**When** caller 调用 `release_agent(sessionId=child)`。  
**Then** Homie 终止 child session 并返回 `ok: true`。

### 场景 2：同级会话不能互相释放

**Given** sibling A 与 sibling B 拥有同一个 parent。  
**When** sibling A 调用 `release_agent(sessionId=siblingB)`。  
**Then** Homie 返回 JSON-RPC `-32000` runtime error，错误文案说明只能释放自己 spawn 的 child。

### 场景 3：无关会话不能释放

**Given** caller 与 target 不在同一 lineage 链路。  
**When** caller 调用 `release_agent(sessionId=target)`。  
**Then** Homie 返回 JSON-RPC `-32000` runtime error，且不终止 target。

## 3. 功能需求

### FR-1：Owned-child allow

`release_agent` 必须仅允许 `lineage_relation(context, target) == "child"` 时执行 `terminate_session`。

### FR-2：Sibling/unrelated deny

`release_agent` 必须拒绝 `sibling` 和 `unrelated` target，并返回 safe runtime error。

### FR-3：既有保护不回归

`self`、`parent`、`ancestor` 关系的错误文案和 JSON-RPC error code 不得回归；direct child release 仍可用。

## 4. 实现方案

### 4.1 CLI/MCP 分支

在 `crates/homie-cli/src/main.rs` 的 `release_agent` tool handler 中，复用既有 `lineage_relation`：

- `self`：保留现有 self-release guard。
- `parent`/`ancestor`：保留现有上游释放 guard。
- `child`：允许调用 `HomieClient::terminate_session`。
- `sibling`/`unrelated`/其它关系：拒绝。

### 4.2 测试策略

新增 `crates/homie-cli/tests/mcp_release_owned_child_guard_cli.rs`，通过真实 `homie mcp-stdio --data-dir --session-id` 创建 session lineage 并验证：

- sibling release 被拒绝；
- unrelated release 被拒绝；
- target snapshot 仍能读取，证明没有被误终止；
- 既有 direct child release regression 继续通过。

## 5. 边界情况

| 场景 | 处理方式 |
|------|---------|
| caller 没有 `--session-id` | 视为 `unrelated`，拒绝释放 target |
| target 不存在 | 仍由 runtime/client 返回 safe runtime error |
| target 是 child 的 descendant 而非 direct child | 本阶段拒绝；recursive release/permission 矩阵另行处理 |
| sibling/unrelated release 被拒绝后 | 不调用 `terminate_session`，target 保持可查询 |

## 6. 涉及文件

- `crates/homie-cli/src/main.rs`
- `crates/homie-cli/tests/mcp_release_owned_child_guard_cli.rs`
- `specs/mcp-automation/README.md`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-mcp-release-owned-child-guard/`

## 7. 验收标准

- `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli -- --nocapture`
- `cargo test -p homie-cli --test mcp_release_agent_cli -- --nocapture`
- `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

## 8. 受影响长期规格

- `specs/mcp-automation/README.md`：补充 `release_agent` owned-child 权限规则。

## 9. Beads 跟踪

- Bead: `homie-al5`
- 状态：实现和验证完成后关闭。

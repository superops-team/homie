# Diri MCP Worktree Tools Runtime 设计文档

```yaml
change_id: diri-mcp-worktree-tools
beads: homie-4wg
target_rows:
  - API-004
  - API-003
  - GIT-002
feature_atoms:
  - M12-F002
  - M13-F001
  - M07-F002
```

## 1. 概述

### 1.1 问题/背景

Diri MCP bridge 暴露 `create_worktree`、`list_worktrees`、`remove_worktree`，用于 agent 在同一编排流里为子任务创建和清理 git worktree。Homie 已经实现 runtime/client/CLI 的真实 git worktree create/list/remove，但 `homie mcp-stdio` 仍只在 descriptor 中列出这些工具，payload 未接入，导致 managed agent 无法通过 MCP 使用 worktree 能力。

该能力不依赖截图，可通过临时 git repo 和真实 `homie mcp-stdio --data-dir` 做端到端验证。

### 1.2 目标

- 实现 MCP `create_worktree`，参数与 Diri 一致：`repo`、可选 `branch`、可选 `base`。
- 实现 MCP `list_worktrees`，参数 `repo`。
- 实现 MCP `remove_worktree`，参数 `repo`、`path`、可选 `force`。
- 复用既有 `HomieClient::worktree_create/list/remove`，不重复写 git shell 逻辑。

## 2. 用户场景

### 场景 1：Agent 创建隔离 worktree

**Given** 当前项目是 git repo。  
**When** agent 通过 MCP 调用 `create_worktree(repo, branch, base)`。  
**Then** Homie 创建真实 git worktree，并返回路径、branch、head 等结构化信息。

### 场景 2：Agent 查看 repo worktrees

**Given** repo 中已存在 Homie 创建的 worktree。  
**When** agent 通过 MCP 调用 `list_worktrees(repo)`。  
**Then** Homie 返回包含该 worktree 的列表。

### 场景 3：Agent 清理 worktree

**Given** repo 中存在一个可删除 worktree。  
**When** agent 通过 MCP 调用 `remove_worktree(repo, path, force:true)`。  
**Then** Homie 调用真实 git worktree remove，返回 ok，并且目录不存在。

## 3. 功能需求

### FR-1：create_worktree runtime-backed

`homie mcp-stdio --data-dir` 必须支持 `tools/call` name=`create_worktree`，并通过 `HomieClient::worktree_create` 执行。

### FR-2：list_worktrees runtime-backed

`list_worktrees` 必须返回 `{ "worktrees": [...] }`，内容来自真实 git porcelain 解析路径。

### FR-3：remove_worktree runtime-backed

`remove_worktree` 必须通过 `HomieClient::worktree_remove` 删除指定 path，并返回 `{ "ok": true, "path": path }`。

### FR-4：参数错误安全

缺少 `repo` 或 `path` 时必须返回 JSON-RPC invalid params，不得 panic。

## 4. 实现方案

### 4.1 MCP payload 分支

在 `crates/homie-cli/src/main.rs` 的 `mcp_tool_payload` 中新增三条分支：

- `create_worktree`：解析 `repo`、`branch`、`base`，构造 `homie_proto::WorktreeCreateRequest`。
- `list_worktrees`：解析 `repo`，构造 `homie_proto::WorktreeListRequest`。
- `remove_worktree`：解析 `repo`、`path`、`force`，构造 `homie_proto::WorktreeRemoveRequest`。

### 4.2 测试策略

新增 `crates/homie-cli/tests/mcp_worktree_tools_cli.rs`：

- 初始化真实 git repo；
- MCP create/list/remove 贯穿一个端到端用例；
- 断言 list 包含创建出的 worktree；
- 断言 remove 后路径不存在；
- 另测缺少 repo/path 返回 invalid params。

## 5. 边界情况

| 场景 | 处理方式 |
|------|---------|
| repo 不是 git repo | 透传 runtime safe error |
| branch 已存在 | 透传 git/runtime error |
| remove path 不存在 | 透传 runtime safe error |
| force 未传 | 默认 false |
| no `--data-dir` | 返回 runtime unavailable safe error |

## 6. 涉及文件

- `crates/homie-cli/src/main.rs`
- `crates/homie-cli/tests/mcp_worktree_tools_cli.rs`
- `specs/mcp-automation/README.md`
- `docs/research/diri-parity-lock.md`
- `docs/verification/diri-mcp-worktree-tools/`

## 7. 验收标准

- `cargo test -p homie-cli --test mcp_worktree_tools_cli -- --nocapture`
- `cargo test -p homie-cli --test worktree_cli -- --nocapture`
- `cargo check -p homie-client -p homie-cli`
- `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- scoped `git diff --check`
- `make parity-lock`

## 8. 受影响长期规格

- `specs/mcp-automation/README.md`：将三条 worktree MCP tools 从未实现列表移到 runtime-backed tools。

## 9. Beads 跟踪

- Bead: `homie-4wg`
- 验证完成后关闭。

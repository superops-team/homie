# MCP Automation 组件规格

## 1. 组件定位

`homie-mcp-automation` 定义 Homie 的 CLI automation、hook/notify fail-open forwarder、MCP stdio server、MCP tools、browser/test_run、lineage 和跨 session 权限合同。它是 agent 调用 Homie 编排能力的主要自动化入口。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- 功能验证: FC-012, FC-014, FC-018

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | managed agent / MCP client | 通过 stdio 调 MCP tools |
| 上游 | provider hooks/notify | 上报 session event |
| 下游 | `homie-runtime` | session/worktree/artifact/browser/test execution |
| 下游 | `intent-orchestrator` | spawn/routing decisions |
| 下游 | `session-context-store` | lineage/whoami/children |

## 4. 职责边界

负责：

- `homie mcp-stdio`。
- `homie mcp-tools` 和 `homie mcp-call`。
- hook/notify fail-open forwarding。
- tools: spawn_agent、list_agents、get_status、send_prompt、wait_for_agent、read_output、create/list/remove_worktree、get_artifacts、release_agent、test_run、browser、whoami、list_children、wait_for_children。
- lineage 和 permission enforcement。

不负责：

- provider raw key 管理。
- browser engine 安装。
- UI 渲染。

## 5. 核心接口

```rust
pub trait McpToolHandler {
    async fn handle(&self, tool: &str, args: serde_json::Value, identity: ToolIdentity)
        -> Result<serde_json::Value, McpError>;
}

pub struct ToolIdentity {
    pub session_id: Option<SessionId>,
    pub parent_session_id: Option<SessionId>,
    pub permission_profile_id: PermissionProfileId,
}
```

### 5.1 Runtime-backed MCP tool surface 第一阶段

`homie mcp-stdio` 支持两种模式：

| Mode | Trigger | Behavior |
|------|---------|----------|
| no-runtime | no `--data-dir` | `tools/list` works; `list_agents` returns empty list; `whoami` returns unbound |
| runtime-backed | `--data-dir <dir>` | Opens `HomieClient` against that data dir and dispatches session tools through runtime |

第一阶段已承诺的 runtime-backed tools：

| Tool | Required args | Result |
|------|---------------|--------|
| `list_agents` | none | `agents: Vec<SessionSummary>` from runtime storage |
| `whoami` | optional CLI `--session-id`, `--parent-session-id` | current MCP identity and optional session facts |
| `get_status` | `sessionId` | session status report summary |
| `read_output` | `sessionId` | `outputText` from runtime output log |
| `send_prompt` | `sessionId`, `text`, optional `submit` | writes through `HomieClient::send_text` |
| `spawn_agent` | `cwd` or `workspace`, optional `title` | creates a runtime-backed shell session |
| `wait_for_agent` | `session_id`/`sessionId`, optional `until`, `timeout_s`/`timeoutS` | waits on real runtime status; `done` means `idle` or `exited`; returns `settled`, `timedOut`, current `status`, and `waitedFor` |
| `create_worktree` | `repo`, optional `branch`, optional `base` | creates a real git worktree through `HomieClient::worktree_create` |
| `list_worktrees` | `repo` | returns real git worktree rows from `HomieClient::worktree_list` |
| `remove_worktree` | `repo`, `path`, optional `force` | removes a real git worktree through `HomieClient::worktree_remove` |
| `get_artifacts` | `session_id`/`sessionId` | returns current scanner `artifacts` and Diri-named `listeningPorts` from real session output; PR live stats are out of scope until the PR monitor lane is complete |

未实现但保留在 descriptor 中的 tools must return explicit safe unsupported errors until their dedicated lanes land: `test_run`, `browser`.

## 6. 数据模型

Tool result envelope:

```json
{
  "ok": true,
  "value": {},
  "warnings": [],
  "evidence": []
}
```

Error envelope:

```json
{
  "ok": false,
  "error": {
    "code": "permission_denied",
    "message": "safe message",
    "retryable": false
  }
}
```

## 7. 运行模型与状态机

```text
mcp-stdio starts
  -> list tools from runtime/catalog
  -> read tool call
  -> bind identity
  -> validate permission
  -> dispatch to runtime/orchestrator/browser/test
  -> return safe JSON result
```

Hook/notify:

```text
agent hook invokes CLI
  -> read bounded stdin
  -> parse payload
  -> send hook.report
  -> on any error return success/empty output
```

## 8. 安全与权限

- hook/notify fail-open，不阻塞 agent。
- MCP source 必须绑定 identity；无 identity 的 tool 能力受限。
- 跨 session send_prompt/release_agent 需要 permission profile。
- `release_agent` 第一阶段权限矩阵：`child` 是唯一允许终止的关系；`self`、`parent`、`ancestor`、`sibling`、`unrelated` 均必须在调用 `terminate_session` 前拒绝，其中 `self` 和上游关系保留专用安全文案。
- browser/test_run 不返回 inline image bytes。
- tool args/result 进入 logs/evidence 前必须脱敏。

## 9. 可观测性

- mcp.tool_started。
- mcp.tool_completed。
- mcp.tool_failed。
- hook.report_received。
- hook.report_failed_open。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| runtime unavailable | tool 返回 retryable safe error |
| hook report failed | CLI exit 0，输出 safe empty result |
| wait timeout | 返回 partial statuses，不视为 panic |
| browser engine unavailable | tool 返回 engine_unavailable |

## 11. 测试计划与验收引用

- FC-012: browser/test_run and artifacts。
- FC-014: CLI/hook/notify/MCP automation。
- FC-018: full local quality gate。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M04-F002, M07-F002, M12-F001, M12-F002, M13-F001, M13-F002 |
| Required Diri test mapping | CommandGrammarTests, MCP stdio transcript, lineage denied cases |
| Pre-implementation gaps | tool-by-tool MCP contract |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- Requirements: FR-10, FR-11, FR-14, FR-16
- Beads: `homie-t3u`

本组件当前状态是 `partial`。第 5.1 节允许把未实现的 `browser`、`test_run` 保留在 descriptor 中，与重基线的协议真实性规则冲突，现由本节覆盖：

- `tools/list` 只能发布当前进程可执行且通过 capability readiness 的工具。
- 未实现或依赖 sidecar 不可用的工具不得出现在 `tools/list`；直接调用未知工具返回 JSON-RPC method-not-found。
- tool handler 内部失败使用稳定 tool error；不得把 method-not-found 和 execution failure 都映射为 `-32000`。

### 12.1 冻结工具目录

必须覆盖：

- `spawn_agent`
- `list_agents`
- `get_status`
- `send_prompt`
- `wait_for_agent`
- `read_output`
- `create_worktree`
- `list_worktrees`
- `remove_worktree`
- `get_artifacts`
- `release_agent`
- `test_run`
- `browser`
- `whoami`
- `list_children`
- `wait_for_children`
- `summarize_children`
- `report_to_parent`

每个工具必须有独立 JSON Schema，明确 required、alias、enum、unknown fields、timeout/cancel 和 result envelope。禁止所有工具共享 `additionalProperties: true` 的空 schema。

### 12.2 Lineage 与权限

- identity 必须来自可信 runtime/session binding，不能只信任任意 CLI flag。
- 权限矩阵必须覆盖 self、parent、ancestor、direct child、descendant、sibling 和 unrelated。
- `release_agent` 只允许明确拥有的 child/descendant，其他关系 fail closed。
- `send_prompt`、`summarize_children` 和 `report_to_parent` 必须分别定义读写方向和 provenance。
- recursive lineage 操作必须有深度、数量和 timeout 上限。

### 12.3 Browser/Test Sidecar

- `browser` 和 `test_run` 通过受监管 sidecar/runner 执行，runtime 持有 lifecycle、timeout、cancel 和 artifact refs。
- sidecar 不读取 provider credential，不返回 inline image bytes，不接受任意未约束文件或 shell command。
- sidecar、browser runtime 和所需 assets 必须进入 package dependency closure。
- readiness 失败返回 `engine_unavailable`，不伪造成功或静态 artifact。

### 12.4 完成门禁

- MCP stdio transcript fixture覆盖 initialize、tools/list、tools/call、cancel、error 和 shutdown。
- 每个工具至少有 schema、permission、runtime success 和 negative E2E。
- app、CLI、MCP 使用同一 runtime daemon 和 lineage facts。
- 当前 unsupported error code 回归修复，workspace MCP tests 通过。
- browser/test 在 packaged artifact 内执行通过后才进入公开目录。

## 13. Wave 1A Async Daemon Bridge 修订

权威来源：

- PRD: `prd-spec/features/diri-runtime-daemon-client-transport/2026-08-08-diri-runtime-daemon-client-transport-design.md`
- OpenSpec: `openspec/changes/diri-runtime-daemon-client-transport/`
- Beads: `homie-nep`

- MCP runtime-backed tools 必须持有 async `HomieClient`，不得在 MCP/CLI 进程构造 embedded runtime。
- `control-stdio` 是 bounded stdin/stdout 与 daemon control method 的 bridge，不再拥有 dispatcher。
- CLI 入口必须显式调用 `RuntimeLauncher`；`HomieClient::connect` 自身不 spawn daemon。
- `tools/list` 对 runtime method/stream capability 做 readiness 过滤，不得因 proto 常量存在而发布工具。
- daemon `method_not_found` 必须映射 JSON-RPC `-32601`；transport unavailable、timeout 和 execution failure 使用各自 stable tool error。
- MCP shutdown 只关闭本 client connection；只有显式 daemon admin command 才能触发 `prepare_shutdown`/`shutdown`。
- Wave 1A 只迁移当前真实 runtime-backed handlers；完整 frozen tool catalog 仍按本规格 12.1 的后续 owning change 实现。

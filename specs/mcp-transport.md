# MCP Transport 契约（`homie` MCP tools）

> change_id: `mcp-http-transport-unified` · Beads: `homie-gyj`

本文定义 Homie 对外暴露的 `homie` MCP 工具的**传输与协议契约**，是长期工程契约：端点、认证、
会话身份、工具清单、血统判定与错误码。工具语义变更需另开 PRD；本文只锁 transport 与服务端边界。

## 1. 端点

- transport：MCP `streamable-http`（JSON-RPC 2.0）。
- 绑定：`127.0.0.1:<port>`，**仅回环**，不监听 `0.0.0.0` / 不暴露外网。
- 路径：`POST /mcp`（JSON-RPC 请求/响应）。若后续工具需要流式长任务，再开 `GET /mcp` 的 SSE 流；
  v1 不要求。
- 端口由 daemon 启动时确定（固定端口优先，冲突则动态），写入注入事实文件供注入逻辑读取。

## 2. 认证

- 每个请求必须携带 `Authorization: Bearer <secret>`，否则返回 `401`。
- `<secret>` 为 daemon 生命周期随机生成（或每会话签发），仅存内存，不落日志/SQLite/文件。
- 不启用 Codex MCP 的 `auth`/`oauth`/`chatgpt` 流程；统一走 bearer token。

## 3. 会话身份（血统）

- caller session id 从请求 header `X-Homie-Session-Id` 读取。
- header 缺失或非法 → caller 视为 `None`（trusted，写 verbatim），等价于旧 stdio 下无
  `HOMIE_SESSION_ID` 的场景。
- 血统关系判定（相对 caller）：
  - `caller`（self）
  - `parent`（caller 的 parent）
  - `child`（target 的 parent == caller）
  - `ancestor` / `descendant`（沿 parent 链遍历，带 visited 集防环）
  - `sibling`（同 parent）
  - `unrelated`
- `parent` 与直接 `child` 为 delegation channel，写 **verbatim**；其余关系（含 `unrelated`）写
  `send_prompt`/`send_text` 时加 provenance 前缀：`[message from id:<caller> (<title>), channel: homie — reply with send_prompt to that id]`。

## 4. 工具清单（v1，18 个）

`tools/list` 返回以下工具，`spawn_agent.kind` 的 enum 从 manifest 目录动态推导（launchable
shortLabel + `shell`）：

`spawn_agent`、`list_agents`、`get_status`、`send_prompt`、`wait_for_agent`、`read_output`、
`create_worktree`、`list_worktrees`、`remove_worktree`、`list_children`、`wait_for_children`、
`summarize_children`、`report_to_parent`、`get_artifacts`、`release_agent`、`whoami`、`browser`、
`test_run`。

`whoami` 返回的 `writePolicy` 文本固定为：

> Reads are open across all sessions. Writes to your parent or your direct children are delivered
> verbatim; writes to anyone else are prefixed with a provenance header naming you, so the receiving
> agent knows an unrelated session is talking to it. You cannot send_prompt to yourself, and
> release_agent refuses to kill you or any of your ancestors.

## 5. JSON-RPC 方法

- `initialize` → server info + capabilities（`tools`）。
- `tools/list` → `{ tools: [...] }`。
- `tools/call` → `{ name, arguments }` → `{ content: [...], isError? }`。
- `notifications/cancelled` / `notifications/initialized` → 接受并忽略（v1）。
- `ping` → `{}`。

## 6. 错误码

| 场景 | 状态/错误 |
|---|---|
| 无/错 bearer token | HTTP `401` |
| 非法 JSON-RPC | JSON-RPC `-32700` |
| 未知方法 | JSON-RPC `-32601` |
| 未知工具 | `tools/call` 返回 `isError: true`，`-32602` 语义 |
| 工具内部失败 | `tools/call` 返回 `isError: true`，带 `error` 文本（不泄露密钥/token） |

## 7. 服务端边界

- 工具实现位于 `homie-engine`（daemon），直接调用内部 control 方法；**不再**经 Swift CLI 或
  `homie-mcp` 中转。
- 移除资产：`homie-mcp` crate、`mcp-stdio` / `mcp-tools` / `mcp-call` 子命令、`Sources/HomieMCP`
  的 Swift 工具定义（若全部迁 Rust）。

# Plan: MCP HTTP 统一 transport

change_id: `mcp-http-transport-unified` · Beads: `homie-gyj`

## 目标

把 `homie` MCP 工具的 transport 从 stdio（每会话 `homie-mcp` shim + Swift CLI 子进程）统一为
daemon 内嵌的 `streamable_http` endpoint，删除 stdio 代理资产，减少进程数与通信开销。

## 分阶段实施

### Phase 0：核实 Claude headers 展开（决策 D）

- 验证 Claude `.mcp.json` `type:http` 的 `headers` 是否支持 `${ENV_VAR}` 展开。
- 结果写入 `docs/verification/mcp-http-transport-unified/spec-review.md`，据此确定分支 1/2。

### Phase 1：daemon 内嵌 endpoint + 工具收敛

- `homie-engine` 新增 HTTP listener（`127.0.0.1`），端口/bearer secret 启动时确定并写事实文件。
- 新增 `mcp` 模块：JSON-RPC `streamable-http` handler（`initialize`/`tools/list`/`tools/call`/`ping`）。
- 18 个工具 schema 从 Swift `HomieMCPTools.all` 迁 Rust，`spawnableKinds` 从 manifest 推导。
- 血统逻辑从 `MCPLineage.swift` 迁 Rust，caller 从 `X-Homie-Session-Id` header 读。
- 认证：`Authorization: Bearer` 校验，缺失返回 401。

### Phase 2：注入改造

- `inject.rs`：Codex 注入改为 `url`/`bearer_token_env_var`/`env_http_headers`，删 `command`/`args`。
- `write_claude_mcp_file`：写 `type:http`（分支 1/2），删 `mcp_launch`/`is_executable` 的 `homie-mcp` 探测。
- 注入 env：`HOMIE_MCP_TOKEN`、`HOMIE_SESSION_ID`。

### Phase 3：删除 stdio 资产 + 文档

- 删 `homie-mcp` crate、Swift `mcp-stdio`/`mcp-tools`/`mcp-call`、`Sources/HomieMCP`。
- 更新 README 进程表/模块图/时序图（图 4）。

### Phase 4：验证

- 单测 + 集成 + 端到端 + 安全，证据入 `docs/verification/mcp-http-transport-unified/`。
- 打 tag（minor）。

## 依赖

- 复用 `homie-gateway` 的 HTTP 栈选型（axum/hyper 与 tower），避免重复引入。
- manifest 目录的 `AgentCatalog`/`launchable` 派生 `spawnableKinds`。

## 回退

- 若 Claude headers 不支持 env 展开且 per-session 文件方案复杂度过高，可暂保留 Claude stdio、
  仅 Codex 切 HTTP，并在 PRD 标注——但首选两 agent 统一 HTTP。

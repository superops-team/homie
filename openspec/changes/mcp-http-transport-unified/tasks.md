# Tasks: MCP HTTP 统一 transport

## Phase 0 — 核实
- [x] T0.1 验证 Claude `.mcp.json` `type:http` headers 的 `${ENV_VAR}` 展开行为，记录结论（支持展开 → 决策 D 分支 1）。

## Phase 1 — endpoint + 工具收敛
- [ ] T1.1 `homie-engine` 引入 HTTP 栈（复用 gateway 选型），新增 `127.0.0.1` listener。
- [ ] T1.2 端口分配 + bearer secret 生成，写入注入事实文件。
- [ ] T1.3 实现 `streamable-http` JSON-RPC handler（initialize/tools/list/tools/call/ping/notifications）。
- [ ] T1.4 从 Swift `HomieMCPTools.all` 迁 18 个工具 schema 到 Rust（spawnableKinds 动态推导）。
- [ ] T1.5 血统判定从 `MCPLineage.swift` 迁 Rust，caller 从 `X-Homie-Session-Id` header 读。
- [ ] T1.6 工具方法映射到内部 control 方法（session/worktree/artifacts/release 等）。
- [ ] T1.7 认证：`Authorization: Bearer` 校验，401 拒绝。

## Phase 2 — 注入改造
- [ ] T2.1 Codex：`injection_args` 改 `url`/`bearer_token_env_var`/`env_http_headers`，删 `command`/`args`。
- [ ] T2.2 Claude：`write_claude_mcp_file` 写 `type:http`，删 `mcp_launch`/`is_executable`。
- [ ] T2.3 注入 env `HOMIE_MCP_TOKEN` + `HOMIE_SESSION_ID`。

## Phase 3 — 删除 + 文档
- [ ] T3.1 删 `homie-mcp` crate（workspace 成员 + Cargo.toml）。
- [ ] T3.2 删 Swift `mcp-stdio`/`mcp-tools`/`mcp-call` 与 `Sources/HomieMCP`。
- [ ] T3.3 更新 README（进程表/模块图/图 4 时序图）。

## Phase 4 — 验证
- [ ] T4.1 单测：注入参数、血统判定、schema 对拍。
- [ ] T4.2 集成：daemon 起 endpoint，POST initialize/tools/list/tools/call + 401。
- [ ] T4.3 端到端（若环境允许）：真实 Codex/Claude 经 HTTP MCP 编排。
- [ ] T4.4 安全：无 token 泄露、仅回环可达。
- [ ] T4.5 全量 `cargo test -p homie-engine` / `swift build` 绿。
- [ ] T4.6 证据 + tag + 关 Beads `homie-gyj`。

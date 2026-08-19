# Spec Review Report — mcp-http-transport-unified

- Beads: `homie-gyj`
- change_id: `mcp-http-transport-unified`
- 日期: 2026-08-19

## 1. 范围

评审 `homie` MCP 工具 transport 从 stdio（每会话 `homie-mcp` shim + Swift CLI 子进程）
统一为 daemon 内嵌 `streamable_http` endpoint，删除 stdio 代理资产。

## 2. Phase 0 核实结果（决策 D）

**结论：采用分支 1（优先）。** Claude `.mcp.json` `type:http` 的 `headers` **支持
`${ENV_VAR}` 环境变量展开**，无需退化为 per-session 文件。

官方文档（docs.anthropic.com/en/docs/claude-code/mcp）§"Environment variable
expansion in `.mcp.json`" 明确：

| 语法 | 语义 |
|------|------|
| `${VAR}` | 展开为环境变量 `VAR` 的值 |
| `${VAR:-default}` | `VAR` 已设置则用其值，否则用 `default` |

`headers` 字段（用于 HTTP server 认证）在 env 展开的适用范围内。文档给出直接范例：

```json
{
  "type": "http",
  "url": "${API_BASE_URL:-https://api.example.com}/mcp",
  "headers": { "Authorization": "Bearer <YOUR_API_KEY>" }
}
```

> 注：`<YOUR_API_KEY>` 为占位符；官方文档原文写作 `${API_KEY}`，运行时展开为环境变量值。

因此 `write_claude_mcp_file` 可一次写入（`<...>` 为占位符，实际写入 `${VAR}` 触发 env 展开）：

```json
{
  "mcpServers": {
    "homie": {
      "type": "http",
      "url": "http://127.0.0.1:<port>/mcp",
      "headers": {
        "Authorization": "Bearer <HOMIE_MCP_TOKEN>",
        "X-Homie-Session-Id": "${HOMIE_SESSION_ID}"
      }
    }
  }
}
```

每会话 spawn 时注入 `HOMIE_MCP_TOKEN` / `HOMIE_SESSION_ID` env 即生效，无需 per-session 文件。

## 3. 额外核实（影响实现细节）

| 发现 | 影响 |
|------|------|
| `type` 字段接受 `streamable-http` 作为 `http` 的别名 | 可写 `"type":"http"`，与 PRD 决策 E 一致 |
| 有 `url` 但缺 `type` 是配置错误（Claude Code 把无 `type` 条目读成 stdio） | 注入时必须显式写 `"type":"http"`，不能省略 |
| `headers` 值若有首尾空白，Claude Code 会告警且不 trim，按原样使用 | 注入逻辑生成 header 时不得带多余空白 |
| 动态 headers 可用 `headersHelper`（连接时生成），会覆盖同名静态 header | 本方案用 `${ENV_VAR}` 展开即可，无需 headersHelper |

## 4. 技术选型评审（可行性）

| 决策 | 结论 | 依据 |
|------|------|------|
| daemon 内嵌 MCP HTTP listener（不新增进程） | **采纳** | daemon 本就常驻并持有会话注册表/PTY/事件流（权威数据源）；复用 `homie-gateway` 的 axum/hyper 栈 |
| 工具实现收敛 Rust daemon（删 Swift stdio 代理） | **采纳** | 工具逻辑贴近权威数据源；删 `homie-mcp` shim 每会话 -1 常驻进程 + 每工具调用 -1 Swift 短进程 |
| 会话身份经 `X-Homie-Session-Id` header 传递 | **采纳** | Codex `env_http_headers` 已确认「请求时读 env」（源码 `codex-rs/config/src/mcp_types.rs`）；Claude `headers` 支持 `${ENV_VAR}` 展开 |
| 认证经 `Authorization: Bearer`（daemon 级随机 secret） | **采纳** | 每 daemon 生命周期随机、仅回环可用、不落日志/SQLite |
| endpoint 协议 streamable HTTP JSON-RPC 2.0 | **采纳** | 与 MCP spec 一致；当前工具均一次性返回，可先只做请求/响应，SSE 留待长任务 |

## 5. 组件合同评审

`specs/mcp-transport.md` 新增，评审结论：

- 端点路径 `/mcp`、JSON-RPC 方法（initialize/tools/list/tools/call/notifications）、
  认证 header、会话身份 header、18 工具清单、血统判定、错误码与 PRD FR-1~FR-3 对齐。
- 工具语义不变（read 开放、write 归属、whoami 策略文本），仅 transport 与服务端语言迁移。

## 6. 依赖评估

| 依赖 | 状态 | 理由 |
|------|------|------|
| `axum`/`hyper`/`tower` | 已有（homie-gateway 选型） | daemon 内嵌 listener 复用，不新增 |
| `tokio` | 已有 | runtime 线程（同 gateway 内嵌方式） |
| manifest 目录 `AgentCatalog`/`launchable` | 已有 | 派生 `spawnableKinds` |
| 删除 `homie-mcp` crate | 待执行 | workspace 成员 + Cargo.toml 引用 |
| 删除 Swift `mcp-stdio`/`mcp-tools`/`mcp-call` | 待执行 | `Sources/homie-cli` + `Sources/HomieMCP` |

## 7. 结论

Phase 0 核实完成：决策 D 确定分支 1（Claude headers 支持 env 展开）。技术选型与
组件合同评审通过，与 PRD FR-1~FR-6 及 OpenSpec 任务 T1~T4 对齐，可进入 Phase 1 实现。

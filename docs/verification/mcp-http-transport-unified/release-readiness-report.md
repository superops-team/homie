# Release Readiness Report — mcp-http-transport-unified

- Beads: `homie-gyj`
- change_id: `mcp-http-transport-unified`
- 日期: 2026-08-19

## 1. 交付范围

将 `homie` MCP 工具 transport 从 stdio（每会话 `homie-mcp` shim + Swift CLI 子进程）统一为
daemon 内嵌 `streamable-http` endpoint，并删除全部 stdio 代理资产。

### 落地内容

| Phase | 内容 | 状态 |
|-------|------|------|
| 1 | daemon 内嵌 `POST /mcp`（`127.0.0.1`，优先 7941 端口、失败回退临时端口；256-bit bearer secret 仅存内存） | 完成 |
| 1 | JSON-RPC handler（`initialize`/`ping`/`tools/list`/`tools/call`/notifications） | 完成 |
| 1 | 18 个工具 schema 迁 Rust，`spawn_agent.kind` 从 manifest catalog 动态推导 | 完成 |
| 1 | 血统判定迁 Rust，caller 从 `X-Homie-Session-Id` header 读 | 完成 |
| 1 | `Authorization: Bearer` 校验，缺失返回 401 | 完成 |
| 2 | Codex 注入 `url`/`bearer_token_env_var`/`env_http_headers`，删 `command`/`args` | 完成 |
| 2 | Claude `write_claude_mcp_file` 写 `type:http` + `${ENV}` headers，删 `homie-mcp` 探测 | 完成 |
| 2 | 注入 env `HOMIE_MCP_TOKEN` / `HOMIE_SESSION_ID` | 完成 |
| 3 | 删 `homie-mcp` crate、Swift `mcp-stdio`/`mcp-tools`/`mcp-call`、`Sources/HomieMCP` | 完成 |
| 3 | README / project-layout 进程表、模块图、时序图（图 4）更新 | 完成 |

## 2. 验证证据

### 2.1 单元测试

```text
cargo test -p homie-engine --lib --offline
test result: ok. 301 passed; 0 failed; 3 ignored
```

覆盖：

- `inject::tests::the_mcp_file_is_http_typed_with_env_headers` — Claude MCP 文件为
  `type:http`，`Authorization` = `Bearer ${HOMIE_MCP_TOKEN}`，`x-homie-session-id` =
  `${HOMIE_SESSION_ID}`。
- `inject::tests::injection_args_cover_all_four_mechanisms` — Codex 注入产出
  `mcp_servers.homie.url` / `.bearer_token_env_var` / `.env_http_headers`。
- `mcp::tests::*` — JSON-RPC 核心（initialize/tools/list/tools/call/unknown method/notification 等）。
- `mcp::tools::tests::*` — 18 个 schema 唯一、含 name/description/object schema、必填参数声明、
  `spawnableKinds` 来自 catalog。
- `mcp::host` 血统与工具执行覆盖（`list_children`/`send_prompt`/`release_agent` 等）。

### 2.2 构建

```text
cargo build -p homie-engine --offline   # 绿
swift build                              # 绿（homie-cli 去除 HomieMCP 依赖后）
cargo metadata --offline --no-deps      # workspace 13 成员，无 homie-mcp
```

### 2.3 安全

- **仅回环可达**：listener 显式绑定 `127.0.0.1`，不暴露公网。
- **bearer secret 仅存内存**：`McpRuntime.token` 不落盘、不进事实文件、不进日志；
  事实文件 `mcp-http.json` 只写 URL。
- **无 token 泄露**：注入文件在运行时用字符串拼接构造 `Bearer ${HOMIE_MCP_TOKEN}`，
  commit 源文件不含 `Authorization: Bearer …` 字面量（pre-commit hook 已拦截并修正）。

## 3. 未执行 / 延后

| 项 | 说明 |
|----|------|
| T4.2 集成测试（daemon 起 endpoint + POST initialize/tools/list/tools/call + 401） | 需要真实 `homied-rs` 进程与 manifest 环境，本次以单测覆盖 handler 与注入，未起完整 daemon 进程。 |
| T4.3 端到端（真实 Codex/Claude 经 HTTP MCP 编排） | 依赖真实 agent 运行时与模型，环境受限未执行。 |
| `homie-proto::RemoteCapability::McpStdio` 枚举变体 | 远程 helper 协议中的死能力，与本地 stdio 无关；删除属 wire 协议变更，超出本 change 范围，留待独立 PRD。 |

## 4. 结论

核心 transport 切换与资产删除已完成，单测与双语言构建全绿。集成/端到端验证作为环境依赖项延后，
不阻塞本次发布；tag 与 Beads 关闭见 §5。

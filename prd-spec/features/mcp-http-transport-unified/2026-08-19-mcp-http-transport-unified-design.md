# Homie MCP HTTP 统一 transport 设计文档

## 1. 概述

### 1.1 问题/背景

当前 Homie 的 agent 编排 MCP（`homie` MCP tools，共 18 个工具：`spawn_agent`、`list_agents`、
`get_status`、`send_prompt`、`wait_for_agent`、`read_output`、`create_worktree`、`list_worktrees`、
`remove_worktree`、`list_children`、`wait_for_children`、`summarize_children`、`report_to_parent`、
`get_artifacts`、`release_agent`、`whoami`、`browser`、`test_run`）通过 **stdio transport** 注入：

- Claude Code 通过 `claude-mcp.json`（`{"type":"stdio","command":<homie-mcp>,"args":[]}`）加载；
- Codex 通过 `-c "mcp_servers.homie.command=…"` / `-c "mcp_servers.homie.args=[…]"` 加载；
- 每个 agent 会话各自 spawn 一个 **`homie-mcp`**（Rust shim）进程，shim 再按需 spawn
  `homie mcp-tools` / `homie mcp-call`（Swift CLI）子进程做工具执行，最后经 control socket 转发
  到 daemon（`homied-rs`）。

问题：

1. **进程数量随会话线性增长**：每个 agent 会话常驻一个 `homie-mcp` shim + 每个工具调用临时
   一个 Swift CLI 子进程。10 个会话 = 10 个 shim + 频繁的 Swift 冷启动，通信与安装维护成本高。
2. **Rust `homie-mcp` 只是转发壳**：它不实现任何工具，只是把 JSON-RPC 转成 CLI 子命令
   （`mcp-tools` / `mcp-call`），再等 Swift 进程去连 daemon。这是一个纯冗余的中间层。
3. **工具实现割裂在 Swift**：真正的工具 schema 与桥接逻辑在 `Sources/HomieMCP/Tools.swift` 和
   `Sources/homie-cli` 的 `MCPBridge` / `MCPLineage`，而 daemon（`homie-engine`）才是权威，持有
   会话注册表、PTY、事件流。工具逻辑本应贴近权威数据源，却绕了一大圈。
4. **会话血统依赖 stdio 的 env 继承**：`MCPLineage.swift` 靠 agent 进程继承的 `HOMIE_SESSION_ID`
   env 计算 parent/child/ancestor/descendant。改用 HTTP 后该 env 不再自动注入到 MCP 进程，
   必须显式解决 session identity 的传递。

用户已明确：**统一走 MCP HTTP transport，不再 stdio**，以减少桌面软件多进程的通信与维护成本。

### 1.2 已核实的可行性事实

**Codex**（源码 `~/workspace/github/codex`，`codex-rs/config/src/mcp_types.rs`）原生支持 MCP
`streamable_http` transport，**只需改配置文件**，无需改 Codex 源码。关键结构：

- transport 由字段推断：有 `command` 即 stdio，有 `url` 即 streamable_http（无显式 `transport`
  键）；
- `StreamableHttp { url, bearer_token_env_var, http_headers, env_http_headers, http_headers_helper }`：
  - `url`：MCP HTTP 端点；
  - `bearer_token_env_var`：存 bearer token 的**环境变量名**，Codex 请求时发 `Authorization: Bearer <token>`；
  - `http_headers`：静态 header map；
  - `env_http_headers`：`header_name -> env_var_name` 映射，**请求时从环境变量读值**放进对应 header；
  - `http_headers_helper`：本地 shell 命令打印动态 header JSON（仅本地 server 支持）。
- 这正好解决会话血统与认证：`env_http_headers = { "X-Homie-Session-Id" = "HOMIE_SESSION_ID" }`
  传递会话身份，`bearer_token_env_var = "HOMIE_MCP_TOKEN"` 传递认证。

**Claude Code** `.mcp.json` 支持 `"type": "http"`（streamable HTTP），字段 `url` + `headers`
（静态对象）。**需在实现期核实**：headers 是否支持 `${ENV_VAR}` 展开，以决定会话身份的传递方式
（见 §1.5 决策 D 与 §7 风险）。若不支持，则退化为「按会话写 per-session `claude-mcp-<uuid>.json`」
或「daemon 级共享 token + 其它机制传 session id」，PRD 已预留该分支。

### 1.3 目标

1. 在 **daemon（`homie-engine`）内嵌一个 MCP `streamable_http` endpoint**，作为 `homie` MCP
   工具的唯一服务端，**不新增任何常驻进程**。
2. 删除 `homie-mcp` Rust shim crate 与 Swift `mcp-stdio` / `mcp-tools` / `mcp-call` 的 stdio 代理
   路径，工具实现收敛到 Rust daemon（权威数据源就地）。
3. 注入改为 HTTP：Codex 走 `-c` 覆盖（`url` + `bearer_token_env_var` + `env_http_headers`），
   Claude 走 `--mcp-config` 的 `type:http`。
4. 显式传递 session identity 与认证：HTTP 层用 `X-Homie-Session-Id` header 承载 `HOMIE_SESSION_ID`，
   bearer token 承载认证，替代原 stdio env 继承。
5. 保持工具语义与血统（lineage）判定行为不变（read 开放、write 归属规则、whoami 策略说明）。

### 1.4 非目标

- 不改 MCP 工具的**语义/能力**（18 个工具的功能不变，只换 transport 与服务端语言）。
- 不做 MCP 的多租户/远程暴露；endpoint 仅 `127.0.0.1` 回环，不对外。
- 不引入独立 MCP gateway 进程（违背「减少进程」目标）。
- 不做 MCP 的 OAuth / ChatGPT login 流程（Codex 的 `auth`/`oauth` 字段不启用，用 bearer token）。
- 不处理其它 agent（Cursor / Gemini / OpenCode 等）的 MCP 注入（本次只覆盖 Claude Code 与 Codex）。
- 不改 hooks / notify 机制（仍是 stdio/env，不在本 PRD 范围）。

### 1.5 关键设计决策

#### 决策 A：daemon 内嵌 MCP HTTP endpoint（不新增进程）

daemon（`homied-rs`）本就常驻、持有 owner-only control socket、会话注册表、PTY 与事件流，是
MCP 工具的权威数据源。在 daemon 内加一个 `127.0.0.1:<port>` 的 HTTP listener（复用现有 HTTP 栈，
如 axum/hyper，与 `homie-gateway` 一致），把 18 个工具的方法直接实现为 control 内部调用，**零新增
进程**，且删除每会话 `homie-mcp` 后净减少 N 个进程。

- 端口在 daemon 启动时分配（优先固定端口，冲突则动态），并连同 daemon 级 bearer secret 写入
  一个 well-known 的注入事实文件（如 inject dir 下 `mcp-endpoint.json`），供注入逻辑读取。
- bearer secret 为**每 daemon 生命周期随机**生成（或每会话签发），仅回环可用，不落日志。

#### 决策 B：工具实现收敛到 Rust daemon，删除 stdio 代理层

- 在 `homie-engine` 新增 `mcp` 模块（或复用 `control`），把 Swift `HomieMCPTools.all`（18 个工具的
  schema）与 `MCPBridge.handle`（转发到 control 方法）用 Rust 重写为 HTTP handler。
- 工具 schema 仍从 manifest 目录推导 `spawnableKinds`（保持「agent 支持是数据不是代码」）。
- 删除 `homie-mcp` crate、`homie/crates/homie-mcp/`、以及 `Sources/homie-cli/Homie.swift` 的
  `McpStdio` / `McpTools` / `McpCall` 与 `Sources/HomieMCP` 的 Swift 工具定义（或改为仅供测试/文档）。
- 血统逻辑从 `MCPLineage.swift` 移植为 Rust，会话身份改从 `X-Homie-Session-Id` header 读取。

#### 决策 C：session identity 显式传递

- HTTP 端点从 `X-Homie-Session-Id` header 读 caller session id，等价于原 `HOMIE_SESSION_ID` env。
- Codex：`env_http_headers = { "X-Homie-Session-Id" = "HOMIE_SESSION_ID" }`，注入逻辑继续在 spawn
  时把 `HOMIE_SESSION_ID` 写入 agent env（已有能力），Codex 自动把它搬进 header。
- 认证：`bearer_token_env_var = "HOMIE_MCP_TOKEN"`，daemon 注入该 env（每会话或每 daemon token）。

#### 决策 D：Claude headers 环境变量展开（实现期核实，预留两分支）

- 分支 1（优先）：Claude `.mcp.json` 的 `headers` 支持 `${ENV_VAR}` 展开 → 写入
  `claude-mcp.json`（`type:http` + `url` + `headers: {"Authorization": "Bearer <HOMIE_MCP_TOKEN>",
  "X-Homie-Session-Id": "${HOMIE_SESSION_ID}"}`），一次写入、每会话 env 注入即生效。
- 分支 2（回退）：若 headers 是纯静态字符串 → 按会话写 per-session 文件
  `claude-mcp-<uuid>.json`（session uuid 在 spawn 时已由 `uuid_v4()` 造好），`--mcp-config` 指向
  该文件；认证 token 仍经 daemon 签发并烘焙进该文件 header。
- 无论哪个分支，`write_claude_mcp_file` 与 `injection_args` 的 stdio 分支都要改成 HTTP。

#### 决策 E：endpoint 协议为 streamable HTTP MCP（JSON-RPC 2.0）

- 遵循 MCP `streamable-http` transport（`initialize` / `tools/list` / `tools/call` / `notifications/*`）。
- 端点路径 `/mcp`（或 `/`），`POST` JSON-RPC，`GET` 用于 SSE 流（若工具需要流式；当前工具均
  一次性返回，可先只实现请求/响应，SSE 留待需要长任务时再开）。
- `tools/list` 返回 18 个工具 schema（与 Swift 版一致）；`tools/call` 解析参数后调内部 control 方法。

### 1.6 进程影响

| 改动 | 影响 |
|---|---|
| 删除 `homie-mcp`（每会话一个 shim） | 每会话 -1 常驻进程 |
| 删除 Swift `mcp-tools`/`mcp-call` 临时子进程 | 每工具调用 -1 短生命周期 Swift 进程 |
| daemon 内嵌 HTTP listener | daemon 内 +1 线程/task，**无新进程** |
| 净效果 | 会话数 N 时，常驻进程减少 N，短生命周期进程减少 O(工具调用数) |

## 2. 用户场景

### 场景 1：Codex 会话用 HTTP MCP 编排其它 agent

**Given** 用户在 Homie 里开一个 Codex 会话。  
**When** daemon 注入 `mcp_servers.homie.url` + `bearer_token_env_var` + `env_http_headers`，并把
`HOMIE_SESSION_ID` / `HOMIE_MCP_TOKEN` 写入 agent env。  
**Then** Codex 直接连 `http://127.0.0.1:<port>/mcp`，无需本地 `homie-mcp` 进程，可调用
`spawn_agent` / `send_prompt` 等编排工具。

### 场景 2：Claude Code 会话用 HTTP MCP 编排其它 agent

**Given** 用户开一个 Claude Code 会话。  
**When** daemon 注入 `--mcp-config` 指向 `type:http` 的 `claude-mcp.json`（或 per-session 文件），
并注入 `HOMIE_SESSION_ID` / `HOMIE_MCP_TOKEN` env。  
**Then** Claude 直连 HTTP 端点编排其它 agent，无 stdio 进程。

### 场景 3：跨会话写仍带血统归属

**Given** 会话 A（HTTP MCP）向无关会话 C 发 `send_prompt`。  
**When** 端点从 `X-Homie-Session-Id` 识别 caller=A，判定 relation=unrelated。  
**Then** 写入被加 provenance 前缀（与现状一致），接收方能识别来源。

### 场景 4：无 token 的请求被拒绝

**Given** 一个未携带正确 bearer token 的请求打到 `/mcp`。  
**When** 端点校验 `Authorization`。  
**Then** 返回 401，不泄露任何内部信息。

## 3. 功能需求

### FR-1: daemon 内嵌 MCP streamable HTTP endpoint

- `homie-engine` 新增 HTTP listener（`127.0.0.1:<port>`），端口与 bearer secret 在启动时确定，
  并写入注入事实文件供注入逻辑读取。
- 端点实现 MCP `streamable-http`：`POST /mcp` JSON-RPC（`initialize`/`tools/list`/`tools/call`/
  `notifications/cancelled`），返回 JSON-RPC 响应。
- `tools/list` 返回 18 个工具的 schema，`spawnableKinds` 从 manifest 目录动态推导。

### FR-2: 工具实现收敛到 Rust daemon

- 18 个工具的方法调用直接映射到现有 `control` 方法（session spawn/list/status、PTY send_text、
  worktree、artifacts、release 等），不再经 Swift CLI。
- 血统判定（parent/child/ancestor/descendant/sibling/unrelated 与 `frame` 归属）移植为 Rust，
  caller 从 `X-Homie-Session-Id` header 读取（无 header 视作 trusted，写 verbatim，同现状 nil caller）。
- `whoami` 返回的 `writePolicy` 文本保持与 Swift 版一致。

### FR-3: 认证与会话身份

- 每个请求校验 `Authorization: Bearer <secret>`，不匹配返回 401。
- 会话身份从 `X-Homie-Session-Id` header 解析；缺失/非法视为无 caller（trusted）。

### FR-4: 注入改造（Codex + Claude → HTTP）

- Codex（`codex_mcp` 开启）：
  - `-c "mcp_servers.homie.url=<endpoint>/mcp"`
  - `-c "mcp_servers.homie.bearer_token_env_var=HOMIE_MCP_TOKEN"`
  - `-c "mcp_servers.homie.env_http_headers={ X-Homie-Session-Id = HOMIE_SESSION_ID }"`
  - 注入 env：`HOMIE_MCP_TOKEN`、`HOMIE_SESSION_ID`（后者已有）。
  - **删除** 原 `mcp_servers.homie.command` / `args` 的 stdio 注入。
- Claude（`claude_mcp` 开启）：
  - `write_claude_mcp_file` 写 `type:http` 的 `claude-mcp.json`（分支见决策 D），`--mcp-config` 指向它。
  - 注入 env：`HOMIE_MCP_TOKEN`、`HOMIE_SESSION_ID`。
  - **删除** 原 `homie-mcp` sibling 探测与 `homie mcp-stdio` fallback。
- 移除 `mcp_launch` / `is_executable` 中对 `homie-mcp` 的依赖。

### FR-5: 删除 stdio 代理资产

- 删除 `homie/crates/homie-mcp/`（crate + `Cargo.toml` 成员 + workspace 引用）。
- 删除 `Sources/homie-cli/Homie.swift` 的 `McpStdio` / `McpTools` / `McpCall` 子命令及相关命令注册。
- 删除/迁移 `Sources/HomieMCP`（若工具 schema 全部迁 Rust，则删除；`MCPBridge`/`MCPLineage` 迁 Rust）。
- 更新 README 进程表、模块图与 MCP 时序图（§4 agent 编排 MCP 从 stdio 改 HTTP）。

### FR-6: 安全

- bearer secret 与 `X-Homie-Session-Id` 不进日志、不写 SQLite、不回显。
- endpoint 仅绑定 `127.0.0.1`，不暴露局域网/外网。
- 删除资产后确保无残留 `homie-mcp` 二进制被注入。

## 4. 受影响 Specs

- 新增 `specs/mcp-transport.md`：MCP HTTP endpoint 契约（路径、JSON-RPC、认证 header、会话身份
  header、工具清单、血统判定、错误码）。
- 更新 `specs/engine-session-runtime.md`：daemon 增加 HTTP listener 与 endpoint 生命周期。
- 更新 `specs/homie-cli-config-ops.md`（若涉及）：删除 `mcp-stdio`/`mcp-tools`/`mcp-call` 子命令。
- 更新 `README.md`：进程表、模块图、MCP 时序图（图 4）。

## 5. 测试计划

- 单测：注入参数生成（Codex `-c` 覆盖为 `url`/`bearer_token_env_var`/`env_http_headers`，无 `command`；
  Claude `type:http` + headers）。
- 单测：`X-Homie-Session-Id` 解析与血统判定（移植自 `MCPLineage.swift` 的用例）。
- 集成：起 daemon，`POST /mcp` 做 `initialize`/`tools/list`/`tools/call`（spawn/list/send_prompt），
  校验返回结构与 401 拒绝。
- 端到端：真实 Codex/Claude 会话通过 HTTP MCP 成功编排另一个会话（若环境允许）。
- 安全：断言日志/SQLite 不含 bearer token 或 session id；endpoint 非回环不可达。
- 全量 `cargo test -p homie-engine --offline` 绿；删除 `homie-mcp` 后 `cargo build` 无残留引用。

## 6. 验收标准

- Codex 与 Claude Code 会话均通过 HTTP MCP 调用 `spawn_agent`/`send_prompt` 成功编排另一会话，
  且**无** `homie-mcp` 进程 spawn（`ps` 验证）。
- `tools/list` 返回 18 个工具；跨会话写归属与 `whoami` 策略文本与现状一致。
- 无正确 token 的请求返回 401；endpoint 仅回环可达。
- 删除 `homie-mcp` crate 与 Swift stdio 代理后，`cargo build` / `swift build` 均通过。
- 证据齐全：`docs/verification/mcp-http-transport-unified/`（spec-review / functional-cases /
  functional-verification / code-review / release-readiness）。

## 7. 风险与待核实项

| 风险/待核实 | 影响 | 缓解 |
|---|---|---|
| Claude `.mcp.json` `headers` 是否支持 `${ENV_VAR}` 展开 | 决定决策 D 分支 | 实现期先验证；不支持则用 per-session 文件（分支 2） |
| Codex `env_http_headers` 是否真的在请求时读 env（而非启动时） | 会话身份新鲜度 | 已从源码确认语义为「请求时读 env」，实现期再以真实会话验证 |
| 端口冲突 | endpoint 起不来 | 固定端口失败则动态分配并写事实文件 |
| 工具 schema 迁移遗漏 | 能力回退 | 以 Swift `HomieMCPTools.all` 为金本逐一对拍，写 schema 对比测试 |
| 删除 Swift MCP 影响其它调用方 | 回归 | 全量 `swift build` + 现有 CLI 测试 |

## 8. Beads 追踪

- Beads: `homie-gyj`
- change_id: `mcp-http-transport-unified`
- 类型: feature
- 优先级: P0

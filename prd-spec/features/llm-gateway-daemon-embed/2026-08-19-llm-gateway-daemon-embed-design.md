# Homie LLM proxy 内嵌 daemon + 协议收敛 OpenAI-only 设计文档

## 1. 概述

### 1.1 问题/背景

Homie 当前本地常驻进程包括：`homied-rs`（daemon）、`homie-gateway`（独立 LLM 代理进程）、
`homie-mcp`（每会话 MCP shim）、`homie-holder`（PTY 存活边界）。其中：

- `homie-gateway` 是一个**独立 axum 进程**，同时服务两套 wire 协议：`POST /v1/responses`
  （OpenAI Responses，Codex）与 `POST /v1/messages`（Anthropic Messages，Claude Code）。
- 用户诉求：把 **MCP + LLM proxy 统一内嵌进 daemon**，本地常驻收敛到「daemon + CLI + GUI」的
  轻量架构；同时 **LLM 协议只保留 OpenAI 一种**，放弃 Anthropic Messages，让所有 OpenAI 兼容
  agent 统一纳管。

本 PRD 落实 LLM proxy 内嵌 + 协议收敛（MCP 内嵌已在 `mcp-http-transport-unified` 单独立项）。
两者共享同一目标——daemon 成为唯一本地 HTTP 面——但 transport 层独立，故拆分推进、互不阻塞。

### 1.2 已核实的可行性事实

1. `homie-gateway` 已是「lib + 薄 bin」结构：`lib.rs` 暴露 `auth/config/db/policy/routes/state/
   upstream/usage` 九个模块，`main.rs` 只做 `inject` 预览与 `serve()`（读配置→开 Db→建 Upstream→
   建 AppState→`axum::serve`）。
2. `homie-gateway` 对 `homie-engine` 的依赖**仅**存在于 `inject.rs`（re-export `homie_engine::inject`），
   `routes/upstream/auth/db/policy/usage` 均不依赖 engine。删除该 re-export 即可打破循环，使
   `homie-engine → homie-gateway(lib)` 成为可能。
3. daemon（`homie-engine`）目前**未接入 gateway**：`homied-rs.rs` 里 `gateway: None` 硬编码，
   engine 的 `Cargo.toml` 无 tokio/axum/reqwest/rusqlite 依赖。即 LLM 纳管链路至今是半成品，
   内嵌改造无存量行为破坏。
4. gateway 的 db/usage/policy/auth 均为**同步 rusqlite**（`Arc<Mutex<Connection>>`），只有 axum
   handler 与 upstream 转发是 async；这与 daemon 的同步线程模型天然契合，只需额外起一个
   tokio runtime 线程 host axum。
5. Claude Code 的 manifest 里 `claudeGateway` 已是 `false`，未启用 LLM 纳管；删 Anthropic 协议
   与 `claude_gateway` 注入无存量行为破坏。

### 1.3 目标

1. 把 `homie-gateway` 从独立进程收敛为**库 crate**，删除其 bin（`main.rs`），由 daemon 内嵌调用，
   本地常驻进程 -1。
2. 协议收敛：**只保留 OpenAI Responses**（`POST /v1/responses`），删除 Anthropic Messages
   （`POST /v1/messages`）及其路由/注入/模型映射。
3. Claude Code 退出 LLM 纳管：删除 `claude_gateway` 注入路径与 `ANTHROPIC_*` env 注入；
   Claude Code 保留 hooks + MCP 编排，流量回归原生 Anthropic 凭证（`~/.claude`）。
4. virtual key 签发内聚到 daemon：spawn 时 daemon 直接调用 `GatewayApiKeyStore.create` 签发
   `sk-…` 并注入 agent env，不再依赖 gateway 的 `/admin/keys` HTTP 面。
5. 统一 OpenAI 纳管面：任何 manifest 声明 `codexGateway`（或泛化后的 `gateway`）的 agent 都
   走同一 OpenAI Responses 协议（本 PRD 只收敛协议与内嵌，具体新 agent 的 manifest 适配属后续）。

### 1.4 非目标

- 不做 MCP 内嵌（`mcp-http-transport-unified` 已单独立项）。
- 不扩展具体新 agent 的 manifest（只收敛协议面，OpenAI 兼容 agent 的逐个纳管属后续 feature）。
- 不改变虚拟 key 模型、用量记录、策略/配额、模型路由的**语义**（仅删 Claude 分支）。
- 不动 `homie-node` 的凭证解析（`credentials` 模块与 `credentialSource: node` 保留）。
- 不做 Anthropic Messages ↔ OpenAI Responses 的协议翻译（放弃即删除，不兼容 Claude）。
- 不改 hooks / notify / 远程 SSH 链路。

### 1.5 关键设计决策

#### 决策 A：`homie-gateway` 降级为纯库，daemon 内嵌调用（不搬代码）

保留 `homie-gateway` crate 作为**库**，删除 `[[bin]]` 与 `main.rs`，让 `homie-engine` 依赖
`homie_gateway` 库。daemon 起一个 tokio runtime 线程 `axum::serve(homie_gateway::routes::router(state))`。

- 优点：零代码搬移，`routes/upstream/auth/db/policy/usage` 原地复用，测试原地保留。
- 打破循环：删除 `homie-gateway/src/inject.rs`（re-export）与 Cargo 里 `homie-engine` 依赖；
  `inject` 逻辑本就全部在 `homie-engine`，gateway 不再需要它。
- 新依赖方向：`homie-engine → homie-gateway → (homie-node, homie-usage)`。

#### 决策 B：daemon 新增 tokio runtime 线程 host axum

- daemon 现有 `std::thread` 同步 accept 循环（control socket）不动。
- 新增一个 dedicated `tokio::runtime::Runtime`（`new_multi_thread`，1~2 worker），在 `std::thread`
  里 `rt.block_on(axum::serve(listener, router))`。
- engine `Cargo.toml` 新增 `tokio`/`axum`/`reqwest`/`rusqlite` 依赖（对齐 gateway 现有版本）。
- 同步 store（`GatewayApiKeyStore`/`UsageStore`/`Db`）由 axum handler 跨 `std::sync::Mutex` 共享，
  与现状一致（gateway 里就是 `Arc<Mutex<Connection>>`，axum handler 里直接锁，无跨线程异步问题）。

#### 决策 C：协议收敛 OpenAI-only，删 Anthropic

- 删 `POST /v1/messages` 路由、`handle_messages`、`route_key("/messages")→Some("claude")`、
  `claude` 模型路由分支。
- 删 `homie_engine::inject` 的 `claude_gateway_env`（`ANTHROPIC_BASE_URL`/`ANTHROPIC_AUTH_TOKEN`）
  与 `InjectionSpec.claude_gateway` 字段、`gateway_env` 里的 claude 分支。
- `models` map 收敛为单键 `codex`（或改为通用 `default`/`codex` 单一模型路由，实现期定）。
- `specs/llm-gateway.md` 的 §2/§5/§7 移除 Anthropic 契约。

#### 决策 D：virtual key 签发内聚 daemon spawn

- daemon 持有 `AppState`（含 `GatewayApiKeyStore`），`ControlServer`/`InjectionConfig` 增加对该
  state 的访问。
- spawn agent（manifest 声明 gateway opt-in）时，daemon 调 `keys.create(Some(session_id))` 签发
  `sk-…`，构造 `GatewayRuntime { base_url, virtual_key }` 注入。
- 删除独立 gateway 的 `/admin/keys` HTTP 面（或降级为仅 debug/doctor 的本地 CLI 手段，实现期定）。
- master key 语义保留：作为本地 debug/doctor 的认证凭据，但不再承担签发主流程。

#### 决策 E：端口与配置归属

- `homie.local.json` 的 `gateway.listen`（默认 `127.0.0.1:7338`）仍由 daemon 读取；daemon 成为
  该端口的唯一监听者。
- daemon 启动失败（端口占用）沿用现有「AddrInUse → 视为 singleton 已运行」语义，但需区分
  LLM 端口占用与 control socket 占用（实现期定）。

### 1.6 进程影响

| 改动 | 影响 |
|---|---|
| `homie-gateway` 独立 bin 删除，降级为库 | 本地常驻 -1 进程 |
| daemon 内嵌 axum（/v1/responses） | daemon 内 +1 tokio 线程，无新进程 |
| 删 Anthropic Messages | 协议面 -1（Claude Code 退出 LLM 纳管） |
| virtual key 签发内聚 | 删 `/admin/keys` HTTP 面，spawn 时内嵌签发 |

最终本地常驻（配合 MCP 内嵌后）：`homied-rs`（内嵌 control + MCP + LLM）+ `homie-holder`；
GUI/CLI 为前台/on demand。

## 2. 用户场景

### 场景 1：Codex 走 daemon 内嵌 LLM proxy 纳管

**Given** 用户开一个 Codex 会话（manifest `codexGateway: true`）。  
**When** daemon spawn 时签发虚拟 key，注入 `HOMIE_CODEX_GATEWAY_KEY` 与 `-c model_provider=homie`
等覆盖。  
**Then** Codex 直连 `http://127.0.0.1:7338/v1/responses`，无独立 `homie-gateway` 进程，流量被
纳管、用量被记录、策略被应用。

### 场景 2：Claude Code 回归原生 Anthropic 凭证

**Given** 用户开一个 Claude Code 会话。  
**When** daemon 不再注入 `ANTHROPIC_BASE_URL`/`ANTHROPIC_AUTH_TOKEN`。  
**Then** Claude Code 用自己 `~/.claude` 的原生凭证直连 Anthropic；hooks 编排与 MCP 编排仍可用，
但该流量不进入 Homie 用量/配额。

### 场景 3：OpenAI 兼容 agent 统一纳管

**Given** 一个 OpenAI 兼容 agent 的 manifest 声明 gateway opt-in。  
**When** daemon 注入同一 OpenAI Responses 协议。  
**Then** 该 agent 与 Codex 走完全相同的代理、用量、策略、模型路由路径。

### 场景 4：无独立 gateway 时 daemon 仍可编排

**Given** 用户启动 Homie。  
**When** daemon 起来后内嵌 LLM proxy 监听。  
**Then** 无需任何额外进程，即可完成 MCP 编排 + LLM 代理。

## 3. 功能需求

### FR-1: `homie-gateway` 降级为库

- 删除 `homie/crates/homie-gateway/src/main.rs` 与 `[[bin]]`；保留 `lib.rs` 九个模块。
- 删除 `homie-gateway/src/inject.rs` 与 Cargo 里 `homie-engine` 依赖。
- `homie-engine` 新增 `homie-gateway` 依赖（workspace）。

### FR-2: daemon 内嵌 LLM proxy listener

- daemon 新增 tokio runtime 线程，`axum::serve(homie_gateway::routes::router(state))` 绑定
  `homie.local.json` 的 `gateway.listen`。
- `AppState` 由 daemon 构造并持有（Db 打开 `gateway.sqlite3`、Upstream、master_key、models、policy）。
- daemon 启动/端口错误语义明确（区分 control socket 与 LLM 端口占用）。

### FR-3: 协议收敛 OpenAI-only

- 删 `POST /v1/messages` 路由与 `handle_messages`。
- 删 `route_key` 的 `claude` 分支，`models` 收敛为单一 OpenAI 模型路由键。
- 删 `homie_engine::inject::claude_gateway_env` 与 `InjectionSpec.claude_gateway`、`gateway_env` 的
  claude 分支。

### FR-4: virtual key 签发内聚 daemon

- daemon spawn（manifest gateway opt-in）时调 `GatewayApiKeyStore.create` 签发虚拟 key，构造
  `GatewayRuntime` 注入。
- 删除独立 gateway 的 `/admin/keys` HTTP 面（或降级为 debug/doctor CLI）。
- `InjectionConfig.gateway` 的 `None` 硬编码移除，改为由 daemon 实际签发/填充。

### FR-5: 文档与 Spec 收敛

- 更新 `specs/llm-gateway.md`：§2/§5/§7 删 Anthropic 契约，§3 补「虚拟 key 由 daemon spawn 内嵌签发」。
- 更新 README：进程表（删 homie-gateway 独立进程）、模块图、时序图（图 5 LLM gateway 改为 daemon 内嵌）。

## 4. 受影响 Specs

- `specs/llm-gateway.md`：协议收敛（删 Anthropic）、虚拟 key 签发归属、daemon 内嵌契约。
- `specs/engine-session-runtime.md`：daemon 新增 LLM proxy listener 生命周期。
- `README.md`：进程表、模块图、时序图。

## 5. 测试计划

- 单测：`route_key` 无 claude 分支；`models` 单键路由；`injection_args` 无 claude 注入；
  `GatewayRuntime` 由 daemon 签发。
- 集成：daemon 起 LLM proxy，`POST /v1/responses` 转发 + 用量记录 + 策略；`/v1/messages` 返回 404。
- 端到端：Codex 经 daemon 内嵌 proxy 完成一次转发；Claude Code 不再注入 ANTHROPIC_* env。
- 安全：虚拟 key 不回显、不落日志；`/admin/keys` 移除后无 HTTP 签发面。
- 全量 `cargo test -p homie-gateway -p homie-engine --offline` 绿；`cargo build` 无 `homie-gateway` bin。

## 6. 验收标准

- 无 `homie-gateway` 独立进程，daemon 内嵌 `POST /v1/responses` 正常代理 Codex。
- `POST /v1/messages` 返回 404；Codex 走 OpenAI 协议纳管，Claude Code 走原生 Anthropic 且不注入
  `ANTHROPIC_*`。
- daemon spawn 时内嵌签发虚拟 key，无 `/admin/keys` HTTP 面。
- `cargo build` 无 `homie-gateway` 二进制；全量测试绿。
- 证据齐全：`docs/verification/llm-gateway-daemon-embed/`（spec-review / functional-cases /
  functional-verification / code-review / release-readiness）。

## 7. 风险与待核实项

| 风险/待核实 | 影响 | 缓解 |
|---|---|---|
| daemon 同步模型 host tokio/axum 的线程安全 | 潜在死锁/阻塞 | gateway store 本就 `Arc<Mutex>`，axum handler 短锁；用独立 tokio runtime 线程隔离 |
| 循环依赖残留 | 编译失败 | 删 `homie-gateway` 对 engine 的依赖后 `cargo build` 验证 |
| 端口占用语义混淆 | daemon 误判 singleton | 区分 control socket 与 LLM 端口两个错误路径 |
| 删 `/admin/keys` 影响 debug/doctor | 调试能力回退 | 保留 master key + 内嵌 CLI 签发手段，或明确删除 |
| Claude 退出纳管导致用量缺失 | 用量统计不完整 | 用户已确认取舍；README 明示 Claude 流量不在 Homie 用量内 |

## 8. Beads 追踪

- Beads: `homie-6md`
- change_id: `llm-gateway-daemon-embed`
- 类型: feature（收敛性重构）
- 优先级: P0

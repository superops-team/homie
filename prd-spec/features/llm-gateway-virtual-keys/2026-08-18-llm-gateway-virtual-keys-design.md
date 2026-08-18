# Homie LLM Gateway：本地 OpenAI/Anthropic 兼容代理 + 虚拟 key + Codex/Claude 配置自动注入（首个纵向切片）设计文档

## 1. 概述

### 1.1 问题/背景

`AGENTS.md` 声明 Homie 拥有「统一 LLM 配置入口点」：真实 provider 凭证只保存在 Homie 本地
配置，受管 agent 收到**虚拟 key** 并调用 Homie 的 **OpenAI-compatible 代理**，Homie 再应用
策略、记录用量、转发到已配置的 provider。但这套能力当前**尚未落地**——代码里没有 LLM 代理
层、没有 base_url 配置、没有虚拟 key 签发。现状是：

- provider 凭证由 Claude Code / Codex 各自的 CLI（`claude auth` / `codex login`）管理，Homie
  通过 `CLAUDE_CONFIG_DIR` / `CODEX_HOME` 把配置目录隔离到 `node/accounts/<provider>/<id>`；
- agent 直接调用真实 provider，Homie 无法统一记录用量、应用策略、下发一次性虚拟 key；
- 模型与请求地址无法在 Homie 侧统一配置，无法「不同 agent 配置不同模型」。

本 PRD 是这条能力链的**首个真实纵向切片**：在 Homie 内落地一个本地 HTTP 网关（对 Codex 暴露
OpenAI Responses 协议、对 Claude Code 暴露 Anthropic Messages 协议），签发虚拟 key，并把
Codex/Claude 的启动配置**自动注入**为指向该网关。打通「虚拟 key → agent 自动指向本地网关 →
网关转发到上游 provider → 记录用量」一条完整链路。

### 1.2 技术选型结论（可行性调研）

候选复用对象为 [aimux](https://github.com/arcships/aimux)（Rust 统一 LLM 访问层库）与
[litellm-rust](https://github.com/LiteLLM-Labs/litellm-rust)（Rust 版 LiteLLM 网关）。两者均
MIT。结论：

- **litellm-rust 与本需求匹配度更高**：它正是「coding agents 的 AI 网关」，已含 axum HTTP
  server（`/v1/messages` `/v1/responses`）、虚拟 key（`GatewayApiKeyStore`，`sk-` 前缀、
  last_used 追踪）、以及 `lite claude` / `lite codex` 的 agent 配置注入（Codex 用 `-c` 覆盖
  `model_provider`/`model_providers.*`，Claude 用 `ANTHROPIC_BASE_URL`/`ANTHROPIC_AUTH_TOKEN`）。
- **aimux 定位是上游访问层库**，不含 HTTP 网关、不含虚拟 key 签发，适合后续作为「转发引擎/
  provider 广度扩展」使用，不适合直接承担代理层本身。
- **复用方式**：litellm-rust 是纯 binary（无 lib.rs），不能 `cargo add`；本阶段采用
  **vendor 其 `src/proxy`/`src/http`/`src/providers` 源码**进 Homie 新 crate 的方式复用，避免
  子进程多一层生命周期/配置同步成本。

### 1.3 目标

1. 新增 `homie-gateway` crate：本地 HTTP 网关，监听 `127.0.0.1` 本地端口（默认 `7338`）。
2. 实现虚拟 key 签发（`sk-` 前缀、唯一、可撤销、可追踪 last_used），并持久化到本地 SQLite。
3. 实现网关鉴权：master key + 虚拟 key，`x-api-key` / `Authorization: Bearer` 双 header。
4. 实现上游转发：MVP 仅支持 **OpenAI-compatible provider**（单个上游 base_url + api_key）。
5. 扩展 `homie-engine` 的 spawn 注入（`inject.rs`），使 Codex/Claude 启动时自动指向本地网关并
   携带虚拟 key。
6. 记录每个虚拟 key 的请求用量（模型、token、时间）到本地 SQLite。

### 1.4 非目标

- 不接入 aimux / 不扩展 Anthropic 原生、多模态、329 provider（属 child Bead）。
- 不实现 per-agent 默认模型映射的 UI/配置面（属 child Bead）。
- 不实现配额/限流/策略/审计（属 child Bead）。
- 不把 Claude Code/Codex 的**登录凭证**直接接入网关上游（MVP 上游用独立录入的 OpenAI 兼容
  凭证，属 child Bead）。
- 不实现 `/v1/realtime`、`/v1/audio`（litellm-rust 有，但本切片不需要）。
- 不改造 `homie-node` 的远程节点 / `homie-mcp` MCP 代理。

## 2. 用户场景

### 场景 1：录入上游 provider 凭证

**Given** 用户有 OpenAI（或任意 OpenAI 兼容）服务的 base_url 与 api_key。  
**When** 用户在本地配置（ignored 文件）录入该凭证。  
**Then** 网关启动时加载该凭证，用于上游转发；凭证不进入 git、不进 agent 可见配置。

### 场景 2：为 agent 签发虚拟 key

**Given** 网关已启动。  
**When** Homie 为某个 agent session 签发虚拟 key（`sk-...`）。  
**Then** 生成唯一虚拟 key 并持久化，agent 用它调用网关，网关据此识别调用方。

### 场景 3：Codex 自动指向本地网关

**Given** Codex manifest 已声明 `codexMCP`/injection。  
**When** spawn Codex session。  
**Then** 注入 `-c model_provider="homie" -c model_providers.homie.base_url=... -c
model_providers.homie.wire_api="responses" -c model_providers.homie.env_key=...`，Codex 经网关
请求 OpenAI Responses API，使用虚拟 key。

### 场景 4：Claude Code 自动指向本地网关

**Given** Claude manifest 已声明 injection。  
**When** spawn Claude session。  
**Then** 注入 `ANTHROPIC_BASE_URL` / `ANTHROPIC_AUTH_TOKEN` 环境变量，Claude Code 经网关请求
Anthropic Messages API，使用虚拟 key。

### 场景 5：用量记录

**Given** agent 经网关完成一次请求。  
**When** 网关转发并返回响应。  
**Then** 按虚拟 key 记录模型、token、时间到本地 SQLite，供后续查询/统计。

## 3. 功能需求

### FR-1: 本地 HTTP 网关（homie-gateway）

- 监听 `127.0.0.1:<port>`，端口可配置，默认 `7338`（与 `homie-node` 的 `7337` 不冲突）。
- 路由：`POST /v1/responses`（OpenAI Responses，供 Codex）、`POST /v1/messages`（Anthropic
  Messages，供 Claude Code）。
- 请求体解析、响应回显、流式（SSE）转发到上游。
- 网关只绑本地回环地址，不对外暴露（对齐 `SECURITY-MODEL` 本地信任边界）。

### FR-2: 虚拟 key 签发与鉴权

- `GatewayApiKeyStore`：`create(label)` 生成 `sk-<uuid><uuid>` 唯一 key；`delete(id)` 撤销；
  `list()` 列表；`accepts(key)` 校验并更新 `last_used_at`。
- 鉴权：master key 或虚拟 key，`x-api-key`（裸 key）与 `Authorization: Bearer` 两种 header，
  `Bearer` 优先。
- 虚拟 key 持久化到本地 SQLite（MVP 不丢失；litellm-rust 原实现为内存 HashMap，需替换）。

### FR-3: 上游 OpenAI-compatible 转发

- 单一上游 provider：`base_url` + `api_key`，来自本地 ignored 配置。
- `/v1/responses` 与 `/v1/messages` 均转发到该上游；调用方永远看不到、不携带上游真实 key。
- 上游 key 由网关持有，与虚拟 key 严格分离。

### FR-4: agent 配置自动注入（扩展 inject.rs）

- Codex：追加 `-c model_provider="homie"`、`-c model_providers.homie.base_url="<gateway>/v1"`、
  `-c model_providers.homie.wire_api="responses"`、`-c model_providers.homie.env_key="<env>"`，
  并注入该虚拟 key 到对应环境变量。
- Claude：注入 `ANTHROPIC_BASE_URL="<gateway>"`、`ANTHROPIC_AUTH_TOKEN="<virtual-key>"`。
- 每 session 每次签发（或复用）一个虚拟 key，注入到对应 agent 环境，不影响其他 agent。

### FR-5: 用量记录

- 网关按虚拟 key 记录每次请求的 `model`、输入/输出 token 估算、时间，写入本地 SQLite。
- 复用 `homie-usage` 的 OpenAI 用量估算（`openai_estimate`）。

### FR-6: 安全边界

- 上游真实 key 与虚拟 key 明文只存本地 ignored 文件与本地 SQLite，绝不进 git、不进 agent 可见
  配置、不进日志。
- 网关鉴权失败返回 `401`；错误响应脱敏（不泄露上游 key、不泄露完整敏感 prompt）。

## 4. 实现方案

### 4.1 模块边界

```text
homie/crates/homie-gateway/
├── Cargo.toml
├── src/
│   ├── main.rs        # 二进制入口：加载配置、启动 axum、绑定 127.0.0.1
│   ├── config.rs      # 端口、上游 base_url/api_key 加载（本地 ignored 文件）
│   ├── state.rs       # AppState：虚拟 key store、上游 client、用量 store
│   ├── http/
│   │   ├── mod.rs     # 路由注册
│   │   ├── responses.rs  # POST /v1/responses（OpenAI Responses）
│   │   └── messages.rs   # POST /v1/messages（Anthropic Messages）
│   ├── auth/
│   │   ├── mod.rs     # 鉴权中间件（master key + 虚拟 key）
│   │   ├── master_key.rs
│   │   └── api_keys.rs   # GatewayApiKeyStore（vendor 自 litellm-rust）
│   ├── providers/
│   │   └── openai.rs  # 上游 OpenAI-compatible 转发 + SSE 流式
│   └── usage.rs       # 按虚拟 key 的用量记录（SQLite）
```

### 4.2 数据模型

```rust
// auth/api_keys.rs（vendor 自 litellm-rust，持久化替代 HashMap）
struct GatewayApiKeyRecord { id: String, label: Option<String>, key: String,
                             created_at: u64, last_used_at: Option<u64> }
struct CreatedGatewayApiKey { id, label, key, created_at, last_used_at }

// usage.rs
struct UsageRecord { key_id: String, model: String, occurred_at: i64,
                     input_tokens: i64, output_tokens: i64 }
```

### 4.3 上游凭证配置（本地 ignored 文件）

- 默认路径 `~/.config/homie/homie.local.json`（或 `HOMIE_CONFIG` / `HOMIE_CONFIG_DIR`
  覆盖），内容：`base_url`、`api_key`、`listen`、`master_key`。采用 JSON（非 TOML），
  使 Swift CLI 与 Rust 二进制读写同一份字节（见 PRD2 `homie-cli-config-ops` 决策 A）。
- 该文件加入 `.gitignore`（已由 `*.local.json` / `homie.local.*` 覆盖）。

### 4.4 I/O 模型

- 网关 crate 使用 `tokio` + `axum`（符合 `docs/research/rust-package-selection.md` 的异步约定）；
  这是新增依赖，需在 `Cargo.toml` 明确 owner crate 与 feature flags。
- 持久化用 `rusqlite`（workspace 已有），虚拟 key 与用量各一张表。

### 4.5 注入挂载点

- 扩展 `homie/crates/homie-engine/src/inject.rs::injection_args()` 与 `control/handlers.rs`
  的 env 注入，新增 `codex_gateway` / `claude_gateway` 两个机制（对齐现有
  `codex_mcp` / `claude_mcp` / `claude_hooks` 结构）。
- 虚拟 key 由 spawn 时向网关的本地管理接口申请（或由 engine 直接经共享 store 签发），随 agent
  argv/env 注入。

### 4.6 首阶段关闭口径

- 网关可启动、可转发 `/v1/responses` 与 `/v1/messages` 到 OpenAI 兼容上游。
- 虚拟 key 可签发/校验/撤销/持久化，鉴权失败返回 401。
- Codex/Claude spawn 时自动注入指向本地网关的配置。
- 用量按虚拟 key 落库。
- 不接 aimux、不做模型映射 UI、不做配额策略。

## 5. 边界情况

| 场景 | 处理 |
|------|------|
| 上游 base_url/api_key 未配置 | 网关启动失败并给出明确错误，不静默降级 |
| 上游请求失败/超时 | 透传错误，脱敏，不泄露上游 key |
| 虚拟 key 不存在/已撤销 | 返回 `401`，不转发 |
| master key 未配置 | 明确警告（本地边界内可接受，但不静默放行到非回环地址） |
| 虚拟 key 重启后丢失 | 持久化到 SQLite，重启可恢复 |
| 流式响应中断 | 正确关闭 SSE，记录已产生用量 |
| 端口被占用 | 启动报错，可配置换端口 |

## 6. 涉及文件

- `homie/crates/homie-gateway/*`（新增 crate，vendor litellm-rust 的 auth/providers 源码）
- `homie/Cargo.toml`（注册 workspace member；新增 `axum`/`tokio` 依赖，遵循依赖添加政策）
- `homie/crates/homie-engine/src/inject.rs`（新增 codex/claude 网关注入）
- `homie/crates/homie-engine/src/agent.rs`（`InjectionSpec` 新增 gateway 字段）
- `homie/crates/homie-engine/manifests/codex.json` / `claude-code.json`（声明 gateway 注入）
- `specs/`（新增 `specs/llm-gateway.md` 组件合同：虚拟 key、协议、用量）
- `docs/research/rust-package-selection.md`（记录 axum/tokio 选择）
- `.gitignore`（忽略 `homie.local.json`，已由现有规则覆盖）

## 7. 验证计划

### 7.1 单元测试

- 虚拟 key store：创建/校验/撤销/持久化/last_used 更新。
- 鉴权中间件：master key、虚拟 key、`x-api-key` vs `Bearer` 优先级、401。
- 上游转发：`/v1/responses` 与 `/v1/messages` 的请求构造与响应回显（wiremock 模拟上游）。
- 用量记录：按虚拟 key 落库。
- inject.rs：codex `-c` / claude env 注入的 argv/env 形状。

### 7.2 集成测试

- 网关真实启动（绑 127.0.0.1），用虚拟 key 走 `/v1/responses` 与 `/v1/messages`，转发到
  wiremock 上游，断言用量落库。
- spawn 路径：manifest 声明 gateway 注入后，session argv/env 包含正确网关指向。

### 7.3 门禁

- `cargo check --workspace`
- `cargo fmt --all --check`
- `cargo test -p homie-gateway`
- `cargo test -p homie-engine inject`

## 8. 验收标准

1. `homie-gateway` 可编译、可启动，监听 127.0.0.1 本地端口。
2. 虚拟 key 可签发/校验/撤销/持久化，鉴权失败返回 401。
3. `/v1/responses` 与 `/v1/messages` 可转发到 OpenAI 兼容上游，流式可用。
4. Codex/Claude spawn 时自动注入指向本地网关的配置与虚拟 key。
5. 用量按虚拟 key 持久化。
6. 上游真实 key 不进 git、不进 agent 可见配置、不进日志。
7. OpenSpec alignment 对齐本 PRD，Beads `homie-f91` 关闭。

## 9. Beads 追踪与 child Bead 拆解

- Beads: `homie-f91`
- change_id: `llm-gateway-virtual-keys`
- 类型: feature
- 优先级: P0

### 后续 child Bead（本 PRD 只声明，不实现）

| change_id | 内容 |
|-----------|------|
| `llm-gateway-provider-expansion` | 接入 aimux，扩展 Anthropic 原生 / 多模态 / 329 provider 覆盖 |
| `llm-gateway-model-routing` | per-agent 默认模型映射 + 网关 model router（不同 agent 不同模型） |
| `llm-gateway-policy-quota` | 虚拟 key 配额 / 限流 / 策略 / 审计 |
| `llm-gateway-credential-login` | 把 Claude Code / Codex 登录凭证接入网关上游（复用现有 login） |

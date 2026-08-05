# Homie V1 架构设计文档

## 1. 概述

### 1.1 背景

Homie 的目标是构建一个 Rust + GPUI 的高性能跨平台桌面应用，用一个本地优先的控制面统一管理多个后台 coding agent，包括 Codex、Claude Code、OpenCode 以及后续兼容 agent。项目还需要统一 LLM 配置入口：真实 provider key 只存在 Homie 本地配置中，Homie 给各 agent 签发虚拟 key，agent 通过 Homie 暴露的 OpenAI-compatible proxy 发起推理请求。

第一版实现不能只停留在 UI shell 或单个 agent wrapper。它必须跑通一个最小端到端链路：用户在桌面应用中创建 agent session，后台运行时启动 agent，Homie 记录 session/context，agent 使用 Homie 的虚拟 key 访问 LLM proxy，桌面 UI 可以看到 session 状态和输出。

本设计参考成熟本地 agent 编排桌面应用的工程形态：桌面 UI 与后台运行时分离、用稳定协议连接、后台进程拥有 PTY/session/log/status 等状态、UI 只消费状态和渲染输出；同时结合 Homie 的核心差异：LLM credential custody、virtual key、OpenAI-compatible proxy、global context/memory/task/orchestrator。

### 1.2 目标

- 初始化可长期演进的 Rust workspace 和 crate 边界。
- 第一版实现本地桌面应用 + 后台运行时 + 控制协议 + agent session 的最小端到端闭环。
- 支持至少一个真实 agent runtime 的启动、输入、输出、状态记录和终止；V1 默认真实 runtime 选择 Codex，OpenCode 和 Claude Code 作为同一 adapter contract 下的后续目标。
- 建立统一 LLM 配置和虚拟 key 代理模型，agent 不直接持有真实 provider key。
- 建立基于 SQLite 的本地存储层，统一维护 agent、runtime、profile、skills、MCP、权限、LLM 配置、session、context、usage 和任务关系。
- 建立 session context 存储，记录 agent 会话、工作目录、agent 类型、profile、状态、输出摘要和关键事件。
- 建立 agent profile 配置入口，支持未来多个 agents 形态：每个 agent 可绑定 runtime、skills、MCP servers、权限策略和 LLM profile。
- 为 memory、task、intent orchestrator 留出稳定边界，但 V1 只实现最小可用骨架。
- 在架构上为 macOS 首发和后续 Linux/Windows 扩展保留清晰平台 seam。

### 1.3 非目标

- V1 不实现完整多 agent 自动协作。
- V1 不实现复杂长期记忆检索和语义索引，只提供可扩展的 memory controller 边界。
- V1 不实现完整插件市场或第三方 agent SDK。
- V1 不追求所有 agent 的深度状态检测一致；先跑通一个真实 agent，并让其他 agent 通过同一 contract 渐进接入。
- V1.0 不承诺 runtime 在 app 退出后继续存活；V1.0 必须持久化 session/context/usage，以便 app 重启后展示历史状态。runtime 独立保活作为 V1.1 目标。
- V1.0 不实现 MCP server proxy 执行链路；MCP server 配置和 profile 绑定先入库，实际代理、调用转发和 MCP tool 耗时统计在后续工作补充开发。
- 远端执行主机、节点账户、跨机器 handoff/fork 属于 V1.x 阶段，不进入 V1.0 最小闭环，但属于第一个产品版本路线图。

## 2. 用户场景

### 场景 1: 启动一个本地 agent session

**Given** 用户打开 Homie 并选择一个工作目录。
**When** 用户创建一个 Codex session。
**Then** Homie 后台运行时启动 Codex agent，桌面 UI 展示 session 行、终端输出、运行状态和基本操作。

### 场景 2: agent 通过 Homie 访问 LLM

**Given** 用户已在 Homie 本地配置真实 provider key。
**When** managed agent 发起 OpenAI-compatible 请求。
**Then** agent 使用 Homie 签发的 virtual key 请求本机 proxy，Homie 校验 key、映射 provider/model、转发请求并记录 usage，不把真实 key 写入 agent 配置或日志。

### 场景 3: 桌面应用重启后恢复 session 列表

**Given** 用户关闭并重新打开 Homie。
**When** 后台运行时仍有 session 记录或可恢复状态。
**Then** UI 从 session registry/context store 读取 session 列表，并展示最近状态、工作目录、agent 类型和历史输出入口。

### 场景 4: 后续新增 agent

**Given** 项目要新增一个 agent runtime。
**When** 开发者添加 agent manifest/adapter。
**Then** 不需要改 UI、LLM proxy、context store 或 task controller 的核心合同，只扩展 agent adapter 层和状态检测规则。

### 场景 5: 管理 agent profile

**Given** 用户希望配置多个不同用途的 agent，例如代码实现、代码审查、文档整理或任务拆解。
**When** 用户在 Homie 中创建 agent profile。
**Then** Homie 允许用户选择 runtime、LLM profile、skills、MCP servers、权限策略和默认 workspace scope，并把配置写入 SQLite，由 runtime 在启动 session 时冻结为本次 session 的有效配置。

## 3. 功能需求

### FR-1: Rust workspace 与 crate 边界

V1 初始化 Rust workspace，按职责拆分 crate，避免把 UI、运行时、协议、LLM proxy、storage 混在一个二进制中。

建议 workspace：

```text
homie/
├── Cargo.toml
├── crates/
│   ├── homie-app/          # GPUI 桌面应用入口
│   ├── homie-ui/           # 设计系统、通用组件、图标、状态 glyph
│   ├── homie-term/         # terminal grid/buffer 渲染、输入编码、选择、搜索
│   ├── homie-proto/        # 控制协议、事件、数据模型、错误 envelope
│   ├── homie-client/       # UI 到 runtime 的异步客户端
│   ├── homie-runtime/      # 后台运行时：session、PTY、agent process、registry、status
│   ├── homie-agents/       # agent manifest、agent profile、adapter contract、skills/MCP/权限配置
│   ├── homie-llm/          # provider config、virtual key、OpenAI-compatible proxy
│   ├── homie-context/      # session context、事件、workspace facts
│   ├── homie-memory/       # durable memory 边界，V1 可为空实现
│   ├── homie-task/         # task model 与本地任务状态，V1 可为空实现
│   ├── homie-storage/      # 本地持久化、schema、索引、atomic write
│   └── homie-cli/          # doctor、runtime status、MCP/agent hook shim 的后续入口
├── assets/
├── scripts/
└── docs/
```

V1 可以只实现 P0 crate，但目录和依赖方向必须先正确。

依赖方向：

```text
homie-app
  -> homie-ui
  -> homie-term
  -> homie-client
  -> homie-proto

homie-client
  -> homie-proto

homie-runtime
  -> homie-proto
  -> homie-agents
  -> homie-context
  -> homie-llm
  -> homie-storage

homie-llm
  -> homie-proto
  -> homie-storage

homie-context / homie-memory / homie-task
  -> homie-storage
  -> homie-proto
```

禁止 `homie-runtime` 依赖 `homie-app` 或 `homie-ui`。

### FR-2: 双进程运行模型

V1 采用双进程模型：

- `homie-app`：GPUI 桌面应用，负责窗口、导航、terminal pane、session list、设置页和用户交互。
- `homie-runtime`：后台运行时，负责 agent process、PTY、session registry、output log、状态检测、context 写入和本机控制 socket。

设计理由：

- agent session 生命周期不应绑定 UI 窗口生命周期。
- 后台运行时可以独立重启、恢复或被 CLI 诊断。
- UI 不直接拥有 PTY 和 child process，避免渲染层与进程生命周期耦合。
- 后续远端运行时可以复用同一控制协议。

V1 可先由 `homie-app` 启动并守护本机 `homie-runtime`，但协议边界必须真实存在，不允许用进程内全局状态替代。

### FR-3: 控制协议与事件模型

`homie-proto` 定义 UI/client 与 runtime 之间的稳定协议。

V1 协议最低包含：

```text
hello
agent.runtime.list
agent.profile.create
agent.profile.update
agent.profile.list
agent.profile.set_default
session.spawn
session.list
session.attach
session.input
session.resize
session.terminate
session.read_output
events.subscribe
llm.virtual_key.issue
llm.proxy.status
skills.list
mcp.server.list
permission.profile.list
context.session.summary
```

控制通道建议使用本机 Unix domain socket + newline-delimited JSON。后续 Linux 可复用 Unix socket，Windows 使用 named pipe 或 TCP loopback + owner-only token。

事件必须包含递增 `seq`，client 记录最后 `seq`，断线重连后通过 `events.subscribe{since_seq}` 恢复。V1 事件 ring 可固定 4096 条。

事件类型：

```text
runtime.ready
runtime.unhealthy
session.created
session.updated
session.output
session.terminated
llm.request.started
llm.request.completed
llm.request.failed
tool.call.started
tool.call.completed
tool.call.failed
metrics.write_failed
context.updated
```

协议原则：

- unknown enum/value 必须 lenient decode。
- mutating request 必须有 request id。
- 错误统一为 `{ code, message, retryable, details }`。
- 所有事件和响应不得包含真实 provider key、raw Authorization、cookie 或完整敏感 tool args。

### FR-4: Agent adapter contract

`homie-agents` 提供 agent runtime manifest、agent profile registry 和 adapter contract。新增 runtime 应优先通过数据文件和 adapter 配置完成，而不是改 UI 分支；新增具体 agent 应优先通过 profile 配置完成，而不是复制 runtime 代码。

V1 runtime descriptor：

```rust
struct RuntimeDescriptor {
    id: RuntimeId,
    display_name: String,
    binary: Option<String>,
    argv_template: Vec<String>,
    env: BTreeMap<String, String>,
    env_scrub_prefixes: Vec<String>,
    status_authority: StatusAuthority,
    resume: Option<ResumeSpec>,
    approve: Option<ApproveSpec>,
    llm_proxy: LlmProxyRequirement,
}
```

V1 agent profile：

```rust
struct AgentProfile {
    id: AgentProfileId,
    name: String,
    runtime_id: RuntimeId,
    llm_profile_id: LlmProfileId,
    skill_ids: Vec<SkillId>,
    mcp_server_ids: Vec<McpServerId>,
    permission_profile_id: PermissionProfileId,
    default_workspace_scope: WorkspaceScope,
    enabled: bool,
    created_at: DateTime,
    updated_at: DateTime,
}
```

两者关系：

- `RuntimeDescriptor` 描述“怎么启动和观察某类 runtime”，例如 Codex、Claude Code、OpenCode。
- `AgentProfile` 描述“用户配置的某个 agent”，绑定一个 runtime，并配置该 agent 使用的 LLM profile、skills、MCP servers 和权限策略。
- session 启动时必须把 `AgentProfile` 解析成不可变的 `EffectiveAgentConfig`，记录到 session 中；后续 profile 修改不影响已经运行的 session。
- `agent.profile.set_default` 只设置新 session 默认 profile，不改变既有 profile 的 enabled 状态，也不影响已运行 session。

`StatusAuthority`：

- `ProcessOnly`：只根据进程是否运行判断。
- `ScreenPrimary`：根据 terminal screen 和进程状态判断。
- `HookPrimary`：优先使用 agent hook/notify 回调，screen 作为补充。

环境处理规则：

- 不继承危险 agent 状态变量，如已有 session id、provider raw key、Authorization。
- 显式设置 `TERM=xterm-256color`、`COLORTERM=truecolor`，移除 `NO_COLOR`。
- 真实 provider key 不写入 agent env。
- agent 只能拿到 Homie virtual key 和 local proxy base URL。

V1 至少落地一个 runtime descriptor、一个 agent profile 和一个可运行 adapter。

### FR-4A: Agent profile 配置中心

Homie 需要一个统一配置入口，管理未来多 agents 形态下的关系。V1 默认真实 runtime 为 Codex，其他 runtime 的 descriptor 可以后续加入同一模型。

```text
AgentProfile
  -> RuntimeDescriptor
  -> LlmProfile
  -> SkillBinding[]
  -> McpServerBinding[]
  -> PermissionProfile
  -> WorkspaceScope
```

V1 必须支持以下配置对象：

```rust
struct LlmProfile {
    id: LlmProfileId,
    name: String,
    provider_id: ProviderId,
    default_model: String,
    allowed_models: Vec<String>,
    temperature: Option<f32>,
    max_output_tokens: Option<u32>,
}

struct SkillDefinition {
    id: SkillId,
    name: String,
    source: SkillSource,
    enabled_by_default: bool,
}

struct McpServerConfig {
    id: McpServerId,
    name: String,
    command: Option<String>,
    url: Option<Url>,
    env: BTreeMap<String, SecretOrPlainRef>,
    enabled: bool,
}

struct PermissionProfile {
    id: PermissionProfileId,
    name: String,
    filesystem: FilesystemPolicy,
    network: NetworkPolicy,
    shell: ShellPolicy,
    approval: ApprovalPolicy,
}
```

session 启动时冻结的有效配置：

```rust
struct EffectiveAgentConfig {
    session_id: SessionId,
    agent_profile_id: AgentProfileId,
    runtime_id: RuntimeId,
    llm_profile_id: LlmProfileId,
    provider_id: ProviderId,
    skill_ids: Vec<SkillId>,
    mcp_server_ids: Vec<McpServerId>,
    permission_profile_id: PermissionProfileId,
    workspace_scope: WorkspaceScope,
    virtual_key_id: VirtualKeyId,
    frozen_at: DateTime,
}
```

配置规则：

- profile 是用户面对的主要配置单元。
- runtime 是执行引擎，不承载用户策略。
- LLM profile 只引用 provider，不保存 raw key。
- MCP server 可以声明 secret ref，但不能内联真实 secret。
- V1 只保存 MCP server 配置和 agent profile 绑定，不启动 MCP server proxy，不向 agent 注入 MCP proxy endpoint。
- skills 是可被 agent profile 选择的能力集合，V1 可先只存 metadata 和启用状态。
- permission profile 必须显式绑定，禁止隐式默认 full access。
- 所有 profile 变更写入 SQLite，并保留 `created_at/updated_at`；V1 不要求完整审计日志，但 schema 要预留 `config_events`。
- 至少有一个 default agent profile；若 default profile 被禁用，启动新 session 必须返回 `agent_profile_unavailable`。

### FR-5: PTY/session runtime

`homie-runtime` 拥有 session 生命周期：

- spawn agent process；
- 分配 PTY；
- 读取 stdout/stderr/terminal bytes；
- 写 output log；
- 更新 headless screen；
- 运行 status reducer；
- 持久化 session record；
- 接收 input/resize/terminate；
- 向 client 发送 session/output/status events。

V1 session model：

```rust
struct SessionRecord {
    id: SessionId,
    agent_profile_id: AgentProfileId,
    runtime_id: RuntimeId,
    workspace: PathBuf,
    title: String,
    status: SessionStatus,
    created_at: DateTime,
    updated_at: DateTime,
    last_seen_at: Option<DateTime>,
    output_tail_offset: u64,
    virtual_key_id: Option<VirtualKeyId>,
    llm_profile_id: Option<LlmProfileId>,
    permission_profile_id: PermissionProfileId,
    effective_config_id: EffectiveConfigId,
    context_ref: Option<ContextRef>,
}
```

`SessionStatus`：

```text
starting
working
idle
needs_input
done
failed
terminated
unknown
```

PTY 平台 seam：

- Unix/macOS：使用 `openpty`/`fork`/`exec` 或成熟 crate，支持 resize、process group kill、fd hygiene。
- Windows：V1 不实现，但 `Pty` trait 预留 ConPTY shape。

### FR-6: Terminal grid 与 GPUI 渲染

V1 UI 不直接解析 agent 语义，但需要高性能显示 terminal 输出。

最小方案：

- runtime 维护 headless terminal screen；
- client attach 后收到 screen snapshot 和后续 row diff；
- `homie-term` 维护 `GridBuffer`；
- GPUI element 只绘制 cell buffer；
- unchanged rows 不重新 shape；
- resize 由 UI 计算 cols/rows 后发送 runtime。

V1 可以先实现普通文本输出视图，但 crate 边界必须为后续 grid/rle diff 渲染留出位置。若初期选择纯文本模式，也必须定义从纯文本升级到 grid 的路径：

```text
V1a: append-only output pane
V1b: headless screen snapshot
V1c: row diff + damage tracking
V1d: scrollback + selection + find
```

### FR-7: LLM provider config、virtual key 与 proxy

`homie-llm` 是 Homie 的核心差异，V1 必须定义并实现最小链路。

V1 数据模型：

```rust
struct ProviderConfig {
    id: ProviderId,
    kind: ProviderKind,
    base_url: Url,
    api_key_ref: SecretRef,
    default_model: String,
    model_aliases: BTreeMap<String, String>,
}

struct VirtualKey {
    id: VirtualKeyId,
    session_id: SessionId,
    agent_profile_id: AgentProfileId,
    provider_id: ProviderId,
    allowed_models: Vec<String>,
    expires_at: DateTime,
    revoked_at: Option<DateTime>,
}
```

V1 proxy endpoint：

```text
POST /v1/chat/completions
POST /v1/responses     # 可先返回 not_implemented，但协议保留
GET  /v1/models
```

认证：

- agent 使用 `Authorization: Bearer <HOMIE_VIRTUAL_KEY>`。
- proxy 校验 virtual key 是否存在、未过期、未撤销、session/agent scope 匹配。
- proxy 根据 provider config 注入真实 provider key。
- proxy 响应和日志不得包含真实 key。

usage：

- V1 记录 request id、session id、agent profile id、provider id、model、started_at、completed_at、status、token usage（若 provider 返回）。
- LLM profile 由 agent profile 绑定；session 启动时根据 effective config 签发 virtual key。
- 失败也必须记录 safe error code。

### FR-7A: LLM 统一流量指标、token 成本与工具调用耗时

Homie 承接统一 LLM 流量入口，V1 必须把 proxy 设计为 usage、cost 和 latency 的事实源，而不是只做透明转发。

V1 指标维度：

```text
local_user
session_id
agent_profile_id
runtime_id
provider_id
llm_profile_id
model
request_kind
status
tool_name
mcp_server_id
```

LLM request 指标：

```rust
struct LlmUsageRecord {
    id: UsageRecordId,
    request_id: RequestId,
    session_id: SessionId,
    agent_profile_id: AgentProfileId,
    runtime_id: RuntimeId,
    provider_id: ProviderId,
    llm_profile_id: LlmProfileId,
    model: String,
    status: RequestStatus,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cached_input_tokens: Option<u64>,
    cache_read_tokens: Option<u64>,
    cache_write_tokens: Option<u64>,
    cache_hit_rate: Option<Decimal>,
    reasoning_tokens: Option<u64>,
    total_tokens: Option<u64>,
    unit_price_input: Option<Decimal>,
    unit_price_output: Option<Decimal>,
    currency: Option<String>,
    pricing_snapshot_id: Option<PricingSnapshotId>,
    estimated_cost: Option<Decimal>,
    first_token_latency_ms: Option<u64>,
    total_latency_ms: u64,
    safe_error_code: Option<String>,
    started_at: DateTime,
    completed_at: DateTime,
}
```

工具调用耗时指标：

```rust
struct ToolCallMetric {
    id: ToolCallMetricId,
    session_id: SessionId,
    agent_profile_id: AgentProfileId,
    runtime_id: RuntimeId,
    tool_name: String,
    mcp_server_id: Option<McpServerId>,
    status: ToolCallStatus,
    latency_ms: u64,
    queue_latency_ms: Option<u64>,
    input_bytes: Option<u64>,
    output_bytes: Option<u64>,
    safe_error_code: Option<String>,
    started_at: DateTime,
    completed_at: DateTime,
}
```

成本规则：

- provider 返回 token usage 时，以 provider usage 为准。
- provider 不返回 usage 时，V1 可记录 `unknown`，不强行估算；后续再接 tokenizer。
- cache hit rate 的单请求口径为 `cache_read_tokens / input_tokens`；当 provider 只返回 `cached_input_tokens` 时，先映射到 `cache_read_tokens`。
- 聚合 cache hit rate 必须用 `sum(cache_read_tokens) / sum(input_tokens)` 重新计算，不能对单请求百分比做平均。
- `cache_write_tokens` 表示本次写入 provider prompt/cache 的 token 数；provider 不返回时记录 `unknown`。
- pricing 必须按 provider/model 维度配置，写入 SQLite，不写死在代码里。
- 成本计算只记录估算值，字段名使用 `estimated_cost`，避免和账单实际扣费混淆。
- 每条 usage 记录必须保存 pricing snapshot id 和 currency，避免模型价格更新后历史成本被重新解释。
- 所有 cost/token/cache 统计必须可按 session、agent profile、runtime、provider、model 聚合。

工具调用规则：

- Homie 直接代理的 tool 调用必须记录耗时。
- V1 不实现 MCP server proxy，因此 MCP tool 调用耗时在 V1 只保留 schema 和 UI/诊断占位，不作为 E2E 准出项。
- agent 内部未经过 Homie 的工具调用，V1 不强行统计；后续通过 agent hook/MCP proxy 接入。
- tool args 和 result 默认不写入指标表，只记录 safe metadata、字节数、耗时、状态和 safe error code。
- 对 timeout、cancel、permission denied、upstream error 必须有稳定 safe error code。

UI/诊断要求：

- V1 UI 至少展示当前 session 的 token 使用、估算成本和最近工具调用耗时摘要。
- `homie doctor` 或 `homie llm proxy-status` 应能输出 proxy health、最近失败数和 usage 表是否可写。
- 指标写入失败不能阻塞 LLM 响应返回，但必须产生 `metrics.write_failed` runtime event。

### FR-8: Context store

`homie-context` 负责 session context 和 workspace facts。

V1 context 只做结构化记录，不做复杂检索：

```rust
struct SessionContext {
    session_id: SessionId,
    workspace: PathBuf,
    agent_profile_id: AgentProfileId,
    events: Vec<ContextEvent>,
    latest_summary: Option<String>,
}

enum ContextEvent {
    UserInput,
    AgentOutput,
    ToolCall,
    ToolResult,
    LlmRequest,
    LlmResponse,
    StatusChange,
}
```

V1 必须写入：

- session created；
- user input；
- agent output summary 或 output offset；
- LLM request/response metadata；
- status changes。

V1 不要求自动总结，但接口要允许后续 summarizer 写入 `latest_summary`。

### FR-9: Memory、Task、Orchestrator 骨架

V1 不实现复杂能力，但必须建立不会阻碍演进的边界。

`homie-memory`：

- 定义 `MemoryRecord`、`MemoryCandidate`、`MemoryStore` trait；
- V1 默认 no-op 或 file-backed minimal store；
- 禁止将 raw secret、完整 provider request、敏感 tool args 写入 memory。

`homie-task`：

- 定义 `TaskRecord`、`TaskStatus`、`TaskEvent`；
- V1 可以只服务于 UI 内部任务和后续 Beads 对接；
- 不把 Beads 数据库作为 runtime 强依赖。

`homie-orchestrator`（可先在 `homie-runtime` 内部模块实现，后续独立 crate）：

- V1 支持简单 intent routing：
  - `new_session`；
  - `send_to_active_session`；
  - `show_session`；
  - `configure_provider`；
  - `list_tasks`。
- 不做多 agent 自动计划。

### FR-10: Storage

`homie-storage` 统一本地持久化。V1 明确使用 SQLite 作为本地事实源，用关系模型维护 provider、LLM profile、runtime、agent profile、skills、MCP、权限、session、context、usage、task 等对象之间的关系。

V1 推荐目录：

```text
~/Library/Application Support/Homie/      # macOS
~/.local/share/homie/                     # Linux 后续
%APPDATA%/Homie/                          # Windows 后续
```

V1 文件：

```text
homie.sqlite                  # SQLite 主库，WAL 模式
homie.sqlite-wal              # SQLite WAL
homie.sqlite-shm              # SQLite shared memory
secrets/                      # encrypted local secret envelope，不放 raw key
runtime/output/<session>.log  # 大流式输出日志；SQLite 保存 offset 和索引
```

SQLite 是当前实现基线，不再使用分散 JSON 文件作为主事实源。JSONL 可以作为 debug/export 产物，但不能成为 runtime 必需的状态源。

V1 SQLite 表规划：

```text
providers(id, kind, name, base_url, api_key_ref, created_at, updated_at)
llm_profiles(id, provider_id, name, default_model, allowed_models_json, params_json, created_at, updated_at)
model_pricing(id, provider_id, model, input_price_per_million, output_price_per_million, cached_input_price_per_million, currency, effective_at, created_at)
pricing_snapshots(id, provider_id, model, input_price_per_million, output_price_per_million, cached_input_price_per_million, currency, source_pricing_id, captured_at)
runtime_descriptors(id, kind, display_name, binary, argv_template_json, env_json, env_scrub_json, status_authority, created_at, updated_at)
agent_profiles(id, name, runtime_id, llm_profile_id, permission_profile_id, workspace_scope_json, enabled, is_default, created_at, updated_at)
skills(id, name, source_json, enabled_by_default, created_at, updated_at)
agent_profile_skills(agent_profile_id, skill_id, enabled)
mcp_servers(id, name, transport, command, url, env_refs_json, enabled, created_at, updated_at)
agent_profile_mcp_servers(agent_profile_id, mcp_server_id, enabled)
permission_profiles(id, name, filesystem_json, network_json, shell_json, approval_json, created_at, updated_at)
effective_agent_configs(id, session_id, agent_profile_id, runtime_id, llm_profile_id, provider_id, permission_profile_id, virtual_key_id, skill_ids_json, mcp_server_ids_json, workspace_scope_json, frozen_at)
sessions(id, agent_profile_id, runtime_id, llm_profile_id, permission_profile_id, effective_config_id, workspace, title, status, output_log_path, output_tail_offset, virtual_key_id, created_at, updated_at, last_seen_at)
context_events(id, session_id, kind, safe_payload_json, output_offset, created_at)
virtual_keys(id, session_id, agent_profile_id, provider_id, key_hash, allowed_models_json, expires_at, revoked_at, created_at)
usage_records(id, request_id, session_id, agent_profile_id, runtime_id, provider_id, llm_profile_id, model, request_kind, status, input_tokens, output_tokens, cached_input_tokens, cache_read_tokens, cache_write_tokens, cache_hit_rate, reasoning_tokens, total_tokens, unit_price_input, unit_price_output, currency, pricing_snapshot_id, estimated_cost, first_token_latency_ms, total_latency_ms, started_at, completed_at, safe_error_code)
tool_call_metrics(id, session_id, agent_profile_id, runtime_id, tool_name, mcp_server_id, status, latency_ms, queue_latency_ms, input_bytes, output_bytes, started_at, completed_at, safe_error_code)
tasks(id, title, status, agent_profile_id, session_id, metadata_json, created_at, updated_at)
config_events(id, subject_type, subject_id, event_kind, safe_payload_json, created_at)
metrics_write_failures(id, metric_kind, subject_id, safe_error_code, created_at)
```

写入要求：

- SQLite 使用 WAL 模式，应用启动时执行 schema migration。
- 所有外键关系必须开启 `PRAGMA foreign_keys = ON`。
- profile 关系必须有唯一约束：同一 agent profile 下同一 skill/MCP server 只能绑定一次；同一 provider/model/effective_at 只能有一条 pricing 记录。
- default agent profile 必须通过数据库约束或事务逻辑保证最多一个 enabled default。
- 对高频 output bytes 不写 SQLite blob，写入 `runtime/output/<session>.log`，SQLite 只保存路径、offset、索引和摘要。
- context/usage/task 写 SQLite，禁止各模块各自维护独立 JSON 状态。
- token、cost、cache hit rate、latency、tool call metrics 写 SQLite，支持按 session/agent profile/runtime/provider/model 聚合查询。
- metrics 写入失败只记录 safe error code，不能把 raw request、raw response、tool args 或 result 写入失败表。
- corrupt state 不得静默覆盖，必须 quarantine。
- V1 secret 存储使用 encrypted local secret envelope，不直接依赖 macOS Keychain；必须写明 threat model，不允许明文 raw key。
- migration 可以只支持向前升级；不要求兼容旧 schema，因为本项目不保留向后兼容。

### FR-11: Desktop V1 UI

V1 UI 目标是可用的工作台，不做营销页。

首屏布局：

- 左侧 session sidebar；
- 中央 active session terminal/output；
- 顶部或底部状态区；
- 设置入口；
- provider/virtual key 状态可见但不展示真实 key。

V1 操作：

- 新建 session；
- 选择 workspace；
- 创建/编辑 agent profile；
- 为 agent profile 选择 runtime、LLM profile、skills、MCP servers 和 permission profile；
- 选择 agent profile 启动 session；
- 启动/终止 session；
- 输入消息或 terminal input；
- 查看 session 状态；
- 打开 provider config 设置；
- 显示 LLM proxy health、token 使用、缓存命中率、估算成本、请求耗时和工具调用耗时简表。

UI 不直接写 runtime state；全部通过 `homie-client` 调用协议。

### FR-12: CLI 与诊断

V1 可提供最小 CLI：

```text
homie doctor
homie runtime status
homie session list
homie llm proxy-status
```

CLI 使用同一 `homie-client` 和 `homie-proto`，不能绕过 runtime 直接改 state 文件。

### FR-13: Worktree 与 workspace 管理

第一个产品版本需要提供 worktree/workspace 能力：

```text
worktree.create
worktree.list
worktree.remove
worktree.overview
project.add
```

V1.0 可先只实现 workspace path 选择和 SQLite 记录；V1.1 补齐 git worktree 创建、清理、overview 和 UI 管理。

### FR-14: 完整桌面工作台功能面

Homie 桌面工作台需要覆盖以下 surface：

- session sidebar：project sections、pinned、archive、drag reorder、multi-select、hover actions。
- new session popover：选择 agent profile、workspace、runtime 可用性。
- command palette。
- quick open。
- session overview board/list。
- history / transcript scan / resume。
- terminal pane：header、status chips、scrollback、selection、find、return-to-live、exited/resume/archive overlays。
- settings：General、Terminal、Provider/LLM、Agent Profiles、Permissions、Remote。
- account/status footer：usage、update、proxy health。

V1.0 只要求基础 sidebar、active output、new session、provider/profile settings 和 usage summary；其余 surface 必须在 V1.x roadmap 中保留，并逐项拆 OpenSpec。

### FR-15: Native system integration

macOS native integration 由 Swift 或 Rust `objc2` 平台模块承载：

- menu bar status rollup；
- native notifications；
- notification actions for approve/deny；
- optional status sounds；
- traffic-light/window chrome；
- packaging metadata and entitlements。

V1.0 可不实现 menu bar/notifications/sounds，但目录结构和 Swift/Rust platform seam 必须预留。

### FR-16: Session lifecycle controls

第一个产品版本需要覆盖完整 session lifecycle：

```text
session.spawn
session.list
session.attach
session.input
session.resize
session.terminate
session.archive
session.unarchive
session.reopen_last
session.hibernate
session.wake
session.history
session.resume_from_history
```

V1.0 最小闭环只实现 spawn/list/input/resize/terminate/read_output；archive/reopen/hibernate/wake/history resume 在后续 runtime work 中补齐。

### FR-17: Artifact、Port、PR 与资源信息

Homie 需要把 agent session 的外部产物变成可见 metadata：

- artifacts；
- listening ports；
- pull requests；
- git branch/worktree；
- resource usage；
- runtime health。

V1.0 只要求 schema 和 UI 占位；V1.x 补 artifact scanner、port scanner、PR monitor 和 resource governor。

### FR-18: Homie MCP control surface

这里的 MCP control surface 指 Homie 对外暴露控制能力，让 agent 可以查询/创建/管理 Homie session 和任务；它不同于 MCP server proxy。

V1.0 暂不实现 MCP control surface，但第一个产品版本需要保留：

- `homie mcp-stdio` 或等价入口；
- session list/spawn/status tools；
- task read/update tools；
- strict permission and audit；
- 所有 tool 调用写入 tool metrics。

### FR-19: Packaging、Updater 与 Release

第一个产品版本需要正式桌面分发链路：

- universal macOS app bundle；
- icons、Info.plist、entitlements；
- codesign；
- notarization；
- DMG/zip artifact；
- update feed；
- in-app update check；
- release performance gate。

V1.0 dev loop 不实现 packaging/updater，但需要在 specs 和 quality gates 中保留 release 准出。

### FR-20: Remote execution roadmap

第一个产品版本路线图需要包含远端执行能力，但不进入 V1.0 最小闭环：

- remote execution host registry；
- per-host account/profile 状态；
- node health；
- node usage merge；
- checkpoint/move/fork/handoff；
- SSH fallback；
- token-based node enrollment；
- no provider credential transfer。

这些能力必须作为 V1.x 独立 PRD/OpenSpec，不得混入 V1.0 本地闭环实现。

## 4. 实现方案

### 4.1 第一阶段：工程骨架

创建 Rust workspace，包含：

- `homie-proto`
- `homie-client`
- `homie-runtime`
- `homie-agents`
- `homie-llm`
- `homie-context`
- `homie-storage`
- `homie-ui`
- `homie-term`
- `homie-app`
- `homie-cli`
- SQLite schema migration 初版

此阶段验收：

- `cargo build --workspace` 通过；
- `cargo test --workspace` 通过；
- `cargo clippy --workspace -- -D warnings` 通过；
- `cargo run -p homie-app` 打开一个最小 GPUI 窗口；
- `cargo run -p homie-cli -- doctor` 输出 runtime/config 检查结果。
- `homie-storage` 能创建 `homie.sqlite` 并执行 schema migration。

开发前依赖基线：

- 必须先阅读 [Homie 大型项目目录结构规范](../../../docs/architecture/project-layout.md)。
- 必须先阅读 [Homie 开发规范](../../../docs/development/standards.md)。
- 必须先阅读 [Homie 准出门禁规范](../../../docs/development/quality-gates.md)。
- 必须先阅读 [Homie V1 Rust 包选型调研](../../../docs/research/rust-package-selection.md)。
- SQLite 优先使用 `rusqlite` + `bundled`，V1 不引入 ORM。
- LLM proxy 优先使用 `axum` + `tower` + `tower-http` + `reqwest`。
- terminal emulation 优先使用 `alacritty_terminal`。
- async/process/control socket 优先使用 `tokio`。
- CLI 优先使用 `clap` derive。
- logs/diagnostics 优先使用 `tracing`。
- cost/price 使用 `rust_decimal`，不使用 float 存金额。
- schema/config 校验使用 `serde` + `schemars`。
- PTY 实现先做 spike：`portable-pty` vs Unix seam + future ConPTY。
- encrypted local secret envelope 先做 spike：`age` vs RustCrypto primitives；未完成组件 spec 和测试向量前不得实现自定义 crypto envelope。
- 除非组件 spec 明确批准，禁止自研已有成熟 crate 能覆盖的能力。

### 4.2 第二阶段：runtime + protocol

实现本机 runtime：

- owner-only Unix socket；
- `hello`；
- `session.spawn/list/input/resize/terminate/read_output`；
- `events.subscribe`；
- session registry；
- SQLite session/config 读写；
- append-only output log；
- status reducer 最小实现。

### 4.3 第三阶段：agent adapter

实现 agent descriptor 与至少一个 adapter：

- runtime descriptor 读取；
- agent profile 读取和 effective config 冻结；
- Codex runtime adapter；
- env scrub；
- proxy env 注入；
- process/PTY spawn；
- screen/process status。

### 4.4 第四阶段：LLM proxy

实现本机 OpenAI-compatible proxy：

- provider config；
- LLM profile；
- secret ref；
- virtual key issue；
- `/v1/chat/completions` 转发；
- `/v1/models`；
- SQLite usage_records 写入；
- model_pricing 读取和 estimated_cost 计算；
- tool_call_metrics 写入；
- metrics.write_failed event；
- 安全日志。

### 4.5 第五阶段：UI 端到端

实现桌面工作台：

- session sidebar；
- terminal/output pane；
- new session flow；
- agent profile 设置；
- provider settings；
- usage summary；
- runtime connection state。

## 5. 组件 spec 影响

| 组件 | 是否影响 | 原因 | 需要更新 |
|------|----------|------|----------|
| `specs/desktop-shell/README.md` | 是 | 定义 GPUI app shell 与 UI 边界 | 后续创建 |
| `specs/runtime-supervisor/README.md` | 是 | 定义后台 runtime、session、PTY、生命周期 | 后续创建 |
| `specs/agent-adapter-contract/README.md` | 是 | 定义 Codex 默认 runtime、runtime descriptor、agent profile 和 adapter contract | 后续创建 |
| `specs/llm-proxy/README.md` | 是 | 定义 OpenAI-compatible proxy | 后续创建 |
| `specs/virtual-key-credentials/README.md` | 是 | 定义 encrypted local secret envelope、真实 key、virtual key、secret ref | 后续创建 |
| `specs/session-context-store/README.md` | 是 | 定义 context event/session context | 后续创建 |
| `specs/storage-indexing/README.md` | 是 | 定义 SQLite schema、migration、WAL、外键关系和 output log 索引 | 后续创建 |
| `specs/memory-controller/README.md` | 是 | 定义 V1 no-op/minimal memory 边界 | 后续创建 |
| `specs/task-controller/README.md` | 是 | 定义 V1 task model 与 Beads 边界 | 后续创建 |
| `specs/intent-orchestrator/README.md` | 是 | 定义 V1 intent routing | 后续创建 |
| `specs/observability/README.md` | 是 | 定义 LLM tokens、估算成本、请求耗时、工具调用耗时和聚合维度 | 后续创建 |
| `specs/worktree-controller/README.md` | 是 | 定义 workspace/worktree 创建、列表、清理和 overview | 后续创建 |
| `specs/session-lifecycle/README.md` | 是 | 定义 archive、hibernate、wake、reopen、history resume | 后续创建 |
| `specs/native-system-integration/README.md` | 是 | 定义 menu bar、notification、sounds、window chrome、Swift/native seam | 后续创建 |
| `specs/packaging-updater/README.md` | 是 | 定义 app bundle、签名、公证、更新和 release gate | 后续创建 |
| `specs/remote-execution/README.md` | 是 | 定义 remote host/node/account/handoff roadmap | V1.x |
| `specs/homie-mcp-control/README.md` | 是 | 定义 Homie MCP control surface，不等同于 MCP server proxy | 后续创建 |

## 6. 边界情况

| 场景 | 处理方式 |
|------|----------|
| runtime socket 存在但无服务 | client 尝试连接失败后提示 runtime unhealthy，可由 app 重启 runtime |
| state 文件损坏 | quarantine，不覆盖原文件，UI 显示恢复错误 |
| agent binary 不存在 | session.spawn 返回 `agent_not_found`，UI 提示安装或配置路径 |
| provider key 未配置 | virtual key 不签发，agent session 可启动但 LLM proxy 标记 unavailable |
| virtual key 过期 | proxy 返回 401 `virtual_key_expired`，runtime 可为 active session 重新签发 |
| agent 输出过快 | output log 持续写入，UI 只消费 snapshot/diff，避免阻塞 PTY pump |
| UI 关闭 | V1 不承诺 runtime 继续运行；退出前 flush SQLite state/context/usage，重启后展示历史状态 |
| runtime 崩溃 | V1 至少保留 state/output/context；自动恢复 live PTY 可作为 V1.1 |

## 7. 测试计划

### 7.1 单元测试

- protocol encode/decode；
- unknown enum lenient decode；
- agent env scrub；
- virtual key scope/expiry/revoke；
- provider config model alias；
- model pricing cost calculation；
- LLM usage latency aggregation；
- tool call latency metric recording；
- storage atomic write；
- context event append；
- status reducer。

### 7.2 集成测试

- runtime control socket hello/list；
- spawn shell/agent fake process；
- input/output log；
- session terminate；
- LLM proxy fake provider；
- virtual key cannot access wrong session/provider/model；
- `usage_records` 记录 token 使用、缓存命中率、估算成本、首 token 延迟和总耗时；
- `tool_call_metrics` 记录 Homie 直接代理的工具调用耗时和 safe error code；MCP tool proxy 相关用例后续补充；
- state reload。

### 7.3 E2E 验证

- 启动 app；
- 配置 fake provider；
- 创建 agent session；
- agent 通过 proxy 请求 fake provider；
- UI 展示输出与 usage；
- UI 展示当前 session token 使用、缓存命中率、估算成本和 Homie 直接代理的工具调用耗时摘要；
- 重启 app 后 session 列表仍可见。

### 7.4 安全测试

- agent env 不包含真实 provider key；
- logs/events/context/memory 不包含 Authorization/raw key；
- expired/revoked virtual key 被拒绝；
- provider request failure 不泄漏 header；
- usage/tool metrics 不记录 raw request、raw response、tool args 或 result；
- `.githooks/pre-commit` 保持通过。

## 8. 验收标准

- `prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md` 被评审通过。
- 对应组件 spec 创建并覆盖 P0 合同。
- OpenSpec 拆解出可实现任务。
- Rust workspace 能构建并运行最小 GPUI app。
- 本机 runtime 能通过协议启动至少一个真实或 fake agent session。
- agent LLM 请求经过 Homie virtual key 和 proxy。
- LLM proxy 写入 token usage、cache hit rate、estimated cost、request latency 和 safe error code。
- Homie 直接代理的 tool 调用写入耗时指标；MCP server proxy 后续实现时补充 MCP tool 指标 E2E。
- 真实 provider key 不出现在 agent env、日志、events、context、report 中。
- UI 能展示 session 列表、active session 输出和基本状态。

## 9. Beads 追踪

- Beads issue: `homie-9c9`
- change_id: `homie-v1-architecture`
- spec-id: `prd-spec/features/homie-v1-architecture/2026-08-05-homie-v1-architecture-design.md`

后续应拆分子 issue：

- `homie-runtime-p0`
- `homie-llm-proxy-p0`
- `homie-desktop-shell-p0`
- `homie-agent-adapter-p0`
- `homie-context-store-p0`
- `homie-worktree-controller`
- `homie-session-lifecycle`
- `homie-native-system-integration`
- `homie-packaging-updater`
- `homie-mcp-control-surface`
- `homie-remote-execution-roadmap`

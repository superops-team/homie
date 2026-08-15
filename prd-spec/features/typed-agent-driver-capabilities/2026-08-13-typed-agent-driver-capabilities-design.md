# Typed Agent Driver Capability 设计文档

## 1. 概述

### 1.1 问题/背景

Homie 当前 agent 体系以 PTY + manifest 为核心：manifest 描述二进制、resume、注入、approve/deny 和屏幕检测规则。这个设计对长尾 agent 很高效，新增 agent 可以是数据变更。

但一等 agent 正在出现更多结构化能力需求：

- 在运行中的 turn 中追加/steer 用户消息；
- 精确 cancel 当前 turn，而不是只依赖 PTY 输入或进程信号；
- rollback/fork 到 provider 原生会话游标；
- model discovery、reasoning effort、service tier；
- rich activity、background work、usage event；
- provider permission request 的结构化响应。

这些能力很难只靠 terminal screen detection 表达。Waku 的 `DriverControl` / `DriverEvent` 提供了一个参考：对支持结构化协议的 provider 使用 typed driver，统一输出 provider-neutral events；对能力缺失的 provider 返回 unsupported。Homie 可以引入可选 typed capability 层，同时保留 manifest 作为基础运行和长尾 agent 机制。

### 1.2 目标

1. 为 first-class agents 定义 typed capability layer。
2. 保留现有 manifest/PTY/holder/status reducer 架构，不做替换式重写。
3. 让 UI/CLI/MCP 能查询某个 session 支持哪些能力。
4. 优先支持最小能力集：`cancel_turn`、`steer_message`、`model_options`、`native_resume_cursor`。
5. 为后续 rollback/fork、usage、background work 留出协议边界。

### 1.3 非目标

- 不把所有 agent 都改为 typed driver。
- 不取消 manifest screen detection。
- 不在第一阶段实现 computer use 或 browser automation。
- 不引入 provider 云账号体系。
- 不要求所有 provider 能力一致。

## 2. 用户场景

### 场景 1: 运行中追加指令

**Given** Codex/Claude/OpenCode session 正在执行。  
**When** 用户在 composer 中输入补充说明并点击 steer。  
**Then** 如果 provider 支持 steer，Homie 将消息注入当前 turn；如果不支持，UI 给出明确 fallback：排队为下一轮或提示不支持。

### 场景 2: 精确取消当前 turn

**Given** agent 正在运行工具或流式输出。  
**When** 用户点击 Stop。  
**Then** Homie 优先调用 typed driver 的 cancel；不支持时回退到现有 PTY/进程策略。

### 场景 3: 模型选择

**Given** 用户创建一等 agent session。  
**When** 打开 model picker。  
**Then** Homie 显示该 provider 当前可用模型、reasoning effort、service tier；如果 driver 不支持 discovery，使用静态 manifest 或隐藏高级选项。

### 场景 4: provider 原生恢复

**Given** provider 通过 hook/notify/SDK 返回原生 session id/thread id。  
**When** Homie 需要 resume/fork/rollback。  
**Then** typed capability layer 用明确的 `ProviderResumeCursor` 记录来源和 provider-specific 字段。

## 3. 功能需求

### FR-1: Capability 查询

每个 session 能返回能力集合：

- `prompt`
- `cancelTurn`
- `steerMessage`
- `respondPermission`
- `modelDiscovery`
- `nativeResumeCursor`
- `rollback`
- `fork`
- `usageEvents`
- `backgroundWork`

能力默认 false，只有 typed driver 或 manifest 明确声明后才 true。

### FR-2: Provider-neutral event

新增内部事件模型，至少包含：

- connected / disconnected；
- text delta；
- reasoning delta；
- permission request；
- turn started / turn finished；
- error；
- usage updated；
- native cursor updated。

第一阶段可只把这些 event 接入 store/Engine 内部，不必全部暴露到 UI。

### FR-3: Typed driver 不破坏 PTY continuity

现有 holder、PTY、output log、screen reducer 仍是 session 生命周期基础。typed driver 是附加控制面，不拥有 session persistence 的唯一事实源。

### FR-4: Manifest 与 typed driver 协同

manifest 继续定义：

- binary/spawn args；
- env scrub；
- injection；
- screen status rules；
- approve/deny fallback；
- long-tail agent metadata。

typed driver 只为一等 agent 提供增强能力。

### FR-5: 安全边界

typed driver 事件不得记录 secret、Authorization、cookie、完整敏感 prompt payload。driver error 必须短消息化，详细日志只进内部诊断。

## 4. 实现方案

### 4.1 新模块边界

建议新增 Rust Engine 模块：

```text
homie/crates/homie-engine/src/driver/
├── mod.rs
├── capability.rs
├── event.rs
├── codex.rs
├── claude.rs
└── opencode.rs
```

核心 trait：

```rust
trait AgentDriverControl {
    fn capabilities(&self) -> DriverCapabilities;
    fn cancel_turn(&self) -> DriverResult<()>;
    fn steer_message(&self, text: String) -> DriverResult<()>;
    fn respond_permission(&self, request_id: String, option_id: String) -> DriverResult<()>;
    fn model_options(&self) -> DriverResult<Vec<ModelOption>>;
}
```

所有方法默认 unsupported，避免每个 provider 被迫实现所有能力。

首阶段只实现抽象、fake driver 和 capability 查询链路，不接入真实 Codex/Claude/OpenCode provider。真实 provider driver 必须作为后续 child Bead/独立 OpenSpec 执行，并分别证明其 native API、权限请求、日志脱敏和 fallback 行为。

### 4.2 Session 集成

`Session` 增加可选 `driver_handle`：

- spawn 时按 agent id/provider 判断是否创建；
- holder/PTY 仍照常启动；
- driver event 进入 status reducer 或 store projection；
- driver unavailable 不影响基础 terminal session。

authority 规则：

- holder/PTY/output log/screen reducer 仍是 session 生命周期和屏幕状态的事实源。
- typed driver 只能提供增强控制与结构化 signal，不可绕过 holder 启动、session persistence 或现有安全清理。
- driver capability 必须在 session snapshot 中以显式 `supported/unsupported` 暴露，不允许 UI 根据 agent id 猜测。
- driver event 若与 screen reducer 冲突，第一阶段只记录 diagnostic，不改变 visible session status。

### 4.3 协议暴露

后续 OpenSpec 需要定义是否新增 control methods：

- `session.capabilities`
- `session.steer`
- `session.cancel_turn`
- `agent.models`

第一阶段可以先内部接入 `agent.readiness` 或 session snapshot，避免协议一次扩太大。

首阶段协议限制：

- 可以只在现有 session snapshot/model 中暴露 capabilities。
- 不新增 `session.steer` 或 `session.cancel_turn` wire method，除非 OpenSpec 证明 UI/CLI 调用路径、fallback、权限和测试证据齐备。
- 不修改 MCP tool surface；MCP 暴露 typed control 另开变更。

### 4.4 Provider 优先级

建议顺序：

1. Codex：已有 notify/MCP 注入基础，收益高。
2. Claude Code：已有 hook/MCP 注入基础，但 stdin/permission 行为需谨慎。
3. OpenCode：如本地 server/API 可用，适合 typed control。
4. Cursor/Grok/Gemini 等后续评估。

### 4.5 首阶段关闭口径

`homie-kcq` 首阶段只关闭 capability layer 的最小可验证骨架：

- `DriverCapabilities`、unsupported error、fake driver contract tests；
- session snapshot 或等价查询能显示 fake driver capabilities；
- 无 typed driver 的 manifest agent 行为不变；
- typed cancel/steer 真实 provider 接入不在本阶段；
- 安全脱敏规则写入 tests 或 review evidence。

## 5. 边界情况

| 场景 | 处理方式 |
|------|----------|
| driver 启动失败 | session 仍以 PTY/manifest 模式运行，capabilities 降级 |
| steer 不支持 | UI 显示排队/不支持，而不是静默失败 |
| cancel typed 失败 | 回退现有停止策略，并记录诊断 |
| provider 返回未知事件 | 丢弃或记录为 diagnostic，不影响 session |
| driver 与 screen reducer 状态冲突 | reducer 仍按既定 authority 决策，typed event 进入明确定义的 signal |

## 6. 涉及文件

- `homie/crates/homie-engine/src/session.rs`
- `homie/crates/homie-engine/src/control.rs`
- `homie/crates/homie-engine/src/agent.rs`
- `homie/crates/homie-engine/src/status/*`
- `homie/crates/homie-engine/src/mcp/*`
- `homie/crates/homie-proto/src/methods.rs`
- `homie/crates/homie-proto/src/model.rs`
- `homie/crates/homie-client/src/*`
- `homie/crates/homie-app/src/store/mod.rs`
- `homie/crates/homie-app/src/composer.rs`
- `homie/crates/homie-app/src/launcher.rs`
- `homie/crates/homie-app/src/settings.rs`
- agent manifests for first-class agents

## 7. 验证计划

### 7.1 单元测试

- unsupported driver 默认返回明确错误。
- capability serialization 稳定。
- driver event 转 reducer signal 不泄漏敏感内容。
- typed cancel 失败时 fallback 被调用。

### 7.2 集成测试

- 使用 fake driver 启动 session，验证：
  - capabilities 出现在 snapshot；
  - unsupported 操作返回稳定错误；
  - driver event 不覆盖 screen reducer visible status；
  - cancel/steer fallback 只在 OpenSpec 明确启用时才执行。
- 真实 Codex/Claude smoke 不属于首阶段关闭条件；每个真实 provider 接入需要独立 child change。

### 7.3 回归测试

- 没有 typed driver 的 agent 仍可正常 spawn/resume/status detection。
- shell/generic 不暴露 typed capabilities。
- holder adoption 后 capabilities 可恢复或明确降级。

### 7.4 风险控制

| 风险 | 控制 |
|------|------|
| typed driver 变成替代运行时，破坏 PTY continuity | 首阶段只做 fake driver 和 capability 查询，holder/PTY 仍是事实源 |
| provider API 细节未确认导致抽象漂移 | 真实 provider 接入拆 child Bead；本阶段方法默认 unsupported |
| event 泄漏敏感 prompt/token | fake event 和脱敏测试覆盖；禁止记录 Authorization、cookie、完整 prompt payload |
| UI/MCP 一次性扩面 | 首阶段不新增 MCP tool，不新增 steer/cancel wire method |
| capability 与 screen status 冲突 | typed event 只作 diagnostic/signal，不覆盖 visible status |

## 8. 验收标准

1. Homie 有明确 typed driver capability 抽象。
2. 至少一个 fake driver 覆盖 prompt/cancel/steer/model discovery 的 contract test。
3. 现有 manifest agent 行为不回退。
4. UI/CLI 能知道一个 session 是否支持 steer/cancel 等能力。
5. OpenSpec alignment 明确首阶段不接真实 provider、不改 MCP、不改变 session authority。
6. Beads `homie-kcq` 更新为已验证状态后才可关闭。

## 9. Beads 追踪

- Beads: `homie-kcq`
- change_id: `typed-agent-driver-capabilities`
- 类型: feature
- 优先级: P1

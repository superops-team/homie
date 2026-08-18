# Codex ACP Harness 与 Chat Session Surface 设计文档

## 1. 概述

### 1.1 问题/背景

Homie 当前的 agent 体系以 **PTY + manifest + 屏幕检测** 为核心：每个 session 是一个真实
terminal，manifest 描述二进制、resume、injection、approve/deny 与 screen rules，状态检测
靠读取 agent 在屏幕上绘制的内容。这套设计对长尾 agent 很高效，但存在结构性瓶颈：

- 一等 agent（Codex/Claude Code/OpenCode）的对话本体是**结构化消息流**（用户 turn、助手
  message/thinking、tool call、permission request、plan），不是终端字节流。靠屏幕检测推断
  "working / needs-you / done" 只能做到概率正确，无法精确表达 tool call 的 pending/running/
  completed/failed、permission 请求的具体选项、以及 turn 级别的 start/finish。
- PTY transcript 是**单维字节流**，无法自然渲染富文本消息、tool 卡片、diff、permission 弹窗。
  用户看到的是一屏终端，而非一个可交互的 chat canvas。
- `typed-agent-driver-capabilities`（homie-kcq）已为结构化能力定义了 provider-neutral 的
  `DriverCapabilities`/`DriverEvent` 抽象与 fake driver 骨架，但明确**未接入任何真实
  provider**，真实 provider driver 被拆为独立 child Bead。
- 业界已出现开放标准 **ACP（Agent Client Protocol，agentclientprotocol.com）**：JSON-RPC 2.0
  over stdio，允许任意 editor/host 与 coding agent 双向通信。OpenAI 提供 `codex-acp` crate，
  将 Codex CLI 包装成 ACP server。这为 Homie 提供了一条**不依赖终端屏幕**的结构化接入路径。

本 PRD 是 Homie 首个 **ACP harness + 非 PTY Chat Session Surface** 的纵向切片：以 Codex 为
第一个真实 provider，通过 `codex-acp` 建立 ACP host 侧 harness，把结构化 session/event 投影
到 GPUI 的 New Session chat canvas，包含 composer 的 send/steer/stop、visual transcript、
approval 语义，并配套 Apple/design-engineering 项目级视觉与动效规范。

### 1.2 目标

1. 明确并锁定协议选型：**ACP + pinned `codex-acp`** 作为结构化接入路径；direct app-server
   只作 adapter 内部/历史参考，不作为正式运行时协议。
2. 定义 backend harness 的模块边界：ACP host 进程管理、`initialize`/`session/*` 握手、
   `session/update` 事件流、`fs/*` 文件代理、permission 请求转发。
3. 定义 typed session/capability：session 级能力协商、结构化 transcript 数据模型、event 投影
   到 UI 的边界（与 `typed-agent-driver-capabilities` 的 `DriverCapabilities`/`DriverEvent`
   对齐，不另起一套冲突模型）。
4. 定义 GPUI New Session chat canvas：composer 的 send / steer / stop、visual transcript/
   event projection、非 PTY 渲染路径。
5. 定义 approval 语义：`allow`/`deny`（once）+ `always allow`/`always deny`（for session）。
6. 学习并固化 **Comet GPUI chat 模块边界**，评估 **gpui-base / gpui-component 组件复用
   兼容性门禁**，输出 Apple/design-engineering 项目级视觉与动效规范。
7. 本阶段交付**设计与规范**，不承诺完成真实 provider 端到端运行（真实 `codex-acp` 进程对接
   属后续 child Bead 的代码落地）。

### 1.3 非目标

- 不把 ACP harness 变成唯一 session 运行时；PTY/manifest/holder/screen reducer 仍是长尾
  agent 与 shell session 的基础。ACP 是**附加结构化控制面与投影面**。
- 不一次性接入 Claude Code / OpenCode 的真实 ACP server；本阶段只锁定 Codex 作为首个 provider，
  其他 provider 属后续 child Bead。
- 不在本阶段实现 computer use / browser automation。
- 不引入 provider 云账号体系；认证走现有本地配置与 `codex-acp` 的认证代理语义。
- 不把 composer 做成通用富文本编辑器；首阶段只支持纯文本输入 + send/steer/stop。
- 不覆盖 Swift CLI/MCP 的 typed control 暴露；MCP 面另开变更。

## 2. 用户场景

### 场景 1: 新建非 PTY 会话并对话

**Given** 用户选择 Codex 并创建 New Session。  
**When** 会话以 ACP host 启动（而非 PTY terminal）。  
**Then** Homie 展示 chat canvas：composer 在底部，transcript 按 message/tool/plan 结构化渲染；
用户在 composer 输入并 send，Codex 的响应以消息流实时追加，而非终端字节。

### 场景 2: 运行中 steer

**Given** Codex turn 正在运行（可能正在 tool call）。  
**When** 用户在 composer 输入补充说明并点击 steer。  
**Then** Homie 通过 ACP 将消息注入当前 turn；若 provider 不支持 steer，UI 明确 fallback
（排队为下一轮或提示不支持），不静默失败。

### 场景 3: 精确 stop

**Given** turn 正在流式输出或执行工具。  
**When** 用户点击 Stop。  
**Then** Homie 通过 `session/stop`（保持 session）或 `session/cancel`（终止）语义停止当前
turn，并给出明确视觉状态（stopping → stopped）。

### 场景 4: 审批 tool / permission

**Given** Codex 发起需要审批的操作（文件写、命令执行、permission request）。  
**When** 事件到达 transcript。  
**Then** Homie 渲染审批卡片，用户可选 allow / deny（once）或 always allow / always deny
（for session）；选择映射回 ACP permission 响应。

### 场景 5: 会话恢复

**Given** 已有 ACP 会话。  
**When** Homie 重启或重新打开该 session。  
**Then** 通过 `session/load` 恢复结构化 transcript（而非 PTY 重放）；若无法恢复则明确降级为
不可恢复状态并提示。

## 3. 功能需求

### FR-1: ACP 协议选型与 pinning

- 协议统一使用 **ACP**（JSON-RPC 2.0 over stdio）。
- `codex-acp` 使用 pinned 版本（与 gpui 一样 pin 到具体 commit/rev，写入 `homie/Cargo.toml`
  的 `[workspace.dependencies]`），避免上游漂移。
- direct app-server 仅作为 adapter 内部/历史参考实现保留，不进入正式运行时路径、不新增依赖。

### FR-2: ACP host harness（backend）

新增 Rust Engine 模块承载 ACP host 职责：

- 启动/托管 `codex-acp` 子进程（stdio 双向 JSON-RPC）；
- `initialize` 握手与能力协商；
- `session/new` / `session/load` / `session/prompt` / `session/stop` / `session/cancel` /
  `session/set_model` 等 host→agent 调用；
- 接收 `session/update` 通知（agent_message_changed / agent_thought_changed / plan /
  tool_call / available_commands_update / current_mode_update / session_status_update）；
- 处理 `fs/read_text_file` / `fs/update_text_file` 等 host 服务的请求（agent→host）；
- 认证/登录代理（`authenticate` 与 provider 本地配置协同）。

harness 是**有界异步运行时**：每个 ACP session 一个 host loop，事件经 channel 进入
store/reducer，不阻塞 UI 主线程。

### FR-3: typed session/capability 对齐

- 复用 `typed-agent-driver-capabilities` 定义的 `DriverCapabilities` / `DriverEvent` 抽象，
  ACP 作为其**第一个真实 provider driver** 实现（`codex-acp` driver）。
- session snapshot 暴露结构化能力（prompt / steerMessage / cancelTurn / respondPermission /
  modelDiscovery / nativeResumeCursor / usageEvents），由 driver 显式声明，不允许 UI 根据
  agent id 猜测。
- ACP event 投影为 provider-neutral `DriverEvent`，进入 store/reducer；与 screen reducer 冲突
  时，ACP session 以结构化 event 为事实源，但仍遵守既有安全脱敏与 authority 规则。

### FR-4: GPUI New Session chat canvas

新增 GPUI 表面承载非 PTY 对话：

- **composer**：底部输入区，send / steer / stop 三个动作；纯文本输入（复用/适配现有
  `query_editor` 文本路径，不新造文本系统）。
- **transcript**：按 message（user/assistant）、thinking、tool call（含 status）、plan、
  permission request 结构化渲染；支持滚动、增量追加、稳定 ID。
- **visual event projection**：把 `DriverEvent` 投影为视觉元素，tool call 卡片展示 pending/
  running/completed/failed，permission 渲染审批卡片。
- 会话切换、新建、恢复的生命周期与现有 sidebar/workbench 集成，不另造第二套导航。

### FR-5: approval 语义

- ACP permission/approval 支持四态：
  - `allow`（once）：仅本次请求；
  - `deny`（once）：仅本次请求；
  - `always allow`（for session）：本 session 后续同类请求自动允许；
  - `always deny`（for session）：本 session 后续同类请求自动拒绝。
- 语义规则固化在 session 级状态（按 permission kind 记忆），映射回 ACP permission 响应。
- 审批卡片必须展示可读的资源/动作摘要，禁止泄露 secret、Authorization、完整敏感 payload。

### FR-6: 设计一致性（Apple 设计原则 / design taste）

- 视觉与动效遵循 Apple HIG 的核心原则：清晰（clarity）、遵从（deference）、深度（depth），
  以及一致性、直接操纵、反馈、隐喻、用户控制。
- 建立项目级 **design tokens**（语义色、字号阶梯、圆角、间距、动效时长/缓动曲线），与现有
  `homie-ui` tokens 对齐，chat canvas 不引入第二套视觉语言。
- 动效遵循 `specs/gpui-interaction-contract.md` §7 平台偏好：reduce motion 时 snap/短 fade，
  无连续动画帧需求。
- 输出独立的 Apple/design-engineering 规范文档（见 §6 涉及文件），作为后续 UI 实现的一致性
  依据。

### FR-7: Comet GPUI 模块边界学习 + gpui-base/gpui-component 兼容性门禁

- 学习 Zed Comet 的 chat 模块如何拆分（message/transcript/composer/input 的边界与生命周期），
  提取可复用的**模块边界原则**，而非直接拷贝代码。
- 评估 gpui-base / gpui-component（Zed 拆出的 GPUI 组件库）与当前 pinned gpui revision 的
  **兼容性门禁**：是否能复用其组件（Button/ListRow/Dialog/TextField 等）而不引入版本冲突或
  GPL 污染；若不可复用，明确自研 primitive 的边界（对齐 `specs/ui-components.md`）。
- 该评估产出为**决策记录**，不直接引入未验证的新依赖。

## 4. 实现方案

### 4.1 模块边界（backend）

建议新增 Rust Engine 模块：

```text
homie/crates/homie-engine/src/acp/
├── mod.rs            # ACP host 入口与生命周期
├── host.rs           # 子进程托管 + JSON-RPC stdio 循环
├── protocol.rs       # ACP JSON-RPC 消息/通知 DTO（serde）
├── session.rs        # ACP session 状态机（new/load/prompt/stop/cancel）
├── event.rs          # ACP event -> DriverEvent 投影
├── approval.rs       # permission 请求与四态审批记忆
└── fs_proxy.rs       # fs/read_text_file / fs/update_text_file 代理
```

核心 trait（与 typed-agent-driver-capabilities 对齐）：

```rust
trait AgentDriverControl {
    fn capabilities(&self) -> DriverCapabilities;
    fn prompt(&self, text: String) -> DriverResult<()>;
    fn steer_message(&self, text: String) -> DriverResult<()>;
    fn cancel_turn(&self) -> DriverResult<()>;
    fn respond_permission(&self, request_id: String, option: PermissionOption) -> DriverResult<()>;
}
```

`codex-acp` driver 实现上述 trait，能力由 `initialize` 协商结果填充。

### 4.2 ACP 数据模型（非 PTY transcript）

结构化 transcript 是 ACP session 的投影，而非终端字节：

```text
SessionTurn { id, role: User|Assistant, kind: Message|Thinking|ToolCall|Plan|Permission, status, blocks }
MessageBlock { text_delta, attachments? }
ToolCallBlock { tool, input_summary, status: Pending|Running|Completed|Failed, output_summary? }
PermissionBlock { request_id, kind, resource, options: [AllowOnce|DenyOnce|AlwaysAllow|AlwaysDeny] }
```

transcript 存储到持久层（复用现有 registry/persistence 的 session 存储，避免另起 schema），
支持 detach 后 `session/load` 恢复。

### 4.3 GPUI 表面边界（frontend）

```text
homie/crates/homie-app/src/chat/
├── mod.rs            # ChatSurface 实体
├── transcript.rs     # transcript 渲染（message/tool/plan/permission）
├── composer.rs       # composer 输入 + send/steer/stop
├── approval_view.rs  # 审批卡片
└── projection.rs     # DriverEvent -> 视觉元素投影
```

遵循 `specs/gpui-shell.md` 的 render contract：render 只读 prepared state、派生有界展示值、
构建元素树，不启动任务、不访问磁盘/网络/进程、不 mutate 域状态。

### 4.4 认证与凭据

- 复用现有本地配置与 `authenticate` 语义，真实 provider key 仍由 Homie 本地配置持有，
  不泄漏到 managed agent 配置。
- ACP 认证流（浏览器登录等）作为 provider 侧实现细节，harness 只代理握手结果。

### 4.5 首阶段关闭口径（设计交付）

本 PRD 是**设计/规范交付**，关闭口径为：

- ACP 协议选型与 pinning 决策已固化为文档；
- backend harness 模块边界、数据模型、event 投影契约已定义；
- GPUI chat canvas 的模块边界、composer/transcript/approval 交互契约已定义；
- approval 四态语义已定义；
- Comet 模块边界学习结论 + gpui-base/gpui-component 兼容性门禁结论已记录；
- Apple/design-engineering 视觉与动效规范文档已产出；
- 真实 `codex-acp` 进程端到端运行与完整 UI 实现属后续 child Bead，不属本阶段。

## 5. 边界情况

| 场景 | 处理方式 |
|------|----------|
| ACP 子进程启动失败 | session 降级为 PTY/manifest 模式或明确报错，不静默 |
| `initialize` 协商失败 | 记录 diagnostic，capability 置 unsupported |
| provider 不支持 steer | UI 明确 fallback（排队/不支持） |
| `session/stop` 与 `session/cancel` 语义 | stop 保持 session，cancel 终止；UI 展示 stopping→stopped |
| fs 代理请求越权 | 按工作区/权限策略拒绝，记录 diagnostic |
| permission 事件含敏感内容 | 渲染前脱敏，禁止记录 secret |
| ACP event 与 screen reducer 冲突 | ACP session 以结构化 event 为事实源，遵守安全 authority |
| transcript 无法恢复 | 明确降级为不可恢复并提示 |

## 6. 涉及文件/规范

- `homie/crates/homie-engine/src/acp/*`（新增）
- `homie/crates/homie-engine/src/driver/*`（typed-agent-driver-capabilities 既有，对齐扩展）
- `homie/crates/homie-app/src/chat/*`（新增）
- `homie/crates/homie-proto/src/*`（session snapshot 能力字段，如需要）
- `specs/gpui-shell.md`、`specs/gpui-interaction-contract.md`、`specs/ui-components.md`
  （chat canvas 遵守并可能补充 chat 表面契约）
- `specs/engine-session-runtime.md`（ACP harness 与 PTY authority 的边界）
- `docs/design/apple-design-principles.md`（新增，Apple/design-engineering 规范）
- `docs/research/comet-gpui-chat-boundaries.md`（新增，Comet 学习结论）
- `docs/research/gpui-component-compat-gate.md`（新增，兼容性门禁结论）
- `homie/Cargo.toml`（pinned `codex-acp` 依赖，需 license 审计）

## 7. 验证计划

### 7.1 设计验证（本阶段）

- ACP 协议 DTO 与 `codex-acp` 实际 API 对齐（以 pinned 版本为准，需真实拉取验证或明确标注
  "以 pinned rev 为准待代码阶段确认"）。
- 数据模型/event 投影契约经 spec review，无空泛表述。
- Apple/design 规范覆盖清晰/遵从/深度与 reduce-motion 等平台偏好。
- gpui-base/gpui-component 兼容性门禁结论可追溯（license、版本、能力覆盖）。

### 7.2 后续代码阶段验证（child Bead 交付，本阶段只列出）

- harness 单测：JSON-RPC 编解码、event→DriverEvent 投影、approval 四态记忆。
- 集成：fake ACP server 下 `session/new`→`prompt`→`session/update` 全链路。
- GPUI：composer send/steer/stop、transcript 增量渲染、审批卡片四态。
- 回归：无 ACP 的 manifest agent 行为不变；shell/generic 不暴露 ACP 能力。

### 7.3 风险控制

| 风险 | 控制 |
|------|------|
| ACP 版本漂移导致抽象失效 | pin `codex-acp` 到具体 rev，DTO 以 pinned rev 为准 |
| ACP harness 变成替代运行时破坏 PTY continuity | 明确 ACP 是附加面，PTY/manifest 仍为基础 |
| gpui-base/gpui-component 引入版本冲突或 GPL 污染 | 兼容性门禁先于任何依赖引入，license 审计前置 |
| 设计规范空泛、无法落地 | spec review 阶段按 16 维度检查，拒绝空泛表述 |
| approval 泄露敏感内容 | 脱敏规则写入设计，代码阶段以测试固化 |

## 8. 验收标准

1. ACP 协议选型与 `codex-acp` pinning 决策已固化为文档。
2. backend harness 模块边界、ACP 数据模型、event 投影契约已定义且经 spec review。
3. GPUI chat canvas 的 composer/transcript/approval 交互契约已定义。
4. approval 四态（allow/deny once + always allow/deny for session）语义已定义。
5. Comet 模块边界学习结论 + gpui-base/gpui-component 兼容性门禁结论已记录。
6. Apple/design-engineering 视觉与动效规范文档已产出。
7. OpenSpec 拆解与 alignment 对齐本 PRD，明确本阶段为设计交付、真实 provider 对接属后续。
8. Beads `homie-sc6` 更新为已验证状态后才可关闭。

## 9. Beads 追踪

- Beads: `homie-sc6`
- change_id: `codex-acp-harness-runtime`
- 类型: feature
- 优先级: P0
- 关联: `homie-kcq`（typed-agent-driver-capabilities，上游抽象）

# Desktop Shell 组件规格

## 1. 组件定位

`homie-app` 和 `homie-ui` 负责 Homie 的 GPUI 桌面工作台、设计系统、窗口、侧边栏、终端 pane、浮层、右侧 inspector、设置页、菜单栏、通知和用户交互。它只消费 `homie-client` 与 `homie-proto` 提供的状态，不直接拥有 runtime、PTY、SQLite 或 provider credential。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- Gap-closure PRD: `prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- Gap-closure OpenSpec: `openspec/changes/diri-engine-migration/`
- 功能验证: `docs/verification/reference-parity-v1/functional-cases.md`
- 覆盖 Case: FC-008, FC-009, FC-012, FC-017, FC-018
- Gap-closure Case: FC-DIRI-007, FC-DIRI-008

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | 用户输入 | 键盘、鼠标、菜单、通知 action |
| 下游 | `homie-client` | 订阅 runtime 状态、发送 session 操作 |
| 下游 | `homie-proto` | 使用稳定 DTO、事件、错误模型 |
| 下游 | `homie-term` | 挂载 terminal grid、selection、find |
| 下游 | macOS bridge | 菜单栏、通知、窗口 chrome、更新入口 |

## 4. 职责边界

负责：

- Reference parity 设计 token、brand mark、status glyph 和 floating surface recipe。
- 主窗口、侧边栏、terminal pane、command palette、quick open、overview、history、worktrees sheet、settings、right inspector。
- 键盘映射、焦点、Esc cascade、multi-select、drag reorder、hover card、inline rename。
- UI fidelity preview fixtures 和 screenshot gate。
- live-connected workbench：通过 `homie-client` 读取 session projection、attach selected session、发送输入和 resize，不允许再用只改本地文案的假操作替代真实 runtime action。

不负责：

- PTY、agent process、runtime recovery。
- SQLite repository 和 migration。
- 真实 provider key、virtual key 签发。
- MCP tool 执行。
- live session registry 或直接调用 runtime/storage 写状态。

## 5. 核心接口

```rust
pub trait ShellClient {
    fn subscribe_state(&self) -> StateStream;
    fn dispatch(&self, command: ShellCommand) -> Result<(), ShellError>;
}

pub enum ShellCommand {
    SpawnSession(SpawnSessionUiRequest),
    SelectSession(SessionId),
    SendText { session_id: SessionId, text: String, submit: bool },
    ResizeSession { session_id: SessionId, cols: u16, rows: u16 },
    ArchiveSession(SessionId),
    OpenSettings(SettingsTab),
    OpenCommandPalette,
    OpenQuickOpen,
}
```

UI action 必须经过 `ShellCommand` 或 `homie-client` API，不允许直接修改 runtime/storage。

`homie-client` 已经是 `homie-app` 的 runtime 边界。`homie-app` 不允许直接依赖 `homie-runtime` 的 live registry，也不允许为了展示交互而直接写 storage session lifecycle。

## 6. 数据模型

UI projection:

```rust
pub struct WorkspaceProjection {
    pub sessions: Vec<SessionRow>,
    pub projects: Vec<ProjectSection>,
    pub selected_session: Option<SessionId>,
    pub usage: UsageDisplay,
    pub update: UpdateDisplay,
}

pub struct SessionRow {
    pub id: SessionId,
    pub title: String,
    pub agent_kind: AgentKind,
    pub status: SessionStatus,
    pub attention: AttentionLevel,
    pub cwd: String,
    pub branch: Option<String>,
    pub pinned: bool,
    pub archived: bool,
}
```

Preferences:

- window placement。
- sidebar visible/width/order/pinned/collapsed/archive。
- inspector open/width/tab。
- terminal theme/font。
- default agent、last spawn host、sounds、updates、hibernate、memory limit。

Design token source:

- Diri token baseline: `diri/diri/crates/diri-ui/src/tokens.rs`, `components.rs`, `status.rs`, `brand.rs`。
- Homie token owner: `crates/homie-ui`。
- `homie-app` 应优先消费 `homie-ui` token，不在 app shell 中新增散落的设计常量。

## 7. 运行模型与状态机

```text
app start
  -> load preferences
  -> connect homie runtime through homie-client
  -> subscribe state/events
  -> render shell projection
  -> user action dispatches ShellCommand
  -> runtime event updates projection
```

UI 状态机必须支持：

- disconnected / connecting / connected / degraded。
- no session / selected session / archived session / remote active。
- command palette、quick open、history、settings、worktrees、overview 等互斥浮层。
- live-connected 是默认数据模式；preview-only 只能作为显式测试/降级场景，且不得暴露会失败的 live 操作按钮为真实操作。

## 8. 安全与权限

- UI 不显示 raw provider key、Authorization、cookie、完整 tool args/result。
- notification approve/deny 必须来自 manifest 声明，未知 agent 不显示 quick approve。
- browser/test screenshot 只显示文件路径或用户主动打开，不内联敏感 bytes。
- Remote settings 中 token file path 可显示，token 内容不可显示。

## 9. 可观测性

必须记录 safe UI events：

- shell.connected / shell.disconnected。
- shell.command_dispatched。
- shell.render_degraded。
- shell.notification_action。
- shell.update_action。

日志中只允许 session id、agent kind、safe status、错误码和脱敏路径。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| runtime 断连 | 显示 disconnected 状态并 backoff 重连 |
| event gap | 请求 state snapshot |
| terminal attach 失败 | 显示 retry overlay |
| settings 保存失败 | 显示错误，不更新本地 optimistic state |
| notification action 失败 | 记录 safe error 并提示用户回到 app |

## 11. 测试计划与验收引用

- FC-008: Terminal interaction。
- FC-009: Desktop shell and UI fidelity。
- FC-012: Artifact surfaces。
- FC-017: update UI flow。
- FC-018: full local quality gate。
- FC-DIRI-007: Diri design token parity。
- FC-DIRI-008: app shell placeholder-copy regression and compile smoke。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M01-F001, M01-F002, M02-F001, M02-F002, M03-F001, M04-F001, M05-F001, M06-F001, M08-F001, M09-F001, M19-F002, M20-F002 UI |
| Required Diri test mapping | Diri app screenshots and UI interaction flows |
| Pre-implementation gaps | UI state matrix and side-by-side screenshot gates |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- Requirements: FR-06, FR-07, FR-08, FR-09, FR-16
- Beads: `homie-t3u`

本组件当前状态是 `partial`。`homie-app` 仍直接依赖 `homie-storage`，部分 pin/archive/settings/remote/update 操作只修改本地 state 或提示文案。这些路径违反本组件第 4、5 节合同，必须删除而不是保留 fallback。

### 12.1 强制数据流

```text
user action
  -> typed ShellCommand
  -> homie-client
  -> owning service/runtime
  -> durable write or live operation
  -> authoritative event/snapshot
  -> UI projection update
```

- `homie-app` 的 production dependencies 不得包含 `homie-runtime` 或 `homie-storage`。
- 本地 optimistic state 只能用于可回滚的视觉过渡；服务失败时不得保留成功状态。
- settings、pin、archive、order、worktree、remote、notification action 和 update action 都必须有 typed command。
- Inspector、History、Usage、Quick Open 和 Terminal 必须消费真实 projection，不以 fixture 作为默认数据源。

### 12.2 完整 UI 状态矩阵

必须覆盖：

- runtime: connecting、connected、degraded、disconnected、reconnecting；
- session: none、starting、running、needs_input、idle、hibernated、archived、exited、unreachable；
- overlays: palette、file quick open、switcher、history、settings、worktrees、overview、terminal find；
- inspector: Info、Changes、Artifacts；
- native: menu bar、notification、sound、approve/deny、update；
- layout: narrow/minimum/standard/large window；
- input: keyboard navigation、focus restore、Esc cascade、multi-select、drag/hover/rename。

### 12.3 验证规则

- 行为测试必须调用 ShellCommand/client seam 并断言 service/event 结果。
- source-code substring 测试不能作为功能准出；只允许用于极窄的静态 guard，且不得因 rustfmt 换行失败。
- UI parity 必须同时包含结构断言、真实 app interaction 和 Diri/Homie side-by-side screenshot。
- 首帧测试必须证明 GPUI 线程不执行阻塞 runtime、SQLite、process 或 network 调用。
- M01-M09 只有在真实 runtime-backed flow 和 visual/interaction evidence 都通过后才能按模块声明完成。

## 13. Wave 1A Runtime Client Bridge 修订

权威来源：

- PRD: `prd-spec/features/diri-runtime-daemon-client-transport/2026-08-08-diri-runtime-daemon-client-transport-design.md`
- OpenSpec: `openspec/changes/diri-runtime-daemon-client-transport/`
- Beads: `homie-nep`

- app 必须创建固定 2-worker Tokio runtime 承载 `HomieClient` transport 和 service bridge。
- app 必须先通过显式 `RuntimeLauncher` ensure daemon，再异步 connect；launcher/connect 不能阻塞 GPUI 首帧。
- session list/spawn/send/resize/snapshot/events 必须通过 async client 和 authoritative event/snapshot 回写 projection。
- app 不得在 client transport 路径构造 `RuntimeSupervisor` 或读取 holder/output log。
- settings 等现存 direct-storage path 由 T-103 清理；它们不得继续作为 live session projection 来源。
- runtime disconnect 必须投影 connecting/degraded/reconnecting/disconnected，不得回退到 embedded runtime。
- terminal stream reset 必须显示可重试状态并从 last confirmed offset 重新 open。
- source-text substring test 不能证明 async bridge、first-frame 或 shared-daemon 行为；这些必须由 compile、interaction 和 cross-process E2E 覆盖。

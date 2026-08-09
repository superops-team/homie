# Agent Adapter Contract 组件规格

## 1. 组件定位

`homie-agents` 定义 agent runtime descriptor、agent profile、manifest schema、status rule、approval/resume/hook/notify 能力声明和 `EffectiveAgentConfig` 冻结规则。它让 Codex、Claude Code、OpenCode、Gemini、Cursor、shell 和其他 Reference catalog agent 通过同一合同接入 Homie。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- Gap-closure PRD: `prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- Gap-closure OpenSpec: `openspec/changes/diri-engine-migration/`
- 功能验证: FC-005, FC-006, FC-013, FC-014, FC-018
- Gap-closure 功能验证: FC-DIRI-004, FC-DIRI-005

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `homie-runtime` | session spawn/status 读取 descriptor 和 effective config |
| 上游 | `homie-app` | agent readiness 和 new-agent UI 读取 catalog projection |
| 下游 | `homie-storage` | runtime descriptor、agent profile、effective config 持久化 |
| 下游 | `homie-llm` | 为 managed agent 请求 virtual key/proxy 配置 |

## 4. 职责边界

负责：

- Reference 19-agent catalog 的 manifest schema、加载、校验和 readiness projection。
- status authority：process、screen、hooks。
- status reducer：将 process、screen、hooks、notify、user input、tick 等信号折叠为 canonical session status。
- hook/notify parser：把 Claude/Codex 等工具回调解析为稳定事件，并在进入 runtime 前完成脱敏。
- approval/deny keystroke、resume、hook/notify 配置。
- agent profile 与 runtime descriptor 解析成不可变 `EffectiveAgentConfig`。
- env scrub 和 argv template 数据合同。

不负责：

- 实际 PTY/process 启动。
- provider raw key 解密。
- UI 状态渲染。
- MCP tool 执行。

## 5. 核心接口

```rust
pub struct RuntimeDescriptor {
    pub id: RuntimeId,
    pub display_name: String,
    pub binary: String,
    pub argv_template: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub env_scrub_prefixes: Vec<String>,
    pub status_authority: StatusAuthority,
    pub resume: Option<ResumeSpec>,
    pub approve: Option<AgentKeystroke>,
    pub deny: Option<AgentKeystroke>,
}

pub struct EffectiveAgentConfig {
    pub session_id: SessionId,
    pub profile_id: AgentProfileId,
    pub runtime: RuntimeDescriptor,
    pub llm_proxy: ManagedLlmProxyConfig,
    pub permission_profile_id: PermissionProfileId,
}
```

## 6. 数据模型

Combined manifest fields:

- Top-level detection manifest:
  - `schemaVersion`
  - `id`
  - `version`
  - `statusModel`
  - `rules`
- Nested `agent` descriptor:
  - `displayName`
  - `shortLabel`
  - `glyph`
  - `aliases`
  - `binary`
  - `spawnArgs`
  - `sessionIDFlag`
  - `statusAuthority`
  - `firstClass`
  - `resume.style`
  - `resume.token`
  - `returnToLoginShell`
  - `env`
  - `envScrubPrefixes`
  - `injection`
  - `foregroundExecNames`
  - `approve`
  - `deny`

Rules:

- `assets/agent-descriptors/<id>.json` is the single source for both catalog metadata and detection rules.
- Full status manifests must declare at least one detection rule.
- Process-only manifests may have an empty rules list.
- `shell` and `generic` do not represent named agent binaries and must not appear in binary readiness probes.
- Unknown agent ids use a conservative fallback only for catalog/detection projection: process authority, not first-class, no binary, no resume, no approve/deny quick action. Production session spawn must reject that fallback and must not start a shell for an unknown agent.
- Resume is valid only when the descriptor has `resume.style == latest`, or when an id source exists through `sessionIDFlag`, Claude hooks, or Codex notify.

Agent profile fields:

- runtime id。
- LLM profile id。
- skills。
- MCP servers。
- permission profile。
- workspace scope。
- enabled/default。

## 7. 运行模型与状态机

```text
load manifests
  -> validate schema
  -> seed runtime descriptors
  -> user profile selects runtime + llm + permission
  -> spawn freezes EffectiveAgentConfig
  -> running session never changes after profile edit
```

Status reducer:

```text
process-only | screen-primary | hook-primary
  -> blocked / working / idle / done / exited / hibernated
```

Gap-closure status signal flow:

```text
HookEvent | NotifyEvent | ScreenObservation | PtyOutputActivity | UserKeystroke | ProcessExit | Tick
  -> StatusReducer
  -> SessionStatus + NeedsInputDetail + turn_completed
```

Reducer requirements:

- `ProcessOnly` only transitions from starting to working on output and exits on process exit.
- `ScreenPrimary` uses manifest screen observations as canonical status input.
- `HooksPrimary` lets hook events lead state while screen observations arbitrate blockers.
- Subagent events update bookkeeping only and must not overwrite the parent session status.
- Startup grace, idle confirmation, blocker clear scans, hook authority window, and staleness timeout must be deterministic and testable without sleeping.

Agent catalog/readiness:

```text
combined manifests
  -> AgentCatalog(sorted by id)
  -> descriptor(id) / resolve(name)
  -> launchable descriptors
  -> readiness_with_resolver(binary -> path?)
```

Readiness requirements:

- Readiness probes must be a resolver/stat check, not a subprocess launch.
- A missing binary returns `available=false` for that agent and must not make catalog load fail.
- The readiness item must include the descriptor projection so clients can render unavailable agents without another lookup.
- Readiness is an adapter contract only in this component; runtime decides when to call it and how to resolve login-shell PATH.

## 8. 安全与权限

- manifest 不能声明 raw provider key。
- env scrub 必须移除 `Authorization`、provider key、cookie 和已有 session credential。
- unknown approval keystroke 默认不可自动执行。
- disabled default profile 不能启动 session。
- running session 使用冻结 config，不受后续 profile 修改影响。
- hook/notify payload 中的 `token`、`secret`、`authorization`、`cookie`、`password`、`api_key` 等字段必须结构化脱敏后才能进入日志、事件、报告或 reducer diagnostic。
- URL query 中的 secret-bearing 参数必须脱敏。
- Unknown hook events fail-open with a safe summary and must not create quick approve/deny actions.
- Subagent hook events update bookkeeping only and must not overwrite the parent session title, status, or needs-input state.

## 9. 可观测性

事件和日志：

- agent.catalog_loaded。
- agent.manifest_invalid。
- agent.readiness_checked。
- agent.effective_config_created。
- agent.hook_parsed。
- agent.hook_parse_failed。
- agent.status_transitioned。

日志不得包含完整 argv 中的 secret-bearing 参数。
Hook parse failure 必须 fail-open：记录安全摘要，不阻塞 agent 运行。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| manifest schema invalid | catalog load fail closed，指出 agent id |
| binary missing | readiness unavailable，不影响其他 agent |
| default profile disabled | spawn 返回 stable error |
| status rule panic/invalid regex | 启动前 schema 检查失败 |

## 11. 测试计划与验收引用

- FC-005: agent catalog manifest parity。
- FC-006: protocol readiness contract。
- FC-013: no real provider key leak。
- FC-014: hook/notify manifest-driven behavior。
- FC-018: full local quality gate。
- FC-DIRI-004: status reducer needs-input/idle/subagent/process-exit parity。
- FC-DIRI-005: hook parser stable event and redaction parity。
- FC-DA-001: Diri combined manifest catalog fields。
- FC-DA-002: alias resolve and unknown-id fallback。
- FC-DA-003: launchable readiness projection。
- FC-DA-004: Claude/Codex/Cursor/Gemini golden screen parity。
- FC-DA-005: hook/notify stable event and hostile redaction parity。
- FC-DA-006: strict bundled manifest decode。
- FC-DA-007: focused Rust quality gates。
- FC-DA-008: security hook gate。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Diri atom | Homie owner | Contract section | Verification |
|-----------|-------------|------------------|--------------|
| M16-F001 Agent catalog/manifest/readiness | `assets/agent-descriptors`, `homie-agents` | Combined manifest fields, Agent catalog/readiness | FC-DA-001, FC-DA-002, FC-DA-003, FC-DA-006 |
| M16-F002 Detection regions/predicates/reducer/golden screens | `homie-agents::detect`, `homie-agents::status` | Status reducer, golden screen parity | FC-DA-004, FC-DIRI-004 |
| M08-F001 Notification rollup and needs-input modal data | `homie-agents::hooks`, `homie-agents::status` | Security and permissions, hook/notify parser | FC-DA-005, FC-DIRI-005 |
| M17-F001 Core needs-input/attention semantics | `homie-proto`, `homie-agents` | `NeedsInputDetail`, `RiskHint`, status authority | FC-DA-005 |

Diri test mapping:

| Diri test/source | Homie equivalent |
|------------------|------------------|
| `AgentKindTests.catalogLoadsBundledManifests` | `manifest_catalog::bundled_catalog_projects_diri_manifest_fields` |
| `AgentKindTests.catalogResolvesUserTypedNames` | `manifest_catalog::catalog_resolves_aliases_and_falls_back_for_unknown_ids` |
| `AgentKindTests.resumeNeedsEitherAnIDSourceOrLatestSemantics` | `manifest_catalog::resume_specs_preserve_diri_resume_semantics` |
| `AgentReadiness.inspect` | `manifest_catalog::readiness_projects_launchable_agents_only` |
| `ManifestAndRegionTests.everyBundledManifestDecodesStrictly` | `manifest_catalog::every_bundled_manifest_decodes_strictly` |
| `GoldenScreenTests` | `golden_screens` integration test |
| `HookParsing.parseClaudeHook` / `parseCodexNotify` | `hook_parser` integration test |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.
- This component can claim first-stage agent detection parity only after FC-DA-001 through FC-DA-008 are recorded in `docs/verification/diri-agent-detection/release-readiness-report.md`.

| Mandatory field | Value |
|-----------------|-------|
| Owned feature atoms | M16-F001, M16-F002, M08-F001, M17-F001 agent attention subset |
| Required Diri test mapping | Agent catalog/readiness, combined manifest strict decode, resume semantics, golden screen detection, hook/notify redaction |
| Pre-implementation gaps | full terminal status parity, complete approval/resume UI wiring, downstream storage/runtime integration gates |

## 16. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- Requirements: FR-04, FR-13, FR-16
- Beads: `homie-t3u`

本组件当前状态是 `partial`。19 个 manifest、golden screen、hook parser 和 reducer 测试通过，只证明 adapter library 的部分合同成立；runtime 仍固定启动 shell，因此不能声明 agent runtime parity。

### 16.1 Runtime 接入合同

每次 agent spawn 必须：

1. 从持久化 profile 选择启用的 runtime descriptor、LLM profile 和 permission profile。
2. 用登录 shell PATH 或等价 resolver 完成 readiness。
3. 冻结 `EffectiveAgentConfig`，后续 profile 修改不得改变 running session。
4. 按 manifest 构造 binary、argv、env scrub、hook/notify/MCP injection 和 initial prompt。
5. 只注入 scoped virtual key 和 local proxy URL，不注入 provider raw key。
6. 把 process、screen、hook、notify、user input、tick 和 exit 信号送入同一 reducer。
7. 按 manifest resume 语义恢复原 agent session，不得退化为新 shell。

### 16.2 禁止 shortcut

- 不允许把 manifest catalog 数量相等当作运行时完成证据。
- 不允许 runtime 忽略 descriptor binary/argv 而固定启动 `/bin/sh`。
- 不允许 UI 或 CLI 自行拼接 agent command。
- 不允许用 source-text 或静态 readiness fixture 代替真实 fake-binary spawn/resume E2E。

### 16.3 完成门禁

- 每个 first-class manifest 都有 strict decode、readiness、spawn command projection 和 reducer fixture。
- 至少 Claude、Codex、OpenCode、Gemini、Cursor 和 shell 通过 fake-binary process E2E。
- 至少两个可用真实 agent 通过 opt-in local smoke，smoke 不依赖真实 provider key。
- approve/deny、hook/notify、subagent isolation 和 resume 进入 runtime/app/MCP 产品路径。
- unavailable binary、disabled profile、invalid injection 和 revoked virtual key 全部 fail closed。

## 17. T-102 Manifest-Driven Runtime 修订

权威来源：

- PRD: `prd-spec/features/diri-agent-session-runtime/2026-08-09-diri-agent-session-runtime-design.md`
- OpenSpec: `openspec/changes/diri-agent-session-runtime/`
- Beads: `homie-t3u.1`
- Master task: T-102
- Checkpoint: `48f522b`

### 17.1 Catalog 与 Spawn 边界

`homie-agents` 负责把 manifest/profile 投影为 agent-specific plan；`homie-runtime` 负责在
holder-owned PTY 中执行该 plan。

Production spawn 流程：

```text
explicit profile or explicit shell kind
  -> enabled profile/runtime descriptor
  -> strict bundled manifest
  -> bounded readiness
  -> absolute executable
  -> sanitized env + manifest argv/injection
  -> immutable EffectiveAgentConfig
  -> structured holder launch
```

规则：

- production 只读取固定 bundled descriptor source，不接受环境变量 manifest override。
- readiness 只解析 executable，不执行 agent 本体。
- readiness 必须得到 absolute executable regular file。
- `shell` 是显式 kind；unknown/unavailable/disabled agent 不得 fallback 到 `/bin/sh`。
- test catalog 只能通过 Rust constructor/fixture 注入，不得形成 daemon flag/test mode。
- UI、CLI、MCP caller 不得自行拼 agent command。

### 17.2 EffectiveAgentConfig

每个 session 在 holder launch 前冻结：

- profile/runtime/LLM/permission identifiers；
- manifest id/version/status authority；
- absolute executable 和 final argv；
- sanitized env；
- hook/notify/MCP injection decisions；
- resume style/token；
- cwd、parent 和 initial geometry。

Running session 不随 profile/manifest edit 变化。Resume 必须读取 frozen config/launch record；
缺失或不兼容时 fail closed，不得从 mutable profile 重建成不同 agent。

若既有 storage 表无法完成 effective-config/session 原子 linkage，T-102 implementation 必须
阻塞并请求 storage owner/spec 变更，不得私自增加 schema migration。

### 17.3 Environment 与 Credential 边界

- child env 从 reviewed baseline 构造，再应用 manifest env scrub 和 explicit values。
- provider raw key、Authorization、cookie、已有 agent session credential 不得默认继承。
- raw values 不得进入 holder argv、launch metadata、logs、events 或 evidence。
- T-102 可以消费 credential owner 提供的 scoped virtual-key reference/local proxy input，
  但不负责 issuance 或 provider forwarding。
- 缺少 managed proxy input 时，不得发明 production key 或回退到真实 provider credential。

### 17.4 Status Authority 与 Runtime Signal

每个 live session 使用 frozen manifest 的 authority 创建一个 stateful reducer：

```text
ProcessReady/Exit
PTY Output Activity
Manifest Screen Observation
Claude Hook
Codex Notify
User Input
Tick
  -> one StatusReducer
  -> canonical status + needs-input + turn-complete
```

合同：

- runtime 不得为每次 status read 新建固定 `ScreenPrimary` reducer。
- screen detection 使用目标 `ManifestEngine`，不以 agent-agnostic phrase classifier 代替。
- hook/notify parser 输出 allowlisted structured signal，不输出 caller-selected final status。
- subagent event 不能覆盖 parent status/title/needs-input。
- runtime 先持久化 reducer outcome，再发布 event。
- raw hook/notify payload 不持久化。
- daemon restart 可以清零 debounce counters，但必须用 holder live evidence 和 persisted
  behavior/checkpoint 重建，不能用 storage row 伪造 running。

### 17.5 Direct Resume

- ID-based resume 需要 verified agent session id。
- latest resume 只对 manifest 明确声明 `latest` 的 agent 有效。
- resume 直接执行 manifest resume argv，不先启动 shell 再注入文本。
- fresh spawn 有效的 hook/MCP injection 只有在 manifest 明确允许 resume 时才重放。
- resume 保持 Homie session identity 和 frozen authority，并建立新 output epoch。
- remote migration/handoff 不属于本组件 T-102 合同。

### 17.6 Rollback 与完成门禁

任一 readiness、effective config、injection、holder launch 或 child readiness 失败：

```text
terminate/reap launched fixture/session holder
  -> remove live registry state
  -> roll back uncommitted session/effective config
  -> return stable redacted error
```

完成门禁：

- 19 manifest strict decode 和现有 reducer/hook suites 保持 GREEN。
- fake manifest 使用真实 executable、holder、PTY 验证 argv/env/output。
- profile edit 不改变 running session。
- unavailable/unknown/disabled/invalid config 全部 fail closed，无 shell fallback。
- process/screen/hook/notify/input/tick 进入同一 reducer。
- direct resume 和 restart reconstruction E2E 通过。
- 无 production fake catalog、environment override、raw provider key 或 remote placeholder。

# Diri 7ba3407 全功能对齐重基线设计

```yaml
change_id: diri-7ba3407-parity-rebaseline
status: approved_for_planning
beads: homie-t3u
baseline_repository: diri/
baseline_commit: 7ba3407
baseline_scope:
  - desktop-ui
  - runtime-session-pty
  - cli
  - mcp
  - remote-node
  - usage
  - updater
  - packaging-performance
supersedes_planning:
  - reference-parity-v1
  - diri-parity-child-tasks
```

## 1. 概述

### 1.1 背景

Homie 是对 Diri 的 Rust + GPUI 重构。此前的 `reference-parity-v1` 和一批微切片 change 已迁移部分数据模型、协议 DTO、解析器、UI 表面和测试夹具，但没有形成可运行的端到端产品闭环。

截至 2026-08-08，代码审计确认：

- `homie-client` 在应用进程内直接创建 `RuntimeSupervisor`，尚不是 Diri 式独立 runtime client。
- runtime 仅能固定启动 `/bin/sh`，19 个 agent manifest 没有驱动真实 agent 启动。
- protocol 常量、MCP descriptor 和组件 spec 声明的能力大于实际 dispatch 能力。
- Desktop UI 的 pin、archive、remote、updater 等操作仍有只修改本地状态或文案的路径。
- remote node、LLM HTTP proxy、updater 安装、签名公证和真实性能门禁尚未实现。
- context、memory、task、orchestrator 只有孤立模型，未接入产品路径。
- workspace 测试存在 app source-text、MCP error contract 和 runtime holder/PTY 回归。
- 现有 OpenSpec change 普遍只有 `tasks.md`，缺少 OpenSpec CLI 要求的 proposal、design 和 capability specs。

因此，不能继续用“存在同名 crate、DTO、静态 UI 或局部测试”代表 Diri parity。本设计固定以内嵌 `diri/` 的提交 `7ba3407` 为唯一功能基线，并将后续开发重组为可独立验收的纵向闭环。

### 1.2 目标

1. 建立覆盖 Diri `7ba3407` 全部用户能力、协议方法、MCP 工具、后台进程和发布链路的冻结矩阵。
2. 用真实运行路径替代 in-process shortcut、静态 UI、fixture-only 和 source-text 断言。
3. 按依赖顺序交付 runtime、桌面产品、自动化、远端、usage/LLM 和发布能力。
4. 为每个需求建立 PRD -> component spec -> OpenSpec task -> RED/GREEN test -> evidence -> Beads 的双向追踪。
5. 保留 Homie 的 virtual key、context、memory、task 和 orchestration 扩展，但不得以扩展功能替代 Diri 基线能力。
6. 只有所有冻结矩阵条目通过真实 E2E 和准出门禁后，才允许声明 Diri parity complete。

### 1.3 非目标

- 不逐行翻译或机械复制 Diri 源码。
- 不保留当前错误架构的兼容层、fallback 或双实现。
- 不把 Windows/Linux 支持纳入本次 macOS parity 准出。
- 不在本 change 中直接完成所有代码实现；本 change 负责冻结需求、长期合同、纵向任务和验证基线。
- 不重新打开或改写历史 change 的交付结论；历史文档保留，但不再作为当前实施入口。
- 不把 Homie 扩展能力纳入“Diri parity complete”的必要定义；扩展能力有独立准出状态。

## 2. 用户场景

### 场景 1：本地 Agent 会话在桌面应用中持续运行

**Given** 用户已配置可用 agent profile，且 runtime daemon 已启动  
**When** 用户从 Homie 创建 agent session、关闭应用后重新打开  
**Then** session 继续运行，终端可重连并从已确认 offset 恢复，状态、标题、输入和输出都来自真实 runtime

### 场景 2：CLI 和 MCP 操作同一运行时

**Given** 桌面应用、CLI 和 MCP client 连接同一 runtime endpoint  
**When** 任一入口创建、读取、等待、发送、归档或释放 session  
**Then** 所有入口观察到同一持久化事实、事件序列和权限结果，不存在 in-process 私有 registry

### 场景 3：远端执行和 handoff 不泄漏凭据

**Given** 用户配置了 remote node 和 Homie provider profile  
**When** session 在远端创建或执行 move/fork handoff  
**Then** checkpoint、node protocol、日志和 agent config 不包含 provider raw key，失败时 source session 保持可恢复

### 场景 4：桌面产品表面都执行真实操作

**Given** 用户打开 sidebar、Quick Open、History、Settings、Inspector 或通知 action  
**When** 用户执行 pin、archive、resume、worktree、approve、update 等动作  
**Then** 动作通过 client/protocol 到达 owning service，成功后由事件更新 UI，失败时不提交 optimistic 假状态

### 场景 5：发布包可验证、更新、回滚并满足性能预算

**Given** release host 持有合法 Developer ID 和 notarization 配置  
**When** 执行 release pipeline 并从已发布 feed 安装更新  
**Then** universal app、CLI、holder、sidecar 和 node 依赖闭包完整，签名、公证、stapling、安装、回滚和性能门禁都有可复现证据

## 3. 功能需求

### FR-01：冻结基线与完成判定

- 唯一基线必须是仓库内 `diri/` 的 commit `7ba3407`。
- `docs/research/diri-7ba3407-capability-matrix.md` 必须覆盖 20 个 Diri 模块和所有跨模块能力。
- 每个能力条目必须有 Diri source、Diri test、Homie owner、当前状态、目标验证和依赖。
- 状态只能使用 `implemented`、`partial`、`missing`、`blocked`。
- `implemented` 必须同时具备真实实现、真实产品接线、自动化验证和 evidence。
- 任一必要条目不是 `implemented` 时，整体状态必须是 `not_parity_complete`。

### FR-02：协议与独立 Runtime Client

- runtime 必须作为独立后台进程拥有 session、PTY、event bus 和 durable registry。
- control channel 使用 versioned NDJSON request/response/event envelope；大数据和 terminal grid 使用独立 data/attachment channel。
- client 必须支持 request correlation、heartbeat、断线重连、event sequence resume、gap 后 full snapshot 和 backpressure。
- app、CLI、MCP 和 remote 必须通过同一 client/protocol 访问 runtime。
- 删除 `HomieClient` 在调用方进程内创建 `RuntimeSupervisor` 的路径，不保留兼容 fallback。
- 协议目录中的每个方法必须有 handler，或从公开目录删除；禁止 advertised-but-unsupported。

### FR-03：Runtime、PTY、Holder 与会话生命周期

- runtime 必须支持 spawn、attach、read、send、resize、wait、kill、release、archive/unarchive、hibernate/wake、history/resume 和 migrate。
- holder 必须独立持有 PTY 和 output log，runtime/app 崩溃不得终止 live child。
- output log 必须按 offset 寻址，attach 必须返回 holder stat、replay window、screen checkpoint 和当前状态的一致快照。
- process tree、stop/continue、memory sampling、resource governor、terminate 和 crash recovery 必须覆盖真实子进程。
- daemon 必须实现 prepare_shutdown 和 shutdown，按顺序 flush storage、events、usage/context 和 output index。

### FR-04：Agent 启动、检测、Resume 与权限

- 19 个冻结 manifest 必须驱动 binary、argv、env scrub、injection、status authority、approve/deny 和 resume。
- spawn 必须冻结 `EffectiveAgentConfig`，包括 profile、runtime descriptor、permission profile 和 managed LLM config。
- readiness 必须基于登录 shell PATH 或等价可复现 resolver，不执行 agent 本体。
- hooks、notify、screen、process 和 timeout 信号必须进入同一 status reducer。
- resume 必须使用 manifest 声明的 session id/latest 语义，不能退化为新 shell。

### FR-05：核心模型、Storage 与一致事实源

- session、project、worktree、history、artifact、usage、profile、permission 和 event 必须以 SQLite/repository 为持久化事实源。
- output bytes 不进入 SQLite blob；SQLite 只保存索引、offset、checkpoint 和安全摘要。
- migration 必须 forward-only，schema 过新 fail closed。
- UI 不得直接写 storage；runtime/service repository 是写入 owner。
- pin、archive、order、settings、history tracking 和 remote facts 必须可在重启后恢复。

### FR-06：桌面 Workbench 与 Sidebar

- Workbench 必须包含真实 session projection、terminal pane、inspector、状态栏、overlay 和 disconnected/degraded 状态。
- Sidebar 必须支持 project/session 分组、状态 glyph、hover card、rename、pin/archive、drag reorder 和 multi-select。
- 所有可变操作必须经 `ShellCommand`/`homie-client`，成功状态由 runtime event 回写。
- app 启动不得在 GPUI 首帧线程执行阻塞 runtime、SQLite、process 或网络操作。

### FR-07：Terminal 完整交互

- terminal 必须渲染真实 runtime grid，支持 damage、cursor、selection、copy/paste、find、keyboard encoding、resize 和 scrollback。
- scrollback 必须按 row/offset fetch，不能读取完整日志后在 UI 内切片。
- theme、font metrics 和 repaint pacing 必须有截图和性能证据。
- terminal 行为测试必须通过真实 PTY 或确定性 terminal fixture，不得用 source-text 断言替代。

### FR-08：导航、设置与 macOS 原生能力

- Command Palette、文件 Quick Open、Overview、Ctrl-Tab Switcher 和 History Resume 必须完整可用。
- Quick Open 必须包含目录扫描、缓存、git-aware ranking 和明确的失效刷新规则。
- Settings 必须覆盖 General、Terminal、Resources、Remote，并通过 owning service 持久化。
- macOS menu bar、native notification、sound 和 approve/deny action 必须执行真实路径。
- 所有 overlay 必须遵守统一 focus、keyboard 和 Esc cascade。

### FR-09：Inspector、Git、Worktree、Artifact、PR 与端口

- Inspector 的 Info、Changes、Artifacts 必须有真实 tab state 和数据源。
- diff 必须覆盖 tracked、untracked 和 base/head comparison，并支持大 diff 虚拟化或分页。
- worktree 必须支持 locate、create、list、remove 和 cleanup safety。
- artifact scanner、PR monitor、browser preview、port list 和 port forward 必须连接 live session。
- browser preview 和 port forward 失败必须返回稳定、可重试的 safe error。

### FR-10：完整 CLI

- CLI 必须覆盖 Diri 的 session get/read/send/wait/spawn/release/archive undo、status、artifacts、forward、ports forward 和 events subscribe。
- session selector 必须支持稳定 id、title 和 prefix resolution，并对歧义 fail closed。
- human、JSON 和 NDJSON 输出必须有固定 grammar 和 fixture。
- CLI 必须通过 client 访问 runtime，不能直接读取 live registry 或拼 storage 状态。

### FR-11：MCP、Lineage、Browser Sidecar 与测试自动化

- MCP tools 必须包含 spawn/list/status/send/wait/read/worktree/artifact/release/test/browser/whoami/children。
- 基线额外工具 `summarize_children` 和 `report_to_parent` 必须进入冻结目录和权限矩阵。
- 每个 tool 必须有精确 JSON Schema，未知字段策略和 stable JSON-RPC error mapping。
- lineage 必须覆盖 self、parent、ancestor、child、sibling、unrelated 和 recursive descendants。
- `browser` 和 `test_run` 只有在 sidecar/runner 可用且通过 E2E 后才能出现在 `tools/list`。
- sidecar 必须纳入 package 依赖闭包，图片只返回路径/引用，不内联敏感 bytes。

### FR-12：Remote Node、Account 与 Handoff

- first-party node 必须实现 hello/capability/token auth、remote spawn、events、provider account 和 usage。
- host prefs sync 必须使用固定 secretless allowlist。
- move/fork handoff 必须实现 preflight、checkpoint、增量传输、quarantine restore、provider-native resume/fork 和 lease commit。
- checkpoint 必须排除 credential、provider home、`.env*`、SSH、`.git`、build/dependency、symlink 和超限文件。
- companion listener 只允许 loopback 或显式 Tailscale bind，token owner-only 且可撤销。

### FR-13：Usage、LLM Proxy 与 Virtual Key

- usage 必须支持 Claude/Codex transcript 增量扫描、offset/tail hash cache、pricing snapshot、本地汇总、fleet merge 和 UI。
- estimated cost 与 billed cost 必须分开，历史 cost 不随价格表更新漂移。
- Homie LLM proxy 必须提供 OpenAI-compatible HTTP endpoint、provider routing、SSE streaming 和 safe error mapping。
- managed agent 只能获得 scoped virtual key 和 local proxy URL。
- provider raw key 只能在 encrypted envelope 和 upstream Authorization 的短期内存中出现。
- metrics 写入失败不能中断已成功的 provider response。

### FR-14：Context、Memory、Task 与 Orchestrator 产品接入

- context、memory、task 和 orchestrator 必须通过 runtime/client/CLI/MCP/UI 至少形成一个真实纵向闭环。
- context 只保存安全事件、lineage、artifact/task/memory 引用和 output offset，不复制 raw output。
- memory candidate 必须有 source event、redaction 和 permission gate。
- task 必须覆盖 create、claim、block、complete、return，并与 session owner 失活恢复联动。
- orchestrator 路由必须确定、可审计，高风险歧义要求用户确认。
- 本需求属于 Homie 扩展，单独报告状态，不阻塞纯 Diri parity 的完成判定。

### FR-15：Updater、Packaging、签名与性能

- package 必须构建 arm64/x86_64 universal app，包含 CLI、runtime、holder、sidecar 和必要资源。
- release 必须执行 Developer ID、hardened runtime、notarization、stapling、DMG、update zip 和 feed 生成。
- updater 必须执行 feed fetch、HTTPS allowlist、SHA256、bundle/team/version、codesign/spctl、stage、helper swap 和 rollback。
- update 只能由用户动作触发重启，不能静默终止 live session。
- packaged app startup、session attach、terminal repaint、内存和更新检查必须有真实性能预算与测量证据。

### FR-16：SDD/TDD、Evidence 与最终准出

- 每个实施切片必须先有独立 Bead、中文 PRD、受影响 component spec 和完整 OpenSpec。
- 每个 task 必须按 RED -> GREEN -> REFACTOR -> EVIDENCE 执行。
- 测试必须断言行为、协议或渲染结果；禁止以源代码字符串存在作为核心功能证据。
- evidence 状态只能是 `pass`、`blocked`、`not_run`、`partial`、`fail`。
- scoped slice pass 不等于整体 parity pass。
- 最终准出必须通过 workspace tests、真实 app/CLI/MCP/runtime/remote/updater E2E、安全检查、截图对比、性能和 package 安装。

## 4. 方案设计

### 4.1 纵向交付波次

| 波次 | Change ID | 主要需求 | 准出结果 |
|------|-----------|----------|----------|
| Wave 0 | `diri-7ba3407-parity-rebaseline` | FR-01, FR-16 | 需求、合同、任务和证据基线可信 |
| Wave 1A | `diri-runtime-daemon-client-transport` | FR-02 | app/CLI/MCP 连接独立 runtime |
| Wave 1B | `diri-agent-session-runtime` | FR-03, FR-04 | 真实 agent session 可恢复运行 |
| Wave 1C | `diri-storage-core-facts` | FR-05 | 统一持久化事实和迁移合同 |
| Wave 2A | `diri-desktop-workbench-sidebar` | FR-06 | runtime-backed workbench/sidebar |
| Wave 2B | `diri-terminal-interaction` | FR-07 | 完整 live terminal 交互 |
| Wave 2C | `diri-navigation-settings-native` | FR-08 | 导航、设置和 macOS 原生闭环 |
| Wave 2D | `diri-inspector-git-artifacts` | FR-09 | inspector/git/worktree/artifact 闭环 |
| Wave 3A | `diri-cli-complete-surface` | FR-10 | CLI grammar 与 runtime E2E |
| Wave 3B | `diri-mcp-browser-automation` | FR-11 | MCP/lineage/browser/test E2E |
| Wave 4A | `diri-remote-node-handoff` | FR-12 | 远端 spawn/account/handoff E2E |
| Wave 4B | `diri-usage-llm-proxy` | FR-13 | usage/fleet/proxy/virtual-key E2E |
| Wave 4C | `homie-control-plane-integration` | FR-14 | Homie 扩展纵向闭环 |
| Wave 5A | `diri-updater-packaging-performance` | FR-15 | 可签名、安装、更新、回滚的包 |
| Wave 5B | `diri-7ba3407-final-parity-gate` | FR-01..FR-16 | 全矩阵和最终 release gate 通过 |

实施约束：

- 每个 wave change 在编码前单独创建 PRD、Bead 和 OpenSpec。
- Wave 1 是 Wave 2 和 Wave 3 的硬依赖。
- Wave 3B 依赖 Wave 2D 的 artifact/browser owner。
- Wave 4A 和 4B 可在 Wave 1 稳定后并行。
- Wave 5A 可以提前建设脚本，但最终准出依赖全部运行时 sidecar 固定。
- Wave 5B 不实现功能，只执行最终交叉验证和缺口封板。

### 4.2 架构原则

```text
GPUI / CLI / MCP / Remote control
                 |
          homie-client
                 |
    versioned control + data channels
                 |
        homie runtime daemon
        /       |          \
   holder     storage    domain services
      |                      |
     PTY              agents/llm/context/task
```

- runtime daemon 是 live session 唯一 owner。
- SQLite 是 durable facts 唯一 owner，output log 文件是大流式输出唯一 owner。
- `homie-client` 只实现 transport、correlation、reconnect 和 typed methods。
- UI 不直接依赖 runtime 或 storage。
- CLI/MCP 不重复实现业务状态拼装。
- Homie 扩展通过稳定服务边界接入，不侵入 PTY、protocol 和 Diri 基线语义。

### 4.3 OpenSpec 产物

为同时满足仓库工作流和当前 OpenSpec CLI，每个实施 change 必须包含：

```text
openspec/changes/<change-id>/
├── proposal.md
├── design.md
├── specs/
│   └── <capability>/spec.md
├── plan.md
├── tasks.md
└── alignment-report.md
```

`proposal/design/specs/tasks` 用于 OpenSpec CLI artifact completeness；`plan/tasks/alignment-report` 用于仓库现有 SDD/TDD 和证据追踪。两组产物必须引用同一 PRD、Bead 和需求编号。

## 5. 边界情况

| 场景 | 处理方式 |
|------|----------|
| Diri 基线出现未登记源文件或测试 | 先更新冻结矩阵和受影响 spec，再允许实现 |
| Diri 行为与 Homie 安全基线冲突 | 保留用户行为，采用 virtual key/secretless adaptation，并在 PRD 明确差异 |
| 当前 Homie 已有同名 API 但语义不完整 | 状态保持 partial，删除错误 API 或补齐真实语义，不添加兼容 fallback |
| runtime event sequence 出现 gap | client 丢弃增量 projection，重新请求 full snapshot |
| app/runtime/holder 任一进程崩溃 | 按 ownership 恢复；无法确认 live 状态时标 degraded/detached，不伪造 running |
| remote/handoff 中断 | source 不变，target 使用 quarantine，lease commit 前不切换 owner |
| release host 缺少证书或 notarization 权限 | gate 记 blocked，不得使用 ad-hoc 签名替代 pass |
| 真实 GUI/remote/perf 环境不可用 | 对应 gate 记 not_run 或 blocked，不得写 pass_with_scope_limit |
| 历史 change 声称完成但当前测试回归 | 当前矩阵降级，创建新 gap-closure task，历史证据不改写 |

## 6. 受影响长期组件规格

| Component spec | 修订内容 |
|----------------|----------|
| `specs/runtime-supervisor/README.md` | 独立 daemon、holder、resource、shutdown、migration |
| `specs/agent-adapter-contract/README.md` | manifest 驱动真实 spawn 和 effective config |
| `specs/desktop-shell/README.md` | 删除直接 storage/runtime 路径，补齐完整 UI 状态矩阵 |
| `specs/storage-indexing/README.md` | service-owned repository、持久化事实和迁移准出 |
| `specs/mcp-automation/README.md` | 精确 schema、完整 tools、lineage、browser sidecar |
| `specs/remote-node-handoff/README.md` | node server、account、handoff 和 network E2E |
| `specs/llm-proxy/README.md` | 真实 HTTP/SSE/provider forwarding 和 usage |
| `specs/virtual-key-credentials/README.md` | secret envelope 和跨远端/sidecar 传播规则 |
| `specs/packaging-updater/README.md` | universal/sign/notarize/install/rollback/perf |
| `specs/session-context-store/README.md` | runtime-backed context integration |
| `specs/task-controller/README.md` | 产品入口和 session lifecycle integration |
| `specs/memory-controller/README.md` | source/redaction/permission 的真实 repository |
| `specs/intent-orchestrator/README.md` | UI/CLI/MCP 到 runtime/task 的执行闭环 |
| `specs/observability/README.md` | 合法状态词和整体/切片准出分离 |

## 7. 测试计划

| 层级 | 必须验证 |
|------|----------|
| Contract | Diri wire fixtures、MCP schemas、CLI grammar、unknown value/error mapping |
| Unit | reducer、parser、permission、pricing、migration、redaction、state machine |
| Integration | runtime/client socket、holder adoption、storage transactions、fake provider streaming |
| Process E2E | app/CLI/MCP 共用 runtime、daemon restart、agent resume、process-tree terminate |
| Remote E2E | node auth、remote spawn、account、checkpoint transfer、move/fork、failure recovery |
| UI E2E | first frame、sidebar/terminal/navigation/settings/inspector/native notification |
| Release E2E | packaged launch、codesign/spctl、notarization、update install/rollback |
| Performance | cold/warm startup、attach、replay、repaint、memory、event lag |
| Security | raw-key propagation denial、path/command injection、MCP lineage、bundle trust |

所有外部依赖优先使用本地 fake/stub：

- fake provider HTTP/SSE server；
- loopback fake node；
- temporary git repositories/worktrees；
- fixture transcript/feed/update bundle；
- deterministic PTY command；
- disposable user data directory。

真实 provider、真实远端主机和真实 release credential 只用于最后 smoke，不作为单元或集成测试前提。

## 8. 验收标准

### 8.1 本重基线 change

- Bead `homie-t3u` 指向本 PRD，状态与文档一致。
- 冻结矩阵覆盖 20 个模块和新增遗漏能力。
- 所有受影响 component spec 引用本 PRD，并明确当前实现不等于完整合同。
- OpenSpec proposal、design、capability spec、plan、tasks、alignment-report 完整。
- 16 维 spec review 无阻断项。
- 文档 format、路径、alignment 和 secret scan 通过。

### 8.2 最终 Diri parity

- 冻结矩阵全部必要条目为 `implemented`。
- `cargo test --workspace`、clippy、format、security 和 package gates 全部通过。
- app、CLI、MCP、runtime、remote、updater 的真实 E2E 全部通过。
- Diri/Homie side-by-side UI 和交互报告通过。
- 发布包通过 Developer ID、notarization、stapling、安装、更新和回滚。
- 性能预算有真实 packaged measurement。
- 所有未运行门禁均为零；不存在 `pass_with_scope_limit` 等非法状态。
- Beads 只在证据与 delivered state 一致时关闭。

## 9. Beads 追踪

| Bead | 用途 | 关闭条件 |
|------|------|----------|
| `homie-t3u` | 本重基线 PRD/spec/OpenSpec | 规格、对齐和评审证据通过 |
| `homie-h7n` | 历史 reference parity parent | 保留历史，不作为新实施入口 |
| `homie-h7n.1`..`homie-h7n.5` | 历史 group gaps | 由新 wave Beads 显式接管后再处理状态 |

后续每个 wave 必须创建自己的 Bead，metadata 至少包含 `change_id`、`baseline_commit=7ba3407` 和 `parent_change_id=diri-7ba3407-parity-rebaseline`。

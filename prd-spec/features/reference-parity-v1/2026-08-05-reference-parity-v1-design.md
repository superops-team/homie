# Homie Reference Parity V1 产品规格设计文档

## 1. 概述

### 1.1 背景

Homie 的长期目标是成为统一管理多个后台 coding agent 的 Rust + GPUI 桌面应用，并由 Homie 统一托管 LLM provider 配置、虚拟 key、OpenAI-compatible proxy、全局 context、memory、task 和 orchestration。当前 Homie 已有基础架构、SQLite bootstrap、CLI smoke 和本地最小版本文档，但这些内容只覆盖了核心技术基座，尚未覆盖参考产品 Reference 的完整产品面。

本需求要求以 `/Users/bytedance/workspace/github/reference` 为参考基线，对 Reference 当前首版产品功能和产品设计进行 1:1 复刻，并在 Homie 的项目规范、开发规范、代码规范、测试规范和项目管理规范下形成可执行的 PRD/spec。复刻目标不是复制 Reference 的代码或进程划分，而是让 Homie 第一个正式产品版本在用户可感知能力、交互面、自动化面、远端执行面、设计系统和准出门禁上完整对齐 Reference。

本 PRD 是 `reference-parity-v1` 的需求事实源。后续实现不得只完成“能创建 agent session”的最小闭环后宣称 V1 完成；只有本规格中列出的 Reference 功能项全部达到验收标准，才算 Homie Reference Parity V1 完成。

### 1.2 参考基线

本次盘点使用以下 Reference 文件作为产品和工程参考：

| 参考文件 | 用途 |
|----------|------|
| `/Users/bytedance/workspace/github/reference/README.md` | 顶层产品定位、进程架构、agent 支持范围 |
| `/Users/bytedance/workspace/github/reference/reference/README.md` | Rust + GPUI app、侧边栏 preview、远端 host 配置 |
| `/Users/bytedance/workspace/github/reference/reference/PLAN.md` | GPUI 迁移计划、完整设计 token、UI surfaces、键盘映射、性能预算 |
| `/Users/bytedance/workspace/github/reference/reference/PORT.md` | Rust engine port 状态、PTY、检测、registry、control socket |
| `/Users/bytedance/workspace/github/reference/reference/NODE.md` | 远端 node、账号、usage、handoff/move/fork |
| `/Users/bytedance/workspace/github/reference/reference/PACKAGING.md` | macOS 打包、签名、公证、DMG |
| `/Users/bytedance/workspace/github/reference/reference/UPDATING.md` | 自动更新、信任模型、安装回滚 |
| `/Users/bytedance/workspace/github/reference/reference/PERF.md` | 性能门禁和 packaged release gate |
| `/Users/bytedance/workspace/github/reference/Sources/<ReferenceProtocol>/Methods.swift` | Reference control protocol 方法和事件 |
| `/Users/bytedance/workspace/github/reference/Sources/<ReferenceCore>/Resources/manifests/*.json` | agent manifest、状态检测、approval/resume 规则 |
| `/Users/bytedance/workspace/github/reference/Sources/<ReferenceMCP>/Tools.swift` | MCP tool surface |
| `/Users/bytedance/workspace/github/reference/Sources/<reference-cli>/<ReferenceCLI>.swift` | CLI、hook/notify、mcp-stdio、doctor |
| `/Users/bytedance/workspace/github/reference/reference/crates/<reference-app>/src/*` | GPUI app shell、导航、设置、侧边栏、终端、inspector、usage、updates |
| `/Users/bytedance/workspace/github/reference/reference/crates/<reference-ui>/src/*` | 设计系统、图标、status glyph、brand marks |

### 1.3 目标

- 建立 Homie Reference Parity V1 的完整功能矩阵，确保 Reference 已有产品能力在 Homie 首版中都有明确需求、任务和验收。
- 将 Reference 的产品设计 design 转换为 Homie 的 GPUI 设计系统规格，包括窗口、侧边栏、终端面板、浮层、右侧 inspector、状态 glyph、菜单栏、通知、声音和键盘映射。
- 在 Homie 架构中复刻 Reference 的本地 agent orchestration 能力：多 agent session、PTY、output log、headless screen、状态检测、持久化、恢复、archive/hibernate/wake/reopen、历史扫描和 worktree。
- 在 Homie 架构中复刻 Reference 的自动化面：CLI、hook/notify fail-open forwarder、MCP stdio server、agent orchestration tools、browser/test_run 工具和事件订阅。
- 在 Homie 架构中复刻 Reference 的远端能力：remote hosts、first-party node、远端 spawn、repo 定位、偏好同步、node account、usage 汇总、session move/fork/handoff。
- 在 Homie 约束下增强 Reference 的 LLM 相关能力：真实 provider key 由 Homie 管理，managed agent 只获得 Homie virtual key 和 local proxy URL。
- 建立实现前的 OpenSpec plan、tasks 和 alignment report，保证后续实现不能脱离 PRD/spec。
- 建立验证策略：单测、集成、E2E、真实 PTY、UI screenshot、性能、打包、更新、安全、MCP 和远端 node 全部有准出门禁。

### 1.4 非目标

- 不要求 Homie 与 Reference 二进制、socket 协议、bundle id、配置文件或数据文件保持兼容。
- 不把 Reference 的 Swift daemon 架构照搬为 Homie 的长期事实源。Homie 的核心业务事实源仍在 Rust crate 和 SQLite 中，Swift 只作为 macOS native integration 边界。
- 不允许为了快速复刻而绕过 Homie 的 virtual key、provider credential custody、storage、security 和 evidence 规范。
- 不以旧的 Homie 最小 V1 范围作为准出标准。此前 `homie-v1-architecture` 和 `local-basic-v1` 是基础切片，不足以定义 Reference parity 的完成状态。
- 不为 Reference 旧接口写兼容层，除非后续用户明确要求迁移 Reference 数据或接管 Reference 运行中 session。

## 2. 版本定义

### 2.1 Reference Parity V1 完成定义

Homie Reference Parity V1 是一个完整产品版本，不是单个技术切片。完成必须同时满足：

1. 功能矩阵中的 P0/P1 项全部实现并有证据。
2. Reference 产品设计中的主界面、工作台、侧边栏、终端面板、浮层、设置页、菜单栏和通知行为达到视觉和交互对齐。
3. 本地、远端、MCP、CLI、hook/notify、worktree、history、artifact/port/PR、usage、updater、packaging、性能门禁全部通过。
4. Homie 的 LLM proxy、virtual key 和 credential 安全要求没有被 Reference parity 需求绕开。
5. Beads issue、PRD/spec、组件 spec、OpenSpec、验证证据和 release readiness report 状态一致。

### 2.2 实现分期

后续实现可以按阶段推进，但 release gate 不得按阶段降级：

| 阶段 | 目的 | 可独立验收 | 是否可作为 V1 发布 |
|------|------|------------|-------------------|
| P0 Foundation | 协议、runtime、storage、agent catalog、terminal core | 是 | 否 |
| P1 Local Product | 本地 agent session、UI shell、sidebar、terminal、history、worktree、usage | 是 | 否 |
| P2 Automation | CLI、MCP、hook/notify、browser/test_run、artifact/PR/port | 是 | 否 |
| P3 Remote/Node | remote hosts、node、account、handoff、fleet usage、companion access | 是 | 否 |
| P4 Ship | packaging、updater、perf/fidelity/security gate | 是 | 是 |

## 3. 用户场景

### 场景 1: 本地创建多个 agent session

**Given** 用户打开 Homie 并选择一个 Git 仓库。  
**When** 用户通过 New Agent、快捷键或命令面板启动 Codex、Claude Code、OpenCode、Gemini、Cursor 或 shell。  
**Then** Homie 创建 session，启动对应 agent runtime，显示侧边栏 session 行、终端输出、状态 glyph、标题、分支、cwd 和可操作控制。

### 场景 2: agent session 需要用户输入

**Given** 某个后台 agent 停在权限审批、确认问题或选择器。  
**When** Homie 的状态检测规则或 hook/notify 上报识别出 `needs_input`。  
**Then** 侧边栏、菜单栏、通知和声音给出一致提醒；如果 manifest 声明了安全 approve/deny keystroke，通知可以执行对应输入；否则只提示用户回到 session。

### 场景 3: 使用终端面板完成日常操作

**Given** 用户选中一个 session。  
**When** 用户输入、粘贴、滚动、选择文本、搜索、调整字体、切换侧边栏或窗口尺寸。  
**Then** 终端 grid 与 PTY 尺寸一致，scrollback/selection/find 可用，resize 不产生明显跳帧，隐藏 pane 不浪费渲染资源。

### 场景 4: 通过 worktree 并行开发

**Given** 用户希望把多个 agent 分派到同一个 repo 的不同分支。  
**When** 用户从 New Agent、MCP tool 或 worktrees sheet 创建 worktree。  
**Then** Homie 创建安全的 git worktree，记录 session 与 worktree 关系，支持 overview、cleanup 建议和非破坏性移除确认。

### 场景 5: 使用命令面板和 quick open 导航

**Given** Homie 管理多个项目和 session。  
**When** 用户使用 `Cmd-K`、`Cmd-P`、Ctrl-Tab、overview、history 或快捷键跳转。  
**Then** Homie 提供 Reference 对齐的 fuzzy ranking、session actions、folder index、live preview、history resume 和键盘语义。

### 场景 6: 查看右侧 inspector

**Given** 用户选中一个 session。  
**When** 用户打开 inspector 的 Info、Changes、Artifacts tab。  
**Then** Homie 展示 session 信息、worktree diff、artifact、PR、preview、port、检查结果和评论线程摘要，远端 session 的 diff 通过 runtime/host 读取而不是本地执行远端路径。

### 场景 7: 使用 MCP 编排其他 agent

**Given** 某个 agent 接入 Homie MCP stdio server。  
**When** agent 调用 `spawn_agent`、`wait_for_agent`、`read_output`、`create_worktree`、`get_artifacts`、`test_run` 或 `browser`。  
**Then** Homie 通过统一 runtime 和权限模型执行动作，保留 lineage，限制跨 session 写入，返回脱敏、结构化结果。

### 场景 8: 在远端 host 或 node 上运行 agent

**Given** 用户在 Settings -> Remote 添加 SSH host 或 Homie node。  
**When** 用户选择远端 host 启动 session、同步偏好、定位同 repo、move/fork 现有 session。  
**Then** Homie 在目标机器运行 agent，维持 session 记录、usage、artifacts 和 handoff 审计，并遵守 Homie 的 virtual key 和 credential custody 规则。

### 场景 9: 查看 usage 和成本

**Given** 用户同时使用多个 provider、agent 和远端节点。  
**When** 用户查看侧边栏 footer、account popover 或 settings。  
**Then** Homie 展示 session/today/month usage、token、cache read/write、cost、rate/quota 信息，并可从 Homie proxy 和 transcript/node ledger 合并数据。

### 场景 10: 升级和发布

**Given** 用户安装了签名、公证的 Homie app。  
**When** 新版本发布。  
**Then** Homie 后台检查但不自动重启，显示 update pill，用户手动下载、验证、重启更新；失败时保留可恢复旧 bundle。

## 4. 功能需求

### FR-1: Reference 功能覆盖矩阵必须成为准出依据

Homie 必须维护 Reference parity coverage matrix。矩阵至少覆盖以下功能面：

| Reference 功能面 | Homie V1 要求 | 优先级 |
|-------------|---------------|--------|
| Rust + GPUI 桌面 app | `homie-app` + `homie-ui` 实现完整工作台 | P0 |
| 双进程或可独立 runtime 架构 | UI 通过 client/protocol 访问 runtime，不直接拥有 PTY/process | P0 |
| PTY、output log、headless screen | runtime 拥有 PTY、offset log、headless emulator、read/replay | P0 |
| holder/会话保活 | app 退出不杀 session，runtime crash 不丢 PTY/output 的 holder-equivalent 机制 | P0 |
| session registry/persistence | SQLite 记录 session/project/worktree/history/artifact/state | P0 |
| agent manifests/status rules | data-driven manifest，支持 Reference 已有 agent catalog 和检测规则 | P0 |
| sidebar/product design | Reference 侧边栏结构、拖拽、pin/archive、hover card、多选、footer 对齐 | P0 |
| terminal pane | header、chips、grid、find、scrollback、selection、overlays 对齐 | P0 |
| command palette/quick open/switcher/overview/history | 全部导航 surface 和键盘语义对齐 | P1 |
| worktrees | create/list/remove/overview/cleanup 建议 | P1 |
| settings | General、Terminal、Resources、Remote tabs | P1 |
| right inspector/workbench | Info、Changes、Artifacts、diff virtualization | P1 |
| menu bar/notifications/sounds | macOS native status rollup、通知 action、声音反馈 | P1 |
| usage accounting | local + proxy + transcript + node usage 汇总 | P1 |
| hook/notify/MCP/CLI | Reference 自动化面完整复刻，并按 Homie 权限模型收口 | P1 |
| artifact/port/PR/browser pool | artifact scanner、port forward/listening port、PR monitor、browser/test_run | P1 |
| remote hosts/node/handoff | SSH fallback、first-party node、account、move/fork、fleet usage | P2 |
| companion/iOS access | Tailscale endpoint、pairing token、remote config | P2 |
| packaging/updater/perf gate | universal app、签名公证、DMG、auto-updater、packaged perf gate | P0 |

准出要求：

- `docs/research/reference-feature-coverage.md` 必须更新到所有项目至少为 `covered-by-reference-parity-v1` 或明确有后续 Beads。
- 不允许出现 `missing`、`partial` 而没有阻塞说明。

### FR-2: Homie 架构边界必须承载 Reference parity

Homie 继续采用项目规范中的 Rust workspace 分层：

- `homie-app`：GPUI 应用壳和窗口。
- `homie-ui`：设计系统、图标、status glyph、浮层组件。
- `homie-term`：grid buffer、terminal element、输入编码、selection、scrollback、find。
- `homie-proto`：控制协议、event、frame、grid、错误 envelope。
- `homie-client`：runtime/node client、reconnect、event resume、attachment。
- `homie-runtime`：PTY、process、session、output log、headless screen、status、registry、resource governor。
- `homie-agents`：agent catalog、manifest schema、adapter contract、approval/resume/hook 配置。
- `homie-llm`：virtual key、provider routing、OpenAI proxy、usage metrics。
- `homie-context`、`homie-memory`、`homie-task`：Homie 增值上下文、记忆和任务边界。
- `homie-storage`：SQLite schema、migration、repository。
- `homie-cli`：doctor、status、session/worktree/events、hook/notify、mcp-stdio、mcp-call。

硬约束：

- UI 不直接写 runtime/storage 状态。
- runtime 不依赖 UI。
- agent adapter 不直接持有真实 provider key。
- remote/node 不成为新的 credential 事实源，除非后续 PRD 明确改变 Homie 的 credential custody 策略。
- 所有新增依赖必须在 research/spec 中说明。

### FR-3: Agent catalog 与 Reference manifest 1:1 对齐

Homie 首版必须内置并测试 Reference 当前 manifest catalog 中的 agent：

```text
claude-code, codex, opencode, gemini, cursor, shell, generic,
qoder, pi, kilo, kimi, copilot, kiro, devin, hermes, grok,
antigravity, droid, amp
```

每个 agent 至少包含：

- id、display name、short label、glyph/brand。
- binary 查找规则。
- argv/env 模板和环境清理规则。
- status authority：`process`、`screen`、`hooks`。
- resume 能力声明。
- approval/deny keystroke 声明。
- blocker、working、idle、done、risk 状态规则。
- manifest schema 测试和 golden screen 测试。

Reference 已有 first-class status detection 的 agent 在 Homie 中不得退化为仅 process 状态；无法完全识别时必须标记为 blocked 并补充 fixture，而不是 silently fallback。

### FR-4: Session runtime 必须对齐 Reference 生命周期

Homie runtime 必须支持：

- `session.spawn`：本地或远端创建 session，支持 agent kind/profile、cwd、initial prompt、parent、initial geometry、new worktree、host、same repo。
- `session.list` 和 `state.snapshot`：返回 session、project、resource、artifact 和 selection 所需状态。
- `session.attach`：数据通道或等价 attachment，传输 grid、modes、input、resize、scroll、ping/pong。
- `session.input` / `session.send_text`：发送输入，支持 submit。
- `session.resize`：首个 geometry 立即发送，后续 debounce/pacing。
- `session.kill` / `session.remove` / `session.archive` / `session.unarchive`。
- `session.hibernate` / `session.wake`：按资源策略冻结或恢复空闲 session。
- `session.rename` / `session.mark_seen` / `session.set_owner`。
- `session.resume` / `session.reopen_last` / `session.history` / `session.resume_from_history`。
- `session.read_screen` / `session.read_scrollback` / `session.read_scrollback_cells` / `session.read_output`。
- `session.read_diff`：对本地和远端 session 都由 runtime/host 解析，不由 UI 在本机拼远端路径。
- parent/child/lineage：支持 agent 之间的 delegation 关系。

运行模型要求：

- app 退出不杀 agent session。
- runtime 重启后可以从 SQLite/output log/holder-equivalent 恢复 session 列表和可读输出。
- PTY owner 或 holder-equivalent 负责降低 runtime crash 对 live session 的影响。
- output log 使用 offset-addressed append/read，high-volume bytes 不写入 SQLite blob。
- headless emulator 是状态检测事实源之一，不能只 grep raw bytes。

### FR-5: 控制协议和事件模型必须对齐 Reference 并扩展 Homie 能力

Homie protocol 至少覆盖 Reference methods，并加入 Homie LLM/profile/task/memory 方法：

```text
hello
state.snapshot
client.set_active
governor.configure
agent.readiness
session.spawn/list/kill/remove/rename/resume/send_text/resize/set_owner
session.read_screen/read_scrollback/read_scrollback_cells/read_diff/mark_seen
session.hibernate/wake/archive/unarchive/reopen_last/history/resume_from_history
worktree.create/list/remove/overview
project.add
host.sync_prefs
host.locate_repo
events.subscribe
events.wait
hook.report
test.run
browser.act
llm.virtual_key.issue/revoke
llm.proxy.status
agent.profile.create/update/list/set_default
skills.list
mcp.server.list
permission.profile.list
context.session.summary
task.list/create/update
memory.search/write_candidate
```

事件至少覆盖：

```text
runtime.ready
runtime.unhealthy
session.created
session.spawned
session.updated
session.resources
session.status
session.needs_input
session.output
session.artifact
session.archived
session.removed
project.updated
worktree.created
worktree.removed
llm.request.started
llm.request.completed
llm.request.failed
tool.call.started
tool.call.completed
tool.call.failed
metrics.write_failed
context.updated
events.dropped
```

协议要求：

- control channel 使用本机 owner-only transport，macOS/Linux 为 UDS，Windows 预留 named pipe seam。
- event 有递增 seq，订阅支持 `since_seq`，断线后从 ring buffer 恢复。
- JSON decode 对 unknown enum/value 宽容。
- 错误统一 safe error envelope，不泄漏 key、Authorization、cookie、raw prompt 或完整敏感 tool args。

### FR-6: Terminal 渲染和输入必须对齐 Reference 设计

`homie-term` 必须支持：

- grid buffer：cols、rows、cursor、style、ANSI/default/rgb color、full snapshot、row diff。
- batched background quads 和 shaped text runs。
- shaped-line cache keyed by row content hash + font。
- cursor visible、blink-on-focus、block cursor。
- terminal themes，默认对齐 Reference Dark。
- font fallback：box drawing、spinner、emoji。
- input encoding：control chars、arrows、function keys、modifiers、bracketed paste、paste。
- scrollback：wheel outside alt-screen 拉取历史，alt-screen/mouse-reporting 转发 scroll frame。
- selection：跨 live grid + scrollback seam 选择，`Cmd-C` copy。
- find：debounced search、next/previous、wrap、history/live 统一高亮。
- hidden panes：暂停绘制并释放 GPU-side cache。
- resize pacing：首个 size 立即发，拖拽 pacing，cols-only reflow hold，避免一帧跳动。

验收必须包含：

- captured grid fixture decode/paint。
- fish/vim/agent TUI 真实输入验证。
- 1000 行 scrollback 选择和搜索。
- resize/侧边栏切换不丢最后一行，不出现明显上下跳。

### FR-7: 桌面产品设计必须对齐 Reference design

Homie 的视觉和交互设计以 Reference `PLAN.md` 和 `reference-ui` tokens 为基线。

设计 token：

- radii：chip 5、badge 6、row 7、card 10、panel 12。
- type：meta 11 med、sectionHeader 11 semibold、row 13、rowEmphasized 13 med、title 13 semibold、displayTitle 15 semibold、metaMono 11 med mono。
- ink：attention、danger、fresh、agent working color。
- fills：hover 0.06、multi-select 0.08、selected 0.10、subtle 0.06。
- metrics：titleBar 42、rowHeight 28、sidebar default 248、drag 200-400、min window 900x560。
- motion：row select 0.16、overlay fade 0.12、breathe 2.6s、sweep 2.4s、pulse 1.8s。

主要 surface：

- window chrome：hidden titlebar、full-size content、blurred or opaque-per-Homie decision、traffic lights offset、persistent placement。
- leading sidebar：top bar、New Agent、pinned/project/archive sections、account footer、drag reorder、multi-select、hover card、inline rename、project actions、session actions、drop-to-archive、sidebar toggle。
- status glyph：brand mark、working sweep/breathe、needsInput pulse、done unseen fresh、idle muted、hibernated dim、shell caret。
- terminal pane：header、branch、agent name、chips row、grid padding、find bar、scrolled pill、exited/resuming/archive/remote-active overlays。
- floating surfaces：command palette、quick open、find bar、popover、settings、history、worktrees sheet。
- command palette：actions + sessions，fuzzy scorer 与 Reference ranking 对齐。
- quick open：folder index、cache warm、git-aware ranking、default agent/shell launch。
- Ctrl-Tab switcher：live preview/filmstrip/release-to-commit。
- overview：running/needsInput/done/asleep/ended lanes，board/list，bulk close。
- history：scan Claude/Codex/provider transcripts，resume。
- worktrees sheet：cards、overview、cleanup suggestion、confirmation。
- settings：General、Terminal、Resources、Remote。
- menu bar extra：attention rollup、300w popover。
- notifications：needs-input actions，manifest-driven approve/deny。
- sounds：needsInput、done、frozen，可在 settings 关闭。
- trailing inspector：Info、Changes、Artifacts，diff virtualization，artifact/PR/status projection。
- workbench split：primary/auxiliary pane ratio，inspector width persistence。

键盘映射必须对齐 Reference，包括：

```text
Cmd-T, Cmd-Shift-N, Option-Cmd-T, Cmd-W, Cmd-Shift-T,
Cmd-K, Cmd-P, Cmd-Shift-O, Cmd-Shift-H, Cmd-B,
Option-Cmd-W, Cmd-,, Cmd-F, Cmd-G, Shift-Cmd-G,
Cmd-1..8, Cmd-9, Cmd-Up/Down/Left/Right, Cmd-[, Cmd-],
Ctrl-Cmd-Up/Down, Cmd-J, Cmd-R, Cmd-Shift-W,
Ctrl-Tab, Esc cascade, middle-click close
```

### FR-8: Worktree 和 project 管理必须对齐 Reference

Homie 必须支持：

- `project.add`。
- worktree create/list/remove/overview。
- session spawn 时 `new_worktree` 和 `worktree_branch`。
- 同 repo 远端定位：`same_repo_as` 和 `host.locate_repo`。
- worktree overview 中展示 path、branch、project root、session、status、dirty、merged、age、stale suggestion。
- cleanup 只对 safe stale suggestion 生效，dirty、unmerged、main 或风险项不能直接删除。
- remove 默认 `force=false`，force 需要显式确认和审计。

### FR-9: History 和 transcript resume 必须对齐 Reference

Homie 必须扫描 agent 自有 transcript 和 Homie session records：

- Claude Code、Codex 首版必须支持真实历史 resume。
- OpenCode/Gemini/Cursor 等如有可恢复 transcript，要通过 manifest 声明。
- history entry 包含 provider conversation id、agent kind、cwd、title、transcript path、last active、created at、cwd exists。
- 已被 Homie tracking 的 session 不重复显示为历史候选。
- dead cwd 的 resume 在 UI 中禁用或要求用户选择新 cwd。

### FR-10: Artifact、port、PR 和 browser/test surface 必须对齐 Reference

Homie 必须支持：

- artifact scanner：PR、Linear issue、preview URL、generic link。
- listening port detector：展示 local/remote preview URL。
- PR monitor：state、review decision、mergeability、checks passed/failed/pending、comments/reviews/thread resolved、additions/deletions。
- terminal toolbar chips：最多显示 4 个，高价值 PR 优先，其余 overflow。
- inspector artifacts tab 展示完整列表。
- `get_artifacts` MCP tool 返回结构化 artifact 和 PR stats。
- `test.run` 支持 chromium/webkit/firefox、steps、auth seeding、profile、a11y/screenshot observe。
- `browser` tool 支持 open/snapshot/click/fill/type/press/hover/select/check/scroll/get/wait/screenshot/console/back/close/list。

### FR-11: Usage、cost 和 cache accounting 必须对齐 Reference 并接入 Homie proxy

Homie 必须提供：

- provider usage：session/today/month。
- token 字段：input、output、cache read、cache write、total。
- cost 字段：使用 pricing snapshot，历史 cost 不随新价格漂移。
- active session window：例如 Claude five-hour block。
- transcript incremental scanner：byte offset cache，处理 truncate/replacement。
- Homie proxy authoritative records：request id、provider、model、tokens、cache、latency、cost、tool latency。
- node usage ledger：远端节点 usage.sqlite 或 Homie 等价 ledger 汇总。
- UI 展示：sidebar footer、account popover、settings/version row。

安全要求：

- usage、metric、trace 不保存 raw prompt、raw response、raw Authorization、cookie 或完整 tool args/result。
- metrics 写失败不能阻塞 LLM 响应，必须产生 `metrics.write_failed`。

### FR-12: LLM proxy、virtual key 和 credential custody 必须保持 Homie 特性

Reference 依赖 agent 自己的 provider login；Homie 必须在产品对齐基础上加入自己的 LLM custody 规则：

- real provider key 只存在 Homie local config/secret envelope 和短期内存。
- managed agent 环境中只注入 Homie virtual key 和 local proxy URL。
- virtual key 必须有 session/profile/provider/model scope、过期、撤销和审计。
- agent runtime adapter 不得写真实 key 到 argv、env、config、log、event、artifact。
- OpenAI-compatible proxy 支持 streaming、tool-call pass-through、safe error mapping、usage/cost accounting。
- 远端 node 默认通过 Homie virtual key 访问用户授权的 Homie proxy 或 node-local Homie proxy，不复制 provider raw key。

### FR-13: Remote hosts、node、handoff 必须对齐 Reference 产品能力

Homie Settings -> Remote 必须支持：

- host catalog：id、name、ssh、default cwd。
- node config：endpoint、token file、node id。
- owner-only host config 存储。
- SSH fallback 和 first-party node 两种执行路径。
- remote spawn：本地 app 在远端 host/node 创建 session。
- host sync prefs：同步 agent 偏好，不同步 credential。
- host locate repo：按 remote origin 找远端 checkout。
- companion access：Tailscale bind host、display host、pairing token、pairing URL、enable/disable。

Homie node 必须支持：

- node hello、capability negotiation、owner-only token。
- per-node account/profile label、login/status/default/list。
- Codex/Claude 等 provider session identity，不与 Homie raw provider key 混淆。
- session move/fork/handoff：preflight、checkpoint、content-addressed transfer、quarantine restore、provider-native resume/fork、lease commit。
- failure before commit aborts both sides；committed move 通过新 move 反向恢复，不做破坏性 rollback。
- fleet usage 汇总。

### FR-14: CLI、hook/notify 和 MCP 必须对齐 Reference 自动化面

`homie-cli` 必须提供：

```text
homie doctor
homie status
homie session list/spawn/kill/remove/rename/send-text/read-output/archive/unarchive/hibernate/wake/history/resume-from-history
homie worktree create/list/remove/overview
homie artifacts get
homie events subscribe/wait
homie ports list/forward
homie hook <event>
homie notify ...
homie mcp-stdio
homie mcp-tools
homie mcp-call --tool <tool>
homie llm proxy-status
homie runtime status
```

hook/notify 要求：

- fail-open：任意错误不阻塞 agent 原流程。
- stdin 读取有限制和超时。
- payload 脱敏后上报 `hook.report`。
- SessionStart 可返回 session title 等安全 output。

MCP tools 至少包含：

```text
spawn_agent
list_agents
get_status
send_prompt
wait_for_agent
read_output
create_worktree
list_worktrees
remove_worktree
get_artifacts
release_agent
test_run
browser
whoami
list_children
wait_for_children
```

MCP 权限要求：

- tool 调用必须绑定 session identity 和 lineage。
- 跨 session 写入必须符合 Homie permission profile。
- browser/test_run 不返回 inline image bytes，只返回文件路径和结构化摘要。

### FR-15: Resource governor、active state 和性能策略必须对齐 Reference

Homie 必须支持：

- `client.set_active`：app frontmost/occluded 状态，用于降低后台 tick。
- `governor.configure`：idle hibernate threshold、hard memory bytes。
- session resources event：memory bytes、listening ports、artifacts。
- terminal repaint pacing：active 50ms、background no invalidation。
- usage scan debounce：无 timer-driven idle work。
- scrollback cache cap。
- hidden/resident terminal capacity 策略，默认最多 3 个 resident renderer 或等价预算。

### FR-16: Packaging、updater 和 release 必须对齐 Reference 准出

Homie 首版发布必须支持：

- macOS `.app` bundle。
- universal binary 或明确的 architecture matrix。
- icon、Info.plist、entitlements。
- Developer ID signing、hardened runtime、notarization、stapling。
- DMG 和 update zip。
- auto-updater feed JSON。
- 更新信任模型：codesign verify、Team ID、bundle id、spctl、version match、HTTPS host pin、strictly newer only。
- 用户可见更新 flow：check、update pill、download progress、restart-to-update。
- helper swap：等待 pid 退出、旧 bundle rename、ditto unpack、失败恢复旧 bundle、install log。
- release script：version bump、tests、package、notarize、perf gate、feed update。

Homie 不允许无提示自动重启 live agent app。下载和安装必须由用户触发。

### FR-17: Packaged performance gate 必须对齐 Reference

Homie 必须建立 packaged artifact gate，而不是只测开发 binary：

- deterministic sidebar/workbench/terminal fixture。
- normal 和 large window 两档。
- physical footprint。
- mean/peak idle CPU。
- no autonomous decorative repaint loop。
- app-owned process 精确 PID 管理，不用 name-based kill。
- 默认预算初始值由 Homie spec 确定；目标不得弱于 Reference 的量级，除非有明确硬件/架构差异说明。
- release readiness 必须记录 macOS 版本、硬件、PID、footprint、avg CPU、peak CPU。

### FR-18: Storage、preferences 和 migration 必须对齐产品面

Homie SQLite 和 owner-only config 必须保存：

- sessions、projects、worktrees、artifacts、ports、PR statuses。
- output log index、scrollback offsets、screen checkpoints。
- agent manifests/profile/effective config。
- LLM provider/profile/virtual key/usage/pricing snapshot。
- preferences：default agent、last spawn host、start at login、confirm close、sounds、updates、hibernate after、memory limit、terminal theme/font、window placement、sidebar width/visibility/order/pinned/collapsed/archive、inspector state、quick open roots、last selected session。
- host catalog、remote/node config、companion access token。
- history scan cache、usage scan cache。

Migration 要求：

- forward-only。
- migration test 覆盖空库和已有库。
- 不写兼容旧 Homie 试验数据的 fallback，除非用户单独要求迁移。

### FR-19: 安全和隐私必须高于 Reference 复刻便利性

所有实现必须遵守：

- 不提交真实 provider key、virtual key signing secret、Authorization、cookie、private key、local agent credential、raw prompt、完整 tool args/result。
- remote/node token、hosts token file、companion token owner-only。
- agent env/config 只能得到 virtual key 或授权范围内的 token。
- logs/events/context/memory/metrics/report 全部脱敏。
- browser/test artifacts 不内联敏感 screenshot bytes。
- update helper 不执行未验证 bundle。
- git worktree cleanup 不删除 dirty/unmerged/main 风险路径。
- hook/notify fail-open 不能成为注入敏感数据的路径。

### FR-20: Homie Context、Memory、Task 和 Orchestration 必须接入首版

Reference 主要是 agent orchestrator；Homie 还必须把以下自有能力接入同一产品面：

- session context：每个 session 的 cwd、agent、profile、events、summary、artifacts、usage、parent/children。
- task controller：用户任务和 agent task 可被 session 领取、更新、阻塞和归还。
- memory controller：首版可只做写入候选和检索边界，但不得把 raw prompt/secret 写入 memory。
- intent orchestrator：命令面板、New Agent、MCP spawn 和 user prompt 可路由到合适 agent/profile。
- 后续多 agent 协作基于 lineage、task state、worktree 和 permission profile。

## 5. 边界情况

| 场景 | 处理方式 |
|------|----------|
| Reference 功能与 Homie 安全模型冲突 | 保留用户可感知能力，按 Homie security/credential custody 重设实现路径 |
| 某 agent manifest 无法稳定识别状态 | 阻塞该 agent parity，补 golden fixture，不 silently degrade |
| app 与 runtime 断连 | client backoff，events resume，attachment 重新 full snapshot |
| runtime crash | holder-equivalent 保持 PTY/output，恢复 registry；无法保持时标记 blocked 并记录 session 状态 |
| 同一 session 多客户端不同尺寸 | 首版要定义 geometry ownership/role；冲突时提示，不静默 thrash |
| 远端路径在本机不存在 | 所有远端 diff/read/worktree 通过 host/node 执行，不在 UI 本机执行路径 |
| update 下载成功但验证失败 | 删除 staged bundle，保留当前版本，显示安全错误 |
| worktree dirty 或 unmerged | cleanup disabled，必须人工确认或另建任务 |
| usage scan 遇到 transcript truncate | 重置 offset 并记录 ledger 修正，不重复计费 |
| MCP tool 长时间 wait | 不阻塞 async runtime cooperative pool，支持 timeout 和 cancellation |

## 6. 受影响组件 spec

| 组件 spec | 影响 | 要求 |
|-----------|------|------|
| `specs/desktop-shell/README.md` | 新增/更新 | 窗口、sidebar、terminal pane、surfaces、inspector、menu bar、notifications |
| `specs/runtime-supervisor/README.md` | 新增/更新 | PTY、holder、session lifecycle、output log、resource governor |
| `specs/agent-adapter-contract/README.md` | 新增/更新 | Reference manifest parity、status rules、approval/resume/hook |
| `specs/llm-proxy/README.md` | 新增/更新 | virtual key proxy、usage/cost/streaming |
| `specs/virtual-key-credentials/README.md` | 新增/更新 | real key custody、virtual key scope、remote/node policy |
| `specs/session-context-store/README.md` | 新增/更新 | session context、history、lineage、artifact summary |
| `specs/storage-indexing/README.md` | 更新 | SQLite schema、migration、preferences、output index |
| `specs/observability/README.md` | 新增/更新 | logs、metrics、events、evidence、redaction |
| `specs/task-controller/README.md` | 新增/更新 | task ownership、agent task state、orchestration |
| `specs/memory-controller/README.md` | 新增/更新 | memory write candidate、redaction、source |
| `specs/intent-orchestrator/README.md` | 新增/更新 | New Agent、MCP spawn、palette action routing |
| `specs/packaging-updater/README.md` | 新增 | bundle、signing、notarization、update trust |
| `specs/remote-node-handoff/README.md` | 新增 | host catalog、node、accounts、move/fork/handoff |
| `specs/mcp-automation/README.md` | 新增 | CLI、MCP tools、hook/notify、browser/test_run |

## 7. 测试计划

### 7.1 Spec gate

- PRD 自审：无 TBD、无缺失功能项、无与 Homie 安全模型冲突。
- Reference coverage matrix：每项有 Homie FR、OpenSpec task、verification。
- 组件 spec impact review：受影响组件全部更新或明确作为后续阻塞任务。

### 7.2 Unit 和 contract

- protocol JSON/frame/grid roundtrip。
- manifest schema 和 golden screen 状态检测。
- storage migration/repository/preferences。
- virtual key scope/revoke/expiry。
- terminal input encoding、grid update、scrollback cache。
- fuzzy ranking、settings validation、worktree cleanup safety。
- updater version/signature decision。

### 7.3 Integration

- fake runtime + real PTY session。
- Codex、Claude Code、OpenCode、shell smoke。
- hook/notify fail-open。
- MCP tool call against local runtime。
- output log detach/replay。
- artifact/port/PR scanner。
- LLM proxy fake provider success/failure/streaming。
- remote host/node local loopback harness。

### 7.4 E2E/manual

- app launch to first frame。
- create session -> type -> output -> status -> archive -> unarchive -> resume。
- daemon/runtime restart recovery。
- history scan/resume。
- worktree create and cleanup。
- needs-input notification approve/deny。
- browser/test_run flow。
- remote spawn and same repo locate。
- move/fork/handoff dry run and real local harness。
- old app update to new app。

### 7.5 UI 和性能

- deterministic preview screenshots：empty、typical、stress。
- sidebar、terminal、settings、history、worktree、overview、inspector screenshot comparison。
- narrow/min window no overlap。
- packaged perf gate：normal/large memory + idle CPU。
- resize churn and long-use memory retention。

### 7.6 Security

- `.githooks/pre-commit`。
- `git diff --check`。
- secret scan。
- capability diff for new network/filesystem/process/secret paths。
- update artifact signature/notarization verification。
- no raw key/log/event/metric leak regression tests。

## 8. 验收标准

V1 验收必须满足：

1. 所有 FR-1 到 FR-20 有实现、测试和 evidence。
2. `docs/research/reference-feature-coverage.md` 无未解释的 `missing` 或 `partial`。
3. 所有受影响组件 spec 已存在且测试计划回填。
4. OpenSpec alignment report 判定 `pass`。
5. `make full-check`、`make gauntlet` 或 V1 等价门禁通过。
6. app packaged release gate 通过。
7. 至少一次真实本机 Codex、Claude Code、OpenCode、shell session smoke 通过。
8. 至少一次 MCP orchestration E2E 通过。
9. 至少一次 remote/node harness 通过。
10. 至少一次 updater old-to-new 验证通过。
11. Beads `homie-h7n` 在 release-readiness-report 证据完整后才能 close。

## 9. Beads 追踪

- Beads issue：`homie-h7n`
- change_id：`reference-parity-v1`
- spec-id：`prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- 当前状态：spec draft


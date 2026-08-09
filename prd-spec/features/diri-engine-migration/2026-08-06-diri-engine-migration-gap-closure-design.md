# diri 功能复刻差距闭环设计文档

## 1. 概述

### 1.1 背景

Homie 的目标是完整复刻并重构仓库内 `diri/` 参考工程的功能、产品设计元素和 design token。当前仓库已经存在 `reference-parity-v1` 和 `diri-engine-migration` 两条文档与实现线，且有一批从 Diri 迁移过来的 Rust crate、agent manifest、终端模块和 UI token。

本次重新检查当前代码后，结论是：Homie 还不能被视为完成 Diri 全功能复刻。已有文档中存在“计划标为完成，但代码仍包含占位、stub 或未接线实现”的状态漂移。尤其是 runtime、桌面主界面、scrollback、status reducer、hook 解析和 design token 对齐仍有缺口。

本 PRD 是 `diri-engine-migration` 的差距闭环迭代文档，目的不是重新定义 Homie 产品方向，而是把当前识别到的 Diri parity 缺口转化为可执行、可验证、可关闭 Beads 的规格。

### 1.2 当前检查结论

| 范围 | 当前状态 | 影响 |
|------|----------|------|
| `homie-runtime` | 已有 `pty` 模块，但 `RuntimeSupervisor::spawn_shell/send_text/read_output` 仍只创建 SQLite session、追加文件日志，没有真正持有 live PTY session | 用户无法通过 Homie 创建可交互的真实 agent/ shell 会话 |
| `homie-app` | 主界面仍显示 `Next implementation slices`、`PTY-backed execution is the next runtime slice` 等占位文案 | 产品体验没有达到 Diri 的 sidebar + terminal + inspector 工作台形态 |
| `homie-term::scrollback` | 文件中仍有 `Stub types for diri-specific concepts`，`ReadScrollbackCellsResult` 为空元组，fetch/apply 路径没有真实协议结果 | 终端回滚、远端读取和历史视口不满足 Diri 语义 |
| `homie-agents` | 已迁移 manifest 检测，但 Diri `status` reducer 与 `hooks` 解析未迁移 | needs input、turn complete、subagent 隔离、hook authority、anti-flicker 等状态行为不完整 |
| `homie-ui` | 只包含部分 radius、motion、sidebar、fuzzy 等 token；Diri 的 typography、toolbar metrics、surface、spring、memory badge 等 token 未完整对齐 | 设计系统不能作为 Diri 产品设计 1:1 复刻的事实源 |
| `openspec/changes/diri-engine-migration/plan.md` | 标题写“完成”，但正文仍列出 scrollback、命令面板、status reducer、hooks 等延期项 | 项目管理状态与实际交付不一致 |
| `openspec/changes/reference-parity-v1/tasks.md` | 任务状态仍为 `todo` | 不能作为完成 Reference/Diri parity 的证据 |

### 1.3 目标

- 修正 `diri-engine-migration` 的状态漂移，明确当前 Homie 与 Diri 的真实差距。
- 让 Homie runtime 通过 `RuntimeSupervisor` 启动和管理真实 PTY session，而不是用文件日志模拟 session。
- 将 Diri 的 status reducer 和 hook parsing 能力迁移到 Homie 的 agent/status 层。
- 将 `homie-term::scrollback` 从 stub 替换为可用于 live grid、历史缓存和协议读取的真实视口模型。
- 补齐 Homie design token，使 `homie-ui` 能覆盖 Diri 的 radius、typography、metrics、motion、semantic colors、surface、memory badge 和 row fill 语义。
- 移除 `homie-app` 主界面的实现占位文案，形成 Diri 风格的 sidebar + terminal + inspector preview shell，并接入已有 terminal buffer 与 command palette 模型。
- 记录验证证据，保证后续不能再用“文档已完成”替代真实可运行能力。
- 保持 Homie 已确定的 app/client/runtime 分层：`homie-app` 只能通过 `homie-client`/protocol 或只读 preview 数据消费 runtime 状态，不直接拥有 PTY、SQLite 写入或 live session registry。
- 将本迭代不能闭环的 Diri 远端、MCP、updater、完整 RootView/StoreRuntime 迁移列为后续 Beads/OpenSpec 阻塞项，而不是在 release readiness 中默认为完成。

### 1.4 非目标

- 不迁移 Diri 的 Swift daemon 作为 Homie 长期事实源。Homie 的业务核心继续保留在 Rust crate 中。
- 不要求与 Diri 的 bundle id、数据目录、socket path 或二进制名称兼容。
- 不绕过 Homie 的 LLM proxy、virtual key、provider credential custody、安全日志和存储规范。
- 不在本迭代一次性完成 Diri 所有远端 node、updater、MCP 和完整 GPUI root/store runtime 的深度迁移；这些范围若仍缺失，必须在 evidence 中明确列为剩余项，而不能标为完成。
- 不为了快速对齐添加向后兼容层或旧接口 fallback。
- 不允许为了让 `homie-app` 快速显示 live session 而绕过 `homie-client`/protocol 分层。若当前客户端 crate 尚未具备能力，必须在 OpenSpec 中把 client/protocol 接线列为前置任务。

## 2. 用户场景

### 场景 1: 用户启动真实本地会话

**Given** 用户打开 Homie，并选择一个工作目录。  
**When** 用户通过 UI 或 CLI 创建 shell/Codex/Claude Code 会话。  
**Then** Homie 创建真实 PTY，启动子进程，实时采集输出，允许输入、resize、terminate，并在 SQLite 中记录会话状态与 output log 索引。

### 场景 2: 用户在 Homie 中看到 Diri 风格工作台

**Given** 用户打开 Homie 主窗口。  
**When** 没有真实 session 或 runtime 还未连接。  
**Then** 首屏仍呈现 Diri 对齐的 sidebar、terminal pane、inspector、toolbar、状态 chip 和命令入口，而不是未来实现计划或占位卡片。

### 场景 3: agent 进入 needs input

**Given** Claude Code 或 Codex 输出中出现权限请求、确认问题或 done/idle 信号。  
**When** manifest 检测、hook callback 或 notify 事件进入 Homie。  
**Then** status reducer 统一仲裁状态，避免屏幕闪烁，隔离 subagent 状态，并把 `needs_input`、`working`、`idle`、`done_unseen` 等状态提供给 sidebar、terminal header 和通知。

### 场景 4: 用户查看终端历史

**Given** 一个会话输出超过当前可见 grid。  
**When** 用户滚动、搜索或跳回 live。  
**Then** Homie 能从 live grid 和 scrollback cache 组合视图，必要时根据绝对行号请求历史 cells，并保证 alt screen、mouse reporting、wheel routing 语义符合 Diri。

### 场景 5: 用户检查 Diri design parity

**Given** 设计或实现人员需要确认 Homie 是否复刻了 Diri 的产品设计元素。  
**When** 查看 `homie-ui` token、组件测试和主界面 preview。  
**Then** 能看到与 Diri 对齐的 radius、typography、metrics、motion、semantic colors、surface、row fill、status glyph、agent logo、memory badge 等稳定 token，而不是散落在 `homie-app` 里的硬编码颜色和尺寸。

## 3. 功能需求

### FR-1: 真实 PTY runtime 接线

Homie 必须让 `RuntimeSupervisor` 管理 live PTY session：

- 创建 session 前必须校验 cwd、binary/argv 和权限边界；PTY spawn 失败不得留下状态为 `created` 的半成品 session。
- `spawn_shell` 必须启动真实进程，并保存 live session handle。
- `send_text` 必须写入 PTY master，而不是仅追加日志文件。
- `read_output` 必须读取 offset-addressed output log；输出来自 live PTY pump。
- `resize`、`terminate`、`archive`、`hibernate`、`wake` 的行为必须与 live session 状态一致。
- runtime 关闭或重启后，历史 output 仍可读；当前可声称的 holder-equivalent 范围仅限已有证据覆盖的 holder-owned PTY、supervisor drop/reopen adoption、terminate cleanup、exited/detached restore。完整 Diri crash matrix、process tree 和 resource governor 不得提前宣称完成。
- 单元和集成测试必须覆盖真实 `/bin/sh` 或等价 shell 路径，不允许只验证文件写入。

### FR-2: Diri status reducer 迁移

Homie 必须迁移并适配 Diri 的 status reducer：

- 支持 `Authority::HooksPrimary`、`ScreenPrimary`、`ProcessOnly`。
- 支持 Claude hook、Codex turn complete、screen observation、PTY activity、user keystroke、process exit、tick。
- 支持 startup grace、idle confirmation、blocker clear scans、hook authority window、staleness timeout。
- 支持 subagent start/stop bookkeeping，子 agent 状态不得污染 parent canonical status。
- 输出 `SessionStatus`、`NeedsInputDetail`、turn completed 事件。

### FR-3: Hook/notify parsing

Homie 必须迁移 Diri hook parsing 能力：

- 解析 Claude Code session start、prompt submit、pre tool use、permission request、notification、stop、subagent、session end。
- 解析 Codex notify/turn complete 信号。
- 输出稳定的 `HookEvent`/`NotifyEvent` 枚举，输入使用 fixture 驱动的 `serde_json::Value` 或等价结构，避免把原始 JSON 解析逻辑散落到 runtime。
- 对包含 token、secret、authorization、cookie、password 的 payload 做结构化脱敏。
- 无法解析的 hook 必须 fail-open：记录安全摘要，不阻塞 agent 运行，不泄漏原始敏感字段。

### FR-4: Scrollback 真实模型

`homie-term::scrollback` 必须移除 stub 状态：

- `ReadScrollbackCellsResult` 必须是包含 `first_row`、`rows`、`total_rows` 或等价信息的真实结构。
- `begin_fetch` 必须基于当前视口、可见行数和缓存缺口生成有效请求。
- `complete_fetch` 必须解码并缓存返回 rows，校验 row count、absolute row 和 codec 错误。
- `apply_geometry` 必须维护 live start row、total rows、max offset。
- `enter_alt_screen` 必须清理历史视口并回到 live。
- wheel routing 必须区分 alt screen/mouse reporting passthrough、本地滚动和 scrollback。

### FR-5: Design token 完整对齐

Homie 的 `homie-ui` 必须补齐 Diri 的设计 token：

- token 对齐源文件为 `diri/diri/crates/diri-ui/src/tokens.rs`、`components.rs`、`status.rs`、`brand.rs`。测试必须引用这些文件中的关键常量或等价硬编码期望，避免只验证 Homie 自己的当前值。
- Radius：chip、badge、row、card、panel。
- Typography：meta、section header、row、row emphasized、title、display title、meta mono。
- Metrics：title bar、toolbar edge inset、traffic light lane、toolbar gaps、control size、chip height、row height、new agent footer、traffic light offset。
- Motion：snap、pop、settle、footer pin、row select、overlay fade、seam slide、breathe、sweep、ping、risk ping、shell blink、tick hz。
- Semantic colors：light/dark、sidebar、floating stroke、floating surface、sidebar surface、text tone。
- Fill、Space、MemoryFormat。
- token 测试必须与 Diri 关键常量保持一致。

### FR-6: Homie app 去占位并呈现 Diri 风格工作台

Homie 主应用必须去掉“未来实现计划”类占位文案：

- 不再显示 `Next implementation slices`、`PTY-backed execution is the next runtime slice` 等未交付说明。
- 首屏采用 Diri 风格的三栏结构：sidebar、terminal/workbench、inspector。
- sidebar 显示 runtime、session、agent catalog、usage/update/status 等真实或 preview 数据。
- terminal pane 使用 `TerminalElement` 和 `GridBuffer` 展示实时或 preview grid。
- inspector 显示 Info、Changes、Artifacts 三类 surface 的 preview 或真实数据。
- 命令面板至少覆盖 New Terminal、Quick Open、Toggle Sidebar、Settings、Check Updates，并使用 Diri 对齐的 fuzzy ranking。

### FR-7: 状态和文档必须与实际交付一致

- `openspec/changes/diri-engine-migration/plan.md` 不得在存在延期项时标记为完成。
- 已完成、部分完成、未完成、阻塞必须分开标注。
- 必须新增或更新 `openspec/changes/diri-engine-migration/tasks.md` 和 `alignment-report.md`，把本 PRD 每个 FR 映射到具体任务、测试和证据路径。
- `docs/verification/diri-engine-migration/` 必须记录真实命令、退出码和未运行原因。
- `reference-parity-v1` 不能因为覆盖矩阵写了 `covered-by-reference-parity-v1` 就视为实现完成。

## 4. 实现方案

### 4.1 Runtime 接线

在 `homie-runtime` 内新增 live session registry，将现有 `Pty`、`HeadlessScreen`、output log、storage session summary 串起来。

建议最小实现：

- `RuntimeSupervisor` 持有 `Mutex<HashMap<String, LiveSession>>`。
- `spawn_shell` 先校验 binary/cwd，再启动 PTY pump；只有 PTY spawn 成功后才创建或更新 SQLite session 为 `starting/running`，失败路径不写半成品 `created` session。
- `send_text` 优先写 live PTY；若 session 不在 live registry，只允许返回明确错误或进入后续 resume 路径，不静默写日志。
- `read_output` 从 output log 读取，兼容已经存在的历史文件。
- 增加真实 PTY 集成测试，使用短生命周期 shell 命令验证输出和状态。
- protocol/client 缺口必须与 runtime 一起拆分：`homie-app` 或 `homie-cli` 的 live 操作经由 client/protocol 进入 runtime，不直接拿 `Storage` 或 live registry。

### 4.2 Agent 状态层

在 `homie-agents` 中新增 `status` 与 `hooks` 模块：

- 复用 Diri reducer 的纯函数状态机结构，适配 `homie-proto::model` 类型。
- hook parsing 独立于 runtime，可单测。
- runtime pump 将 screen observation、PTY activity、process exit 送入 reducer。

### 4.3 Terminal scrollback

在 `homie-term` 中把 `scrollback.rs` 改为真实数据模型：

- 用 `Vec<Vec<GridCell>>` 作为历史 rows 的最小可用返回类型。
- 保留 codec trait 以便未来接入 RLE wire format。
- 为 cache miss、row mismatch、alt screen、wheel route 添加单元测试。

### 4.4 UI token 与 app shell

在 `homie-ui` 中补齐 Diri token，`homie-app` 只消费 token，不再散落硬编码。

`homie-app` 当前不需要一次性迁移 Diri 的完整 `RootView/StoreRuntime/DaemonClient`，但必须做到：

- 视觉结构对齐 Diri 工作台。
- 不显示实现路线图式占位内容。
- 所有 preview 数据可替换为真实 runtime/session 数据。
- 命令面板和 sidebar/inspector 状态模型有测试。
- 如果当前 GPUI 版本无法完整迁移 Diri 的 `RootView`，本迭代只允许实现 Diri 对齐的 preview shell；真实 session 操作必须等 client/protocol 接线完成后再开放。

### 4.5 OpenSpec 与实施顺序

本 PRD 通过 spec review 后，必须先更新 `openspec/changes/diri-engine-migration/`：

1. `plan.md`：从“完成”改为“gap-closure in progress”，列出现有完成项、阻塞项和本迭代范围。
2. `tasks.md`：按 FR-1 到 FR-7 拆成 SDD/TDD 任务，每个任务包含 RED、GREEN、验收和证据路径。
3. `alignment-report.md`：证明本 PRD、组件 spec、OpenSpec task 和测试之间一一映射。

推荐实现顺序：

1. Runtime 真实 PTY 接线和测试。
2. Status reducer 与 hook parser。
3. Scrollback 真实模型。
4. Design token parity。
5. App preview shell 去占位。
6. 文档状态与 release readiness 证据收敛。

## 5. 组件 spec 影响

| 组件 spec | 影响 | 需要更新内容 |
|-----------|------|--------------|
| `specs/runtime-supervisor/README.md` | 是 | live PTY session registry、output log、status reducer 输入、terminate/resize/restart 边界 |
| `specs/agent-adapter-contract/README.md` | 是 | status authority、hook/notify parsing、needs input 详情、脱敏要求 |
| `specs/desktop-shell/README.md` | 是 | Diri 风格工作台、sidebar、terminal、inspector、command palette、design token 使用边界 |
| `specs/session-context-store/README.md` | 是 | session status、output index、history/read_output 语义 |
| `specs/observability/README.md` | 是 | hook parse failure、status transition、runtime process exit 的安全日志 |
| `specs/storage-indexing/README.md` | 可能 | 若新增 live session/output offset 字段或 repository API，需要更新 |

## 6. 边界情况

| 场景 | 处理方式 |
|------|----------|
| shell/agent binary 不存在 | 创建 session 前返回明确错误，不写半成品 session；错误信息不得泄漏敏感 env |
| live registry 中找不到 session | `send_text` 返回 `SessionNotLive` 或等价错误，不静默追加日志冒充输入成功 |
| PTY pump 写 output log 失败 | session 继续运行，状态仍更新，同时记录脱敏错误并暴露 degraded 状态 |
| hook payload 无法解析 | fail-open，记录安全摘要，不阻塞 agent，不泄漏原文 secret |
| alt screen 中滚轮事件 | 按 Diri 语义 passthrough 或 local routing，不误触历史 scrollback |
| runtime restart 后 live PTY 不存在 | 历史 output 可读，live 操作返回明确状态；有 holder 证据时可 adopt 或恢复 `exited`，缺少 holder 证据时标记 `detached` |
| UI 无真实 session | 显示 Diri 风格 preview/empty state，不显示实现计划或开发说明 |

## 7. 测试计划

### 7.1 RED/单元测试

- `homie-runtime`：真实 shell PTY spawn、input、output、terminate、holder adoption、exited/detached restore。
- `homie-agents`：status reducer 的 startup、working、needs input、idle、subagent、process exit。
- `homie-agents`：hook parsing 与 secret redaction。
- `homie-term`：scrollback request、fetch result、cache row、geometry、alt screen、wheel route。
- `homie-ui`：Diri token 常量 parity。
- `homie-app`：命令面板 ranking 与主界面不包含占位文案的 smoke/文本测试；若没有 UI snapshot harness，至少增加源文本禁止项测试，禁止 `Next implementation slices`、`PTY-backed execution is the next runtime slice` 等文案回归。

### 7.2 集成测试

- PTY -> output log -> read_output 端到端。
- PTY output -> headless screen -> manifest detection -> status reducer 端到端。
- app compile smoke，确认 `homie-app` 能编译并链接新 token/terminal API。

### 7.3 准出命令

最低要求：

```bash
cargo fmt --all -- --check
cargo test --workspace
```

若编译链允许，继续运行：

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

若 `Makefile` 中的完整门禁可用，最终以：

```bash
make full-check
```

作为 release readiness 的最高证据。未运行项必须在 `docs/verification/diri-engine-migration/` 写明原因。

## 8. 验收标准

- [ ] `RuntimeSupervisor::spawn_shell` 启动真实 PTY 进程，测试中能读取 shell 实际输出。
- [ ] `send_text` 写入 live PTY，不再用追加文件模拟用户输入。
- [ ] status reducer 与 hook parser 已迁移并有覆盖 needs input、idle、防闪烁、subagent 隔离的测试。
- [ ] `homie-term::scrollback` 不再包含空元组结果和 stub 概念，滚动/缓存/alt screen 测试通过。
- [ ] `homie-ui` token 覆盖 Diri 关键 token，并有 parity 测试。
- [ ] `homie-app` 首屏不再展示未来实现计划式占位文案，呈现 Diri 风格工作台结构。
- [ ] `openspec/changes/diri-engine-migration/plan.md` 和验证文档状态与实际实现一致。
- [ ] `docs/verification/diri-engine-migration/release-readiness-report.md` 记录真实验证结果、残余风险和后续项。

## 9. Beads 追踪

- Issue: `homie-cj5` — 从 diri 迁移核心引擎到 homie
- Change ID: `diri-engine-migration`
- 文档类型: feature gap-closure iteration
- 本文档路径: `prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md`
- 状态: 待 OpenSpec 更新和实现验证


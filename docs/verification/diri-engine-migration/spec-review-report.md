# diri-engine-migration Gap Closure Spec Review Report

## 1. 总体结论

- 可行性：中高。
- 最大风险：PRD 已正确识别当前 Homie 未完成 Diri 复刻，但若后续实现绕过 `homie-client`/protocol 直接让 `homie-app` 持有 runtime/storage，会破坏 Homie 已确定的长期分层。
- 推荐方向：保留本 PRD 作为 `diri-engine-migration` 差距闭环需求源；先更新 OpenSpec，再按 runtime -> status/hooks -> scrollback -> design token -> app preview shell 的顺序实施。

本次 review 已对 PRD 做过一次整改，重点补充了 app/client/runtime 分层约束、spawn 失败的半成品 session 防护、HookEvent/NotifyEvent 稳定枚举、Diri token 对齐源文件、OpenSpec tasks/alignment 要求和实施顺序。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | 架构一致性 | 初版 PRD 要求 `homie-app` 接入 runtime/session，但没有强调 Homie 既有 `homie-client`/protocol 分层 | 后续实现可能为了快速展示 live session 让 UI 直接依赖 runtime 或 storage，违背 `docs/architecture/project-layout.md` | 已补充：`homie-app` 只能通过 `homie-client`/protocol 或只读 preview 数据消费 runtime 状态 |
| P0 | 数据一致性 | 初版 PRD 没有明确 PTY spawn 失败时是否允许先写 SQLite session | 可能留下状态为 `created` 的半成品 session，继续放大当前“假 session”问题 | 已补充：创建前校验 cwd/binary/权限，PTY spawn 失败不得留下半成品 session |
| P0 | SDD/TDD 适配 | 初版 PRD 只要求修正 plan，没有明确 `tasks.md` 和 `alignment-report.md` | 后续可能再次从聊天上下文直接实现，绕过 OpenSpec 映射 | 已补充：必须新增/更新 OpenSpec tasks 和 alignment report，将每个 FR 映射到任务、测试、证据 |
| P1 | 模块边界 | Hook parsing 要求迁移，但未明确输出类型和解析位置 | 解析逻辑可能散落到 runtime pump，难以测试和脱敏 | 已补充：输出稳定 `HookEvent`/`NotifyEvent` 枚举，解析逻辑留在 `homie-agents` |
| P1 | 设计一致性 | Design token 对齐没有指定 Diri 源文件 | token 测试可能只验证 Homie 当前值，无法防止偏离 Diri | 已补充：对齐源为 Diri `tokens.rs`、`components.rs`、`status.rs`、`brand.rs` |
| P1 | 可落地性 | App shell 要求“Diri 风格工作台”，但未区分 preview shell 和完整 RootView 迁移 | 容易把本迭代扩大成完整 Diri app 搬迁，风险过高 | 已补充：若 GPUI 版本限制存在，本迭代只开放 preview shell；真实 session 操作等待 client/protocol 接线 |
| P2 | 回归测试 | UI 去占位没有最低自动化防线 | 占位文案可能回归且不被测试发现 | 已补充：无 UI snapshot harness 时至少增加源文本禁止项测试 |

## 3. 整改后的完善方案

目标与范围：

- 以 `prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md` 作为本轮需求事实源。
- 解决当前已识别的核心差距：真实 PTY runtime、status reducer、hook parsing、scrollback、design token、app 去占位。
- 纠正已有 OpenSpec 和 verification 文档的状态漂移。

非目标：

- 不迁移 Diri Swift daemon 为 Homie 事实源。
- 不绕过 Homie virtual key、provider credential custody、安全日志和 storage 规范。
- 不在本迭代一次性完成 Diri 远端 node、updater、MCP 和完整 RootView/StoreRuntime。

设计原则：

- UI 不直接拥有 PTY、live session registry 或 SQLite 写入。
- Runtime 接线先证明真实 `/bin/sh` PTY 端到端，再扩展到 agent runtime。
- Status/hook 逻辑保持纯函数和 fixture 驱动，避免散落在 runtime pump。
- Design token 必须以 Diri 源文件为对齐基线。
- 文档状态必须落后于证据，不能领先于实现。

验收标准：

- `RuntimeSupervisor::spawn_shell` 可启动真实 PTY，并通过测试读取 shell 实际输出。
- `send_text` 写 live PTY，不再追加文件冒充输入。
- status reducer、hook parser、scrollback、token parity 均有 RED/GREEN 测试。
- `homie-app` 首屏不再包含未来实现计划式占位文案。
- OpenSpec plan/tasks/alignment 与 PRD、验证报告一致。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| OpenSpec | 更新 `plan.md`、新增/更新 `tasks.md`、新增/更新 `alignment-report.md` | OpenSpec 三件套 | 本 PRD | P0 |
| Runtime | 接线 live PTY registry、spawn/input/output/terminate | `homie-runtime` 代码和真实 PTY 测试 | OpenSpec | P0 |
| Agent status | 迁移 status reducer 和 hook parser | `homie-agents::status/hooks` 和 fixtures | Runtime 状态输入模型 | P0 |
| Terminal | 替换 scrollback stub | `homie-term::scrollback` 和单测 | GridCell 模型 | P1 |
| UI tokens | 补齐 Diri token parity | `homie-ui` token 和 parity 测试 | Diri token 源文件 | P1 |
| App shell | 去占位并呈现 Diri preview shell | `homie-app` shell、palette/sidebar/inspector 测试 | token、terminal API | P1 |
| Evidence | 记录命令、退出码、残余风险 | `docs/verification/diri-engine-migration/release-readiness-report.md` | 所有实现任务 | P0 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| 单元测试 | status reducer | startup、working、needs input、idle、subagent、process exit | Agent status |
| 单元测试 | hook parsing | Claude permission request、Codex notify、secret redaction、unknown fail-open | Agent status |
| 单元测试 | scrollback | cache miss、fetch result、row mismatch、alt screen、wheel route | Terminal |
| 单元测试 | design token | radius、typography、metrics、motion、semantic colors、MemoryFormat | UI tokens |
| 集成测试 | runtime PTY | `/bin/sh` spawn、send_text、read_output、terminate | Runtime |
| 集成测试 | detection pipeline | PTY output -> screen -> manifest -> reducer | Runtime/Agent |
| 回归测试 | app 去占位 | 禁止 `Next implementation slices`、`PTY-backed execution is the next runtime slice` 等文本 | App shell |
| 准出门禁 | workspace | `cargo fmt --all -- --check`、`cargo test --workspace`、尽可能运行 clippy/full-check | Closeout |

## 6. 开发排期

| 阶段 | 时间/顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Phase 0 | 先行 | OpenSpec plan/tasks/alignment 更新 | 防止再次从聊天上下文直接实现 | OpenSpec 文档 |
| Phase 1 | 第 1 批 | Runtime live PTY 接线 | PTY 跨平台只做 Unix，Windows 保持明确 unsupported | Runtime 测试通过 |
| Phase 2 | 第 2 批 | Status reducer + hook parser | 类型适配可能影响 `homie-proto` | Agent 测试通过 |
| Phase 3 | 第 3 批 | Scrollback stub 替换 | 协议读取结果需保持最小模型 | Terminal 测试通过 |
| Phase 4 | 第 4 批 | Design token parity | GPUI 类型差异需要用等价常量表达 | UI token 测试通过 |
| Phase 5 | 第 5 批 | App preview shell 去占位 | 不开放未接线 live 操作 | App 编译和文本回归通过 |
| Phase 6 | 收尾 | Evidence 和 release readiness | 未运行门禁必须标 `not_run` | 验证报告 |

## 7. 待确认问题

- `homie-client` crate 当前在 workspace 规范中存在，但代码树中尚未出现；OpenSpec 需要确认本轮是新增 `homie-client`，还是先通过 `homie-proto` + runtime 内部测试验证，UI 仅做 preview shell。
- 是否需要把远端 node、MCP、updater 的剩余 Diri parity 缺口拆成新的 Beads 子任务，避免 `homie-cj5` 无限扩大。
- 是否要求本轮完成真实 GPUI 截图/视觉回归门禁；如果没有现成 harness，应先以源文本回归和编译 smoke 作为最低 UI gate。

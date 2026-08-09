# Bingo Component Spec Review

```yaml
change_id: diri-module-inventory
reviewed_target: specs/*/README.md
requested_reviewer: bingo
bingo_attempt: failed_tool_call_limit
fallback_reviewer: trae-cli-main
status: needs_revision
```

## 1. Overall Verdict

- 可行性：中。
- 最大风险：当前 `specs/*` 大多是 Homie 架构愿景级合同，能表达组件边界，但还不能逐项承接 `docs/research/diri-module-inventory.md` 的 `Mxx-F###` 功能原子项。直接按现有 specs 进入实现，仍可能出现“模块名对齐，Diri 行为未对齐”的问题。
- 推荐方向：先把每个组件 spec 增加 `Diri parity mapping`、`Feature atom ownership`、`Verification gates` 三个表，再进入逐模块 PRD/OpenSpec。

## 2. Bingo Attempt

`bingo exec` 已被调用做只读审查，但运行中断。Bingo 输出显示：只完成了 specs 目录列表和一次不存在的 `cross-agent-collaboration/README.md` 读取尝试，未读取任一 spec 文件或 inventory 文档完整内容，也未开始审查分析。该输出不能作为有效 review 结论。

## 3. Per-Spec Findings

### specs/agent-adapter-contract/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | Diri 对齐 | 未逐项映射 M16-F001/M16-F002 的 agent catalog、readiness、golden screen、hook/notify redaction | Agent adapter 容易只实现 manifest schema，而漏掉 Diri golden 状态检测和 readiness 行为 | 增加 `Diri parity mapping`：AgentCatalog、ManifestSchema、Reducer、GoldenScreen、HookParsing、AgentReadiness |
| P1 | 验证 | 测试计划没有绑定 `DirijorDetectionTests/*` 和 `DirijorDaemonKitTests/AgentReadiness` | 无法证明 reducer 和检测结果等价 | 增加 golden fixtures、risk/redaction、readiness E2E gates |
| P1 | 安全 | approve/deny、hook payload、subagent 隔离缺少强制失败模式 | 可能误触发 quick action 或泄露 tool args | 增加 fail-closed/unknown-agent/no-quick-action 规则 |

### specs/desktop-shell/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | 产品设计 | 未按 M01..M09 拆出 Diri 视图状态矩阵 | UI 容易继续局部像 Diri，但不是完整 workbench | 增加 Workbench/Sidebar/Terminal/Inspector/Settings/Notifications/Worktrees 的 state matrix |
| P0 | E2E | screenshot gate 只作为测试计划存在，未定义 Diri side-by-side assertion | 可能把单张 Homie 截图当成完成证据 | 增加 Diri reference screenshot、Homie screenshot、差异阈值、人工验收项 |
| P1 | 交互 | Esc cascade、focus、overlay 互斥没有状态机 | 快捷键/浮层行为容易互相覆盖 | 增加 shell overlay state machine |

### specs/intent-orchestrator/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | Diri 对齐 | 未映射 M13/M12 的 MCP lineage、send_prompt、parent/child wait 行为 | 编排器可能只做 Homie 自有 route，不覆盖 Diri MCP automation | 增加 `MCP lineage and automation intent` 章节 |
| P1 | 数据流 | 与 task、memory、runtime、MCP 的调用边界不够具体 | 实现时可能跨层直接调用 | 定义 IntentRequest -> Decision -> Runtime/MCP/Task 的接口 |
| P2 | 测试 | 缺少跨模块 contract tests | 回归难发现 | 增加 orchestrator routing fixture |

### specs/llm-proxy/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | Diri 对齐 | M19 usage accounting 和 Homie virtual-key proxy 的关系没有落到 Diri usage/fleet accounting | usage UI 和 proxy telemetry 可能分裂 | 增加 `Usage record contract` 对齐 M19-F001/M19-F002 |
| P1 | 安全 | provider key custody 与 remote node policy 需要和 `virtual-key-credentials` 双向引用 | remote/node 可能错误复制 raw key | 增加 remote raw-key rejection gate |
| P2 | 验证 | 缺少 pricing/cache/reasoning token fixtures | 成本统计难验证 | 增加 usage/pricing fixture tests |

### specs/mcp-automation/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | Diri 对齐 | 未 method-by-method 映射 Diri MCP tools 与 CLI bridge | MCP surface 很容易漏 tool 或 lineage 行为 | 增加 M13-F001/M13-F002 工具清单、参数、返回 envelope |
| P0 | 权限 | lineage、跨 session 操作、release_agent 权限边界不够可测试 | 可能出现越权 agent 操作 | 增加 parent/child identity、permission profile、denied cases |
| P1 | E2E | stdio MCP server 没有完整协议验收 | 不能证明 agent 可真实调用 | 增加 MCP stdio transcript fixture |

### specs/memory-controller/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P2 | Diri 对齐 | Diri module inventory 中 memory 不是一等 Diri 模块，当前 spec 是 Homie 自有扩展 | 可能干扰 Diri parity 优先级 | 标注为 Homie extension，不能阻塞 Diri parity |
| P1 | 来源 | memory candidate 与 session context/tool result 的边界需更清楚 | 可能把 raw prompt/tool args 写入 memory | 增加 redaction/source citation gates |
| P2 | 验证 | 缺少 retention/approval workflow fixtures | 行为不稳定 | 增加 candidate lifecycle tests |

### specs/observability/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | Diri 对齐 | 未覆盖 Diri EventBus、DaemonLog、metrics write failure 的具体事件 catalog | runtime/app/CLI 证据链可能断裂 | 增加 M14 event bus 与 M19 usage metrics 映射 |
| P1 | 安全 | raw prompt/tool args redaction 规则需要与 hooks/MCP/LLM proxy 统一 | 日志泄漏风险 | 建立全局 safe field whitelist |
| P2 | 验证 | 缺少日志/metrics schema fixture | 难验证可观测性稳定 | 增加 JSONL/event schema tests |

### specs/packaging-updater/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | Diri 对齐 | 未逐项覆盖 M20-F001/M20-F002 的 feed、trust、install、rollback、DMG、notarization、perf | 可能只打出 `.app`，但不能称 Diri release parity | 增加 release pipeline checklist 和 package artifact gates |
| P1 | 验证 | packaged launch 与 notarization/perf gate 区分不清 | 本机可开不等于可发布 | 分层：local app smoke、codesign、DMG、notarization、perf |
| P1 | 回滚 | updater rollback 语义不足 | 更新失败不可恢复 | 增加 rollback state machine |

### specs/remote-node-handoff/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | Diri 对齐 | 未细化 M18-F001/M18-F002：node server/accounts/spawn/checkpoint/handoff/prefs sync/repo locate | remote 可能只实现配置壳 | 增加 remote protocol method table 和 local node E2E |
| P0 | 安全 | provider raw key、node account、companion token 的边界需要和 credential spec 强绑定 | 高危泄漏 | 增加 no-raw-key-to-node gate |
| P1 | 运行 | remote failure/retry/offline semantics 不够具体 | E2E 不稳定 | 增加 remote unreachable/reconnect/failover behavior |

### specs/runtime-supervisor/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | 范围过宽 | 同时覆盖 PTY/session/log/screen/status/resource/checkpoint/migration/event bus | 单个 spec 难以 TDD 实现 | 拆子规格或增加 M14a..M14d 章节 |
| P0 | Diri 对齐 | RT-004/005/009/010 的剩余缺口未转成状态机和验收 | 容易误标 runtime complete | 增加 status reducer authority、hook injection、process sampling、migration gates |
| P1 | 协议 | runtime API 与 `homie-proto` method catalog 未逐项绑定 | client/CLI 对接可能漂移 | 增加 runtime method table |

### specs/session-context-store/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | Diri 对齐 | M05 history/transcript resume 与 M17 core session context 未明确分界 | history scanner 可能无处落地 | 增加 transcript/history/session summary table |
| P1 | 数据模型 | prompt/tool/result context 的敏感字段策略不够具体 | 泄漏或不可追溯 | 增加 source/redaction/citation fields |
| P2 | 验证 | 缺少 resume from missing cwd/transcript 的 fixtures | 恢复行为不清 | 增加 history resume negative cases |

### specs/storage-indexing/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | Diri 对齐 | 没有把 M05/M06/M07/M17/M19 的表字段和唯一约束逐项列出 | 后续 migration 容易反复改 | 增加 table-by-table schema inventory |
| P1 | 数据访问 | 缺少 repository/query API 级合同 | 各模块可能直接写 SQL | 增加 storage API ownership table |
| P1 | 验证 | migration/backfill/backup gates 不够细 | 数据升级风险 | 增加 migration idempotency + downgrade refusal tests |

### specs/task-controller/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P2 | Diri 对齐 | Diri 没有同名一等 task controller，该 spec 是 Homie 自有编排能力 | 不应阻塞 Diri parity 基础实现 | 标注为 Homie extension，并说明与 Diri MCP/lineage 的关系 |
| P1 | 边界 | Beads、本地 task、agent task 三者边界需更明确 | 状态源冲突 | 增加 Beads boundary and sync rules |
| P2 | 验证 | 缺少 claim/block/handoff fixtures | 任务流不可验证 | 增加 task lifecycle tests |

### specs/virtual-key-credentials/README.md

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | 安全 | 与 remote node、LLM proxy、MCP tools 的 raw key 禁止传播规则需要强制引用 | 高危 credential 泄漏 | 增加 cross-spec mandatory gates |
| P1 | Diri/Homie 融合 | Diri 行为与 Homie 自有 virtual-key 架构不是一一对应 | 机械复刻会破坏 Homie 设计 | 增加 “Diri behavior parity / Homie credential adaptation” 双栏 |
| P1 | 验证 | revoke/expiry/scope/audit fixtures 不足 | 虚拟 key 行为不可证 | 增加 issue/revoke/expired/scope-denied tests |

## 4. Cross-Spec Gaps

| 优先级 | 问题 | 涉及 spec | 整改建议 |
|---|---|---|---|
| P0 | 缺少 `Mxx-F### -> spec section -> OpenSpec task -> verification` 的统一索引 | 全部 | 在 `specs/README.md` 增加 Feature Atom ownership table |
| P0 | Diri tests 没有落到各 spec 的验收引用 | 全部 | 每个 spec 增加 `Diri test mapping` 章节 |
| P0 | UI 视觉基线不够强制 | `desktop-shell`, `packaging-updater`, `observability` | 加入 screenshot gate、Diri reference screenshot、blank rejection |
| P1 | 安全规则分散 | `virtual-key-credentials`, `llm-proxy`, `remote-node-handoff`, `mcp-automation`, `observability` | 建立 shared security gate list |
| P1 | Runtime/protocol/client/CLI 方法映射不完整 | `runtime-supervisor`, `mcp-automation`, `desktop-shell`, `storage-indexing` | 增加 method/event/DTO contract matrix |

## 5. Implementation Readiness

| spec | 是否可直接进入 PRD/OpenSpec | 前置补充 |
|---|---|---|
| `agent-adapter-contract` | 否 | Diri golden tests、readiness、hook redaction mapping |
| `desktop-shell` | 否 | UI state matrix、Diri screenshot matrix、interaction gates |
| `intent-orchestrator` | 否 | MCP lineage/automation intent mapping |
| `llm-proxy` | 部分 | usage accounting + remote key custody links |
| `mcp-automation` | 否 | tool-by-tool MCP contract and lineage security |
| `memory-controller` | 部分 | mark as Homie extension, add redaction/source gates |
| `observability` | 部分 | event schema and safe field whitelist |
| `packaging-updater` | 否 | release pipeline gates incl notarization/perf |
| `remote-node-handoff` | 否 | node protocol/account/spawn/handoff matrix |
| `runtime-supervisor` | 否 | split runtime subdomains and method table |
| `session-context-store` | 部分 | history/transcript mapping |
| `storage-indexing` | 否 | table/API inventory |
| `task-controller` | 部分 | mark as Homie extension and Beads boundary |
| `virtual-key-credentials` | 部分 | cross-spec credential gates |

## 6. Recommended Next Actions

1. Update `specs/README.md` with a Feature Atom ownership table.
2. For each component spec, add `Diri parity mapping` and `Diri test mapping`.
3. Split or subsection `runtime-supervisor`, `mcp-automation`, `remote-node-handoff`, and `packaging-updater` before implementation.
4. Add a `make spec-diri-mapping-check` script after the mapping tables stabilize.
5. Only then generate per-module PRD/OpenSpec for implementation.


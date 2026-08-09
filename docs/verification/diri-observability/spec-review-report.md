# Diri observability 第一阶段 Spec Review Report

```yaml
change_id: diri-observability
beads: homie-wm7
report_type: spec-review
source_prd: prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md
source_spec: specs/observability/README.md
status: pass_with_recorded_constraints
review_date: 2026-08-07
```

## 1. 总体结论

- 可行性：高。
- 最大风险：observability 是 L0 foundation lane，若第一阶段实现直接接入 runtime/LLM/storage，会越过当前 worker 的限定写入范围，并与其他 lane 的未完成合同冲突。
- 推荐方向：本阶段只交付可复用合同、safe field whitelist、Diri event/evidence mapping 和纯模型测试；runtime、LLM proxy、storage、client 的真实接入必须作为后续 OpenSpec 变更处理。

本次 review 已对 PRD 做过一次约束校正：明确新建 `homie-observability` 只作为独立纯模型 crate 验证，不修改根 workspace、不改 runtime、不接入 LLM/storage；后续接入必须由对应 lane 另起 PRD/OpenSpec。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P0 | 范围控制 | 初始目标包含 event、metrics、usage、evidence，容易被理解为要直接改 runtime/LLM/storage 接线 | 会违反本 worker 限定写入范围，也可能破坏其他 lane 的并行工作 | 已在 PRD 非目标和方案中明确：本阶段只做规格和纯模型 crate，不接入 runtime/LLM/storage |
| P0 | 安全 | 当前组件 spec 只有禁止敏感信息的原则，没有字段级 allowlist | 下游可以各自发明日志字段，导致 raw prompt/tool args/header 泄漏 | 已要求建立全局 safe field whitelist，未知字段默认丢弃，危险字段 fail closed |
| P1 | Diri 对齐 | 当前 spec 没有把 Diri `EventBus` 的 `seq`、filter 和 `events.dropped` marker 写成合同 | 后续 client/CLI 事件恢复无法证明与 Diri 语义一致 | 已要求 event envelope、event catalog、filter semantics 和 `seq=0` drop marker 测试 |
| P1 | 可测试性 | `metrics.write_failed` 只有一句“不阻断主流程”，没有可执行模型 | LLM proxy/usage 后续可能吞掉 metrics 写失败或泄漏 raw error | 已要求 `MetricsWriteFailure::to_event` 纯模型测试，保留 safe error code 和 retryable |
| P1 | Usage 边界 | usage evidence 未区分摘要字段和 raw transcript/provider payload | usage UI 和 release evidence 可能记录 raw prompt、message body 或 provider response | 已要求 usage projection 只允许 token/cache/cost/latency 等摘要，校验非负 token 和有限 cost |
| P2 | Workspace 发现 | PRD 选择独立 crate、不改根 `Cargo.toml`，短期不会被 `cargo test --workspace` 自动覆盖 | 后续若忘记接入 workspace，observability crate 可能被主门禁遗漏 | 在 release readiness 中标为残余风险；后续 observability integration PRD 必须决定是否接入根 workspace |
| P2 | 文档状态 | Beads 关闭条件未明确 | 可能在只完成模型后误标 broader observability parity 已完成 | 已在 PRD 中限定：`homie-wm7` 仅代表第一阶段完成，broader runtime/LLM接入不在本阶段 |

## 3. 整改后的完善方案

目标与范围：

- 第一阶段建立 `specs/observability/README.md` 的可执行合同。
- 新建 `crates/homie-observability`，只承载 safe fields、safe events、metrics failure、usage evidence、command evidence 的纯模型。
- 所有实现均可通过独立 `--manifest-path` 测试验证。

非目标：

- 不修改 runtime supervisor、LLM proxy、storage repository、client protocol、CLI command。
- 不添加真实事件总线、SQLite 表、socket API、LLM metrics sink。
- 不把本阶段标记为完整 Diri observability parity，只标记为 foundation contract ready。

设计原则：

- Allowlist first：只有显式安全字段可以进入 logs/events/metrics/evidence。
- Diri event semantics first：`seq`、filter、drop marker、event catalog 必须先稳定。
- 主流程与观测写入解耦：metrics 写失败只生成 safe event，不改变主业务结果。
- Evidence honesty：未运行就是 `not_run`，阻塞就是 `blocked`，不允许 helper 自动改成 pass。

验收标准：

- PRD、spec review、functional cases、OpenSpec plan/tasks/alignment 完整。
- `specs/observability/README.md` 已更新 Diri mapping 和 whitelist。
- `cargo test --manifest-path crates/homie-observability/Cargo.toml` 通过。
- functional verification report 逐条记录 FC-OBS-001 到 FC-OBS-006。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|---|---|---|---|---|
| Spec | 更新 `specs/observability/README.md` 的 whitelist/event/metrics/usage/evidence 合同 | 组件 spec | PRD | P0 |
| Functional cases | 设计 FC-OBS-001 到 FC-OBS-006 | `functional-cases.md` | PRD/spec review | P0 |
| OpenSpec | 拆 plan/tasks/alignment | `openspec/changes/diri-observability/*` | functional cases | P0 |
| TDD slice 1 | safe field whitelist | crate API + tests | OpenSpec | P0 |
| TDD slice 2 | event envelope/filter/drop marker | crate API + tests | TDD slice 1 | P0 |
| TDD slice 3 | metrics write failure projection | crate API + tests | TDD slice 1 | P1 |
| TDD slice 4 | usage evidence projection | crate API + tests | TDD slice 1 | P1 |
| TDD slice 5 | command evidence model | crate API + tests | TDD slice 1 | P1 |
| Verification | 运行格式、check、clippy、test、diff gates | verification reports | implementation | P0 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|---|---|---|---|
| 单元测试 | Safe whitelist | 保留允许字段、丢弃 unknown、拒绝 dangerous field | TDD slice 1 |
| 单元测试 | Event semantics | catalog、session/kind filter、`events.dropped` 永远可见 | TDD slice 2 |
| 单元测试 | Metrics failure | sink 失败投影为 safe event，不携带 raw error/message/payload | TDD slice 3 |
| 单元测试 | Usage evidence | token/cost 非负校验、safe projection、secret stripping | TDD slice 4 |
| 单元测试 | Evidence model | gate status 枚举、functional case event、not_run 不变 pass | TDD slice 5 |
| 静态门禁 | Rust crate | fmt、check、clippy、test | Verification |
| 文档门禁 | Traceability | PRD -> OpenSpec -> FC -> evidence 映射 | Verification |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|---|---|---|---|---|
| Phase 0 | 先行 | PRD/spec review | 控制 scope，避免直接接 runtime | PRD 和本报告 |
| Phase 1 | 其次 | functional cases | 保证验证口径先行 | `functional-cases.md` |
| Phase 2 | 其次 | OpenSpec plan/tasks/alignment | 不从聊天上下文直接实现 | OpenSpec 三件套 |
| Phase 3 | 之后 | TDD 纯模型实现 | crate 独立于根 workspace 的发现风险 | crate tests |
| Phase 4 | 收尾 | functional verification、code review、release readiness | 未运行门禁诚实标注 | verification reports |

## 7. 待确认问题

- 后续 integration lane 是否把 `crates/homie-observability` 接入根 workspace，需要在另一个 PRD/OpenSpec 中决定；本阶段为了遵守写入范围不修改根 `Cargo.toml`。
- `homie-llm` 的 usage accounting 与本 crate 的 `UsageEvidence` 是否最终共用同一类型，需要在 `diri-usage-accounting` 变更中确定。
- 真实 EventBus 是归 `homie-runtime`、`homie-client` 还是独立 `homie-observability` runtime helper，需要在 runtime/client lane 接入时决定。

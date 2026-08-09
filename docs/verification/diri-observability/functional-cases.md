# Diri observability 第一阶段功能验证 Case

```yaml
change_id: diri-observability
beads: homie-wm7
source_prd: prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md
source_spec: specs/observability/README.md
status: designed
designed_at: 2026-08-07
```

## 1. 执行原则

- 所有 Case 必须在实现后逐条执行，结果写入 `docs/verification/diri-observability/functional-verification-report.md`。
- 本阶段只验证 observability 纯模型和文档合同，不启动 runtime、LLM proxy、storage 或真实 Diri daemon。
- 所有输出不得包含 raw key、Authorization、cookie、private key、raw prompt、raw request/response、完整 tool args/result。
- 如果命令未运行，报告状态只能是 `not_run` 或 `blocked`，不能写 `pass`。

## 2. Case 清单

### FC-OBS-001: Safe field whitelist 只保留显式安全字段

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 优先级 | P0 |
| 前置环境 | `crates/homie-observability` 已创建 |
| 执行命令 | `cargo test --manifest-path crates/homie-observability/Cargo.toml safe_field -- --nocapture` |
| 输入数据 | 测试内构造包含 `event.name`、`session.id`、`usage.input_tokens`、`authorization`、`raw_prompt`、`tool_args`、unknown field 的 JSON |
| 预期输出 | 安全字段保留；unknown field 丢弃；危险字段不出现在投影结果中；危险字段检测返回 `RedactionBlocked` 或等价错误 |
| 通过标准 | 测试退出码为 0，断言证明危险字段未泄漏 |
| 证据路径 | `docs/verification/diri-observability/functional-verification-report.md#fc-obs-001` |
| 失败处理 | 回到 TDD slice 1 修复 whitelist 和 projector |

### FC-OBS-002: Diri EventBus envelope 和 filter/drop marker 语义

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-2 |
| 优先级 | P0 |
| 前置环境 | `SafeEvent` 和 event catalog 已实现 |
| 执行命令 | `cargo test --manifest-path crates/homie-observability/Cargo.toml event -- --nocapture` |
| 输入数据 | 构造 `session.status`、`session.output`、`worktree.created`、`events.dropped` 四类事件，并分别应用 session filter、kind filter、组合 filter |
| 预期输出 | session filter 只接收指定 session 事件；kind filter 只接收指定 kind；无 session 的 worktree/project 事件不会进入 session filter；`events.dropped` 在任何 filter 下可见且 `seq=0` |
| 通过标准 | 测试退出码为 0，断言事件名、seq、session_id 和 filter 结果符合 Diri contract |
| 证据路径 | `docs/verification/diri-observability/functional-verification-report.md#fc-obs-002` |
| 失败处理 | 回到 TDD slice 2 修复 event model |

### FC-OBS-003: metrics.write_failed 不阻断主流程且只输出安全字段

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-3 |
| 优先级 | P1 |
| 前置环境 | `MetricsWriteFailure` 已实现 |
| 执行命令 | `cargo test --manifest-path crates/homie-observability/Cargo.toml metrics -- --nocapture` |
| 输入数据 | 构造 metrics write failure，包含 safe error code、retryable、component、operation，以及模拟 raw SQL、Authorization、provider request body |
| 预期输出 | `metrics.write_failed` event 只包含 whitelist 字段；主流程成功值保持成功；危险字段被丢弃 |
| 通过标准 | 测试退出码为 0，断言 failure event 中没有 raw SQL、Authorization、request body |
| 证据路径 | `docs/verification/diri-observability/functional-verification-report.md#fc-obs-003` |
| 失败处理 | 回到 TDD slice 3 修复 metrics projection |

### FC-OBS-004: Usage evidence projection 只保留 Diri 对齐摘要字段

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-4 |
| 优先级 | P1 |
| 前置环境 | `UsageEvidence` 已实现 |
| 执行命令 | `cargo test --manifest-path crates/homie-observability/Cargo.toml usage -- --nocapture` |
| 输入数据 | 构造 Claude/Codex-like usage payload，包含 provider、profile_id、session_id、model、token/cache/cost/source/value_kind，以及 raw transcript line、prompt、tool result |
| 预期输出 | 安全 usage projection 保留 token/cache/cost/source/value_kind；raw transcript/prompt/tool result 被剔除；负 token 或 NaN/Inf/负 cost 返回错误 |
| 通过标准 | 测试退出码为 0，断言 valid projection 与 invalid value rejection 均生效 |
| 证据路径 | `docs/verification/diri-observability/functional-verification-report.md#fc-obs-004` |
| 失败处理 | 回到 TDD slice 4 修复 usage validation/projection |

### FC-OBS-005: Evidence helper 保持 gate status 诚实

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-5 |
| 优先级 | P1 |
| 前置环境 | `CommandEvidence` 或等价模型已实现 |
| 执行命令 | `cargo test --manifest-path crates/homie-observability/Cargo.toml evidence -- --nocapture` |
| 输入数据 | 构造 `pass`、`blocked`、`not_run`、`partial`、`fail` evidence，并生成 `verification.functional_case_executed` event |
| 预期输出 | `not_run` 保持 `not_run`；`blocked` 保持 `blocked`；functional case event 包含 case id、status、evidence path 等 safe fields |
| 通过标准 | 测试退出码为 0，断言没有未运行门禁被转为 pass |
| 证据路径 | `docs/verification/diri-observability/functional-verification-report.md#fc-obs-005` |
| 失败处理 | 回到 TDD slice 5 修复 evidence model |

### FC-OBS-006: PRD/spec/OpenSpec/evidence traceability 完整

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-6 |
| 优先级 | P0 |
| 前置环境 | PRD、spec review、functional cases、OpenSpec 三件套已写入 |
| 执行命令 | `test -f prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md && test -f docs/verification/diri-observability/spec-review-report.md && test -f docs/verification/diri-observability/functional-cases.md && test -f openspec/changes/diri-observability/plan.md && test -f openspec/changes/diri-observability/tasks.md && test -f openspec/changes/diri-observability/alignment-report.md` |
| 输入数据 | 文件系统中的规格与验证产物 |
| 预期输出 | 命令退出码为 0 |
| 通过标准 | 所有文件存在；alignment report 映射 FR -> Task -> FC -> Evidence |
| 证据路径 | `docs/verification/diri-observability/functional-verification-report.md#fc-obs-006` |
| 失败处理 | 回到文档阶段补齐缺失文件或映射 |

## 3. 覆盖矩阵

| PRD requirement | Functional case | 覆盖状态 |
|-----------------|-----------------|----------|
| FR-1 Safe field whitelist | FC-OBS-001 | covered |
| FR-2 Diri EventBus parity mapping | FC-OBS-002 | covered |
| FR-3 Metrics write failure contract | FC-OBS-003 | covered |
| FR-4 Usage evidence projection | FC-OBS-004 | covered |
| FR-5 Evidence helper model | FC-OBS-005 | covered |
| FR-6 Component spec and OpenSpec traceability | FC-OBS-006 | covered |

## 4. 执行顺序

1. FC-OBS-006 文档 traceability 可在 OpenSpec 写完后先执行。
2. FC-OBS-001 到 FC-OBS-005 随 TDD slice 逐条执行。
3. 所有 Case 结果统一写入 `functional-verification-report.md`。
4. 任何 P0 Case 失败时不得进入 release readiness pass。

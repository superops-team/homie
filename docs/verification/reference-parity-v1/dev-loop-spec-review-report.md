# Reference Parity V1 Dev Loop Spec Review Report

```yaml
change_id: reference-parity-v1
report_type: dev-loop-spec-review
status: pass_for_case_design
beads: homie-h7n
source_prd: prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md
openspec_plan: openspec/changes/reference-parity-v1/plan.md
openspec_tasks: openspec/changes/reference-parity-v1/tasks.md
reviewed_at: 2026-08-05
dev_loop_step: 1
```

## 1. 总体结论

- 可行性：中。产品目标清晰，但 scope 是 umbrella 级别，不能直接进入实现。
- 最大风险：把 Reference parity 当成单个实现任务，导致组件 spec、功能验证 Case、UI/远端/安全门禁和真实证据断裂。
- 推荐方向：接受当前 PRD 作为需求事实源；先完成可执行功能验证 Case 与 Task/Case 映射；实现前必须拆成子 Beads 并补齐长期组件 specs。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 | 状态 |
|---|---|---|---|---|---|
| P0 | SDD/TDD 适配 | 当前 OpenSpec task 尚未绑定功能验证 Case ID | 后续开发可能先写实现再临时补验收，违反 dev-loop | 新增 `functional-cases.md`，并在 OpenSpec tasks 中建立 Task -> Case 映射 | 本轮修复 |
| P0 | 最小化实现 | `reference-parity-v1` 是 umbrella，不适合一个分支完成 | 一个 PR 同时改 UI/runtime/LLM/remote/updater，无法审查和验证 | 保持 umbrella PRD，只允许从 T-001 起拆子 Beads 和小切片 | 已通过 gate 固化 |
| P0 | 架构一致性 | 多个受影响组件 spec 尚未作为长期合同落地 | 直接实现会绕过 repo 的 `prd-spec/` 与 `specs/` 分层 | 实现前必须创建/更新 listed component specs；T-001 是开发前置任务 | 已通过 gate 固化 |
| P1 | 可执行验证 | PRD 验收标准很完整，但缺少逐条可执行 Case | Step 7 无法逐条执行，也无法证明 P0/P1 覆盖 | 设计功能验证 Case、覆盖矩阵和证据路径 | 本轮修复 |
| P1 | 大模型语义偏差 | “1:1 复刻”容易被解释为复制 Reference 代码或保持兼容 | 可能破坏 Homie 的 Rust/SQLite/virtual-key 架构 | PRD 已声明只复刻产品能力，不复刻二进制/协议/数据兼容；后续 specs 需继续保持 | 已修复 |
| P1 | 安全/凭证 | Reference 的 node-local provider login 与 Homie credential custody 存在策略差异 | remote/node 实现可能复制 provider raw key | `virtual-key-credentials` 与 `remote-node-handoff` specs 必须先定义 virtual-key-safe policy | 已通过 gate 固化 |
| P2 | 可扩展性 | UI fidelity、remote/node、updater 都依赖真实环境 | 无真实 Mac/notary/node 时 release gate 可能长期 blocked | Case 设计中明确 not_run/blocked 规则和证据要求 | 本轮修复 |
| P2 | 存量影响 | 新 PRD 改写了 Reference coverage matrix 的状态词 | 如果误读为已实现，会造成状态偏差 | 文档中明确 `covered-by-reference-parity-v1` 代表需求覆盖，不代表实现完成 | 已修复 |

## 3. 整改后的完善方案

目标与范围：

- `reference-parity-v1` 保持 umbrella PRD，只作为产品能力、设计、自动化、远端、发布和安全的完整需求事实源。
- 任何实现必须从 OpenSpec task 拆出小型子 Beads，并先更新对应 component spec。
- 功能验证 Case 前置设计是进入开发的硬门禁。

非目标：

- 不在本轮 spec review 中直接实现代码。
- 不用单个 PR 完成全部 Reference parity。
- 不用 Reference 旧名称、路径或兼容接口作为 Homie 文档事实源。

设计原则：

- PRD 说明 what/why；`specs/` 说明 component contract；OpenSpec 说明执行任务；verification 说明证据。
- Homie 安全模型优先于 Reference 实现路径。
- 每个 P0/P1 功能至少有一个可执行功能验证 Case。

核心流程：

1. Step 1 spec review 通过后，补 `functional-cases.md`。
2. 将 OpenSpec tasks 映射到 Case ID。
3. 对齐报告检查 FR -> Task -> Case -> Evidence 是否闭环。
4. 只有 component specs 和 Case 设计通过后，才允许进入 SDD/TDD 实现。

兼容与风险控制：

- Reference 只作为代号化参考产品，不写旧名称。
- 不承诺 Reference 数据/进程/socket 兼容。
- 远端/node、LLM proxy、updater、browser/test_run 都按 high-risk gate 执行。

验收标准：

- 旧参考名称全仓库扫描无命中。
- `docs/verification/reference-parity-v1/functional-cases.md` 存在并覆盖 P0/P1。
- `openspec/changes/reference-parity-v1/tasks.md` 有 Task -> Case 映射。
- `docs/verification/reference-parity-v1/dev-loop-alignment-report.md` 判定可进入后续 SDD/TDD gate。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|------|------|--------|------|--------|
| Spec gate | 完成 16 维 spec review | `dev-loop-spec-review-report.md` | PRD/OpenSpec | P0 |
| Verification design | 设计功能验证 Case | `functional-cases.md` | Spec review | P0 |
| OpenSpec mapping | 将 Task 绑定 Case ID | 更新 `tasks.md` | Case 清单 | P0 |
| Alignment | 复核 FR/Task/Case/Evidence | `dev-loop-alignment-report.md` | Task mapping | P0 |
| Component contracts | 创建/更新长期 component specs | `specs/*/README.md` | Alignment pass | P0 |
| Implementation | SDD/TDD 小切片实现 | code + tests | Component specs | P0/P1 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|------|--------|----------|----------|
| Spec tests | PRD/OpenSpec 无旧名称、无断链 | 旧参考名称扫描、`git diff --check` | Step 1/4 |
| Functional cases | P0/P1 FR 覆盖 | FC-001 到 FC-020 | Step 2/7 |
| Unit | protocol、manifest、storage、terminal、virtual key | cargo test focused suites | Step 5/7 |
| Integration | runtime、PTY、LLM proxy、MCP、hook、worktree | cargo test --workspace --tests | Step 5/7 |
| E2E | app、session、remote/node、updater | real app and local node harness | Step 10 |
| Security | secret scan、redaction、updater signature | pre-commit/security gate | Step 7/10 |

## 6. 开发排期

| 阶段 | 时间/顺序 | 工作项 | 风险与缓冲 | 验收物 |
|------|-----------|--------|------------|--------|
| Gate A | 先行 | Spec review + functional cases + task mapping | 无代码实现 | Step 1-4 docs |
| Gate B | Gate A 后 | T-001 component specs and child Beads | umbrella 拆分复杂 | updated `specs/` + Beads |
| Gate C | Gate B 后 | P0 foundation: protocol/runtime/storage/agent/terminal | runtime 和 PTY 风险高 | focused tests + evidence |
| Gate D | Foundation 后 | Local product UI and automation | UI fidelity 风险高 | screenshots + real session smoke |
| Gate E | Local product 后 | remote/node/updater/perf/security | 外部环境依赖 | release readiness evidence |

## 7. 待确认问题

- 是否需要为 T-001 到 T-017 立即创建子 Beads，还是在下一轮按实现优先级逐批创建。
- Homie remote/node 对 provider identity 的最终策略：只允许 Homie proxy virtual key，还是允许 node-local provider login 作为用户显式模式。
- macOS signing/notarization 证书与 release host 是否在本机可用；若不可用，release gate 必须保持 blocked。


# OpenSpec Alignment Report: Diri Agent Detection

```yaml
change_id: diri-agent-detection
beads: homie-v4b
status: aligned_for_implementation
source_prd: prd-spec/features/diri-agent-detection/2026-08-07-diri-agent-detection-design.md
functional_cases: docs/verification/diri-agent-detection/functional-cases.md
plan: openspec/changes/diri-agent-detection/plan.md
tasks: openspec/changes/diri-agent-detection/tasks.md
```

## 1. 对齐结论

- PRD 10 条功能需求均映射到 OpenSpec tasks。
- 每个 OpenSpec task 均绑定至少一个 Functional Case。
- 本 change 的实现范围限定在用户指定写入路径内；runtime/UI/storage/CLI 明确为非目标。
- P0/P1 spec review 问题均有 task 和 case 覆盖。

## 2. PRD -> OpenSpec -> Functional Case

| PRD | OpenSpec Task | Functional Case | 对齐状态 |
|-----|---------------|-----------------|----------|
| FR-1 combined manifest loader | T2 | FC-DA-001, FC-DA-006 | aligned |
| FR-2 19-agent catalog + alias | T3 | FC-DA-001, FC-DA-002 | aligned |
| FR-3 descriptor Diri capability fields | T3 | FC-DA-001, FC-DA-002 | aligned |
| FR-4 readiness projection | T3 | FC-DA-003 | aligned |
| FR-5 full/process-only manifest rules | T2, T4 | FC-DA-004, FC-DA-006 | aligned |
| FR-6 golden screen parity | T4 | FC-DA-004 | aligned |
| FR-7 hook/notify stable events | T5 | FC-DA-005 | aligned |
| FR-8 redaction | T5 | FC-DA-005, FC-DA-008 | aligned |
| FR-9 unknown hook fail-open | T5 | FC-DA-005 | aligned |
| FR-10 spec/test mapping | T1, T6 | FC-DA-007 | aligned |

## 3. Spec Review Finding -> Remediation

| Finding | Remediation Task | Verification |
|---------|------------------|--------------|
| P0 combined manifest 缺失 | T2 | FC-DA-001, FC-DA-006 |
| P0 golden tests 缺失 | T4 | FC-DA-004 |
| P0 hook redaction 不足 | T5 | FC-DA-005 |
| P1 resume style 缺失 | T3 | FC-DA-001, FC-DA-002 |
| P1 readiness projection 缺失 | T3 | FC-DA-003 |
| P1 failure mode 分散 | T1, T3, T5 | FC-DA-002, FC-DA-005 |

## 4. 边界复核

| 边界 | 结论 |
|------|------|
| 写入范围 | 仅写 PRD/OpenSpec/evidence/component spec/assets/homie-agents |
| Runtime | 不修改，不运行真实 PTY |
| Storage | 不修改，不引入 migration |
| UI | 不修改，不做 screenshot gate |
| Security | 不引入 secret，不依赖真实 provider key |

## 5. 最终执行顺序

1. 更新 component spec。
2. 写 catalog/readiness RED tests。
3. 写 golden screen RED tests。
4. 写 hook redaction RED tests。
5. 实现 combined manifest catalog/readiness/redaction。
6. 运行 functional cases 与 quality/security gates。
7. 写 release readiness 和两轮 code review 报告。

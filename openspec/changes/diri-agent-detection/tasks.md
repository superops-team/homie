# OpenSpec Tasks: Diri Agent Detection

```yaml
change_id: diri-agent-detection
beads: homie-v4b
status: ready_for_sdd_tdd
```

## 1. 任务清单

| Task | 描述 | 交付物 | 验收标准 | Functional Case | 状态 |
|------|------|--------|----------|-----------------|------|
| T1 | 更新长期组件规格 | `specs/agent-adapter-contract/README.md` | 包含 Diri parity mapping、readiness、golden、redaction、failure mode、verification gates | FC-DA-007 | completed |
| T2 | 升级 combined manifest 数据 | `assets/agent-descriptors/*.json` | 19 个 manifest 严格解码；full manifest 有 rules；process-only 可空 | FC-DA-001, FC-DA-006 | completed |
| T3 | TDD catalog/readiness API | `crates/homie-agents/src/lib.rs`, `manifest_catalog.rs` | alias/fallback/resume/readiness tests 通过 | FC-DA-001, FC-DA-002, FC-DA-003 | completed |
| T4 | TDD golden screen parity | `crates/homie-agents/tests/golden_screens.rs` | Claude/Codex/Cursor/Gemini golden tests 通过 | FC-DA-004 | completed |
| T5 | TDD hook redaction parity | `detect/redact.rs`, `hooks.rs`, `hook_parser.rs` | nested/header/URL secret 不进入 parsed summaries | FC-DA-005, FC-DA-008 | completed |
| T6 | 执行验证并留证 | `docs/verification/diri-agent-detection/release-readiness-report.md`, code review reports | 实际命令、退出码、结果、残余风险完整记录 | FC-DA-007, FC-DA-008 | completed |

## 2. TDD 执行顺序

1. T3 RED: 增加 catalog/readiness tests，确认当前简化 schema 失败。
2. T3 GREEN: 实现 `AgentCatalog`、`AgentReadiness*`、`ResumeStyle`、`AgentInjection`。
3. T4 RED: 增加 golden screen tests，确认现有 assets 无 detection rules 导致失败。
4. T2/T4 GREEN: 将 assets 升级为 combined manifest，修正 manifest loader。
5. T5 RED: 增加 hostile redaction tests。
6. T5 GREEN: 增强 redaction。
7. Refactor: 只做必要整理，保持 public API 小而稳定。
8. T6: 运行 focused gates 和 review。

## 3. Task 与需求映射

| Task | PRD 需求 |
|------|----------|
| T1 | FR-10 |
| T2 | FR-1, FR-5 |
| T3 | FR-2, FR-3, FR-4 |
| T4 | FR-6 |
| T5 | FR-7, FR-8, FR-9 |
| T6 | AC-1..AC-7 |

## 4. 不做项确认

- 不修改 `crates/homie-runtime`。
- 不修改 `crates/homie-app`、`crates/homie-cli`、`crates/homie-storage`。
- 不更新 `docs/research/diri-parity-lock.md`，因为本阶段不是完整 parity closeout。

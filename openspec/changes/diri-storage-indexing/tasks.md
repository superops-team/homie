# OpenSpec Tasks: Diri Storage/Indexing Parity Phase 1

```yaml
change_id: diri-storage-indexing
beads: homie-q7n
status: ready_for_sdd_tdd
```

## Task List

| Task | Description | Acceptance Criteria | Functional Case | Dependency | Estimate |
|------|-------------|---------------------|-----------------|------------|----------|
| T1 | Complete PRD/spec review and long-lived storage contract | PRD exists; spec review report has no unresolved P0/P1; `specs/storage-indexing/README.md` has phase 1 inventory | FC-STOR-001 | none | S |
| T2 | Write schema inventory RED test | Test checks required tables, fields, indexes and unique constraints through real migrated SQLite | FC-STOR-001 | T1 | S |
| T3 | Implement migration for phase 1 schema additions | Schema version advances; migration idempotent; required fields/indexes/constraints exist | FC-STOR-001, FC-STOR-007 | T2 | M |
| T4 | Keep preferences repository aligned with Diri settings | Settings defaults/load/save pass typed API test | FC-STOR-002 | T3 | S |
| T5 | Add history repository API | Upsert/list/mark tracked behavior passes; `(agent_kind, external_id)` dedupes | FC-STOR-003 | T3 | M |
| T6 | Add project/worktree repository API | Project root/worktree path/branch uniqueness pass; summaries include Diri flags | FC-STOR-004 | T3 | M |
| T7 | Add session core metadata API | Diri SessionRecord phase 1 fields round-trip and FK constraints hold | FC-STOR-005 | T3, T6 | M |
| T8 | Add usage record/query API | Source event dedupes; totals aggregate by filters; token/cache/cost fields preserved | FC-STOR-006 | T3 | M |
| T9 | Run focused functional cases and record results | FC-STOR-001..006 have command output, exit code, pass/fail status | FC-STOR-001..006 | T4-T8 | S |
| T10 | Run local quality gates and readiness report | fmt/check/test/diff/pre-commit attempted; failures classified as lane or external | FC-STOR-007 | T9 | S |
| T11 | Two-round code review | Round 1 explicit issues fixed; round 2 adversarial pass has no P0/P1 | FC-STOR-007 | T10 | S |

## SDD/TDD Rules

- For T2/T5/T6/T7/T8, write or update a focused test before implementation changes for that behavior.
- Keep tests through public APIs where a public API exists.
- SQLite metadata assertions are allowed only for schema inventory and unique/index verification.
- Do not add UI/runtime/CLI behavior to satisfy storage tests.
- Do not weaken existing tests to make migration pass.

## Task To Requirement Mapping

| Requirement | Tasks |
|-------------|-------|
| FR-001 表级 inventory | T1, T2 |
| FR-002 Schema migration | T2, T3 |
| FR-003 唯一约束与关系约束 | T2, T3, T5, T6, T8 |
| FR-004 Repository/query API inventory 与最小实现 | T4, T5, T6, T7, T8 |
| FR-005 安全边界 | T1, T3, T8, T10, T11 |

## Verification Exit Criteria

- `cargo test -p homie-storage` passes.
- Focused FC-STOR commands are executed and recorded.
- Non-storage workspace failures, if any, are not hidden and are listed as blockers in release readiness.
- No file outside the lane write scope is modified by this change.

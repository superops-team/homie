# OpenSpec Alignment Report: Diri Storage/Indexing Parity Phase 1

```yaml
change_id: diri-storage-indexing
beads: homie-q7n
status: aligned
prd: prd-spec/features/diri-storage-indexing/2026-08-07-diri-storage-indexing-design.md
plan: openspec/changes/diri-storage-indexing/plan.md
tasks: openspec/changes/diri-storage-indexing/tasks.md
functional_cases: docs/verification/diri-storage-indexing/functional-cases.md
```

## 1. PRD Requirement Alignment

| PRD Requirement | Spec Contract | OpenSpec Tasks | Functional Cases | Status |
|-----------------|---------------|----------------|------------------|--------|
| FR-001 表级 inventory | `specs/storage-indexing/README.md` 12.1-12.3 | T1, T2 | FC-STOR-001 | aligned |
| FR-002 Schema migration | `specs/storage-indexing/README.md` 12.2 | T2, T3 | FC-STOR-001, FC-STOR-007 | aligned |
| FR-003 唯一约束与关系约束 | `specs/storage-indexing/README.md` 12.2 | T2, T3, T5, T6, T8 | FC-STOR-001, FC-STOR-003, FC-STOR-004, FC-STOR-006 | aligned |
| FR-004 Repository/query API inventory 与最小实现 | `specs/storage-indexing/README.md` 12.3 | T4, T5, T6, T7, T8 | FC-STOR-002..006 | aligned |
| FR-005 安全边界 | `specs/storage-indexing/README.md` 12.5 | T1, T3, T8, T10, T11 | FC-STOR-001, FC-STOR-006, FC-STOR-007 | aligned |

## 2. Diri Atom Alignment

| Atom | Storage Surface | OpenSpec Task | Verification | Excluded Work |
|------|-----------------|---------------|--------------|---------------|
| M05-F002 | `history_entries`, history repository | T5 | FC-STOR-003 | transcript scanner, resume runtime, history UI |
| M06-F001 | `preferences`, settings preferences API | T4 | FC-STOR-002 | settings UI, remote prefs sync execution |
| M07-F001 | `projects`, `worktrees`, project/worktree repository | T6 | FC-STOR-004 | real git detection |
| M07-F002 | worktree session link and cleanup flags | T6, T7 | FC-STOR-004, FC-STOR-005 | real worktree create/remove |
| M17-F001 | session core metadata fields | T7 | FC-STOR-005 | proto/UI model expansion outside storage |
| M19-F001 | `usage_records`, pricing, scan files, rollups, usage repository | T8 | FC-STOR-006 | transcript parser, LLM proxy, usage UI |

## 3. Functional Case Coverage

| Functional Case | Covers Tasks | Gap |
|-----------------|--------------|-----|
| FC-STOR-001 | T2, T3 | none |
| FC-STOR-002 | T4 | none |
| FC-STOR-003 | T5 | none |
| FC-STOR-004 | T6 | none |
| FC-STOR-005 | T7 | none |
| FC-STOR-006 | T8 | none |
| FC-STOR-007 | T3, T9, T10, T11 | none |

## 4. Scope Guard

The plan intentionally excludes UI/runtime/parser/git execution work. Those remain in their own Diri parity lanes and must consume `homie-storage` through repository/query APIs after this contract lands.

Write scope remains limited to:

- `prd-spec/features/diri-storage-indexing/`
- `openspec/changes/diri-storage-indexing/`
- `docs/verification/diri-storage-indexing/`
- `specs/storage-indexing/README.md`
- `crates/homie-storage/**`

## 5. Alignment Verdict

- PRD to component spec: aligned.
- Component spec to OpenSpec tasks: aligned.
- OpenSpec tasks to functional cases: aligned.
- No P0/P1 requirement is unmapped.

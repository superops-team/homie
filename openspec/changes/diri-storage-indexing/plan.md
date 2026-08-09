# OpenSpec Plan: Diri Storage/Indexing Parity Phase 1

```yaml
change_id: diri-storage-indexing
beads: homie-q7n
lane: lane-foundation-storage
status: ready_for_tasks
prd: prd-spec/features/diri-storage-indexing/2026-08-07-diri-storage-indexing-design.md
component_spec: specs/storage-indexing/README.md
functional_cases: docs/verification/diri-storage-indexing/functional-cases.md
```

## 1. Goal

Land the first storage/indexing foundation slice for Diri parity by turning M05/M06/M07/M17/M19 into explicit SQLite schema, unique constraints, indexes, and repository/query APIs inside `homie-storage`.

## 2. Scope

In scope:

- `specs/storage-indexing/README.md` phase 1 schema inventory.
- `crates/homie-storage` SQLite migration, constraints, indexes and repository/query APIs.
- Storage integration tests and verification evidence.

Out of scope:

- UI surfaces, GPUI rendering, screenshots.
- Runtime session process lifecycle.
- Git shell execution or real worktree creation/removal.
- Transcript/history scanner implementation.
- Usage transcript parser or LLM proxy.
- Makefile/scripts changes.

## 3. Module Plan

| Module | Responsibility | Inputs | Outputs | Verification |
|--------|----------------|--------|---------|--------------|
| Spec contract | Durable schema/API contract for M05/M06/M07/M17/M19 | Diri inventory, review reports | Updated `specs/storage-indexing/README.md` | Spec review report |
| Migration | Forward-only SQLite schema version bump | Existing v1/v2 schema | New fields/tables/indexes/constraints | FC-STOR-001, existing migration tests |
| Schema inventory API | Public schema metadata reader for tests/lane readiness | SQLite metadata | Tables/columns/indexes/unique facts | FC-STOR-001 |
| Preferences repository | Diri settings payload persistence | `SettingsPreferences` | Default/load/save settings | FC-STOR-002 |
| History repository | Scanned transcript history facts | `UpsertHistoryEntry` | ordered history list, tracked session link | FC-STOR-003 |
| Project/worktree repository | Repo/worktree facts without git execution | `UpsertProject`, `UpsertWorktree` | project/worktree summaries | FC-STOR-004 |
| Session core metadata | Diri SessionRecord phase 1 subset | `SessionCoreMetadataUpdate` | rich session summary | FC-STOR-005 |
| Usage repository | Safe usage event ledger and aggregation | `RecordUsage`, `UsageQuery` | deduped writes, totals | FC-STOR-006 |
| Evidence/review | Quality gates and two review rounds | code + test results | readiness and review reports | FC-STOR-007 |

## 4. Data Flow

```text
open_or_create
  -> PRAGMA foreign_keys=ON, journal_mode=WAL
  -> migrate v1/v2 existing schema
  -> migrate phase 1 storage/indexing additions
  -> repository APIs read/write typed structs
  -> functional cases verify behavior through public API
```

## 5. Constraints

- No new dependencies.
- No direct access to real user HOME, real provider credentials, real transcript trees, or real git commands.
- SQL writes that mutate multiple related rows must use transactions when needed.
- Stored JSON payloads are safe-field payloads only.
- Higher schema version must fail closed.

## 6. Verification Strategy

| Layer | Command/report |
|-------|----------------|
| Functional cases | `cargo test -p homie-storage <case-test-name> -- --nocapture` |
| Focused crate | `cargo check -p homie-storage`, `cargo test -p homie-storage` |
| Format/diff/security | `cargo fmt --all -- --check`, `git diff --check`, `.githooks/pre-commit` |
| Evidence | `docs/verification/diri-storage-indexing/functional-case-results.md`, `release-readiness-report.md` |
| Review | `code-review-round-1.md`, `code-review-round-2.md` |

## 7. Dependency Impact

This change unblocks later L1/L2 lanes that need a stable storage contract:

- `runtime-supervisor`
- `session-context-store`
- `desktop-shell`
- `remote-node-handoff`
- `llm-proxy`

No dependent lane may assume scanner/runtime/UI behavior is implemented by this change.

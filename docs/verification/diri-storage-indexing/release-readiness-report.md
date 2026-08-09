# Release Readiness Report: Diri Storage/Indexing Parity Phase 1

```yaml
change_id: diri-storage-indexing
beads: homie-q7n
risk_tier: Tier 3 high-stakes SQLite migration/repository
status: pass
executed_at: 2026-08-07
```

## 1. Source Documents

- PRD: `prd-spec/features/diri-storage-indexing/2026-08-07-diri-storage-indexing-design.md`
- Component spec: `specs/storage-indexing/README.md`
- OpenSpec: `openspec/changes/diri-storage-indexing/`
- Functional cases: `docs/verification/diri-storage-indexing/functional-cases.md`
- Beads: `homie-q7n`

## 2. Delivered Scope

- Added phase 1 schema/API inventory for M05/M06/M07/M17/M19 to `specs/storage-indexing/README.md`.
- Added `homie-storage` schema version 3 with session metadata, history, worktree, usage scan/rollup and usage source-event fields.
- Added repository/query APIs for schema inventory, history, project/worktree, session core metadata and usage ledger aggregation.
- Added storage tests for schema inventory, history, worktree, session metadata, usage aggregation and usage validation.

Out of scope and not implemented:

- UI/runtime/git shell/history scanner/usage parser/LLM proxy/remote sync.
- Makefile or scripts changes.

## 3. Gate Results

| Gate | Command | Exit | Result | Notes |
|------|---------|------|--------|-------|
| Spec gate | Review PRD/spec/OpenSpec/evidence | 0 | pass | `spec-review-report.md`, OpenSpec plan/tasks/alignment all present. |
| Functional cases | `cargo test -p homie-storage storage_schema_inventory_covers_diri_phase_one -- --nocapture` | 0 | pass | FC-STOR-001. |
| Functional cases | `cargo test -p homie-storage settings_preferences_round_trip_through_preferences_table -- --nocapture` | 0 | pass | FC-STOR-002. |
| Functional cases | `cargo test -p homie-storage history_repository_upserts_lists_and_tracks_entries -- --nocapture` | 0 | pass | FC-STOR-003. |
| Functional cases | `cargo test -p homie-storage project_worktree_repository_enforces_identity_and_lists_by_project -- --nocapture` | 0 | pass | FC-STOR-004. |
| Functional cases | `cargo test -p homie-storage session_core_metadata_round_trips_diri_record_subset -- --nocapture` | 0 | pass | FC-STOR-005. |
| Functional cases | `cargo test -p homie-storage usage_repository_deduplicates_source_events_and_queries_totals -- --nocapture` | 0 | pass | FC-STOR-006. |
| Functional cases | `cargo test -p homie-storage usage_repository_handles_empty_queries_and_rejects_negative_tokens -- --nocapture` | 0 | pass | FC-STOR-006 boundary regression. |
| Format | `cargo fmt -p homie-storage -- --check` | 0 | pass | Focused storage package format. |
| Format | `cargo fmt --all -- --check` | 0 | pass | Workspace format check. |
| Build | `cargo check -p homie-storage` | 0 | pass | Focused storage package check. |
| Lint | `cargo clippy -p homie-storage --all-targets -- -D warnings` | 0 | pass | Focused storage package clippy. |
| Integration | `cargo test -p homie-storage` | 0 | pass | 16 storage integration tests plus doc tests. |
| Diff safety | `git diff --check -- prd-spec/features/diri-storage-indexing openspec/changes/diri-storage-indexing docs/verification/diri-storage-indexing specs/storage-indexing/README.md crates/homie-storage` | 0 | pass | Lane-scoped whitespace check. |
| Security hook | `.githooks/pre-commit` | 0 | pass | Local hook returned success. |
| Code review | Two local review rounds | 0 | pass | `code-review-round-1.md`, `code-review-round-2.md`. |

## 4. New Dependencies

- No new third-party dependency was introduced by this implementation.
- `crates/homie-storage/Cargo.toml` already had a local diff adding `serde_json.workspace = true` in the dirty worktree; the current storage API uses `serde_json::Value` for safe JSON payloads and would require that dependency.

## 5. Dirty Worktree Note

The repository had many unrelated modified and untracked files before and during this lane. This report only claims the `diri-storage-indexing` lane files and `crates/homie-storage/**` behavior. No Makefile or scripts were edited for this task.

## 6. Residual Risk

- Existing user database compatibility beyond forward-only v1/v2 -> v3 migration was not promised by repo policy and was not expanded.
- Usage pricing is stored as decimal strings; exact billing semantics remain owned by the LLM/usage lane.
- Downstream lanes still need to implement scanners/parsers/runtime/UI against these storage APIs.

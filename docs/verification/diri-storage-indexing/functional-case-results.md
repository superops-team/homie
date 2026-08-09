# Functional Case Results: Diri Storage/Indexing Parity Phase 1

```yaml
change_id: diri-storage-indexing
beads: homie-q7n
status: pass
executed_at: 2026-08-07
```

## Results

| Case | Command | Exit | Result | Notes |
|------|---------|------|--------|-------|
| FC-STOR-001 | `cargo test -p homie-storage storage_schema_inventory_covers_diri_phase_one -- --nocapture` | 0 | pass | Required tables, columns, unique indexes and phase 1 usage/session/history/worktree fields verified through SQLite metadata. |
| FC-STOR-002 | `cargo test -p homie-storage settings_preferences_round_trip_through_preferences_table -- --nocapture` | 0 | pass | Settings defaults and JSON round-trip verified through existing storage bootstrap test. |
| FC-STOR-003 | `cargo test -p homie-storage history_repository_upserts_lists_and_tracks_entries -- --nocapture` | 0 | pass | History upsert dedupes `(agent_kind, external_id)`, orders by recent activity and marks tracked session. |
| FC-STOR-004 | `cargo test -p homie-storage project_worktree_repository_enforces_identity_and_lists_by_project -- --nocapture` | 0 | pass | Project root, worktree path and project/branch uniqueness verified; worktree flags and session link round-trip. |
| FC-STOR-005 | `cargo test -p homie-storage session_core_metadata_round_trips_diri_record_subset -- --nocapture` | 0 | pass | Diri SessionRecord phase 1 fields round-trip through session metadata API. |
| FC-STOR-006 | `cargo test -p homie-storage usage_repository_deduplicates_source_events_and_queries_totals -- --nocapture` | 0 | pass | Usage source-event dedupe and token/cache/cost aggregate query verified. |
| FC-STOR-006b | `cargo test -p homie-storage usage_repository_handles_empty_queries_and_rejects_negative_tokens -- --nocapture` | 0 | pass | Empty usage query returns zero totals; negative token and missing source-event id are rejected before insert. |

## Focused Suite

| Command | Exit | Result |
|---------|------|--------|
| `cargo test -p homie-storage` | 0 | pass; 16 tests across storage integration suites plus doc tests |
| `cargo check -p homie-storage` | 0 | pass |
| `cargo clippy -p homie-storage --all-targets -- -D warnings` | 0 | pass |
| `cargo fmt -p homie-storage -- --check` | 0 | pass |

## Notes

- Initial RED run failed because `HistoryEntryUpsert`, `ProjectUpsert`, `WorktreeUpsert`, `SessionCoreMetadataUpdate`, `RecordUsage`, `UsageQuery` and repository methods did not exist yet. This confirmed the test-first gap.
- Full `cargo test -p homie-storage` initially failed only on old schema version assertions expecting `2`; assertions were updated to `3` after the v3 migration landed.

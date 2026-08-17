# Engine Registry/Session Persistence Split Plan

## 1. Scope

Implement the P0 child refactor for `architecture-audit-governance-2026-08` finding F7.
Separate persistence from live session coordination in `homie/crates/homie-engine/src/registry.rs`
(1,790 lines).

## 2. In Scope

- Extract `PersistedState` + projection fold functions into `registry/persisted.rs`.
- Extract `PersistenceStore` + `JsonEnvelopeStore` + `SplitJsonStore` into `registry/store.rs`.
- Extract migration (`SplitMigrationReport` + `migrate_envelope_to_split`) into `registry/migrate.rs`.
- Extract flush timing + `spawn_persist_flusher` into `registry/flusher.rs`.
- Keep `Registry` as live session coordinator with a thin persistence facade.
- Add focused unit tests for persistence/migration/projection.

## 3. Out Of Scope

- Disk schema / file path / atomic write / migration semantic changes.
- `Registry` public API semantic changes.
- `session.rs` state machine rework.
- New persistence backends.

## 4. Design

Follow `prd-spec/refactors/engine-registry-session-split/`. Four slices:

1. S1 `registry/persisted.rs` — PersistedState + fold projections.
2. S2 `registry/store.rs` — PersistenceStore trait + two impls.
3. S3 `registry/migrate.rs` — migration logic.
4. S4 `registry/flusher.rs` — flush timing + spawn_persist_flusher.

Each slice keeps `cargo test -p homie-engine` green.

## 5. Evidence

- `docs/verification/engine-registry-session-split/spec-review-report.md`
- `docs/verification/engine-registry-session-split/functional-cases.md`
- `docs/verification/engine-registry-session-split/functional-verification-report.md`
- `docs/verification/engine-registry-session-split/code-review-round-*.md`
- `docs/verification/engine-registry-session-split/release-readiness-report.md`

## 6. Risks

| Risk | Control |
|---|---|
| Persistence semantic regresses | envelope→split migration result equality tests |
| Hidden live-session dependency in persistence modules | Static `rg` gate: no session map access in `persisted`/`store`/`migrate`/`flusher` |
| Scope expands into session rewrite | Only move responsibilities, no new behavior |

# Engine Registry/Session Persistence Split Tasks

## T1: Spec and functional cases

- Deliverables: `docs/verification/engine-registry-session-split/functional-cases.md`,
  `openspec/changes/engine-registry-session-split/*`.
- Acceptance: first slice is PersistedState extraction; session rewrite out of scope.

## T2: Extract persisted state (S1)

- Deliverables: `homie/crates/homie-engine/src/registry/persisted.rs`, `registry/mod.rs`.
- Acceptance: `PersistedState` + `fold_session_view`/`repair_persisted_agent_title`/
  `fold_session_status` move; no live-session map access; fold tests added.

## T3: Extract store backends (S2)

- Deliverables: `homie/crates/homie-engine/src/registry/store.rs`.
- Acceptance: `PersistenceStore` + `JsonEnvelopeStore` + `SplitJsonStore` move; atomic write tests added.

## T4: Extract migration (S3)

- Deliverables: `homie/crates/homie-engine/src/registry/migrate.rs`.
- Acceptance: `SplitMigrationReport` + `migrate_envelope_to_split` move; migration equality tests added.

## T5: Extract flusher (S4)

- Deliverables: `homie/crates/homie-engine/src/registry/flusher.rs`, `registry.rs`.
- Acceptance: `registry.rs` < 800 lines; `Registry` keeps live coordination + thin persistence facade.

## T6: Final verification and review

- Deliverables: verification logs/reports under `docs/verification/engine-registry-session-split/`.
- Acceptance: all cases pass; `cargo test -p homie-engine` green; release readiness report exists.

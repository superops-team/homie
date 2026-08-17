# Engine Control Wire/Runtime Split Tasks

## T1: Spec and functional cases

- Deliverables: `docs/verification/engine-control-wire-runtime-split/functional-cases.md`,
  `openspec/changes/engine-control-wire-runtime-split/*`.
- Acceptance: first slice is wire codec only; session rewrite out of scope.

## T2: Extract wire codec (S1)

- Deliverables: `homie/crates/homie-engine/src/control/wire.rs`, `control/mod.rs`.
- Acceptance: `write_message`/`decode`/`encode`/read helper + param parse move; no socket/registry
  dependency; round-trip tests added.

## T3: Extract projections (S2)

- Deliverables: `homie/crates/homie-engine/src/control/codec.rs`.
- Acceptance: `history_entry_to_wire`/`worktree_to_wire` move; projection invariant tests added.

## T4: Extract runtime lifecycle (S3)

- Deliverables: `homie/crates/homie-engine/src/control/runtime.rs`.
- Acceptance: bind loop / subscription / connection guard / idle shutdown / remote restore move;
  behavior unchanged.

## T5: Sink handlers (S4)

- Deliverables: `registry.rs`, `session.rs`, `remote/manager.rs`, `control.rs`.
- Acceptance: `control.rs` < 800 lines; `ControlServer` keeps only routing table; handlers live in
  owned modules.

## T6: Final verification and review

- Deliverables: verification logs/reports under `docs/verification/engine-control-wire-runtime-split/`.
- Acceptance: all cases pass; `cargo test -p homie-engine` green; release readiness report exists.

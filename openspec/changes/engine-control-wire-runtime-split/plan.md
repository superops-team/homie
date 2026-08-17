# Engine Control Wire/Runtime Split Plan

## 1. Scope

Implement the P0 child refactor for `architecture-audit-governance-2026-08` finding F2.
Split `homie/crates/homie-engine/src/control.rs` (3,802 lines) into wire / codec / runtime
layers and sink business handlers into registry/session/remote.

## 2. In Scope

- Extract wire codec (JSON encode/decode, `ControlMessage` read/write) into `control/wire.rs`.
- Extract proto↔domain projections into `control/codec.rs`.
- Extract runtime lifecycle (bind loop, subscription handles, connection guard, idle shutdown,
  remote restore) into `control/runtime.rs`.
- Sink `session_*`/`host_*`/`worktree_*` business handler logic into `registry.rs`/`session.rs`/
  `remote/manager.rs`; `ControlServer` keeps a method routing table.
- Add focused unit tests for codec and projection pure functions.

## 3. Out Of Scope

- Any wire shape / method name / JSON semantic change.
- `homie-proto/src/control.rs` protocol definitions.
- Session state machine rework (`engine-registry-session-split` covers registry persistence).
- App-side changes.

## 4. Design

Follow `prd-spec/refactors/engine-control-wire-runtime-split/`. Four slices:

1. S1 `control/wire.rs` — pure codec + param parse + return assembly.
2. S2 `control/codec.rs` — proto↔domain projections.
3. S3 `control/runtime.rs` — bind loop + lifecycle.
4. S4 handler sinking — `ControlServer` reduced to routing table.

Each slice keeps `cargo test -p homie-engine` green.

## 5. Evidence

- `docs/verification/engine-control-wire-runtime-split/spec-review-report.md`
- `docs/verification/engine-control-wire-runtime-split/functional-cases.md`
- `docs/verification/engine-control-wire-runtime-split/functional-verification-report.md`
- `docs/verification/engine-control-wire-runtime-split/code-review-round-*.md`
- `docs/verification/engine-control-wire-runtime-split/release-readiness-report.md`

## 6. Risks

| Risk | Control |
|---|---|
| Wire shape regresses | Golden method/param/return assertions; protocol unchanged |
| Hidden daemon dependency in extracted modules | Static `rg` gate: no socket/registry imports in `wire.rs`/`codec.rs` |
| Scope expands into session rewrite | Only move responsibilities, no new behavior |

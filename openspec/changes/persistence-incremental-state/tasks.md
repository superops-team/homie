# Persistence Incremental State Tasks

## T1: Spec and functional cases

- Deliverables:
  - `docs/verification/persistence-incremental-state/functional-cases.md`
  - `openspec/changes/persistence-incremental-state/*`
- Acceptance:
  - first slice remains non-default and non-SQLite;
  - all tasks map to cases.
- Verification Cases: FC-01, FC-02

## T2: Add store trait and envelope store

- Deliverables:
  - `homie/crates/homie-engine/src/registry.rs`
- Acceptance:
  - trait supports project/session load/save/delete/flush;
  - envelope store preserves current state file behavior.
- Verification Cases: FC-03, FC-04

## T3: Add split JSON store

- Deliverables:
  - `homie/crates/homie-engine/src/registry.rs`
- Acceptance:
  - split store reads/writes `projects.json` and individual session JSON files;
  - session writes are atomic per file;
  - corrupt session files are quarantined without losing other records.
- Verification Cases: FC-04, FC-05

## T4: Add envelope-to-split migration

- Deliverables:
  - `homie/crates/homie-engine/src/registry.rs`
- Acceptance:
  - dry-run writes nothing;
  - apply writes split store files and backup;
  - source envelope remains present.
- Verification Cases: FC-03, FC-04

## T5: Final verification and review

- Deliverables:
  - verification logs and reports under `docs/verification/persistence-incremental-state/`
- Acceptance:
  - FC-01 through FC-06 pass;
  - code review reports exist;
  - release readiness report exists.
- Verification Cases: FC-06

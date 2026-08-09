# Diri Storage Core Durable Facts Tasks

> Change ID: `diri-storage-core-facts`
> Beads: `homie-t3u.2`
> Master task: `T-103`
> Ordering: `SPEC -> RED -> GREEN -> REFACTOR -> EVIDENCE`

## 0. SPEC Gate

- [x] **S103-SPEC-01** Read the master PRD/OpenSpec, parity lock, storage/indexing
  PRD/OpenSpec/evidence, relevant component specs, all `homie-storage` code/tests, and app/runtime
  call boundaries.
  - Owner: `S103-spec`
  - Files: read-only repository scope
  - Acceptance: baseline facts in PRD match schema version 3 and green test inventory.

- [x] **S103-SPEC-02** Write the Chinese PRD and complete OpenSpec proposal/design/capability
  specs/plan/tasks/alignment/delegation.
  - Owner: `S103-spec`
  - Files: only `prd-spec/features/diri-storage-core-facts/**` and
    `openspec/changes/diri-storage-core-facts/**`
  - Acceptance: every FR maps to requirement, executable task, and verification.

- [x] **S103-SPEC-03** Update the two allowed long-lived component specs.
  - Owner: `S103-spec`
  - Files: `specs/storage-indexing/README.md`, `specs/session-context-store/README.md`
  - Acceptance: v3 is recorded as current baseline; v4 is an additive T-103 contract.

- [x] **S103-SPEC-04** Run 16-dimension review, OpenSpec status/strict validation, consistency,
  allowed-path, and diff checks.
  - Owner: `S103-spec`
  - Files: documentation only
  - Acceptance: no blocking review finding; status and strict validation pass; no product file
    changed.

## 1. RED

- [ ] **S103-RED-01** Reproduce the immutable storage baseline before adding new tests.
  - Owner: `S103-storage-test`
  - Dependencies: `S103-SPEC-04`
  - Files: no source edit; evidence only
  - Command: `cargo test -p homie-storage --tests`
  - Acceptance: all current binaries pass; actual counts and schema version 3 are recorded.

- [ ] **S103-RED-02** Add failing ordered-v4 migration tests.
  - Owner: `S103-storage-test`
  - Dependencies: `S103-RED-01`
  - Files: `crates/homie-storage/tests/ordered_v4_migration.rs`, test fixture/support only
  - Cases: empty `[1,2,3,4]`, real v3 fixture `[4]`, repeat `[]`, injected v4 rollback,
    schema-too-new.
  - Acceptance: only missing v4 contracts fail; all pre-existing storage tests remain green.

- [ ] **S103-RED-03** Add failing effective-config freeze/readback tests.
  - Owner: `S103-storage-test`
  - Dependencies: `S103-RED-02`
  - Files: `crates/homie-storage/tests/effective_config_facts.rs`
  - Cases: atomic session+parent+config bind, immutable readback after profile mutation, duplicate
    freeze conflict, rollback on invalid reference/JSON/hash, no secret material.
  - Acceptance: failures identify missing typed freeze/readback behavior, not the existing table.

- [ ] **S103-RED-04** Add failing runtime recovery fact tests.
  - Owner: `S103-storage-test`
  - Dependencies: `S103-GREEN-02` repository milestone published
  - Files: `crates/homie-storage/tests/runtime_recovery_facts.rs`
  - Cases: atomic checkpoint fields, bounded deterministic candidates, invalid offsets,
    reopen/readback, old-row preservation on failure.
  - Acceptance: tests label PID/status as hints and never assert a row proves liveness.

- [ ] **S103-RED-05** Add failing lineage/remote/update foundation tests.
  - Owner: `S103-storage-test`
  - Dependencies: `S103-GREEN-03`
  - Files: `crates/homie-storage/tests/durable_metadata_foundation.rs`
  - Cases: parent remains canonical, lineage operation id idempotency, handoff operation/lease CAS,
    update receipt legal/illegal phase transitions, secret-field scan.
  - Acceptance: tests cover repository metadata only; no network/install/workflow assertion.

- [ ] **S103-RED-06** Add failing service-owned settings/health/usage/effective/recovery contract
  fixtures.
  - Owner: `S103-proto-test`
  - Dependencies: `S103-SPEC-04`, T-102 shared-file release
  - Files: new/focused `homie-proto` tests only; no production proto file
  - Cases: revision conflict DTO, bounded query, versioned snapshots, safe errors, serde
    round-trip, unknown version/phase rejection.
  - Acceptance: tests compile-fail or behavior-fail solely because frozen shared contracts are
    not implemented.

- [ ] **S103-RED-07** Add failing runtime/client service tests.
  - Owner: `S103-runtime-test`
  - Dependencies: `S103-RED-06`, T-102 runtime-file release
  - Files: focused runtime/client tests only
  - Cases: method capability discovery, settings CAS, health/usage query, config readback, recovery
    summary, storage failure safe errors.
  - Acceptance: no implementation file is modified; direct unknown methods remain
    `method_not_found`.

- [ ] **S103-RED-08** Add failing app direct-storage removal tests/checks.
  - Owner: `S103-app-test`
  - Dependencies: `S103-RED-07`
  - Files: focused app tests/check script only
  - Cases: settings load/save commands use bridge/client, no app normal storage dependency, no
    `open_ready_storage`/`open_or_create`.
  - Acceptance: RED explicitly proves the current direct dependency and source path exist.

- [ ] **S103-RED-09** Add failing CLI direct-storage removal tests/checks.
  - Owner: `S103-cli-test`
  - Dependencies: `S103-RED-07`
  - Files: focused CLI tests/check script only
  - Cases: doctor calls storage health service, usage summary calls service, no CLI normal storage
    dependency or direct open.
  - Acceptance: RED explicitly proves the current direct dependency/path exists.

## 2. GREEN

- [ ] **S103-GREEN-01** Append schema version 4 and implement ordered migration DDL/backfill/indexes.
  - Owner: `S103-storage-impl` (exclusive)
  - Dependencies: `S103-RED-02`
  - Files: `crates/homie-storage/src/lib.rs` only
  - Scope: preferences revision, effective-config additions, runtime recovery, lineage audit,
    handoff extensions, update receipts.
  - Acceptance: `S103-RED-02` passes; no v1-v3 migration behavior is rewritten.

- [ ] **S103-GREEN-02** Implement effective-config freeze/readback repositories.
  - Owner: `S103-storage-impl` (same exclusive owner, serial after GREEN-01)
  - Dependencies: `S103-RED-03`, `S103-GREEN-01`,
    T-102 G3 resolved launch/effective-config contract handoff
  - Files: `crates/homie-storage/src/lib.rs` only
  - Scope: validated bounded snapshots, deterministic hash, atomic session/parent/config bind,
    immutable by-session readback.
  - Acceptance: `S103-RED-03` passes, current session tests remain green, and the exact repository
    API/type handoff required by T-102 G5 is recorded without T-102 editing storage.

- [ ] **S103-GREEN-03** Implement runtime recovery fact repositories.
  - Owner: `S103-storage-impl` (same exclusive owner, serial after GREEN-02)
  - Dependencies: `S103-RED-04`, `S103-GREEN-02`
  - Files: `crates/homie-storage/src/lib.rs` only
  - Scope: atomic upsert/assessment, by-session read, bounded candidate list, validation, storage
    owned flush/checkpoint.
  - Acceptance: `S103-RED-04` passes; no output/grid/blob enters SQLite.

- [ ] **S103-GREEN-04** Implement lineage/remote/update metadata repositories.
  - Owner: `S103-storage-impl` (same exclusive owner, serial after GREEN-03)
  - Dependencies: `S103-RED-05`, `S103-GREEN-03`
  - Files: `crates/homie-storage/src/lib.rs` only
  - Scope: lineage append/read; host/account/handoff typed APIs and CAS; update receipt create/read/
    CAS; stable conflicts.
  - Acceptance: `S103-RED-05` passes; no remote/update workflow is added.

- [ ] **S103-GREEN-05** Implement frozen proto methods and DTOs.
  - Owner: `S103-proto-integration` (exclusive)
  - Dependencies: `S103-RED-06`, `S103-GREEN-04`, recorded T-102 shared-file handoff
  - Files: exact `homie-proto` method/model/test files recorded at claim time
  - Scope: storage health, settings get/update, usage summary, effective config, recovery summary.
  - Acceptance: proto RED fixtures pass; no remote/updater workflow method is advertised.

- [ ] **S103-GREEN-06** Implement runtime owning services and dispatcher handlers.
  - Owner: `S103-runtime-integration` (exclusive)
  - Dependencies: `S103-GREEN-05`, recorded T-102 runtime-file handoff
  - Files: `crates/homie-runtime/src/runtime_actor.rs`, `dispatcher.rs`, exact focused files/tests
  - Scope: storage is actor/service-owned; handlers call typed repositories; recovery invokes
    T-102 verifier before publishing running; shutdown uses storage-owned flush.
  - Acceptance: runtime RED tests pass; capability discovery includes only implemented handlers.

- [ ] **S103-GREEN-07** Implement typed client methods.
  - Owner: `S103-client-integration` (exclusive)
  - Dependencies: `S103-GREEN-06`
  - Files: `crates/homie-client/src/client.rs`, focused client tests
  - Scope: typed methods and stable error mapping for all six frozen methods.
  - Acceptance: client/runtime integration tests pass; caller never sees repository/SQL types.

- [ ] **S103-GREEN-08** Replace app settings storage access with owning service/client.
  - Owner: `S103-app-integration` (exclusive)
  - Dependencies: `S103-GREEN-07`, `S103-RED-08`
  - Files: `crates/homie-app/Cargo.toml`, `src/main.rs`, `src/runtime_bridge.rs`, focused tests
  - Scope: bridge load/save settings asynchronously; authoritative revisioned response; remove
    storage dependency/import/open helper and all fallback.
  - Acceptance: app RED checks pass and settings failure does not display false success.

- [ ] **S103-GREEN-09** Replace CLI doctor/usage storage access with owning service/client.
  - Owner: `S103-cli-integration` (exclusive)
  - Dependencies: `S103-GREEN-07`, `S103-RED-09`
  - Files: `crates/homie-cli/Cargo.toml`, CLI source/tests
  - Scope: doctor uses `storage.health`; usage uses `usage.summary`; remove direct dependency/open.
  - Acceptance: CLI RED checks pass; storage-unavailable is safe and has no direct fallback.

- [ ] **S103-GREEN-10** Run the focused GREEN matrix.
  - Owner: `S103-verification`
  - Dependencies: `S103-GREEN-08`, `S103-GREEN-09`
  - Files: evidence only
  - Commands: focused storage, proto, runtime, client, app, and CLI tests.
  - Acceptance: every RED test is GREEN; pre-existing storage tests remain GREEN.

## 3. REFACTOR

- [ ] **S103-REFACTOR-01** Remove production raw connection/storage exposure in touched runtime
  paths.
  - Owner: `S103-runtime-integration`
  - Dependencies: `S103-GREEN-10`
  - Files: storage/runtime touched implementation and tests; storage file edits remain assigned to
    `S103-storage-impl`
  - Scope: replace `RuntimeSupervisor::storage()` and direct connection checkpoint use where
    touched with owned domain/flush APIs.
  - Acceptance: no app/CLI/runtime consumer uses `Storage::connection()` as a domain API.

- [ ] **S103-REFACTOR-02** Consolidate validation, errors, and phase-transition tables.
  - Owner: `S103-storage-impl` for `lib.rs`; relevant integration owner elsewhere
  - Dependencies: `S103-REFACTOR-01`
  - Files: only T-103 touched files
  - Scope: deduplicate snapshot validation/CAS helpers when duplication is demonstrated; do not
    introduce a framework.
  - Acceptance: all focused tests remain green; diff does not include unrelated cleanup.

- [ ] **S103-REFACTOR-03** Run formatting, lint, dependency, direct-open, and sensitive-field
  scans.
  - Owner: `S103-verification`
  - Dependencies: `S103-REFACTOR-02`
  - Files: formatting may touch only T-103 product files; evidence records outputs
  - Acceptance: format/check/clippy pass; app/CLI cargo trees exclude storage; secret scan is zero.

## 4. EVIDENCE

- [ ] **S103-EVIDENCE-01** Record complete migration and repository evidence.
  - Owner: `S103-verification`
  - Dependencies: `S103-REFACTOR-03`
  - Output: `docs/verification/diri-storage-core-facts/` in implementation session
  - Evidence: baseline/final counts, v3 fixture SHA256, v4 applied list, rollback proof, repository
    idempotency/CAS, security scan.
  - Acceptance: pass/fail/block states are explicit; no fabricated result.

- [ ] **S103-EVIDENCE-02** Record daemon replacement and effective-config recovery E2E.
  - Owner: `S103-e2e`
  - Dependencies: `S103-EVIDENCE-01`
  - Files: create `crates/homie-cli/tests/diri_storage_recovery_e2e.rs` and
    `docs/verification/diri-storage-core-facts/runtime-recovery-e2e.md`
  - Evidence: create session with frozen config, replace daemon, verify holder/output, read back
    same config hash, reject row-only fake running.
  - Acceptance: cross-process proof passes or is recorded blocked with exact reason.

- [ ] **S103-EVIDENCE-03** Record app/CLI service-boundary E2E.
  - Owner: `S103-e2e`
  - Dependencies: `S103-EVIDENCE-01`
  - Evidence: app settings read/write and revision conflict; CLI doctor/usage; dependency and source
    scans.
  - Acceptance: no direct-storage path/fallback; authoritative daemon responses observed.

- [ ] **S103-EVIDENCE-04** Run two-round code review and security review.
  - Owner: `S103-review`
  - Dependencies: `S103-EVIDENCE-02`, `S103-EVIDENCE-03`
  - Evidence: syntax/runtime/API/logic first pass; boundary/resource/semantic/security second pass.
  - Acceptance: P0/P1 resolved; residual risk documented.

- [ ] **S103-EVIDENCE-05** Run workspace quality and OpenSpec alignment gates.
  - Owner: `S103-verification`
  - Dependencies: `S103-EVIDENCE-04`
  - Commands: fmt/check/clippy/tests, OpenSpec strict/status, parity consistency, secret hook,
    `git diff --check`.
  - Acceptance: actual statuses recorded; blocked unrelated gates are not reported pass.

- [ ] **S103-EVIDENCE-06** Publish release readiness without overstating parity.
  - Owner: `S103-release`
  - Dependencies: `S103-EVIDENCE-05`
  - Evidence: requirement-to-test matrix, changed-file inventory, SHA256, known limitations.
  - Acceptance: foundation completion does not independently mark UI/remote/usage/update/package/
    performance parity rows implemented.

- [ ] **S103-EVIDENCE-07** Update or close Bead only after evidence matches delivery.
  - Owner: `S103-release`
  - Dependencies: `S103-EVIDENCE-06`
  - Command: `bd close homie-t3u.2 --reason "Implemented and verified. See ..."` only when ready.
  - Acceptance: Bead remains open/blocked if any required implementation or evidence is missing.

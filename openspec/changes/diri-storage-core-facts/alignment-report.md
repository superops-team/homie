# Diri Storage Core Durable Facts Alignment Report

> Change ID: `diri-storage-core-facts`
> PRD date: 2026-08-09
> Bead: `homie-t3u.2`
> Master task: `T-103`

## 1. Inputs Reviewed

- `AGENTS.md`
- master PRD/OpenSpec for `diri-7ba3407-parity-rebaseline`
- `docs/research/diri-parity-lock.md`
- `docs/architecture/project-layout.md`
- `docs/development/standards.md`
- `docs/development/quality-gates.md`
- `docs/research/rust-package-selection.md`
- storage/indexing PRD/OpenSpec and verification evidence
- relevant storage, context, runtime, adapter, transport, desktop, remote, updater, MCP, and proxy
  component specs
- all `crates/homie-storage` source/tests
- app, runtime, client, CLI, and proto dependency/call boundaries
- Diri `7ba3407` session, node/checkpoint, and updater reference models

## 2. Baseline Truth Alignment

| Fact | PRD | Design/spec | Task/evidence |
|------|-----|-------------|---------------|
| schema version is 3 | §1.1, §2 | design Current Baseline; ordered spec baseline requirement | SPEC-01, RED-01 |
| v1-v3 migration already ordered/transactional | §1.1, FR-01 | ordered spec baseline requirement | RED-01; existing tests preserved |
| effective config table already exists | §1.1, §2 | design baseline/Decision 4 | RED-03 targets API, not table |
| session/history/worktree/usage repositories exist | §1.1, FR-01 | proposal Why; ordered baseline requirement | RED-01; no duplicate implementation |
| app direct-storage is current violation | §1.1, FR-03 | service-owned repository requirement | RED-08, GREEN-08 |
| CLI direct-storage remains from Wave 1A | §1.1, FR-04 | Decision 9 | RED-09, GREEN-09 |
| events are already persisted in `runtime/events.jsonl` | FR-06, §5.2 | Decision 5 | RED-04, GREEN-03/06 |
| durable rows do not prove live process | FR-06 | recovery live-proof requirement | RED-04, EVIDENCE-02 |

No artifact describes the above existing capabilities as missing.

## 3. PRD-to-OpenSpec-to-Task Mapping

| PRD requirement | OpenSpec requirements | RED | GREEN/REFACTOR | Verification |
|-----------------|-----------------------|-----|----------------|--------------|
| FR-01 baseline | ordered: Preserve schema-v3 baseline | RED-01 | all storage GREEN preserves tests | baseline/final counts |
| FR-02 v4 migration | ordered: Ordered v4; idempotency/fail-closed | RED-02 | GREEN-01 | EVIDENCE-01 |
| FR-03 ownership | service: Daemon-owned production storage | RED-08/09 | GREEN-06/08/09; REFACTOR-01 | dependency/source scans |
| FR-04 settings/health/usage | service: Revisioned settings; health/usage; discovery | RED-06..09 | GREEN-05..09 | EVIDENCE-03 |
| FR-05 effective config | effective: immutable; atomic bind; safe readback | RED-03/06/07 | GREEN-02/05..07 | EVIDENCE-01/02 |
| FR-06 recovery | recovery: typed facts; no live proof; separation; restart | RED-04/07 | GREEN-03/06/07 | EVIDENCE-02 |
| FR-07 lineage | metadata: single-source lineage and safe audit | RED-05 | GREEN-01/04 | EVIDENCE-01 |
| FR-08 remote metadata | metadata: remote operation foundation | RED-05 | GREEN-01/04 | EVIDENCE-01 |
| FR-09 update receipt | metadata: durable update receipt | RED-05 | GREEN-01/04 | EVIDENCE-01 |
| FR-10 security | all safe snapshot/metadata scenarios | RED-03..06 | GREEN-02..07; REFACTOR-03 | secret scans/review |
| FR-11 contract/ownership | service discovery + design Decision 8 | RED-06/07 | GREEN-05..07 | delegation gate |
| FR-12 truthful status | metadata: foundation does not close parity | all RED | all GREEN | EVIDENCE-05/06 |

Every PRD requirement has at least one normative requirement, one executable task, and one
verification outcome.

## 4. Master Alignment

| Master contract | T-103 treatment |
|-----------------|-----------------|
| FR-05 migration/repository/recovery foundation | fully mapped to five capability specs |
| UI must not depend on storage | explicit removal acceptance, not merely "no new dependency" |
| T-102 owns holder/process/agent lifecycle | T-103 treats its output as input and blocks shared-file integration |
| durable shared models need coordination | contract-freeze and exclusive proto/runtime owners |
| SQLite output is metadata-only | terminal bytes/grid/blob remain outside SQLite |
| parity must be evidence-based | downstream UI/remote/usage/update rows remain partial |

## 5. Component-Spec Alignment

| Durable contract | Long-lived owner | Change action |
|------------------|------------------|---------------|
| ordered migration/schema/index/repository | storage/indexing | update allowed spec |
| effective config and recovery facts | storage/indexing | update allowed spec |
| parent/lineage/provenance | session-context-store | update allowed spec |
| live holder verification | runtime-supervisor | no edit; T-102 owner preserved |
| resolved launch config | agent-adapter-contract | no edit; T-102 computes, T-103 persists |
| app async boundary | runtime-client-transport/desktop-shell | no edit; existing contract reused |
| handoff workflow | remote-node-handoff | no edit; metadata foundation only |
| updater workflow | packaging-updater | no edit; receipt foundation only |

## 6. Contract-Freeze Alignment

The PRD/design/plan/delegation artifacts agree on exactly six T-103 methods:

1. `storage.health`
2. `settings.get`
3. `settings.update`
4. `usage.summary`
5. `session.effective_config`
6. `runtime.recovery.summary`

They also agree that existing parent/children methods are reused and that remote/updater workflow
methods are not introduced by T-103. Proto/runtime product edits are blocked until T-102 releases
shared files.

The cross-spec review found and resolved one blocking ownership conflict: the initial T-102 draft
also assigned effective-config persistence in `homie-storage/src/lib.rs`. The final contracts now
make T-103 the only storage/schema/repository owner and use this acyclic handoff:

```text
T-102 G3 contract -> T-103 S103-GREEN-02 repository -> T-102 G5
  -> T-102 shared-file release -> T-103 shared integration
```

## 7. 16-Dimension Specification Review

| Dimension | Result | Evidence / resolution |
|-----------|--------|-----------------------|
| 1. Logic coherence | pass | baseline -> RED -> v4/service -> evidence sequence is explicit |
| 2. Content completeness | pass | proposal/design/5 specs/plan/tasks/alignment/delegation present |
| 3. Ambiguity | pass | six methods, ownership, hints-vs-live, one-parent-source are named |
| 4. Model semantic risk | pass | existing v3 facts are not labeled missing; foundation is bounded |
| 5. SDD/TDD fit | pass | every FR maps to scenario and RED/GREEN task |
| 6. Minimal implementation | pass | additive v4, no storage split/framework/compatibility path |
| 7. Backward compatibility | pass | historical migrations/data preserved; obsolete direct path deleted |
| 8. Existing behavior impact | pass | current storage suites are mandatory baseline/regression gates |
| 9. Risk prediction | pass | T-102 conflict, stale PID, secrets, doctor failure are addressed |
| 10. Feasibility | pass | existing runtime owner/client boundary and v3 tables are reused |
| 11. Task decomposition | pass | each task has owner, dependency, file scope, acceptance |
| 12. Scheduling | pass | storage lane proceeds; shared integration waits for T-102 handoff |
| 13. Extensibility | pass | versioned snapshots and durable receipts support later owners |
| 14. Over-design | pass | no generic repository trait or module split; only required metadata |
| 15. Small efficient diffs | pass | exact future files/owners and no unrelated spec/product edits |
| 16. Architecture unity | pass | daemon owns storage; app/CLI use client; live/durable truth separated |

No unresolved blocking spec finding remains.

## 8. Parity Truth Boundary

T-103 may prove:

- schema v4 and ordered migration;
- service-owned durable repository access;
- app/CLI direct-storage removal;
- effective config and recovery fact durability;
- lineage/remote/update metadata foundation.

T-103 alone MUST NOT mark the following implemented:

- Settings UI parity or native macOS action/UI rows;
- recursive lineage permission/report/MCP workflows;
- remote listener/node/checkpoint transfer/handoff workflows;
- usage watcher/fleet/UI;
- updater feed/download/verification/install/rollback;
- packaging or performance thresholds.

## 9. Validation Record

| Check | Status | Result |
|-------|--------|--------|
| Bead consistency | pass | `homie-t3u.2` is `IN_PROGRESS`; spec path, change id, master task, parent, and baseline match |
| OpenSpec status | pass | `4/4 artifacts complete` |
| OpenSpec strict validation | pass | `Change 'diri-storage-core-facts' is valid` |
| document consistency scan | pass | change id, Bead, six methods, schema-v3 baseline, v4 revision, and parity non-closure agree |
| allowed-path scan | pass | this agent changed only the two allowed specs and the two allowed change directories; concurrent T-102 documentation changes were left untouched |
| product-code scan | pass | no modified/untracked path under `crates/`, app manifests, workspace manifest, or lockfile from this task |
| `git diff --check` | pass | no whitespace error |

These statuses come from actual commands run after all specification files were written.

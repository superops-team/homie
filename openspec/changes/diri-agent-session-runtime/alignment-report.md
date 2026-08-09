# T-102 PRD / OpenSpec Alignment Report

## 1. Identity

| Field | Value |
|-------|-------|
| Change | `diri-agent-session-runtime` |
| PRD | `prd-spec/features/diri-agent-session-runtime/2026-08-09-diri-agent-session-runtime-design.md` |
| Bead | `homie-t3u.1` |
| Parent | `diri-7ba3407-parity-rebaseline` |
| Master task | `T-102` |
| Baseline | Diri `7ba3407` |
| Checkpoint | `48f522b` |
| Date | `2026-08-09` |

## 2. Current-Fact Alignment

| Fact | PRD | Proposal/Design | Specs | Tasks |
|------|-----|-----------------|-------|-------|
| 14 tests, 12 passed, 2 failed | §1.2, FR-01 | proposal Why; design Context | local recovery: Current regression facts | 1.1, 4.1 |
| reopen adoption RED | §1.2/1.3 | design Decision 1 | holder continuity: startup reconciliation | 1.1, 1.2, 2.1 |
| live PTY reopen RED | §1.2/1.3 | design Decision 2 | holder continuity: PTY preservation | 1.1, 2.1 |
| holder stat test already GREEN | §1.2, FR-01/03 | proposal/design | holder continuity: stat; local recovery baseline | 1.1, 2.1, 2.2, 4.1 |
| failed lifecycle run leaked temporary holder PID 87051, now manually terminated | §1.2, FR-11, acceptance | D15 | holder continuity: bounded/fixture-cleanable | 1.1, 4.1, 4.2 |
| bulk detach before adoption is root cause | §1.3, FR-02 | design Decision 1 | holder continuity: reconcile before rewrite | 1.2, 2.1, 3.1 |
| holder evidence is live authority | FR-02 | design Decisions 1-2 | holder continuity: authority | 1.2, 2.1 |
| storage row alone cannot prove running | FR-02/10 | design Decisions 1-2 | holder continuity: storage cannot prove | 1.2, 2.1, 3.2 |

No document in this change treats
`runtime_holder_stat_tracks_resize_and_log_offsets` as a RED blocker. Historical Wave 1A evidence
is referenced as a dated snapshot and is not rewritten.

## 3. PRD Requirement Coverage

| PRD requirement | Capability requirement(s) | Design decision(s) | Tasks | Verification |
|-----------------|---------------------------|--------------------|-------|--------------|
| FR-01 RED/GREEN baseline | local recovery: Current regression facts; holder: stat continuity | Context, D1 | 1.1, 4.1 | exact focused output; 5 serial runs |
| FR-02 reconciliation/authority | holder: reconcile before rewrite; holder authority | D1, D2 | 1.2, 2.1 | truth table + existing RED |
| FR-03 holder/PTY continuity | holder: PTY continuity; stat; structured launch; bounds | D3, D4, D15 | 2.1, 2.2, 4.2 | restart/input/output/resize/stat E2E |
| FR-04 manifest spawn/effective config | manifest: explicit runtime; compiled catalog; resolved config; env; launch commit | D5, D6 | 1.3, 2.3 -> T-103 S103-GREEN-02 -> 2.5 | compiled-catalog + fake executable real PTY |
| FR-05 reducer/hooks | status: one reducer; signal convergence; hook; screen | D7, D8 | 1.4, 2.6, 2.7 | status/hook suite + E2E |
| FR-06 process/resource | status: process operations; resource sampling | D9 | 1.5, 2.8 | process tree repeated tests |
| FR-07 governor/hibernate/archive | status: governor; hibernate/wake | D10, D11 | 1.5, 2.9 | same-tree continuity |
| FR-08 resume/local migration substrate | recovery: resume; output; archive; local substrate | D12, D13 | 1.6, 2.10 | recovery suite |
| FR-09 shutdown/recovery | recovery: prepare; shutdown; recovery E2E | D14 | 1.6, 2.11, 4.2 | ACK/holder survival/restart |
| FR-10 security/no fallback | manifest env; tests no production config; exact capability truth | D4-D6, D16 | 2.3-2.5, 3.1, 3.2 | negative/security scans |
| FR-11 SDD/TDD/evidence | all capability release scenarios; panic-safe before/after holder gate | D15, migration plan | all phases, especially 1.1/4.1/4.2 | reports in implementation |

Coverage result: `11/11` PRD requirements have capability requirements, design decisions, tasks, and
verification.

## 4. Capability-to-Task Coverage

### 4.1 `holder-pty-continuity`

| Requirement | RED | GREEN | REFACTOR/EVIDENCE |
|-------------|-----|-------|-------------------|
| startup reconciliation | 1.1, 1.2 | 2.1 | 3.1, 4.1 |
| holder live authority | 1.2 | 2.1 | 3.2, 4.2 |
| PTY/output continuity | 1.1, 1.2 | 2.1, 2.2 | 4.2 |
| stat geometry/offset | retained GREEN 1.1 | 2.2 | 4.1, 4.2 |
| structured launch | 1.3 | 2.2, 2.5 | 3.1, 4.2 |
| bounded cleanup | 1.1 | all process tasks | 4.1, 4.2 |

### 4.2 `manifest-agent-runtime`

| Requirement | RED | GREEN | REFACTOR/EVIDENCE |
|-------------|-----|-------|-------------------|
| explicit runtime selection | 1.3 | 2.3, 2.5 | 3.1, 3.2 |
| readiness without execution | 1.3 | 2.3 | 4.1 |
| resolved effective-config contract and durable freeze | 1.3 | 2.3 contract -> T-103 S103-GREEN-02 persistence -> 2.5 consumption | 4.2 |
| sanitized environment | 1.3 | 2.3, 2.5 | 3.2, 4.2 |
| manifest owns launch/resume/authority | 1.3, 1.4, 1.6 | 2.3, 2.5, 2.6, 2.10 | 4.2 |
| commit after holder readiness | 1.3 | 2.5 | 4.2 |
| no production test config | 1.3 | 2.3 | 3.2 |

### 4.3 `runtime-status-governor`

| Requirement | RED | GREEN | REFACTOR/EVIDENCE |
|-------------|-----|-------|-------------------|
| stateful reducer | 1.4 | 2.6 | 3.1, 4.2 |
| signal convergence | 1.4 | 2.6, 2.7 | 4.2 |
| structured hook/notify | 1.4 | 2.7 | 3.2 |
| manifest screen | 1.4 | 2.6 | 3.1 |
| process identity/signal | 1.5 | 2.8 | 4.1 |
| resource sample | 1.5 | 2.8 | 4.1, 4.2 |
| conservative governor | 1.5 | 2.9 | 3.2 |
| PTY-continuous hibernate/wake | 1.5 | 2.9 | 4.2 |

### 4.4 `local-session-recovery`

| Requirement | RED | GREEN | REFACTOR/EVIDENCE |
|-------------|-----|-------|-------------------|
| direct manifest resume | 1.6 | 2.10 | 3.1, 4.2 |
| output/checkpoint continuity | 1.6 | 2.10 | 4.2 |
| archive/unarchive distinction | 1.5, 1.6 | 2.9, 2.10 | 4.1 |
| local migration substrate only | 1.6 | 2.10 | 3.2, 4.3 |
| prepare quiesce/flush | 1.6 | 2.11 | 4.2 |
| shutdown holder continuity | 1.6 | 2.11 | 4.2 |
| real-process recovery E2E | 1.1, 1.6 | 2.1-2.11 | 4.2 |
| exact regression facts | 1.1 | 2.1 | 4.1, 4.3 |

Coverage result: every capability requirement has at least one RED or retained-GREEN gate, one
GREEN implementation task, and one refactor/evidence gate.

## 5. Master T-102 Alignment

| Master responsibility | Child delivery |
|-----------------------|----------------|
| FR-03 Runtime/PTY/Holder lifecycle | holder reconciliation, real PTY continuity, process control, hibernate/wake, shutdown |
| FR-04 Agent launch/detection/resume/permission | manifest launch/readiness/resolved config contract/authority/direct resume; durable freeze delegated to T-103 |
| holder/process integration | holder structured launch/stat/signal/sample |
| reducer integration | per-session reducer + structured hook/notify/screen/process/input/tick |
| resource governor | bounded conservative local governor |
| resume/migrate/shutdown | direct local resume, local migration substrate, holder-safe shutdown |

The child does not claim:

- full RT-010 remote migration;
- T-202 UI/terminal interaction;
- T-401 remote node/handoff;
- T-402 provider proxy/virtual-key issuance.

## 6. Long-Lived Component Alignment

| Component | Child requirement | Planned/recorded contract |
|-----------|-------------------|---------------------------|
| runtime supervisor | reconciliation authority/order | `specs/runtime-supervisor/README.md` |
| runtime supervisor | holder/PTY/process/resource/resume/shutdown | same |
| agent adapter | manifest launch/effective config/readiness | `specs/agent-adapter-contract/README.md` |
| agent adapter | reducer authority and hook/notify/screen signal ownership | same |
| storage schema/repository | T-103 v4 effective-config freeze/hash/bind/readback | read-only dependency on `homie-t3u.2` / `S103-GREEN-02`; T-102 does not edit its specs or product files |

No other component contract is changed. The cross-change ordering is
`T102 G3 -> T103 S103-GREEN-02 -> T102 G5`; T-103 shared proto/runtime integration still waits for
T-102 release, so the graph has no cycle. A required transport wire, credential, remote, or UI
contract change is an explicit stop condition.

## 7. Scope and Deferral Alignment

| Area | Status in T-102 | Guard |
|------|-----------------|-------|
| holder adoption/PTY continuity | in scope | real holder E2E |
| manifest agent spawn | in scope | no shell fallback |
| reducer/hook wiring | in scope | one canonical reducer |
| process/resource governor | in scope | idle/unattached conservative policy |
| resume/local relaunch | in scope | same session/checkpoint |
| local migration substrate | in scope | internal only |
| remote `session.migrate` | deferred | method/capability absent |
| remote move/fork/handoff | deferred | T-401 |
| UI/terminal interaction | deferred | T-202 |
| provider forwarding/virtual key | deferred | T-402 |

RT-010 remains partial until remote transfer/handoff is separately specified, implemented, and
verified.

## 8. Consistency Checks

- No unresolved marker, stub success, or optional production fallback remains in normative
  requirements/tasks.
- All task owners include file scope, timeout/budget, verification, and cleanup/blocker handling.
- T-102 has no `G-CONFIG/G4` task or `homie-storage/src/lib.rs` ownership; T-103 is the sole
  schema/repository/effective-config persistence owner.
- Production manifests are compiled from committed descriptor JSON through an explicit
  `include_str!` table; packaged binaries do not discover manifest resources from cwd or PATH.
- Real-process suites record holder PID+start-time before/after; RED failure, panic, timeout, and
  success all require panic-safe fixture cleanup and an empty added-holder set.
- Product implementation tasks are ordered RED -> GREEN -> REFACTOR -> EVIDENCE.
- Real daemon/holder/PTY E2E is blocking evidence; mocks are not accepted as release proof.
- Capability publication is tied to real handler availability.
- This S102 documentation task does not modify product code, parity lock, master tasks, other
  specs, or Git history.

Alignment conclusion: complete, pending OpenSpec strict/status and final document scans.

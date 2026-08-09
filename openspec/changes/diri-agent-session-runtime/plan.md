# Diri Agent Session Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `dev-loop` and
> `test-driven-development`; execute exactly one task from `tasks.md` per TraeCLI, attach command
> output and cleanup evidence, and do not start dependent tasks before the listed gate passes.

**Goal:** Build a truthful local agent-session runtime with holder adoption, real PTY continuity,
manifest-driven spawn, stateful status, conservative resource governance, direct resume, and
holder-safe shutdown.

**Architecture:** Keep the Wave 1A daemon/actor/holder boundary. Add per-session reconciliation,
immutable launch/status state inside the runtime actor, structured holder process control, and
bounded worker lanes. Storage remains durable projection input; only verified holder evidence
proves local liveness.

**Tech Stack:** Rust 1.95, Edition 2024, Tokio, Unix domain sockets, the existing
`rustix-openpty`-based PTY implementation, rusqlite, serde/serde_json, existing `homie-agents`,
`homie-proto`, T-103-owned `homie-storage`, and real subprocess E2E.

---

## 1. Execution Rules

### 1.1 TDD order

Every behavioral slice follows:

```text
RED: add/retain one exact failing behavior
  -> run the narrow command and capture expected failure
GREEN: minimum production change
  -> run the narrow command and affected package tests
REFACTOR: remove superseded path and consolidate ownership
  -> rerun all affected tests
EVIDENCE: run real process gates and record cleanup/security/review facts
```

No GREEN task may:

- weaken, ignore, rename, or delete a RED test;
- write final state directly when the reducer owns it;
- infer running from a storage row;
- add fixed-shell fallback for unavailable agents;
- add environment configuration or production test mode;
- mock holder/PTY in a release E2E;
- modify parity state before evidence.

### 1.2 One TraeCLI per task

Each task in `tasks.md`:

- has one primary owner;
- has a maximum active work budget of 4 hours unless marked 6 hours;
- owns a disjoint file set from concurrently runnable tasks;
- ends with narrow verification and exact cleanup;
- reports `pass|blocked|fail`, not aspirational completion;
- does not create a Git commit unless the master execution workflow explicitly requests it.

If the task discovers a required file outside its ownership, it stops as `blocked` and updates the
delegation handoff. It does not make an opportunistic shared-file edit.

### 1.3 Command bounds

Test binaries must enforce their own deadlines. Shell-level commands use the platform's available
timeout wrapper only when already present; the product does not read timeout environment
variables.

| Gate | Deadline |
|------|----------|
| one unit-test binary | 60 s |
| one integration-test binary | 120 s |
| one real daemon/holder E2E case | 60 s |
| one focused package suite | 10 min |
| workspace gate | 20 min |
| OpenSpec strict/status | 60 s |

### 1.4 Process cleanup

Every subprocess fixture owns a ledger:

```text
pre-test holder PID + start-time baseline
absolute temp data dir
daemon pid + start time
holder socket/pid/status paths
holder pid + start time
root child pid + start time
```

Drop/panic/timeout cleanup:

1. enter a panic-safe guard on normal return, RED assertion failure, panic, or timeout;
2. request session kill/holder terminate/daemon shutdown where responsive;
3. wait at most 3 seconds;
4. signal only ledger PIDs whose start time still matches;
5. reap children;
6. remove only the fixture temp directory;
7. assert no ledger process/socket/pid file remains;
8. resample holder PID/start-time facts and assert after-minus-before is empty.

Process-name matching may observe the before/after holder set but may not kill it. `pkill`, user
production data dirs, and pre-existing baseline holders are prohibited.

## 2. File Ownership Map

The paths below apply to implementation after spec approval. The current S102 task writes only
specification files.

| Owner | Primary paths | Exclusive responsibility |
|-------|---------------|--------------------------|
| `R-BASE` | `crates/homie-runtime/tests/support/**`, focused runtime tests | process fixture ledger and RED baseline |
| `G-RECONCILE` | `crates/homie-runtime/src/reconciliation.rs`, focused sections of `src/lib.rs` | startup outcomes and adoption commit order |
| `G-HOLDER` | `crates/homie-runtime/src/holder.rs`, `src/bin/homie-runtime-holder.rs`, holder tests | structured launch/control/stat |
| `G-AGENT-PLAN` | `crates/homie-agents/src/launch.rs`, `src/lib.rs`, agent tests | manifest resolution and immutable launch plan |
| `G-SPAWN` | `crates/homie-runtime/src/agent_launch.rs`, focused `runtime_actor.rs`, proto/client DTOs/tests | actor-level manifest spawn |
| `G-STATUS` | `crates/homie-runtime/src/status_runtime.rs`, focused actor/hook paths | stateful reducer and signal routing |
| `G-PROCESS` | `crates/homie-runtime/src/process_tree.rs`, process tests | identity-safe signal/sample primitives |
| `G-GOVERNOR` | `crates/homie-runtime/src/resource_governor.rs`, focused daemon/actor lifecycle | policy and hibernate/wake |
| `G-RECOVERY` | `crates/homie-runtime/src/session_recovery.rs`, focused dispatcher/client methods | archive/resume/local relaunch |
| `G-SHUTDOWN` | focused daemon/actor shutdown paths and tests | quiesce/flush/holder continuity |
| `R-CLEANUP` | runtime module exports and obsolete code only after all GREEN owners finish | delete superseded paths |
| `E-E2E` | new process E2E files, CLI/client harness only | cross-entry real-process proof |
| `E-DOCS` | `docs/verification/diri-agent-session-runtime/**`, allowed tracking updates | final evidence and release state |

`crates/homie-runtime/src/lib.rs` and `runtime_actor.rs` are serial integration files. Only one
owner may edit either at a time. Parallel workers create/test focused modules first; the designated
integration task wires them.

## 3. RED Phase

### R1: Record exact current baseline and harden fixture cleanup

**Owner:** `R-BASE`
**Budget:** 4 hours
**Files:**

- Modify: `crates/homie-runtime/tests/session_lifecycle.rs`
- Create/Modify: `crates/homie-runtime/tests/support/process_fixture.rs`
- Modify only if required: `crates/homie-runtime/tests/support/mod.rs`

Required work:

- preserve the current two RED tests unchanged in intent and assertion;
- preserve `runtime_holder_stat_tracks_resize_and_log_offsets` as GREEN;
- replace ad hoc test process handling with the exact ledger/Drop cleanup contract;
- add a cleanup self-test that forces an assertion failure in a child test process and confirms no
  fixture resources remain;
- record holder PID+start-time before the suite and require the post-cleanup added-holder set to be
  empty;
- record initial command output before production changes.

Commands:

```text
cargo test -p homie-runtime --test session_lifecycle -- --nocapture
```

Expected RED:

```text
14 tests: 12 passed, 2 failed
runtime_reopen_can_adopt_holder_and_continue_session: detached != running
runtime_spawn_shell_uses_live_pty: detached != running
runtime_holder_stat_tracks_resize_and_log_offsets: ok
```

Gate: exact baseline, zero fixture-owned residual process/socket, and empty holder
after-minus-before set even when RED assertions fail.

### R2: Add startup reconciliation table REDs

**Owner:** `R-BASE` after R1
**Budget:** 4 hours
**Files:**

- Create: `crates/homie-runtime/tests/startup_reconciliation.rs`
- Reuse: test support only

Cases:

- created/starting/running/detached + live holder -> running/adopted;
- idle/needs-input + live holder -> preserved/adopted;
- live storage row + no holder -> detached;
- holder explicit exit -> exited;
- hibernated + stopped holder -> hibernated/adopted;
- archived + live holder -> recovery contradiction;
- no branch starts a duplicate holder.

Expected: new table cases fail on current bulk-detach/adoption behavior; retained holder-stat case
stays green.

### R3: Add manifest launch/effective-config REDs

**Owner:** `G-AGENT-PLAN`
**Budget:** 4 hours
**Files:**

- Create: `crates/homie-agents/tests/runtime_launch_plan.rs`
- Create: `crates/homie-runtime/tests/manifest_spawn.rs`
- Test fixtures: package-local temp executable helpers

Cases:

- strict manifest -> absolute executable + exact argv;
- safe env baseline/scrub;
- unavailable binary -> no process/session/config;
- profile edit after freeze does not change running config;
- explicit shell succeeds; unknown agent does not fall back;
- real fake executable prints argument/env evidence through real PTY.

Expected: tests fail because runtime still starts fixed shell and no immutable launch record exists.

### R4: Add stateful status/hook REDs

**Owner:** `G-STATUS`
**Budget:** 4 hours
**Files:**

- Create: `crates/homie-runtime/tests/runtime_status_engine.rs`
- Extend only test modules in `crates/homie-agents`

Cases:

- manifest authority selected per session;
- repeated status reads do not reconstruct/mutate reducer;
- hook, notify, screen, process, input, and tick converge;
- hook commit precedes event;
- subagent hook does not overwrite parent;
- restart rebuild preserves verified idle/needs-input but not stale running.

Expected: tests fail on fresh fixed `ScreenPrimary` reducer/direct status writes.

### R5: Add process/resource continuity REDs

**Owner:** `G-PROCESS`
**Budget:** 4 hours
**Files:**

- Create: `crates/homie-runtime/tests/process_tree.rs`
- Create: `crates/homie-runtime/tests/resource_governor.rs`

Cases:

- tree STOP verification;
- leaves-first CONT and same child identity;
- PID reuse guard;
- tree size/footprint sample;
- sample failure remains unknown/no kill;
- only idle+unattached+unpinned auto-hibernates;
- hibernate/wake keeps holder/PTY/offset;
- hibernated input returns stable error.

Expected: stop/continue/sample/governor cases fail; current terminate cases remain green.

### R6: Add resume/relaunch/shutdown REDs

**Owner:** `G-RECOVERY`
**Budget:** 4 hours
**Files:**

- Create: `crates/homie-runtime/tests/session_recovery.rs`
- Extend: focused daemon shutdown integration tests

Cases:

- ID/latest manifest resume direct argv;
- missing resume ID fails closed;
- existing live holder is adopted instead of duplicated;
- resume appends output epoch and preserves session metadata;
- failed relaunch preserves record/checkpoint;
- unarchive does not spawn;
- prepare rejects new lifecycle mutations and flushes facts;
- graceful shutdown leaves live/hibernated holder;
- hard restart adopts and continues.

Expected: direct resume/local substrate/shutdown-flush cases fail on current implementation.

## 4. GREEN Phase

### G1: Implement holder-first startup reconciliation

**Owner:** `G-RECONCILE`
**Depends on:** R1, R2
**Budget:** 4 hours
**Files:**

- Create: `crates/homie-runtime/src/reconciliation.rs`
- Modify: focused startup/adoption code in `crates/homie-runtime/src/lib.rs`
- Test: startup reconciliation and session lifecycle

Required API behavior:

```text
collect persisted fact
probe holder
decide ReconciliationOutcome
persist projection
insert registry
```

Delete the `mark_interrupted_sessions_detached()` call from the startup path. Keep the storage
method untouched if another owner uses it; deletion is a later refactor only if `rg` proves no use.

Gate:

- the two existing RED tests turn GREEN;
- holder stat remains GREEN;
- startup table passes;
- no duplicate holder/child;
- zero residual fixtures.

### G2: Extend holder structured launch and live control

**Owner:** `G-HOLDER`
**Depends on:** G1 interface freeze
**Budget:** 6 hours
**Files:**

- Modify: `crates/homie-runtime/src/holder.rs`
- Modify: `crates/homie-runtime/src/bin/homie-runtime-holder.rs`
- Add/modify: holder protocol/process tests

Required behavior:

- structured argv/cwd/env/geometry launch;
- additive stat fields where required by stop/sample;
- STOP/CONT/sample requests with bounded response;
- one-shot owner-only launch-plan transport;
- no argv/env/raw credential logging;
- old live holder stat/adoption remains parseable through additive fields;
- no fixed-agent command fallback in holder.

Gate: holder protocol/unit/process tests and retained stat GREEN.

### G3: Build immutable manifest launch plans

**Owner:** `G-AGENT-PLAN`
**Depends on:** R3
**Budget:** 6 hours
**Files:**

- Create: `crates/homie-agents/src/launch.rs`
- Modify: `crates/homie-agents/src/lib.rs`
- Modify: focused manifest/readiness tests

Required types:

```text
EffectiveAgentConfig
ResolvedAgentExecutable
AgentLaunchPlan
AgentResumePlan
LaunchPlanError
```

Required behavior:

- explicit `include_str!` table compiling every committed
  `assets/agent-descriptors/*.json` into the immutable production catalog;
- packaged daemon/standalone CLI manifest loading with no cwd, PATH, or external resource lookup;
- build/test-time completeness check between committed descriptor inventory and compiled table;
- constructor-injected test catalog;
- absolute executable readiness without agent execution;
- env baseline/scrub;
- manifest argv/injection/status authority/resume projection;
- explicit shell only;
- safe Debug/Display redaction.

Gate: all `homie-agents` and `runtime_launch_plan` tests pass, then publish the exact
`ResolvedEffectiveAgentConfig` type/field handoff to T-103. T-103
`S103-GREEN-02` exclusively implements v4 freeze/hash/atomic bind/readback.

### G5: Wire actor manifest spawn through the real holder

**Owner:** `G-SPAWN`
**Depends on:** G2, G3, T-103 `S103-GREEN-02` repository GREEN handoff
**Budget:** 6 hours
**Files:**

- Create: `crates/homie-runtime/src/agent_launch.rs`
- Modify: focused `crates/homie-runtime/src/runtime_actor.rs`
- Modify: focused module exports in `crates/homie-runtime/src/lib.rs`
- Modify: exact DTOs/client methods and tests in `crates/homie-proto/**`,
  `crates/homie-client/**`

Required behavior:

- typed spawn selects profile or explicit shell;
- resolve in T-102, then freeze/bind through the T-103 repository before launch;
- holder readiness before running/event;
- reverse-order rollback;
- no shell command string;
- no capability publication before handler integration.

Gate: manifest spawn integration tests pass with a real fake executable and PTY.

### G6: Implement actor-owned stateful status runtime

**Owner:** `G-STATUS`
**Depends on:** G3, G5
**Budget:** 6 hours
**Files:**

- Create: `crates/homie-runtime/src/status_runtime.rs`
- Modify: focused actor/session status paths
- Reuse: `homie-agents` reducer/manifest engine

Required behavior:

- one reducer/manifest engine/cursor per live session;
- bounded output replay and incremental screen processing;
- process/output/screen/input/tick signal methods;
- persistence before event;
- side-effect-free status reads;
- restart reconstruction from holder + persisted behavior + checkpoint.

Gate: `runtime_status_engine` tests except external hook ingress pass.

### G7: Route structured hook/notify signals into status runtime

**Owner:** `G-STATUS` after G6
**Budget:** 4 hours
**Files:**

- Modify: focused hook DTO/CLI/runtime handler tests
- Modify: `status_runtime.rs`
- Do not persist raw payload

Required behavior:

- strict allowlisted event DTO;
- parser redaction;
- reducer signal mapping;
- subagent isolation;
- commit-before-event;
- stable invalid payload result.

Gate: complete status/hook RED suite passes.

### G8: Implement identity-safe process-tree signal and sampling

**Owner:** `G-PROCESS`
**Depends on:** R5, G2 request shape
**Budget:** 6 hours
**Files:**

- Modify: `crates/homie-runtime/src/process_tree.rs`
- Modify: process-tree tests

Required behavior:

- enumerate root, descendants, and required process-group peers;
- capture and verify start time;
- STOP + verification;
- leaves-first CONT;
- TERM/CONT -> grace -> KILL/CONT;
- tree size and platform footprint sample;
- safe unknown on races/unsupported sample.

Gate: process tree tests pass serially and under repeated execution.

### G9: Implement conservative resource governor and PTY-continuous hibernate/wake

**Owner:** `G-GOVERNOR`
**Depends on:** G6, G8
**Budget:** 6 hours
**Files:**

- Create: `crates/homie-runtime/src/resource_governor.rs`
- Modify: focused actor/daemon lifecycle wiring
- Modify: holder call integration

Required behavior:

- one bounded daemon timer;
- exact idle/unattached/unpinned eligibility;
- running/needs-input protection;
- sample unknown -> no action;
- STOP hibernate and CONT wake;
- input fail closed while hibernated;
- archive remains terminating;
- stop new ticks during prepare shutdown.

Gate: resource/hibernate tests pass; holder/PTY/offset identity is unchanged across wake.

### G10: Implement direct manifest resume and local relaunch substrate

**Owner:** `G-RECOVERY`
**Depends on:** G3, G5, G6, G9
**Budget:** 6 hours
**Files:**

- Create: `crates/homie-runtime/src/session_recovery.rs`
- Modify: focused actor/dispatcher/proto/client paths
- Modify: history resume call path

Required behavior:

- direct ID/latest resume argv;
- same Homie session ID and new output epoch;
- preserve title/parent/profile/permission/checkpoint;
- adopt existing holder before launch;
- unarchive without spawn;
- failed readiness keeps record/output retryable;
- local checkpoint/relaunch API remains internal;
- no remote `session.migrate` capability or placeholder.

Gate: session recovery suite passes and remote capability remains absent.

### G11: Extend prepare/shutdown flush without terminating holders

**Owner:** `G-SHUTDOWN`
**Depends on:** G6, G9, G10
**Budget:** 4 hours
**Files:**

- Modify: focused runtime actor/daemon shutdown paths and tests
- Preserve: Wave 1A transport ACK ordering

Required behavior:

- quiesce new lifecycle mutations;
- stop new governor ticks;
- bounded drain accepted work;
- flush status/needs-input/screen/output/event/WAL;
- send ACK before teardown;
- leave running/hibernated holders alive;
- hard restart reuses G1 reconciliation.

Gate: shutdown/restart tests and Wave 1A daemon lifecycle tests pass.

## 5. REFACTOR Phase

### F1: Remove superseded production paths and finalize module boundaries

**Owner:** `R-CLEANUP`
**Depends on:** G1-G11
**Budget:** 4 hours
**Files:**

- `crates/homie-runtime/src/lib.rs`
- `crates/homie-runtime/src/runtime_actor.rs`
- focused obsolete helper/tests proved unused by `rg`

Delete:

- startup bulk-detach-before-adopt call/path;
- fixed-shell agent-profile spawn;
- shell-text history resume;
- fresh reducer construction in status reads;
- agent-agnostic complete status classifier;
- terminate-and-new-shell hibernate;
- duplicate state persistence/event paths.

Keep explicit shell manifest behavior.

Gate: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, affected
package tests, and `rg` negative scans pass.

### F2: Run security and consistency negative scans

**Owner:** `R-CLEANUP` after F1
**Budget:** 4 hours
**Files:** tests/scanners only unless a finding requires its owning GREEN task to reopen

Scan/assert:

- no provider raw keys/Authorization/cookies in argv, metadata, logs, snapshots, evidence;
- no `HOMIE_*` production override for manifests/holder/runtime mode;
- no embedded runtime/fake backend;
- no unavailable-agent shell fallback;
- no public remote migrate capability;
- no storage-only running inference;
- no global `pkill`;
- no unbounded queue/task-per-session governor.

Gate: findings are zero or routed back to the owning task and reverified.

## 6. EVIDENCE Phase

### E1: Run focused and repeated local gates

**Owner:** `E-E2E`
**Depends on:** F1, F2
**Budget:** 4 hours
**Commands:**

```text
cargo test -p homie-agents
cargo test -p homie-runtime --lib
cargo test -p homie-runtime --test session_lifecycle -- --nocapture
cargo test -p homie-runtime --test startup_reconciliation -- --nocapture
cargo test -p homie-runtime --test manifest_spawn -- --nocapture
cargo test -p homie-runtime --test runtime_status_engine -- --nocapture
cargo test -p homie-runtime --test process_tree -- --nocapture
cargo test -p homie-runtime --test resource_governor -- --nocapture
cargo test -p homie-runtime --test session_recovery -- --nocapture
cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1
```

The final lifecycle command runs five serial iterations from the harness/script, not by copying
test logic. Each iteration records zero residual fixture resources and an empty holder
PID/start-time after-minus-before set.

Gate: all pass; current two RED are GREEN; retained stat gate remains GREEN.

### E2: Run real daemon/holder cross-entry E2E

**Owner:** `E-E2E` after E1
**Budget:** 6 hours
**Files:**

- Create/modify: dedicated runtime/CLI process E2E only
- No production test-mode changes

Flow:

1. start packaged daemon with absolute fixture data dir;
2. spawn a manifest fake executable through typed client/CLI;
3. verify argv/env/output/status;
4. resize and verify holder stat offsets;
5. SIGKILL daemon, preserve holder, start replacement;
6. verify adoption, storage/snapshot agreement, and input/output continuation;
7. feed structured hook/notify and verify reducer;
8. hibernate/wake same tree and verify continuity;
9. archive/unarchive/resume direct manifest argv;
10. prepare/shutdown and verify holder survival;
11. final explicit session cleanup and zero-resource assertion.

Gate: complete flow passes within per-case bounds without mocks or leaked processes; panic-safe
cleanup also leaves no holder added after the pre-test PID/start-time baseline.

### E3: Record reviews, reports, and scoped parity handoff

**Owner:** `E-DOCS`
**Depends on:** E1, E2
**Budget:** 4 hours
**Files:**

- `docs/verification/diri-agent-session-runtime/spec-review-report.md`
- `docs/verification/diri-agent-session-runtime/test-report.md`
- `docs/verification/diri-agent-session-runtime/e2e-report.md`
- `docs/verification/diri-agent-session-runtime/security-review-report.md`
- `docs/verification/diri-agent-session-runtime/code-review-report.md`
- `docs/verification/diri-agent-session-runtime/release-readiness-report.md`
- parity/task/Bead updates only after reports pass and only by their owner

Required evidence:

- checkpoint and commit SHA;
- exact commands and exit status;
- 2 RED -> GREEN and retained GREEN table;
- timeout/cleanup ledger summary;
- no-fallback/security scan;
- OpenSpec strict/status;
- PRD/OpenSpec/component alignment;
- explicit deferred RT-010 remote/UI/remote-node/provider rows.

Gate: release-readiness can be `pass` only if all blocking gates pass. RT-010 remote remains
partial.

## 7. Phase Gates

| Gate | Required result |
|------|-----------------|
| RED gate | exact 2 RED + retained holder-stat GREEN; new failures prove scoped gaps |
| Adoption gate | existing lifecycle 14/14 and reconciliation matrix pass |
| Manifest gate | real executable direct argv/env under real holder/PTY |
| Status gate | one reducer, structured hooks, commit-before-event |
| Resource gate | verified stop/continue/sample; conservative governor |
| Recovery gate | direct resume, same identity/history, no remote capability |
| Shutdown gate | facts flushed; ACK order preserved; holder survives |
| Cleanup gate | exact fixture process/socket count zero |
| Release gate | strict/status, package/workspace tests, real E2E, reviews, evidence |

## 8. Escalation Conditions

Stop implementation and return to spec review if:

- T-103 `S103-GREEN-02` cannot represent the G3 resolved contract without semantic loss;
- a Wave 1A frame/event/terminal transport change is required;
- a shared holder manager becomes necessary;
- provider raw credentials must enter the runtime;
- remote migration or UI behavior becomes required;
- a retained GREEN test must change;
- cleanup cannot distinguish fixture-owned from user processes.

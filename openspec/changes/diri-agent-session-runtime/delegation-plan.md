# T-102 TraeCLI Delegation Plan

## 1. Delegation Contract

Each delegation executes exactly one unchecked item from `tasks.md`.

Every worker receives:

- Bead `homie-t3u.1`;
- change `diri-agent-session-runtime`;
- checkpoint `48f522b`;
- PRD, design, relevant capability spec, plan, and exact task item;
- owned files and forbidden files;
- expected RED/GREEN command;
- deadline and cleanup procedure.

Every worker returns:

```text
task_id:
status: pass | blocked | fail
files_changed:
commands:
test_results:
cleanup_result:
remaining_risks:
handoff:
```

Rules:

- no worker modifies another task's owned files;
- no worker modifies parity/master tasks/evidence before its phase;
- no worker adds production fallback, environment configuration, fake backend, or remote/UI scope;
- no worker uses `pkill` or cleans pre-existing holder processes;
- no worker claims pass without command output and cleanup result;
- current S102 specification session creates no Git commit.

## 2. Critical Path

```text
R1 -> R2 -> G1
R3 -> G3 -> T103-S103-GREEN-02 -> G5
R4 -------------> G6 -> G7
R5 -> G2 -> G8 -> G9
R6 ---------------------> G10 -> G11
G1+G2+G3+T103-S103-GREEN-02 -> G5
G3+G5 -> G6
G6+G8 -> G9
G3+G5+G6+G9 -> G10
G6+G9+G10 -> G11
all GREEN -> F1 -> F2 -> E1 -> E2 -> E3
```

## 3. Wave Schedule

### Wave A: RED baseline

Maximum concurrency: 3.

| Task | Worker | Owned files | Timeout | Output |
|------|--------|-------------|---------|--------|
| 1.1 R1 | `trae-r-base` | lifecycle test + fixture support | 4h / test 120s | exact 2 RED + 1 GREEN + panic-safe cleanup + before/after holder set |
| 1.3 R3 | `trae-r-manifest` | new agent/runtime manifest tests | 4h / test 120s | manifest spawn RED |
| 1.4 R4 | `trae-r-status` | new status tests | 4h / test 120s | reducer/hook RED |

Serial continuation:

| Task | Worker | Dependency | Output |
|------|--------|------------|--------|
| 1.2 R2 | `trae-r-base` | 1.1 | reconciliation matrix RED |
| 1.5 R5 | `trae-r-process` | 1.1 fixture available | process/resource RED |
| 1.6 R6 | `trae-r-recovery` | 1.1 fixture available | resume/shutdown RED |

Wave A gate:

- current focused suite remains exactly 12/14;
- only the two named historical cases are counted as current blockers;
- holder stat remains green;
- new tests fail for their intended missing behavior;
- no fixture residual;
- holder PID+start-time after-minus-before is empty even after RED assertion failure/panic/timeout.

### Wave B: Adoption and independent foundations

Maximum concurrency: 3, with `lib.rs` integration serialized.

| Task | Worker | Owned files | Dependency |
|------|--------|-------------|------------|
| 2.1 G1 | `trae-g-reconcile` | reconciliation module + startup section of `lib.rs` | 1.1, 1.2 |
| 2.3 G3 | `trae-g-agent-plan` | `homie-agents` launch module/tests | 1.3 |
| 2.8 G8 | `trae-g-process` | process tree module/tests | 1.5 + holder request shape agreed |

After G1 freezes holder-adoption interfaces:

| Task | Worker | Owned files | Dependency |
|------|--------|-------------|------------|
| 2.2 G2 | `trae-g-holder` | holder client/binary/tests | 2.1 |
| T-103 `S103-GREEN-02` | `trae-s103-storage` | T-103-owned v4 effective-config repository | T-102 G3 contract handoff |

Wave B gate:

- two existing RED tests are green without assertion changes;
- holder stat remains green;
- agent launch-plan tests pass;
- storage uses no new schema;
- process signal/sample primitives pass;
- all process fixtures clean.

T-102 does not edit `homie-storage`. If the T-103 repository cannot represent the G3 contract
without semantic loss, both lanes stop as `blocked` and return to cross-spec review.

### Wave C: Runtime integration

Maximum concurrency: 1 for shared actor integration.

| Order | Task | Worker | Owned files |
|-------|------|--------|-------------|
| 1 | 2.5 G5 | `trae-g-spawn` | agent launch module + actor/proto/client spawn integration |
| 2 | 2.6 G6 | `trae-g-status` | status runtime + actor status integration |
| 3 | 2.7 G7 | `trae-g-status` | hook/notify ingress + reducer |
| 4 | 2.9 G9 | `trae-g-governor` | governor + actor/daemon lifecycle |
| 5 | 2.10 G10 | `trae-g-recovery` | recovery module + resume/history integration |
| 6 | 2.11 G11 | `trae-g-shutdown` | prepare/shutdown integration |

Each worker starts from the immediately preceding verified tree. No parallel edits to
`runtime_actor.rs` or `lib.rs`.

Wave C gate:

- direct manifest spawn uses real holder/PTY;
- one reducer per session;
- hook/notify goes through reducer;
- hibernate/wake preserves holder/tree;
- resume is direct manifest argv;
- remote migrate capability is absent;
- shutdown preserves holders.

### Wave D: Refactor

Maximum concurrency: 1.

| Task | Worker | Output |
|------|--------|--------|
| 3.1 F1 | `trae-refactor` | obsolete production paths deleted; module exports clean |
| 3.2 F2 | `trae-review-scan` | security/fallback/authority negative scans |

Wave D gate:

- no fixed-shell agent fallback;
- no shell-text history resume;
- no fresh reducer read path;
- no terminate-and-respawn hibernate;
- no storage-only running inference;
- no remote placeholder;
- fmt/clippy/tests pass.

### Wave E: Evidence

Maximum concurrency: 1 because process E2E uses host process resources.

| Task | Worker | Output |
|------|--------|--------|
| 4.1 E1 | `trae-e-focused` | focused/package/repeated results |
| 4.2 E2 | `trae-e-process` | real daemon/holder cross-entry E2E |
| 4.3 E3 | `trae-e-docs` | reviews, evidence, readiness, scoped tracking handoff |

Wave E gate:

- all blocking commands pass;
- five serial lifecycle runs have zero leak;
- real process E2E has zero leak;
- security/code review have no open blocker;
- RT-010 remote remains partial;
- parity/Bead updates match evidence.

## 4. Worker Prompts

### 4.1 RED worker prompt

```text
You own task <id> only. Read the T-102 PRD, design, your capability spec, plan section, and task
item. Add the smallest test/fixture change that proves the stated current gap. Do not modify
production code. Do not change existing test intent. Run the exact narrow command with the stated
deadline. Record holder PID+start-time baseline, use a panic-safe guard to clean only
fixture-owned PID/start-time/process-group/socket paths, then report residual count and the
after-minus-before holder set. Process-name matching may observe but never kill. Stop blocked if
another owner file is required.
```

### 4.2 GREEN worker prompt

```text
You own task <id> only and start from its verified RED. Make the minimum production change in the
listed files. Do not add compatibility fallback, environment configuration, fake backend, raw-key
handling, remote migration, or UI scope. Run the narrow RED test, affected package suite, and
cleanup assertion. Do not edit shared actor/lib files unless this task explicitly owns them.
```

### 4.3 REFACTOR worker prompt

```text
All GREEN gates are complete. Remove only the obsolete paths listed in F1/F2 and prove no callers
remain with rg. Preserve explicit shell manifest behavior and all public Wave 1A contracts. Run
fmt, clippy, affected tests, negative scans, and cleanup checks. Route any behavioral finding back
to its original owner instead of hiding it in refactor.
```

### 4.4 EVIDENCE worker prompt

```text
Do not change product behavior. Run the exact gates and record command, exit status, counts,
duration, commit/checkpoint, and cleanup. Real-process tests must use packaged daemon, packaged
holder, real PTY, and real local fake executable. Never clean by process name. Mark blocked/fail
truthfully. Do not advance RT-010 remote, UI, remote-node, or provider rows.
```

## 5. Shared-File Locking

| File | Lock owner/order |
|------|------------------|
| `crates/homie-runtime/src/lib.rs` | G1 -> G5 -> G6 -> G9 -> G10 -> G11 -> F1 |
| `crates/homie-runtime/src/runtime_actor.rs` | G5 -> G6 -> G7 -> G9 -> G10 -> G11 -> F1 |
| `crates/homie-runtime/src/holder.rs` | G2 -> G8 integration if needed -> F1 |
| `crates/homie-runtime/src/bin/homie-runtime-holder.rs` | G2 only |
| `crates/homie-runtime/src/process_tree.rs` | G8 only |
| `crates/homie-agents/src/lib.rs` | G3 -> F1 only if export cleanup |
| `crates/homie-storage/src/lib.rs` | T-103 `S103-storage-impl` only; T-102 read-only |
| proto/client spawn/recovery DTOs | G5 -> G10, serial |

Before editing a shared file, a worker verifies the prior owner is `pass` and no uncommitted
concurrent edit exists. If the worktree contains unrelated user changes, the worker preserves them
and edits only its hunk.

## 6. Timeout and Cleanup Escalation

| Condition | Worker action |
|-----------|---------------|
| holder IPC >350ms | return timeout, cleanup exact fixture, report |
| readiness >3s | rollback uncommitted launch, cleanup, report |
| STOP/CONT >2s | leave status unchanged, cleanup test fixture, report |
| process cleanup >3s | start-time checked SIGKILL fixture PID, reap, report |
| test binary >120s | terminate exact test process tree, mark blocked/fail |
| E2E >60s | terminate ledger only, record last completed phase |
| command hangs beyond declared bound | record command/failure and stop task |
| unexpected pre-existing holder found | do not touch; exclude from fixture ledger |
| holder remains in after-minus-before set | mark fail; clean only if ledger PID/start-time matches |

## 7. Handoff Evidence Minimum

No downstream worker may accept a handoff without:

- `git diff --name-only` within owner scope;
- narrow RED/GREEN command and exit code;
- exact pass/fail counts;
- cleanup residual count;
- pre/post holder PID+start-time counts and added-holder set;
- named unresolved risk or `none`;
- confirmation that no production fallback/env override/remote/UI behavior was added.

## 8. Specification-Only Completion

The S102 specification TraeCLI is complete when:

- PRD and all OpenSpec artifacts exist;
- both long-lived specs contain the new contract;
- OpenSpec status is 4/4;
- strict validation passes;
- consistency/path/diff checks pass;
- no product code, parity lock, master task, other spec, or commit was produced by S102.

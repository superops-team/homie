## Context

`diri-agent-session-runtime` is Wave 1B / master T-102. Wave 1A already made the runtime daemon
the sole production owner of `RuntimeSupervisor`, SQLite, live registry, events, and terminal
sources. App, CLI, and MCP use the same async UDS client. This design keeps that boundary.

The current Homie implementation is not empty:

- `homie-runtime-holder` owns a real PTY, child, output log, geometry, epoch/log offsets, and
  process-tree termination;
- `RuntimeSupervisor` can reopen storage and discover holder sockets;
- `homie-agents` owns 19 manifests, readiness projection, screen detection, hook parsers, and a
  reducer;
- storage owns runtime descriptor, profile, effective config, session, and core metadata tables;
- Wave 1A has exact capability registries, bounded actor/lane queues, and holder-safe shutdown.

The missing part is integration and truthful lifecycle reconciliation.

At checkpoint `48f522b`, the exact focused result is:

```text
cargo test -p homie-runtime --test session_lifecycle -- --nocapture
14 tests: 12 passed, 2 failed
```

The two failures are:

```text
runtime_reopen_can_adopt_holder_and_continue_session
  left: detached
 right: running

runtime_spawn_shell_uses_live_pty
  left: detached
 right: running
```

`runtime_holder_stat_tracks_resize_and_log_offsets` passes and is a retained regression gate.

The startup bug is deterministic:

```text
open_inner
  -> mark_interrupted_sessions_detached()
  -> adopt_live_holders()
       -> only mark running when persisted state is created|starting|running
```

By the time adoption reads sessions, those states are already `detached`. The holder is inserted
into the in-memory live registry, but storage/list projection remains detached.

Relevant validated Diri patterns:

- `HolderLauncher.swift`, `HolderServer.swift`, and `HolderTests.swift`: holder-owned PTY,
  duplicate-launch prevention, stat/epoch continuity, bounded tree cleanup;
- `AgentDescriptor` and `InjectionBuilder`: manifest-owned binary/argv/env/resume decisions;
- `AgentSession` and `StatusEngine`: one stateful reducer fed by process/screen/hook/input/tick;
- `ResourceGovernor.swift`: idle/unattached eligibility and non-destructive memory policy;
- `ScreenCheckpoint.swift`: checkpoint is an acceleration cache, not liveness authority;
- `SessionRegistry.relaunch`: same session ID, adopt an existing holder before starting another;
- `SessionMigrator`: remote transfer exists in Diri, but is explicitly outside this change.

Constraints:

- macOS/Unix local runtime only;
- no environment-variable product configuration;
- no embedded runtime or production fake/test mode;
- no provider raw key in agent/holder configuration;
- no compatibility fallback for fixed shell spawning;
- no Wave 1A frame, event, terminal stream, or launcher redesign;
- no remote/UI delivery claim;
- implementation starts only after this spec is approved.

## Goals / Non-Goals

**Goals:**

- make holder live evidence authoritative for liveness while preserving reducer-owned behavior status;
- repair the current two RED tests without weakening assertions;
- retain the existing holder stat/resize/log-offset GREEN behavior;
- execute a selected manifest agent directly under holder-owned PTY;
- freeze effective config and sanitized launch facts for restart/resume;
- run one stateful reducer per session with manifest authority and runtime hook wiring;
- implement verified tree stop/continue/terminate and bounded footprint sampling;
- make hibernate/wake continuous and resource-governed;
- implement same-session manifest resume/relaunch and local migration substrate;
- flush lifecycle facts on shutdown while preserving holders;
- prove all behavior with real daemon/holder/PTY subprocess E2E and fixture-owned cleanup.

**Non-Goals:**

- remote `session.migrate`, SSH/tmux, checkpoint transfer, move/fork, lease, or target quarantine;
- RT-010 completion;
- GPUI/terminal interaction, remote node, provider proxy, or credential issuance;
- a generic plugin system or new RPC framework;
- storage schema/repository implementation, owned exclusively by T-103 `homie-t3u.2`;
- an environment override for manifests, holder path, agent binary, timeouts, or governor limits;
- keeping `/bin/sh` as an automatic fallback for unknown/unavailable agents.

## Decisions

### Decision 1: Reconcile first, mutate projection second

Startup SHALL not call a bulk state rewrite before holder discovery. It SHALL build one
`ReconciliationOutcome` per persisted session from:

- persisted lifecycle/status facts;
- holder IPC result;
- holder process status;
- holder status file exit evidence;
- hibernation/archive intent;
- screen checkpoint and needs-input facts where applicable.

Conceptual outcomes:

```rust
enum ReconciliationOutcome {
    AdoptRunning,
    AdoptBehaviorStatus(SessionStatus),
    AdoptHibernated,
    RestoreExited,
    RestoreArchived,
    MarkDetached,
    Contradiction(RuntimeRecoveryError),
}
```

The exact internal type may differ, but the branches and tests are mandatory.

Reconciliation order:

```text
read persisted session
  -> probe expected holder path
  -> classify holder evidence
  -> decide one outcome
  -> persist projection
  -> insert live registry entry when applicable
  -> publish no event until startup owner is ready
```

Rationale:

- fixes the actual order bug rather than compensating after it;
- makes each state transition independently testable;
- prevents a storage row from becoming liveness authority;
- avoids registry/storage split.

Rejected alternatives:

1. Move `mark_interrupted_sessions_detached()` after adoption. Rejected because a global rewrite
   can still overwrite adopted or reducer-specific states.
2. Make `list_sessions()` synthesize running whenever a socket exists. Rejected because a path is
   not proof and list/status/snapshot would use different authorities.
3. Change the tests to accept detached. Rejected because the holder and PTY are actually live.

### Decision 2: Holder evidence proves liveness; reducer proves behavior

Successful holder `Stat` with `status=running` at the expected session holder path is the local
live-process authority. It is not sufficient to classify `idle` versus `needs_input`.

Rules:

- persisted `created|starting|running|detached` + verified running holder -> live registry +
  `running`;
- persisted `idle|needs_input` + verified running holder -> live registry + preserve specific
  behavior status;
- persisted `hibernated` + verified stopped tree -> live registry + `hibernated`;
- explicit holder exit evidence -> `exited`;
- no verifiable holder evidence for a live candidate -> `detached`;
- storage row alone never yields running;
- archived + unexpected live holder is a recovery contradiction, not silent adoption.

The public snapshot must compose one reconciled view. The runtime must not expose a live holder
with detached storage projection after startup completes.

### Decision 3: Keep per-session holder ownership; require holder-equivalent behavior

T-102 does not require a shared holder-manager process as an implementation shape. The current
per-session holder is retained if it passes:

- daemon/app crash survival;
- duplicate child/writer prevention;
- deterministic adoption;
- process-tree control and resource sampling;
- bounded cleanup;
- real E2E repetition.

Rationale:

- the parity requirement is PTY/session survival behavior, not a Swift process topology copy;
- replacing a working per-session holder with a manager is not needed to fix the current bug;
- a new manager would expand protocol, packaging, and crash surface without current evidence.

If implementation evidence shows a race that cannot be closed with the current holder paths and
launch lock, the change must return to specification review before introducing a manager.

### Decision 4: Holder launch accepts a structured, sanitized plan

The holder SHALL launch arbitrary direct argv and a complete sanitized child environment. It
SHALL NOT receive a shell command string.

The structured launch plan includes:

```rust
struct HolderLaunchPlan {
    session_id: String,
    argv: Vec<OsString>,
    cwd: PathBuf,
    env: Vec<(OsString, OsString)>,
    cols: u16,
    rows: u16,
    output_log: PathBuf,
}
```

The serialized or IPC representation must:

- avoid provider raw keys;
- not place full argv/env in logs, process listings, events, or evidence;
- use owner-only local transport/file semantics;
- validate absolute cwd/executable;
- be consumed exactly once for new holder launch;
- retain only a sanitized restart/resume record.

Existing holders remain adoptable through additive protocol behavior. T-102 does not add a
fixed-shell fallback for old/new launch paths.

### Decision 5: Manifest catalog builds the runtime launch plan

`homie-agents` owns agent-specific launch semantics. Runtime owns when and where a process starts.

Pipeline:

```text
profile id
  -> enabled profile + runtime descriptor
  -> bundled manifest id/version
  -> readiness resolver
  -> absolute executable
  -> argv + injection + env scrub
  -> EffectiveAgentConfig
  -> HolderLaunchPlan
```

Committed `assets/agent-descriptors/*.json` are compiled into
`homie-agents` through an explicit `include_str!` table in a focused bundled-catalog module. The
packaged daemon and standalone CLI consume that immutable in-binary catalog and do not discover
manifest JSON from cwd, PATH, or external resource directories. Tests inject a catalog through
Rust constructors; no daemon flag or environment variable selects a fake catalog. A catalog
completeness test compares the committed descriptor inventory to the explicit compiled table at
build/test time.

Readiness:

- resolves the manifest binary without executing the agent;
- returns one absolute executable;
- rejects missing, non-executable, directory, or ambiguous results;
- may use a bounded login-shell resolver because version managers can alter PATH;
- never accepts caller-supplied shell fragments.

Explicit `shell` uses `/bin/sh -i`. `generic` or unknown IDs require an explicit reviewed command
contract in a later change; they do not inherit shell fallback here.

### Decision 6: T-102 freezes the resolved contract; T-103 persists it

T-102 G3 owns the Rust type/field contract that resolves manifest and runtime inputs into one
`ResolvedEffectiveAgentConfig`. The contract includes profile/runtime/LLM/permission identifiers,
manifest id/version/status authority, absolute executable, final argv, sanitized non-secret
environment, injection/resume decisions, cwd, parent session, and initial geometry.

T-102 does not write `crates/homie-storage/src/lib.rs`, define schema v4, implement repository
methods, hash snapshots, bind sessions, or own readback. T-103 `homie-t3u.2` is the sole
schema/repository/effective-config persistence owner:

```text
T-102 G3 resolved type/field contract handoff
  -> T-103 S103-GREEN-02 v4 repository freeze/hash/bind/readback
  -> T-102 G5 manifest spawn consumes the GREEN repository handoff
```

T-103 storage-only GREEN work does not require T-102 shared runtime/proto file release. Conversely,
T-103 shared proto/runtime integration remains blocked until T-102 releases those files. This
separates the storage-only repository prerequisite from later shared integration and prevents a
dependency cycle.

The running `LiveSession` owns the decoded in-memory contract. Durable resume/relaunch reads the
safe frozen snapshot through the T-103 repository and rejects missing/invalid data rather than
rebuilding from a changed profile.

Rationale:

- establishes one schema/repository owner;
- lets T-103 implement its specified v4 immutable snapshot and atomic bind;
- keeps T-102 focused on agent resolution, holder launch, and live lifecycle;
- keeps raw provider credentials out of the contract and durable snapshot.

### Decision 7: One stateful reducer per live session

The current `session_status_report()` creates a fresh `ScreenPrimary` reducer from the complete
output each read. T-102 replaces that behavior.

Each live session owns:

```text
frozen manifest authority
StatusReducer
ManifestEngine reference
HeadlessScreen/checkpoint cursor
last processed output offset
persisted NeedsInput
```

Signals:

- process ready/exit;
- PTY output activity;
- manifest screen observation;
- Claude hook;
- Codex notify;
- user input;
- tick.

The actor applies a signal, persists the reducer outcome, then publishes an event. Status reads
only project the current canonical state and are side-effect free.

On daemon restart:

- holder evidence restores liveness;
- persisted status/needs-input restores the behavior projection;
- checkpoint + bounded output replay rebuilds screen;
- reducer debounce counters may restart;
- no status becomes running from storage alone.

### Decision 8: Hook/notify reports carry signals, not precomputed final status

The existing CLI parser remains responsible for safe parsing/redaction. The runtime handler SHALL
receive a structured event sufficient to map to `StatusSignal`.

It SHALL NOT:

- directly set `needs_input`/`idle` as the primary path;
- trust caller-provided arbitrary final status;
- let subagent hooks retitle or overwrite parent status;
- persist raw hook payload.

Rationale:

- one reducer must own authority arbitration and anti-flicker;
- hook/screen/process conflict behavior remains deterministic;
- runtime restart has one canonical projection contract.

### Decision 9: Holder owns process-tree signals; runtime owns policy

Holder operations are extended behaviorally to:

```text
stat
signal STOP
signal CONT
terminate tree
sample tree/footprint
```

The holder:

- enumerates root, descendants, and process-group peers;
- records PID start time;
- checks start time immediately before signaling;
- verifies STOP state;
- resumes leaves before root;
- uses TERM + CONT then KILL + CONT for termination;
- returns safe counts/footprint, not command lines or env.

Runtime/resource governor decides when to call these operations. Socket tasks never sample or
signal processes directly.

### Decision 10: Hibernate/wake preserves the same PTY

T-102 changes the current semantics:

```text
current:
hibernate -> terminate holder
wake      -> start new shell

target:
hibernate -> holder STOP verified process tree
wake      -> holder CONT verified same process tree
```

While hibernated:

- holder, PTY, output log, epoch, and session ID remain;
- input returns `session_hibernated`;
- resize may be stored/applied without creating a new process;
- status ticks do not classify the frozen screen as new activity.

Archive/kill are the operations that terminate the tree.

### Decision 11: Resource governor is bounded and conservative

One daemon-level governor timer submits bounded actor work. It does not create one task per
session.

Eligibility for automatic hibernation:

```text
status == idle
AND attachment_count == 0
AND pinned == false
AND holder is live
```

Not eligible:

- starting;
- running;
- needs_input;
- already hibernated;
- archived/exited/detached;
- currently attached.

Memory policy may hibernate an eligible idle session but never silently kill it. Sample failure
records unknown and leaves the process unchanged.

The initial policy uses fixed reviewed defaults and existing persisted settings where already
available. T-102 does not add environment configuration or UI settings.

### Decision 12: Resume is direct manifest relaunch under the same Homie session

Resume is valid only when:

- session is not already live;
- frozen manifest declares a resume style;
- ID-required style has a persisted agent session ID;
- latest style is explicitly declared;
- cwd and absolute executable remain valid.

Resume:

1. reads frozen effective config and screen/output checkpoint;
2. builds manifest resume argv;
3. launches a new holder incarnation with the same Homie session ID;
4. appends to the same output stream with a new epoch boundary;
5. waits for holder readiness;
6. commits live registry/status only after readiness;
7. preserves title, parent, profile, permission, and output history.

History resume uses the same path. It no longer starts a shell and sends a textual command.

### Decision 13: T-102 delivers local migration substrate, not remote migration

The local migration substrate is:

- flush screen/output checkpoint;
- freeze source effective config;
- stop/relaunch/resume under the same Homie session identity;
- withhold target projection until readiness;
- preserve the source record/checkpoint after failure.

T-102 SHALL NOT advertise a production remote `session.migrate` handler. It does not perform:

- git WIP commit/push;
- transcript transfer;
- host lookup/change;
- quarantine restore;
- move/fork lease;
- source/target remote cleanup.

RT-010 therefore remains partial after this change. T-401 owns the remote transaction.

### Decision 14: Shutdown extends Wave 1A without changing ACK order

Prepare shutdown:

```text
reject new lifecycle mutations
  -> stop new governor ticks
  -> drain accepted mutations to deadline
  -> persist reducer/needs-input projection
  -> write screen checkpoint/output cursor
  -> flush event store and SQLite WAL
  -> return prepare result
```

Shutdown:

```text
send ACK
  -> close listener/streams
  -> stop governor
  -> stop actor
  -> leave live/hibernated holders untouched
```

Hard restart uses the same startup reconciliation as normal reopen.

### Decision 15: Timeouts and cleanup are part of the contract

Fixed bounds:

| Operation | Bound |
|-----------|-------|
| holder request | 350 ms |
| holder/agent readiness | 3 s |
| STOP/CONT verification | 2 s |
| TERM grace | 500 ms |
| total holder cleanup | 3 s |
| status/output/resource sample | 10 s |
| one real-daemon phase | 15 s |
| one complete process E2E | 60 s |

Test fixtures record exact resource ownership. Cleanup order:

```text
session kill / holder terminate / daemon shutdown
  -> bounded wait
  -> start-time-checked SIGKILL for fixture PIDs only
  -> reap
  -> assert fixture socket/pid/process count == 0
```

Global `pkill`, user data directories, and pre-existing holder processes are prohibited.

### Decision 16: No production fallback and exact capability truth

The implementation deletes or stops using:

- bulk detach before adoption;
- fixed `/bin/sh` production spawn for agent profiles;
- shell command injection for history resume;
- fresh reducer construction in status read;
- agent-agnostic full status classifier;
- terminate-and-respawn hibernate.

No compatibility shim or fallback remains. Tests use constructor-injected catalogs and real fake
executables, never production flags.

Only implemented handlers appear in Hello capabilities. Remote migration and later UI/remote
methods remain absent.

## Data And State

### Startup truth table

| Persisted | Holder evidence | Registry | Projection |
|-----------|-----------------|----------|------------|
| created/starting/running/detached | running | insert | running |
| idle/needs_input | running | insert | preserve |
| hibernated | stopped/live | insert | hibernated |
| archived | absent | omit | archived |
| live candidate | explicit exit | omit | exited |
| live candidate | absent/unverifiable | omit | detached |
| archived | running | error/blocked recovery | archived, never running |

### State transitions

```text
created -> starting -> running
running -> needs_input -> running
running -> idle
idle -> hibernating -> hibernated -> waking -> idle|running
running|idle|needs_input -> terminating -> exited
running|idle|needs_input -> archiving -> archived
archived -> unarchived(exited) -> resuming -> running
exited -> resuming -> running
live candidate + missing holder -> detached
detached + verified holder at startup -> running|preserved behavior state
```

### Durable and live ownership

| Fact | Owner |
|------|-------|
| PTY fd, child, process tree | holder |
| output bytes, epoch/log offset | holder output log |
| live registry, reducer, governor | runtime actor |
| session/profile/effective-config IDs and canonical projection | storage |
| resolved in-memory launch contract | T-102 runtime/agent adapter |
| immutable safe effective-config snapshot/hash/binding | T-103 v4 repository |
| provider raw key | credential owner, never this change |

## Risks / Trade-offs

- **[Holder says running but process exits immediately after Stat]**
  Mitigation: adoption commits a live candidate, then normal process/status polling moves it to
  exited; input still fails if IPC is gone. No storage-only running fallback.

- **[Preserving idle/needs-input can hide dead holder]**
  Mitigation: preserve only after verified live Stat and insert into live registry.

- **[Manifest upgrade changes resume semantics]**
  Mitigation: resume reads frozen manifest version/launch facts; missing or incompatible facts
  fail closed.

- **[Structured holder plan can expose secrets]**
  Mitigation: raw provider keys are forbidden; owner-only one-shot transport; safe-field scans;
  argv/env never logged.

- **[Stateful reducer and terminal source can duplicate screen work]**
  Mitigation: use one actor-owned status cursor/checkpoint; do not add one parser per client
  attachment.

- **[STOP does not reclaim memory]**
  Mitigation: governor is a continuity feature, not a memory reclamation claim. It reports
  footprint and never claims pages were released.

- **[Resource governor hibernates active work]**
  Mitigation: exact idle + unattached + unpinned eligibility; running/needs-input are excluded.

- **[Resume is irreversible after old process termination]**
  Mitigation: checkpoint and effective config are flushed first; projection changes only after new
  holder readiness; record remains retryable on failure.

- **[T-103 repository handoff is unavailable or contract-incompatible]**
  Mitigation: block T-102 G5 and reconcile the G3 field contract with T-103 before product
  integration. T-102 never edits storage schema/repository as a workaround.

- **[Real process tests leak holders after assertion panic]**
  Mitigation: RAII fixture ownership, exact PID/start-time tracking, bounded cleanup, and final
  zero-resource assertion.

- **[Scope expands into RT-010 remote migration]**
  Mitigation: no remote method/capability, no host fields, and explicit alignment/deferred table.

## Migration Plan

1. Record the exact 2 RED / 1 GREEN baseline and create fixture-owned cleanup utilities.
2. Replace startup bulk detach with per-session reconciliation; turn the two existing RED tests
   green first.
3. Add additive holder control behavior and keep stat/resize/log-offset green.
4. Add manifest launch-plan construction and fake-executable contract tests in `homie-agents`.
5. Hand the G3 resolved type/field contract to T-103 and wait for `S103-GREEN-02` repository
   freeze/hash/bind/readback GREEN.
6. Make holder launch arbitrary structured argv/env and route `session.spawn` through it using the
   T-103 repository handoff.
7. Add stateful per-session status runtime and structured hook/notify signals.
8. Add process-tree signal/sample and bounded resource governor.
9. Change hibernate/wake/archive lifecycle semantics.
10. Add direct manifest resume and local checkpoint/relaunch substrate.
11. Extend prepare/shutdown flush and recovery tests.
12. Run real daemon/holder cross-entry E2E, repeated lifecycle tests, security scans, reviews, and
    evidence.
13. Update parity rows only for behavior proven by final evidence; leave RT-010 remote partial.

Rollback:

- before release, revert the complete T-102 implementation;
- do not restore fixed shell or embedded-runtime production paths as rollback;
- runtime data created by an unaccepted implementation must remain readable or fail closed;
- holder sessions started by the accepted prior version are not globally killed during rollback.

## Open Questions

None for specification approval.

Implementation must return to spec review if it requires:

- the T-103 `S103-GREEN-02` repository contract cannot represent T-102 G3 without semantic loss;
- Wave 1A wire/frame changes;
- a shared holder-manager process;
- provider credential handling;
- remote migration/handoff;
- UI/terminal behavior.

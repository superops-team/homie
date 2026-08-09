# Design: Diri Storage Core Durable Facts

## Context

T-103 starts from a functioning storage implementation, not an empty database layer.
`homie-storage` schema version 3 already applies v1, v2, and v3 in one transaction and exposes
session, history, project/worktree, usage, scan-cache, preferences, and schema-inventory APIs.
The v1 schema already contains `effective_agent_configs`; v2 contains hosts, node accounts, and
handoff records.

The current architecture still violates the intended boundary:

- `homie-app` links `homie-storage` and opens SQLite for settings;
- CLI doctor and usage summary open SQLite;
- runtime exposes `RuntimeSupervisor::storage()` to its actor;
- runtime shutdown directly accesses the SQLite connection;
- effective config rows are not frozen or bound by a public repository contract;
- recovery combines session rows, holder files, output logs, checkpoints, and event logs without
  one typed durable fact contract.

T-102 is active in parallel and owns manifest-driven spawn, holder adoption, PTY continuity, and
runtime lifecycle files. T-103 therefore freezes shared contracts first and delays edits to
shared proto/runtime files until T-102 releases them.

## Goals / Non-Goals

**Goals:**

- add an ordered v4 migration without rewriting v1-v3;
- make daemon/domain services the production owners of typed repositories;
- freeze and read back immutable safe effective configs;
- persist bounded recovery hints and atomically updated checkpoint facts;
- add durable lineage/remote/update metadata foundations;
- remove app and CLI direct storage dependencies through typed client methods;
- retain truthful partial status for workflows outside this foundation.

**Non-Goals:**

- implement T-102 agent/holder lifecycle;
- implement settings UI E2E, remote node/handoff, usage watcher/fleet/UI, or updater install;
- add a compatibility storage path;
- expose SQLite connections or repository types over the wire;
- edit product code during this specification task.

## Current Baseline

| Surface | Present in schema v3 | Missing contract targeted by T-103 |
|---------|----------------------|------------------------------------|
| Migrations | v1-v3 ordered transaction, idempotency, too-new rejection | real v3 fixture -> v4 and injected rollback |
| Effective config | table and session FK column | immutable freeze/readback and atomic session binding |
| Session/history/worktree/usage | typed APIs and green tests | service ownership, no duplicate implementation |
| Recovery | session status + holder/status/log/checkpoint/event files | typed joined facts and atomic checkpoint metadata |
| Lineage | direct parent column and direct-child APIs | idempotent safe provenance/decision audit |
| Remote | host/account/handoff tables | typed repository and operation/lease/hash/phase constraints |
| Update | no table | durable receipt and constrained phase transitions |
| Consumer boundary | app/CLI direct SQLite paths remain | typed daemon/client integration and dependency removal |

## Decisions

### Decision 1: Add v4 to the existing ordered migration chain

The implementation SHALL preserve v1-v3 and append v4. Empty databases apply `[1,2,3,4]`; a real
v3 fixture applies only `[4]`. DDL, backfill, index creation, and insertion of migration version 4
commit in one transaction.

Alternative: rebuild a canonical current schema and infer upgrades. Rejected because it weakens
ordered migration evidence and risks diverging from shipped v1-v3 semantics.

### Decision 2: Keep one SQLite owner and typed methods, not repository traits

The runtime daemon's actor/domain services SHALL own the production `Storage` instance. Storage
exposes narrowly typed methods grouped by domain. The wire carries DTOs, never repository
handles, SQL, transactions, or `rusqlite::Connection`.

Version 4 SHALL add a monotonic revision to each preference row used by settings. The settings
repository compares `expected_revision` and increments revision in the same update transaction.

No new repository-trait hierarchy is required for this slice. The monolithic storage file has
one implementation owner, which is simpler and avoids parallel edits.

Alternative: split storage into multiple crates/modules before adding behavior. Rejected because
it expands the change and conflicts with the user's single-owner rule.

### Decision 3: Use additive relational metadata, not generic JSON-only facts

Identity, revision, operation id, phase, lease, hash, offset, and sequence fields SHALL be
relational columns with indexes/constraints. Versioned safe snapshots may use bounded JSON for
resolved runtime/LLM/permission payloads.

Alternative: put all new data in `config_events.safe_payload_json` or existing handoff JSON.
Rejected because uniqueness, compare-and-set transitions, and recovery bounds would be
unverifiable.

### Decision 4: Effective config is immutable after one atomic bind

The existing `effective_agent_configs` table SHALL gain:

- `snapshot_version`;
- `runtime_snapshot_json`;
- `managed_llm_snapshot_json`;
- `permission_snapshot_json`;
- `config_hash`;
- one-config-per-session uniqueness.

The storage API SHALL atomically create/bind the session and frozen config, or provide equivalent
single-transaction semantics. There is no update method. Readback joins by session id and returns
safe fields only. `virtual_key_id` is a reference; virtual key material is forbidden.

T-102 computes the resolved launch configuration. T-103 owns durable validation, hashing, binding,
and readback.

The cross-change order is:

```text
T-102 G3 freezes the resolved type/field contract
  -> T-103 S103-GREEN-02 implements v4 freeze/hash/bind/readback
  -> T-103 publishes the repository GREEN handoff
  -> T-102 G5 consumes that repository for manifest spawn
```

This storage handoff does not wait for all of T-102. Later T-103 proto/runtime integration does
wait for T-102 to release the exact shared files, which avoids a dependency cycle.

### Decision 5: Recovery rows are hints, never live authority

`session_runtime_recovery` SHALL contain one row per session with:

- holder instance id, PID hint, and holder start time;
- output epoch;
- checkpoint path, output offset, and content sequence;
- checkpointed event sequence;
- last runtime instance id and last observed durable status;
- update timestamp.

The repository joins these fields with existing session output path/tail offset. PID, instance id,
and last status are hints. On restart, T-102/runtime validates holder/process/output evidence
before it can publish `running`. Multi-field recovery updates are atomic and bounded candidate
queries have deterministic ordering.

### Decision 6: Reuse parent_session_id and add audit, not a second graph

`sessions.parent_session_id` remains the direct-parent source of truth. T-103 adds
`lineage_audit_events` for idempotent safe operation/provenance/decision facts. Recursive graph
authorization and MCP workflows remain owned by later changes.

### Decision 7: Extend existing remote metadata and add updater receipts

Existing hosts/accounts/handoff tables remain. The handoff record gains operation id, checkpoint
id, phase, lease id, manifest hash, and stable conflict/idempotency rules. No blobs or secrets are
stored.

`update_receipts` records operation id, source/target versions, phase, feed host, archive hash,
bundle/team identity, staged/previous path references, safe error, and timestamps. It does not
fetch or install anything.

### Decision 8: Remove direct consumer storage paths

The following methods are frozen for T-103 integration:

| Method | DTO contract | Owner |
|--------|--------------|-------|
| `storage.health` | empty -> `StorageHealthResult` | storage service / runtime dispatcher |
| `settings.get` | empty -> `SettingsSnapshot` | settings service |
| `settings.update` | revisioned request -> updated snapshot | settings service |
| `usage.summary` | bounded filters -> safe aggregate | usage query service |
| `session.effective_config` | session id -> safe frozen snapshot | session/config service |
| `runtime.recovery.summary` | bounded filter -> safe recovery summary | runtime admin service |

Existing `session.set_parent`, `session.list_children`, and `session.parent` remain the lineage
methods. T-103 does not add or advertise remote handoff or updater workflow methods.

DTO freeze rules:

- settings updates carry `expected_revision`;
- health contains no live-session claim;
- effective config contains no provider key or virtual-key material;
- recovery DTO labels all persisted process fields as hints/observations;
- bounded filters and stable safe errors are mandatory.

Method constants, DTOs, handlers, typed client methods, and capability discovery are implemented
only after contract freeze and T-102 file release. A method absent from handlers stays absent from
discovery.

### Decision 9: CLI doctor joins the service boundary

T-103 removes the CLI's normal storage dependency along with the app dependency. Doctor reports
`storage.health` through the owning service and does not fabricate runtime liveness. If storage
prevents daemon startup, the launcher/client returns a stable storage-unavailable diagnostic; the
CLI does not fall back to direct SQLite.

This intentionally replaces the historical "doctor opens storage without daemon" internal path.
The repository does not preserve incorrect internal compatibility by default.

## Target Data Flow

```text
app / CLI
  -> homie-client typed method
  -> runtime dispatcher
  -> RuntimeActor / owning service
  -> homie-storage typed method
  -> SQLite v4
  -> authoritative response/event
```

Recovery:

```text
repository candidates
  -> T-102 holder/process/output verification
  -> runtime classification
  -> atomic durable assessment
  -> event/snapshot
```

## Migration Plan

1. Capture current green storage baseline and direct-dependency RED evidence.
2. Add RED tests for v3->v4, rollback, effective config, recovery, metadata, and service boundary.
3. The sole storage implementation owner adds v4 and typed methods serially.
4. Run storage-only GREEN and preserve all existing v3 repository tests.
5. Freeze proto DTO/method names and wait for T-102 shared-file release.
6. Add proto, runtime service/dispatcher, and client integrations under exclusive owners.
7. Switch app settings and CLI doctor/usage to client methods.
8. Delete direct dependencies, open helpers, raw connection use, and fallback paths.
9. Run restart, cross-process, security, workspace, and evidence gates.

Rollback:

- v4 migration failure rolls back the complete migration transaction.
- No schema downgrade is provided.
- Integration rollback is a source rollback before release, not a production dual path.

## Risks / Trade-offs

- **T-102 shared-file conflict**: block proto/runtime integration until T-102 releases exact files;
  storage RED/GREEN can proceed independently.
- **Monolithic storage file**: assign one implementation owner for every edit; tests and docs use
  different owners.
- **Stale PID/status data**: model fields as hints and require holder identity/process validation.
- **Generic snapshots can leak secrets**: version and validate safe fields, bound size, and scan
  fixtures/results.
- **Foundation mistaken for parity closure**: alignment and evidence explicitly retain partial UI,
  remote, usage, update, packaging, and performance rows.
- **Doctor cannot inspect a database that prevents daemon startup**: return a stable startup/storage
  diagnostic without opening a second production connection.

## Open Questions

None for specification approval. Low-level representation may change only if it preserves all
requirements and is revised in PRD/OpenSpec before implementation. Shared contract naming or DTO
field changes require T-102/T-103 owner agreement before product edits.

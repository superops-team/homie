# Proposal: Diri Storage Core Durable Facts

## Why

Homie already has a green schema-v3 storage baseline: ordered transactional v1-v3 migrations,
the `effective_agent_configs` table, session core metadata, and history/worktree/usage
repositories. T-103 must not recreate or misreport those capabilities.

The remaining gap is ownership and durable behavior. `homie-app` still links and opens
`homie-storage` to load/save settings, CLI doctor/usage still open SQLite directly, effective
agent configs cannot be frozen and read back through a repository, and runtime recovery is
assembled from ad hoc session rows and filesystem evidence. Existing lineage/remote tables also
lack service-owned idempotent repository contracts, while updater receipts are not durable.

## What Changes

- Preserve the schema-v3 baseline and add an ordered transactional v4 migration contract.
- Add service-owned repository contracts for settings, health, usage summaries, immutable
  effective config, runtime recovery facts, lineage audit, remote operation metadata, and update
  receipts.
- Require removal of the existing `homie-app -> homie-storage` and
  `homie-cli -> homie-storage` production dependencies after typed daemon/client methods land.
- Freeze the shared proto/client/runtime methods and DTOs needed for integration, without
  modifying shared product code during this specification task.
- Assign one implementation owner to the monolithic `crates/homie-storage/src/lib.rs`.
- Keep T-102 responsible for producing and validating live agent/holder facts; T-103 persists
  those facts and never treats a storage row as proof that a process is live.
- Provide durable metadata foundations only. This change does not independently complete UI,
  remote, usage, updater, packaging, or performance parity.

## Capabilities

### New Capabilities

- `ordered-storage-migrations`: Extend the existing ordered migration chain with atomic v4
  upgrade, rollback, idempotency, and schema-too-new requirements.
- `service-owned-durable-repositories`: Route app/CLI durable reads and writes through typed
  daemon/client services and remove direct storage dependencies.
- `runtime-recovery-facts`: Persist bounded holder/output/checkpoint/event recovery hints while
  requiring runtime verification before reporting live state.
- `effective-agent-config-facts`: Atomically freeze and read back immutable safe effective-agent
  configuration snapshots.
- `durable-metadata-foundation`: Add idempotent lineage audit, remote operation, and update receipt
  metadata without implementing their owning workflows.

## Impact

- Source PRD:
  `prd-spec/features/diri-storage-core-facts/2026-08-09-diri-storage-core-facts-design.md`.
- Bead: `homie-t3u.2`; master task: T-103; parent change:
  `diri-7ba3407-parity-rebaseline`.
- Long-lived specs updated:
  `specs/storage-indexing/README.md` and `specs/session-context-store/README.md`.
- Future implementation primarily affects `crates/homie-storage`; shared proto/client/runtime,
  app, and CLI integration is serialized through the contract-freeze ownership plan.
- No product code, parity lock, master task, or evidence file is modified by this proposal.

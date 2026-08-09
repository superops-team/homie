# Diri Storage Core Durable Facts OpenSpec Plan

> Change ID: `diri-storage-core-facts`
> Source PRD:
> `prd-spec/features/diri-storage-core-facts/2026-08-09-diri-storage-core-facts-design.md`
> Beads: `homie-t3u.2`
> Master task: `T-103`
> Status: ready_for_review

## 1. Summary

本计划从已验证的 schema v3 开始，追加 ordered v4 migration、typed service-owned
repositories、runtime recovery facts、immutable effective config freeze/readback，以及后续
lineage/remote/update 的 durable metadata foundation。它不重新实现现有
session/history/worktree/usage repository。

交付必须删除现存 `homie-app -> homie-storage` 和 `homie-cli -> homie-storage` production
依赖。settings、doctor、usage summary 改经 runtime daemon owning service 和
`homie-client`。T-102 继续负责 agent/holder/process 的 live truth；T-103 只持久化并查询
durable facts。

## 2. Goals

| Goal | Source requirement | Acceptance |
|------|--------------------|------------|
| G-1 保留真实 v3 基线 | FR-01 | 现有 storage tests/计数先绿，RED 不虚构缺口 |
| G-2 ordered v4 migration | FR-02 | empty/v3/rollback/idempotent/too-new 全通过 |
| G-3 service-owned repositories | FR-03, FR-04 | app/CLI 无 storage normal dependency，durable operations 走 client/service |
| G-4 effective config freeze | FR-05 | session/config 原子绑定、immutable safe readback |
| G-5 runtime recovery facts | FR-06 | facts 跨 restart，running 仍需 holder/process verification |
| G-6 metadata foundation | FR-07..FR-09 | lineage/remote/update repository 幂等与 CAS 通过 |
| G-7 security/contract coordination | FR-10, FR-11 | secret scan 为零，共享文件 ownership 无冲突 |
| G-8 truthful evidence | FR-12 | foundation pass 不关闭 downstream parity rows |

## 3. Non-Goals

- 不修复 T-102 holder/PTY/manifest lifecycle。
- 不实现 UI interaction/screenshot、remote network/handoff、usage watcher/fleet/UI 或 updater。
- 不修改 parity lock 或 master tasks。
- 不保留 direct-storage fallback。
- 不拆分 `homie-storage/src/lib.rs` 或增加 repository framework。
- 本规格 TraeCLI 不修改任何产品代码。

## 4. Affected Component Specs

| Component spec | Impact | Required update |
|----------------|--------|-----------------|
| `specs/storage-indexing/README.md` | yes | schema v3 实况、v4、service ownership、recovery/config/metadata API 与 gates |
| `specs/session-context-store/README.md` | yes | parent 单一事实源、lineage audit、safe durable provenance |
| `specs/runtime-supervisor/README.md` | reviewed/no edit | T-102 owns lifecycle; T-103 freeze referenced in PRD/OpenSpec |
| `specs/agent-adapter-contract/README.md` | reviewed/no edit | T-102 computes resolved config; T-103 persists snapshot |
| `specs/runtime-client-transport/README.md` | reviewed/no edit | existing async client boundary reused |
| `specs/desktop-shell/README.md` | reviewed/no edit | already assigns settings direct-storage cleanup to T-103 |
| `specs/remote-node-handoff/README.md` | reviewed/no edit | workflow remains T-401 |
| `specs/packaging-updater/README.md` | reviewed/no edit | workflow remains T-501 |

## 5. Implementation Scope

### 5.1 Storage-owned scope

| Area | Future files/modules | Reason |
|------|----------------------|--------|
| Migration RED | `crates/homie-storage/tests/ordered_v4_migration.rs` | isolated schema contract |
| Config RED | `crates/homie-storage/tests/effective_config_facts.rs` | isolated typed repository contract |
| Recovery RED | `crates/homie-storage/tests/runtime_recovery_facts.rs` | added after H-02 milestone |
| Metadata RED | `crates/homie-storage/tests/durable_metadata_foundation.rs` | added after recovery GREEN |
| Existing regression | `crates/homie-storage/tests/*.rs` | preserve all v3 behavior |
| Migration/repositories | `crates/homie-storage/src/lib.rs` | sole owner `S103-storage-impl` |
| Storage manifest | `crates/homie-storage/Cargo.toml` only if test-only need is proven | no new production dependency expected |

`crates/homie-storage/src/lib.rs` may be edited only by `S103-storage-impl`. All implementation
tasks touching that file run serially. No other TraeCLI may edit it.

### 5.2 Shared integration scope

Shared files are blocked until contract freeze and T-102 release:

| Area | Future files/modules | Exclusive owner |
|------|----------------------|-----------------|
| Durable wire DTO/methods | `crates/homie-proto/src/lib.rs`, `model.rs`, proto tests | `S103-proto-integration` |
| Runtime services | `crates/homie-runtime/src/runtime_actor.rs`, `dispatcher.rs`, focused tests | `S103-runtime-integration` |
| T-102 lifecycle touchpoint | exact runtime lifecycle file identified after T-102 | `S103-runtime-integration`, only after handoff |
| Typed client | `crates/homie-client/src/client.rs`, focused tests | `S103-client-integration` |
| App settings | `crates/homie-app/Cargo.toml`, `main.rs`, `runtime_bridge.rs`, tests | `S103-app-integration` |
| CLI doctor/usage | `crates/homie-cli/Cargo.toml`, `main.rs`, tests | `S103-cli-integration` |

No shared owner starts until `delegation-plan.md` gates are satisfied.

## 6. Schema v4 Plan

| Object | Change |
|--------|--------|
| `preferences` | monotonic revision for compare-and-set settings updates |
| `effective_agent_configs` | add snapshot version, safe runtime/LLM/permission JSON, config hash, session uniqueness |
| `session_runtime_recovery` | one row/session for holder/output epoch/checkpoint/event/runtime hints |
| `lineage_audit_events` | unique operation id and safe actor/subject/relation/action/decision metadata |
| `handoff_records` | operation/checkpoint/phase/lease/manifest hash and indexes; preserve existing facts |
| `update_receipts` | idempotent update identity/phase/hash/bundle/team/path/safe-error receipt |

Existing `sessions.parent_session_id`, output path/tail offset, hosts, node accounts, and handoff
facts are reused. No second parent graph or duplicate output ledger is added.

## 7. Shared Contract Freeze

| Method | Frozen DTO | Integration owner |
|--------|------------|-------------------|
| `storage.health` | `StorageHealthResult` | proto -> runtime -> client -> CLI |
| `settings.get` | `SettingsSnapshot` | proto -> runtime -> client -> app |
| `settings.update` | `SettingsUpdateRequest` / `SettingsSnapshot` | proto -> runtime -> client -> app |
| `usage.summary` | `UsageSummaryRequest` / `UsageSummaryResult` | proto -> runtime -> client -> CLI |
| `session.effective_config` | session id / `EffectiveAgentConfigSnapshot` | proto -> runtime -> client |
| `runtime.recovery.summary` | bounded filter / `RuntimeRecoverySummary` | proto -> runtime -> client/admin tests |

DTO invariants:

- `SettingsSnapshot` includes monotonic revision.
- `SettingsUpdateRequest` includes `expectedRevision`.
- effective config uses versioned bounded safe snapshots and references only.
- recovery summary distinguishes persisted hints from verified live state.
- health and usage do not include raw payloads or secrets.
- remote/updater workflow methods are not added or advertised in T-103.

## 8. Data, State, and Security Impact

| Topic | Impact | Handling |
|-------|--------|----------|
| SQLite | high | ordered v4 transaction; real v3 fixture; rollback injection |
| Effective config | high | immutable, one/session, deterministic hash, safe snapshots |
| Runtime recovery | high | atomic hints/checkpoint facts; T-102 verifies live evidence |
| Settings concurrency | medium | revision compare-and-set |
| Lineage | medium | parent remains canonical; idempotent safe audit only |
| Remote/update | high future impact | constrained metadata only, no workflow claims |
| Credential | high | references only; schema/fixture/result secret scans |
| Output | high volume | bytes/grid/blob outside SQLite |
| Consumer dependencies | high | remove app/CLI storage dependency and fallback |

## 9. TDD Strategy

The executable order is strict:

1. **RED/GREEN foundation**: preserve baseline, add isolated migration/config RED, implement v4,
   then publish the effective-config repository milestone.
2. **RED/GREEN continuation**: add recovery RED -> GREEN, then metadata RED -> GREEN. Each
   compile-fail-capable contract lives in its own integration test binary and every packet starts
   from a clean commit.
3. **Shared integration**: after the T-102 shared release, wire proto/runtime/client/app/CLI.
4. **REFACTOR**: remove raw connection/direct-open/fallback paths and deduplicate only touched
   code while tests remain green.
5. **EVIDENCE**: focused, cross-process, dependency, security, review, and release-readiness gates.

Each task in `tasks.md` is sized for one TraeCLI execution. A task cannot skip its predecessor
phase or share an exclusive file owner concurrently.

Cross-change critical path:

```text
T-102 G3 resolved launch/effective-config contract
  -> T-103 S103-GREEN-02 v4 repository
  -> T-103 repository GREEN handoff
  -> T-102 G5 manifest spawn integration
  -> T-102 shared-file release
  -> T-103 shared proto/runtime/client integration
```

`S103-GREEN-02` waits only for the T-102 G3 contract handoff, not for all of T-102. Migration/config
RED and `S103-GREEN-01` can run in parallel with T-102. Recovery/metadata RED are committed only
after the H-02 milestone, so that milestone remains fully green and directly mergeable by T-102.
T-103 shared integration waits for T-102's exact shared-file release, so the graph has no cycle.

## 10. Test Strategy

| Layer | Required cases | Command/evidence |
|-------|----------------|------------------|
| Baseline | all current storage test binaries and counts | `cargo test -p homie-storage --tests` |
| Migration | empty/v3/rollback/idempotent/too-new | focused storage integration test |
| Repository | settings revision, config freeze, recovery, lineage/remote/update idempotency | focused storage integration test |
| Proto | serde fixtures, unknown version/phase, safe payload | `cargo test -p homie-proto --tests` |
| Runtime/client | handler capability, restart recovery, no partial commit | focused runtime/client tests |
| App | settings load/save via bridge and no storage dependency | app tests + cargo tree/source scan |
| CLI | doctor/usage via client and no storage dependency | CLI E2E + cargo tree/source scan |
| Security | no secret fields/content in schema/fixtures/results | hook + explicit scan |
| Regression | existing history/worktree/usage/session tests | storage/runtime/client/app/CLI suites |

## 11. Release Gates

- OpenSpec status reports `4/4`; strict validate passes.
- 16-dimension spec review has no unresolved blocking issue.
- `cargo test -p homie-storage --tests` baseline and final suites pass.
- v4 migration matrix and repository RED/GREEN evidence exist.
- runtime restart/effective config/recovery integration passes.
- app and CLI normal dependency trees contain no `homie-storage`.
- app/CLI source contains no production direct storage open/fallback.
- shared method capability discovery includes only executable handlers.
- format/check/clippy/test/security/diff gates are recorded with actual statuses.
- downstream parity rows are not closed by foundation-only evidence.
- Bead `homie-t3u.2` closes only after implementation evidence, not after this spec phase.

# T-103 Delegation Plan

> Change ID: `diri-storage-core-facts`
> Bead: `homie-t3u.2`
> Parallel dependency: T-102 / `homie-t3u.1`

## 1. Purpose

本计划把 `tasks.md` 拆成可由单次 TraeCLI 完成的串行或并行 lane，并锁定共享文件 ownership。
核心约束：

1. `crates/homie-storage/src/lib.rs` 全程只有 `S103-storage-impl` 一个 implementation owner。
2. RED test owner 不编辑 production implementation。
3. T-102 释放 shared proto/runtime 文件前，T-103 只冻结合同，不修改这些产品文件。
4. 同一文件同一时刻只有一个 active owner；handoff 必须记录 base commit、dirty files 和已跑
   commands。
5. 本规格会话不修改产品代码，也不提交。

## 2. Contract Freeze

以下名称和语义在实现开始前冻结：

| Method | Required DTO/invariant |
|--------|------------------------|
| `storage.health` | schema/foreign-key/journal/safe identity；不声明 runtime live |
| `settings.get` | `SettingsSnapshot { preferences, revision }` |
| `settings.update` | expected revision；冲突不覆盖 |
| `usage.summary` | bounded filters；safe aggregate |
| `session.effective_config` | versioned immutable safe snapshot/hash；无 key material |
| `runtime.recovery.summary` | bounded candidates；persisted hint 与 verified live 分离 |

不冻结或提前实现：

- remote node/handoff workflow methods；
- updater workflow methods；
- 新的 lineage 同义 methods；复用现有 parent/children methods。

任何 field/method rename 必须先更新 PRD、design、capability spec 和 alignment mapping，再申请
shared-file owner；不能在实现中静默漂移。

## 3. Ownership Matrix

| Owner | Exclusive write scope | May run in parallel with | Must not edit |
|-------|-----------------------|--------------------------|---------------|
| `S103-spec` | 本 change PRD/OpenSpec、两个允许长期 specs | read-only research | 产品代码、parity、master tasks |
| `S103-storage-test` | new/focused storage integration tests and fixtures | spec/proto test | `homie-storage/src/lib.rs` |
| `S103-storage-impl` | `homie-storage/src/lib.rs` | non-storage-file test work | all shared proto/runtime/app/CLI files |
| `S103-proto-test` | focused proto tests | storage lanes after T-102 release | production proto |
| `S103-proto-integration` | exact proto method/model files | no other proto owner | runtime/client/app/CLI/storage lib |
| `S103-runtime-test` | focused runtime/client tests | storage implementation | production runtime |
| `S103-runtime-integration` | exact runtime actor/dispatcher/lifecycle integration files | app/CLI only after API stable | storage lib/proto/client/app/CLI |
| `S103-client-integration` | exact client source/tests | none touching client files | storage/runtime/app/CLI |
| `S103-app-test` | focused app tests/checks | storage/runtime lanes | app production code |
| `S103-app-integration` | app Cargo/main/runtime bridge and tests | CLI integration | storage/proto/runtime/client |
| `S103-cli-test` | focused CLI tests/checks | storage/runtime lanes | CLI production code |
| `S103-cli-integration` | CLI Cargo/source/tests | app integration | storage/proto/runtime/client |
| `S103-verification` | evidence documents/command logs | no source edits | all production code |
| `S103-review` | review evidence; fixes delegated to original owner | verification reads | unclaimed source files |

## 4. `homie-storage/src/lib.rs` Serial Queue

Only `S103-storage-impl` may execute this queue:

1. `S103-GREEN-01`: v4 migration.
2. `S103-GREEN-02`: effective config.
3. `S103-GREEN-03`: recovery facts/flush.
4. `S103-GREEN-04`: lineage/remote/update metadata.
5. `S103-REFACTOR-02`: proven local helper cleanup.

The owner completes and verifies one item before starting the next. It does not delegate partial
hunks. If interrupted, it records:

- current base and worktree status;
- exact modified ranges;
- last passing/failing test;
- whether migration constants or `SCHEMA_VERSION` are internally consistent.

No second owner takes over until the first owner explicitly releases the complete file.

## 5. T-102 Shared-File Gate

The storage prerequisite and the later shared-file gate are distinct:

```text
T-102 G3 contract handoff
  -> S103-GREEN-02 effective-config repository
  -> repository GREEN handoff to T-102 G5
  -> T-102 completes shared runtime/proto work
  -> T-102 releases exact shared files
  -> T-103 shared integration
```

`S103-GREEN-02` waits for T-102 G3 only. It does not wait for T-102 completion. T-102 never edits
`crates/homie-storage/src/lib.rs`; after GREEN-02, `S103-storage-impl` records the exact typed
repository handoff that unblocks T-102 G5.

Before `S103-RED-06`, `S103-RED-07`, `S103-GREEN-05`, or `S103-GREEN-06`:

1. Query Bead `homie-t3u.1` and its active owner.
2. Record T-102 branch/worktree/base commit and dirty shared files.
3. Obtain explicit release of:
   - `homie-proto` method/model files;
   - `homie-runtime/src/runtime_actor.rs`;
   - `homie-runtime/src/dispatcher.rs`;
   - any holder/adoption/lifecycle file required by recovery integration.
4. Merge the recorded exact shared-release SHA with `git merge --no-ff`; never rebase or
   cherry-pick a shared milestone. Then rerun T-102 focused tests.
5. Compare T-102 effective-config/holder observation shapes to this contract freeze.
6. If incompatible, mark integration tasks blocked and update specs before product edits.

Migration/config RED and `S103-GREEN-01` may proceed before the G3 handoff. `S103-GREEN-02`
publishes the fully green H-02 milestone. Recovery RED/GREEN and metadata RED/GREEN then run as
clean committed packets behind that milestone. Shared proto/runtime integration waits for the later
exact-file release because its files overlap T-102.

## 6. Execution Waves

### Wave A: Spec, migration, and config

- `S103-SPEC-01..04`
- `S103-RED-01..03`
- `S103-GREEN-01`

Each RED uses an independent integration test binary. `S103-GREEN-01` may run while T-102 G3 is
active, after migration RED is committed.

### Wave B: Repository milestone and remaining storage

- `S103-GREEN-02` -> H-02 milestone
- `S103-RED-04` -> `S103-GREEN-03`
- `S103-RED-05` -> `S103-GREEN-04`

Parallelism: none inside `lib.rs`; test owner may inspect failures without editing implementation.
`S103-GREEN-02` starts only after the G3 contract handoff and must publish its repository GREEN
handoff before T-102 G5 starts. Recovery and metadata RED are created only after that milestone, so
T-102 never imports intentionally failing tests.

### Wave C: Shared Contract Integration

- T-102 shared-file gate
- `S103-RED-06..07`
- `S103-GREEN-05..07`

Order is proto -> runtime -> client. Capability discovery updates only after runtime handler tests
pass.

### Wave D: Consumer Removal

- `S103-RED-08..09`
- `S103-GREEN-08..09`

App and CLI integration may run in parallel after client API stability because their file sets do
not overlap. Both must delete dependencies and direct-open fallback, not add dual paths.

### Wave E: Refactor and Evidence

- `S103-GREEN-10`
- `S103-REFACTOR-01..03`
- `S103-EVIDENCE-01..07`

Fixes discovered by review return to the original exclusive owner; reviewers do not make
cross-owned edits.

## 7. Per-TraeCLI Handoff Contract

Every delegated prompt SHALL include:

- change id, Bead, task id, owner id;
- absolute allowed write paths;
- forbidden paths;
- exact predecessor evidence to read;
- one RED or GREEN outcome;
- commands to run;
- requirement/scenario ids covered;
- explicit no-commit rule unless the master owner changes it;
- instruction to stop and report if a file has unowned concurrent changes.

Every completion SHALL report:

- changed files;
- test commands and actual counts/status;
- unresolved failure/blocker;
- next owner handoff data;
- `git diff --check` result;
- confirmation that no forbidden file changed.

## 8. Blocker Policy

Mark a task blocked immediately when:

- T-102 still owns a required shared file;
- a frozen DTO cannot represent T-102's resolved config/holder observation without semantic loss;
- a real v3 fixture cannot migrate atomically;
- a required repository would store forbidden secret/raw payload data;
- app/CLI service removal would require a direct-storage fallback;
- unrelated dirty changes make exclusive ownership ambiguous.

A blocker report names the task, file, current owner, evidence, and smallest decision required. It
must not be bypassed with compatibility code or unverified status claims.

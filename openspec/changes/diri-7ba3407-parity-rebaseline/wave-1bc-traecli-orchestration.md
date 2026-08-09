# Wave 1B/1C TraeCLI Orchestration

```yaml
parent_change_id: diri-7ba3407-parity-rebaseline
baseline_commit: 7ba3407
checkpoint_commit: 48f522b
t102:
  change_id: diri-agent-session-runtime
  bead: homie-t3u.1
t103:
  change_id: diri-storage-core-facts
  bead: homie-t3u.2
status: task_packets_ready
product_implementation_started: false
```

## 1. 执行目标

本文件是 T-102/T-103 的跨 change 调度入口。Child change 内的需求、测试和单任务 prompt
分别以以下文件为准：

- `openspec/changes/diri-agent-session-runtime/traecli-task-packets.md`
- `openspec/changes/diri-storage-core-facts/traecli-task-packets.md`

本文件只决定：

- worktree/branch；
- 跨 change 依赖；
- milestone commit handoff；
- 并发上限和文件 ownership；
- 最终集成、失败和回滚策略。

## 2. 当前事实

- 本地 checkpoint commit：`48f522b`。
- T-102/T-103 规格已批准，当前规格改动尚未提交。
- 两个 OpenSpec 均为 4/4 complete 且 strict-valid。
- parity lock 当前有 39 个 `partial` row。
- runtime focused baseline：14 tests，12 passed，2 failed。
- storage focused baseline：全部通过。
- `diri/` 是主工作区中固定在 `7ba3407` 的独立只读参考仓库，不进入 Homie Git tree。

## 3. Worktree 拓扑

| Role | Absolute path | Branch | Write policy |
|------|---------------|--------|--------------|
| Coordinator/integration | `/Users/bytedance/workspace/github/homie` | `feature/diri-runtime-daemon-client-transport` | 规格、合并、最终 evidence；不并行写产品代码 |
| T-102 | `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime` | `wave1b/diri-agent-session-runtime` | 仅 T-102 packets |
| T-103 | `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts` | `wave1c/diri-storage-core-facts` | 仅 T-103 packets |

Worktree 必须从同一个已提交规格基线创建。不得从旧 checkpoint `48f522b` 直接创建，因为
它不含本轮批准的 PRD/OpenSpec。

每个 child worktree 创建后，可建立 ignored read-only symlink：

```text
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/diri
  -> /Users/bytedance/workspace/github/homie/diri

/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/diri
  -> /Users/bytedance/workspace/github/homie/diri
```

symlink 只用于读取固定 baseline，不得修改 reference checkout。

## 4. Coordinator Preflight Packets

### C-00：提交规格基线

**Allowed write:** Git index/commit only。
**Forbidden:** 产品代码修改、push、amend、force。

执行前验证：

```text
cd /Users/bytedance/workspace/github/homie
git diff --check
openspec validate diri-agent-session-runtime --strict
openspec validate diri-storage-core-facts --strict
make parity-lock
```

只 stage 本轮治理、PRD、OpenSpec、component spec 和 spec-review evidence。确认 staged diff
不含 `crates/`、`Cargo.toml`、`Cargo.lock`、`Makefile`、`scripts/` 产品改动后，运行：

```text
/Users/bytedance/workspace/github/homie/.githooks/pre-commit
```

hook 通过后创建一个本地、不 push 的 conventional docs commit。记录 commit SHA 为后续
worktree base。

### C-01：创建隔离 worktree

从 C-00 commit 创建上表两个 branch/worktree。创建前确认目标路径不存在、branch 不存在。
禁止删除或重置现有 worktree。创建后分别运行：

```text
git status --short --branch
openspec validate diri-agent-session-runtime --strict
openspec validate diri-storage-core-facts --strict
make parity-lock
```

再创建只读 `diri` symlink，并验证 reference HEAD 是
`7ba3407758e10cf6ff25d74b14a8b7746bd23aa2`。

### C-02：记录初始 handoff

在两个 Bead notes 记录：

- spec base commit SHA；
- branch；
- absolute worktree；
- active packet；
- initial dirty status；
- reference Diri SHA。

## 5. 跨 Change 关键路径

```text
T-102 R/G independent foundations ─┐
                                  ├─ T-102 G3 contract handoff
T-103 RED + GREEN-01 ─────────────┘
          |
          v
T-103 S103-GREEN-02 effective-config repository milestone
          |
          | exact commit merge into T-102
          v
T-102 G5 manifest spawn -> status/governor/recovery/shutdown
          |
          | T-103 GREEN-03/04 may continue in parallel
          v
T-102 shared-release milestone
          |
          | exact commit merge into T-103
          v
T-103 proto/runtime/client -> app/CLI direct-storage removal
          |
          v
T-103 final branch contains both completed histories
          |
          v
Coordinator final integration and gates
```

## 6. Milestone Merge Protocol

### H-01：T-102 G3 -> T-103 contract handoff

T-102 G3 完成后提交一个单独 commit，报告：

- exact commit SHA；
- `ResolvedEffectiveAgentConfig` 字段和安全约束；
- no-secret scan；
- agent tests；
- 不包含 storage 改动。

T-103 通过 `git show <recorded-sha>` 读取合同。此时不 merge T-102 branch，避免把未完成
runtime 工作带入 storage lane。

### H-02：T-103 repository GREEN -> T-102

T-103 完成 `S103-GREEN-01` 和 `S103-GREEN-02` 后创建 milestone commit。该 commit 只能
包含：

- v4 migration；
- effective-config repository；
- focused storage tests；
- 必要规格/evidence handoff。

T-102 worktree 使用 `git merge --no-ff <recorded-milestone-sha>` 合入 exact commit，并运行：

```text
cargo test -p homie-storage --tests
cargo test -p homie-agents
git diff --check
```

禁止 cherry-pick。Merge 失败时不手工选边；由 T-103 storage owner 解释 storage conflict，
由 T-102 G3 owner 解释 contract conflict。

### H-03：T-102 shared release -> T-103

T-102 完成 G5-G11、focused refactor 和 required focused gates 后创建 shared-release
milestone commit。它必须：

- 包含 H-02 的真实祖先；
- 不修改 T-103-owned storage file；
- runtime/proto/client shared files clean；
- current T-102 tests和 cleanup gates通过。

T-103 worktree 使用 `git merge --no-ff <recorded-shared-release-sha>`。随后先运行 T-102
focused regression，再开始 T-103 shared proto/runtime/client integration。

禁止 cherry-pick；禁止在 merge 过程中顺便改需求。

### H-04：Final integration

T-103 final branch 应包含：

- T-103 storage history；
- H-03 的 T-102 complete history；
- T-103 shared integration、app/CLI removal 和 evidence。

Coordinator 只需 merge T-103 final branch。若 T-102 branch 存在未被 T-103 ancestry 包含
的 commit，则 final integration 立即 blocked，先修复 ancestry，不重复 cherry-pick。

## 7. 波次与并发

| Wave | Runnable packets | Max concurrency | Gate |
|------|------------------|-----------------|------|
| P | C-00 -> C-01 -> C-02 | 1 | 两个 clean worktree |
| A | T102 R1/R3/R4；T103 RED-01..03 | 4 | RED 精确、独立 test binary、无进程泄漏 |
| B | T102 R2/R5/R6、G1/G2/G3/G8；T103 GREEN-01 | 4 | G3 contract + migration GREEN |
| C | T103 GREEN-02 -> H-02 | 1 storage owner | repository handoff可被 T-102 消费 |
| D | T102 G5-G11；T103 RED-04 -> GREEN-03 -> RED-05 -> GREEN-04 | 2 | T-102 shared release |
| E | H-03；T103 RED/GREEN shared integration | 1 shared owner at a time | app/CLI 无 direct storage |
| F | 两边 refactor、focused E2E、review/evidence | 2 only for disjoint evidence | child release readiness |
| G | H-04 + full integration gates | 1 | merged release candidate |

`runtime_actor.rs`、`dispatcher.rs`、`homie-proto` 和 `homie-client` 在 Wave E 前后只能有一个
active owner。`homie-storage/src/lib.rs` 全程只属于 T-103 storage owner。

## 8. Commit Contract

每个 TraeCLI packet：

1. 从 clean worktree 开始。
2. 只修改 allowed paths。
3. 先证明 RED，再写 GREEN。
4. 运行 packet 指定命令、`git diff --check` 和 repository secret hook。
5. 只有 packet status 为 pass 才创建一个 conventional commit。
6. 不 amend、不 rebase 已共享 milestone、不 push。
7. completion report 必须包含 commit SHA、命令/exit/count、dirty status、cleanup 和 handoff。

发现其他 owner 的 dirty changes 时立即 blocked，不 stash、不 reset、不 checkout 覆盖。

## 9. Failure Policy

| Failure | Action |
|---------|--------|
| RED 未按预期失败 | 停止；修规格/测试假设，不写 GREEN |
| 需要越权文件 | blocked；返回 coordinator 重排 owner |
| T-103 repository 无法表达 G3 contract | 两边 blocked；cross-spec review |
| process fixture residual 非零 | fail；只清 fixture ledger，禁止全局 kill |
| shared merge conflict | 原 owner解释并修复；集成人员不猜语义 |
| strict/parity/security gate 失败 | 不创建 milestone、不更新 Bead完成状态 |
| external release credential缺失 | 只影响后续 release wave，不伪造当前 pass |

## 10. Final Gates

Child gates 完成后，Coordinator 至少运行并记录：

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
openspec validate diri-agent-session-runtime --strict
openspec validate diri-storage-core-facts --strict
make parity-lock
/Users/bytedance/workspace/github/homie/.githooks/pre-commit
git diff --check
```

另需执行 child task packets 中的真实 daemon/holder E2E、migration rollback、app settings、
CLI doctor/usage、依赖树和进程泄漏门禁。

只有 evidence 与 delivered state 一致时，才更新 parity rows、master tasks 和 Beads。

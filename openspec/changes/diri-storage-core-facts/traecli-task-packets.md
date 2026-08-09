# T-103 TraeCLI Task Packets

```yaml
change_id: diri-storage-core-facts
bead: homie-t3u.2
master_task: T-103
worktree: /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts
branch: wave1c/diri-storage-core-facts
packet_count: 29
status: ready_for_dispatch
implementation_started: false
```

## 1. 调度总则

本文件是最终可直接投喂 TraeCLI 的执行包，不是实施记录。生成本文件时不得启动任何 packet。
调度器必须按 packet 依赖和外部门禁逐包投喂；不得把多个 packet 合并为一次无边界执行。

所有 packet 共同遵守：

1. 固定 worktree 为
   `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`，固定 branch 为
   `wave1c/diri-storage-core-facts`。每个 packet 必须从 clean worktree 开始并以 clean
   committed handoff 结束。目录不存在、branch 不匹配或存在未交接改动时立即 `BLOCKED`。
2. 不新增或依赖环境变量配置。命令、worktree、读写文件、fixture 和 evidence 路径使用绝对
   路径。不得用环境变量覆盖 data dir、manifest、holder、timeout 或测试模式。
3. 严格 TDD：先运行并记录 packet 的 RED，再做最小改动，再运行 GREEN 和保留回归。不得弱化
   断言、删除既有测试、增加 compatibility fallback 或双路径。
4. 只写 packet 列出的 allowed write paths。所有其他路径均为 forbidden；发现确需越界时停止
   并报告最小规格决策，不得自行扩 scope。
5. `crates/homie-storage/src/lib.rs` 的唯一 owner 是 `S103-storage-impl`。只有
   `T103-P10`、`T103-P11`、`T103-P12`、`T103-P13`、`T103-P21` 可按串行队列编辑该文件。
6. 不使用 `git reset --hard`、`git checkout --`、`git clean`、`git stash`、`pkill`、按进程
   名清理或其他破坏性命令。只清理 packet 自己创建且能以绝对路径/PID/启动时间证明归属的资源。
7. PASS 后按 packet 的 commit contract stage allowed paths，并运行
   `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/.githooks/pre-commit`
   后提交；不得 amend、rebase、push 或提交 forbidden path。FAIL/BLOCKED 不提交，保留现场
   并报告。
8. 每次 handoff 必须报告：packet/task/owner、base SHA、result SHA、changed files、RED/GREEN
   命令及实际状态/计数、cleanup、未解决 blocker、`git diff --check`、forbidden path 零改动。

实现前必须完整读取：

- `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/AGENTS.md`
- `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/prd-spec/features/diri-storage-core-facts/2026-08-09-diri-storage-core-facts-design.md`
- `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/openspec/changes/diri-storage-core-facts/design.md`
- `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/openspec/changes/diri-storage-core-facts/plan.md`
- `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/openspec/changes/diri-storage-core-facts/tasks.md`
- `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/openspec/changes/diri-storage-core-facts/alignment-report.md`
- `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/openspec/changes/diri-storage-core-facts/delegation-plan.md`

## 2. Cross Handoff Contract

无环顺序固定为：

```text
T103-P01 -> T103-P02 -> T103-P03
T103-P02 -> T103-P10
T103-P03 + T103-P10 + T-102 G3 contract
  -> T103-P11 / S103-GREEN-02
  -> H-02 effective-config repository milestone tip
  -> T-102 G5 consumes that milestone
T103-P11 -> T103-P04 -> T103-P12 -> T103-P05 -> T103-P13
  -> T-102 completes and publishes shared-release milestone
  -> merge T-102 shared-release milestone into wave1c/diri-storage-core-facts
  -> T103-P06/P07/P14/P15/P16 shared proto/runtime/client integration
```

门禁细则：

- `T103-P01..P03` 和 `T103-P10` 不等待 T-102。四类 storage RED 分别位于
  `ordered_v4_migration.rs`、`effective_config_facts.rs`、`runtime_recovery_facts.rs`、
  `durable_metadata_foundation.rs`；typed repository test 可以合法 compile-fail，但只影响其
  focused cargo target。
- `T103-P11` 等待 P03、P10 和 T-102 G3 contract handoff，不等待 T-102 G5 或 T-102 完成。
  G3 handoff
  必须给出 40 位 commit SHA、`ResolvedEffectiveAgentConfig` 的绝对 owner path、完整字段/
  类型、safe/redaction 规则、已跑命令和 dirty-file 清单。
- `T103-P11` PASS 后必须创建并发布 milestone commit，commit subject 固定为
  `feat(storage): publish effective config repository milestone`。handoff 必须给 T-102 G5
  提供 40 位 tip SHA、公开类型/方法签名、transaction/rollback 语义、测试命令和结果。该 tip
  ancestry 必须包含 P10/P11，且
  `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests`
  全绿；T-102 只允许 `git merge --no-ff` 此 exact SHA。
- P04 在 P11 milestone 后创建并提交独立 recovery RED；P12 使其 GREEN。P05 在 P12 后创建并
  提交独立 metadata RED；P13 使其 GREEN。P12/P13 不等待 T-102 消费、G5 或 shared release。
- T-102 shared-release handoff 必须给出 40 位 commit SHA、branch/worktree/base、released
  proto/runtime/client/lifecycle 文件绝对路径、dirty-file 清单和 focused tests。调度器必须先把
  该 milestone 以 `git merge --no-ff` 合并到 T-103 branch，禁止 cherry-pick；先重跑 T-102
  storage/agents/runtime/proto/client regressions。只有该 SHA 已是 T-103 `HEAD` 的 ancestor、
  regressions 全绿且 released 文件无并发 owner 时，才能投喂 `T103-P06`、`T103-P07`、
  `T103-P14`、`T103-P15`、`T103-P16`。
- 任何 shape 不兼容、SHA 未记录、milestone 未合并或 shared file 未释放都必须 `BLOCKED`；
  不得用 adapter、fallback、复制 DTO 或 T-103 反向编辑 T-102 owner 文件绕过。

## 3. Packet Index

| Packet | OpenSpec task | Depends on | External gate |
|--------|---------------|------------|---------------|
| `T103-P01` | `S103-RED-01` | SPEC approved | none |
| `T103-P02` | `S103-RED-02` | `T103-P01` | none |
| `T103-P03` | `S103-RED-03` | `T103-P02` | none |
| `T103-P10` | `S103-GREEN-01` | `T103-P02` | none |
| `T103-P11` | `S103-GREEN-02` | `T103-P03`, `T103-P10` | T-102 G3 contract |
| `T103-P04` | `S103-RED-04` | `T103-P11` | H-02 milestone published |
| `T103-P12` | `S103-GREEN-03` | `T103-P04`, `T103-P11` | none; do not wait for T-102 |
| `T103-P05` | `S103-RED-05` | `T103-P12` | none |
| `T103-P13` | `S103-GREEN-04` | `T103-P05`, `T103-P12` | none; do not wait for T-102 |
| `T103-P06` | `S103-RED-06` | SPEC approved | merged T-102 shared-release |
| `T103-P07` | `S103-RED-07` | `T103-P06` | merged T-102 shared-release |
| `T103-P08` | `S103-RED-08` | `T103-P07` | none |
| `T103-P09` | `S103-RED-09` | `T103-P07` | none |
| `T103-P14` | `S103-GREEN-05` | `T103-P06`, `T103-P13` | merged T-102 shared-release |
| `T103-P15` | `S103-GREEN-06` | `T103-P07`, `T103-P14` | merged T-102 shared-release |
| `T103-P16` | `S103-GREEN-07` | `T103-P15` | merged T-102 shared-release |
| `T103-P17` | `S103-GREEN-08` | `T103-P08`, `T103-P16` | none |
| `T103-P18` | `S103-GREEN-09` | `T103-P09`, `T103-P16` | none |
| `T103-P19` | `S103-GREEN-10` | `T103-P17`, `T103-P18` | none |
| `T103-P20` | `S103-REFACTOR-01` | `T103-P19` | none |
| `T103-P21` | `S103-REFACTOR-02` | `T103-P20` | storage owner only |
| `T103-P22` | `S103-REFACTOR-03` | `T103-P21` | none |
| `T103-P23` | `S103-EVIDENCE-01` | `T103-P22` | none |
| `T103-P24` | `S103-EVIDENCE-02` | `T103-P23` | none |
| `T103-P25` | `S103-EVIDENCE-03` | `T103-P23` | none |
| `T103-P26` | `S103-EVIDENCE-04` | `T103-P24`, `T103-P25` | none |
| `T103-P27` | `S103-EVIDENCE-05` | `T103-P26` | none |
| `T103-P28` | `S103-EVIDENCE-06` | `T103-P27` | none |
| `T103-P29` | `S103-EVIDENCE-07` | `T103-P28` | release readiness pass |

## 4. RED Packets

### T103-P01 / S103-RED-01

- **Packet ID:** `T103-P01`
- **依赖:** 已批准 SPEC；Bead `homie-t3u.2` 为 `IN_PROGRESS`
- **优先级:** `P0`
- **Owner:** `S103-storage-test`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/storage-baseline.md`
- **Forbidden paths:** 除上述 evidence 文件外的全部路径，尤其
  `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/**`
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests`
- **GREEN command:**
  - 重跑同一命令；两次都必须保持既有全绿，记录每个 test binary 实际计数和 schema version 3
- **Deadline:** 启动后 `45m`；单次 cargo command `10m`
- **Cleanup:** 不创建产品 fixture/process；删除仅由本 packet 创建的临时日志，保留 evidence
- **Expected result:** 基线 evidence 真实记录 `0 + 6 + 4 + 2 + 5 + 2 + 2` 或当前实际计数；
  若计数漂移，记录实际值和 diff，不伪造通过
- **Handoff/commit contract:** PASS 后只提交 allowed evidence，commit subject
  `test(storage): record T-103 schema v3 baseline`；不 push

**完整可复制 prompt：**

```text
执行 T103-P01 / S103-RED-01。change_id=diri-storage-core-facts，Bead=homie-t3u.2，
owner=S103-storage-test，priority=P0。固定 worktree：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts；固定 branch：
wave1c/diri-storage-core-facts；deadline=45m，cargo timeout=10m。

先完整读取 AGENTS.md、T-103 PRD、design、plan、tasks、alignment-report、delegation-plan，路径均
位于上述 worktree 对应绝对路径。运行 /usr/bin/git -C
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts rev-parse --show-toplevel、
/usr/bin/git -C /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts branch
--show-current 和 /usr/bin/git -C
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts status --short，确认
worktree/branch 精确匹配；allowed path 有未交接改动则 BLOCKED。不得新增环境变量配置，
不得启动实现。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/storage-baseline.md
其余全部 forbidden，尤其 crates/**。

运行 RED/baseline：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests
记录每个 test binary 的名称、passed/failed/ignored 和 schema version 3 证据。再原样重跑作为
稳定性 GREEN；两次都应保持既有基线全绿。计数与 PRD 不同则记录实际值并 FAIL/BLOCKED，
不得改测试或产品代码。

evidence 必须包含 base SHA、时间、完整命令、退出码、实际计数、schema version、结论和无产品
改动声明。运行 /usr/bin/git -C
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts diff --check 和
/usr/bin/git -C /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts status
--short。PASS 后
只 stage allowed evidence，提交 subject：
test(storage): record T-103 schema v3 baseline
不得 amend/rebase/push。报告 result SHA、changed files、命令状态、cleanup、blocker、
diff-check 和 forbidden path 零改动。
```

### T103-P02 / S103-RED-02

- **Packet ID:** `T103-P02`
- **依赖:** `T103-P01` PASS
- **优先级:** `P0`
- **Owner:** `S103-storage-test`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/tests/ordered_v4_migration.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/tests/fixtures/diri_storage_core_facts/**`
- **Forbidden paths:** `crates/homie-storage/src/lib.rs`、所有其他 source/test/doc/Cargo 文件
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration -- --nocapture`
- **GREEN command:**
  - 同一 focused command，供 `T103-P10` 使用；本 packet 不实现 GREEN
  - 保留回归：`/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --lib --test diri_storage_indexing --test local_basic_v1 --test reference_parity_schema --test storage_bootstrap --test usage_scan_cache --test usage_transcript_import`
- **Deadline:** `3h`；focused command `10m`
- **Cleanup:** fixture 使用 test-owned temp dir；真实 v3 fixture 如落盘必须确定性、无凭据并记录
  SHA256；无进程资源
- **Expected result:** empty `[1,2,3,4]`、v3 `[4]`、repeat `[]`、故障 rollback、too-new
  用例以 `ordered_v4_` 命名并只因缺少 v4 contract RED；既有 suites 仍绿
- **Handoff/commit contract:** RED 证据成立后提交 failing tests/fixture，subject
  `test(storage): add ordered v4 migration RED`；不提交实现

**完整可复制 prompt：**

```text
执行 T103-P02 / S103-RED-02，owner=S103-storage-test，priority=P0，依赖 T103-P01 PASS。
固定 worktree=/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，
branch=wave1c/diri-storage-core-facts，deadline=3h。先完整读取 AGENTS.md 与 T-103 PRD/OpenSpec/
tasks/delegation/alignment；做 branch/status/base preflight。不得新增环境变量配置。

只允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/tests/ordered_v4_migration.rs
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/tests/fixtures/diri_storage_core_facts/**
禁止编辑 crates/homie-storage/src/lib.rs、Cargo.toml 和所有其他路径。

严格 TDD。先确认 focused target 尚不存在或没有 ordered_v4_ 用例。添加可编译、行为失败的
ordered_v4_migration binary tests，覆盖 empty applied=[1,2,3,4]、真实 v3 fixture
applied=[4]、repeat=[]、
v4 中途故障后 DDL/DML/version row 全 rollback、schema-too-new fail closed。测试必须通过现有
test harness/schema inspection 到达行为断言，不引用尚不存在的 Rust symbol，确保后续 RED
packet 可独立运行。不得改写 v1-v3 migration 或既有断言。

RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration -- --nocapture
必须只因缺少 v4 contract 失败。保留回归：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --lib --test diri_storage_indexing --test local_basic_v1 --test reference_parity_schema --test storage_bootstrap --test usage_scan_cache --test usage_transcript_import
既有 test binaries 必须保持绿；独立新 target 的预期 RED 不阻塞其他 focused target。未来
GREEN 命令与 RED 相同，
由 T103-P10 执行，本 packet 不写实现。

fixture 必须确定性、无 secret；落盘 fixture 用 /usr/bin/shasum -a 256 记录摘要。清理仅限
test-owned temp paths。运行 git diff --check，确认 forbidden path 零改动。PASS（即预期 RED
被准确观察）后只 stage allowed paths，提交：
test(storage): add ordered v4 migration RED
不得 amend/rebase/push。handoff 报告 base/result SHA、changed files、实际 failure、既有回归、
cleanup 和 diff-check。
```

### T103-P03 / S103-RED-03

- **Packet ID:** `T103-P03`
- **依赖:** `T103-P02` PASS
- **优先级:** `P0`
- **Owner:** `S103-storage-test`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/tests/effective_config_facts.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/tests/fixtures/diri_storage_core_facts/**`
- **Forbidden paths:** `crates/homie-storage/src/lib.rs` 及所有其他路径
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test effective_config_facts -- --nocapture`
- **GREEN command:** 同一 command，供 `T103-P11` 使用；本 packet 不实现
- **Deadline:** `3h`
- **Cleanup:** 关闭 SQLite handle；只删除 test-owned temp DB；fixture 不含 raw/virtual key
- **Expected result:** 原子 session+parent+config bind、profile mutation 后 immutable readback、
  duplicate conflict、invalid reference/JSON/hash rollback、secret absence 均有独立 RED
- **Handoff/commit contract:** commit
  `test(storage): add effective config repository RED`

**完整可复制 prompt：**

```text
执行 T103-P03 / S103-RED-03，owner=S103-storage-test，priority=P0，依赖 T103-P02 PASS。
worktree 固定 /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，
branch 固定 wave1c/diri-storage-core-facts，deadline=3h。读取 AGENTS.md 和全部 T-103 批准
文档，做 branch/status/base preflight；不得新增环境变量配置。

只允许写 effective_config_facts.rs 和其 fixtures 目录的绝对路径；禁止
crates/homie-storage/src/lib.rs 及其他全部路径。沿用单一 storage test owner，不覆盖 P02。

添加独立 effective_config_facts binary RED tests：原子创建
session+parent+frozen config；profile 后续 mutation 不改变 readback；每 session 只允许一次
freeze；invalid foreign reference、unknown snapshot version、oversize/corrupt JSON、hash/bind
失败均整 transaction rollback；持久结果无 provider raw key、virtual key material、
Authorization、Cookie。当前 effective_agent_configs 表已存在，测试不得声称表缺失。typed
repository symbol 尚不存在时允许该独立 binary 合法 compile-fail，不得把 production public
Connection 设计成新领域 API，也不得影响 ordered_v4_migration target。

RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test effective_config_facts -- --nocapture
失败必须指向 freeze/readback/atomicity 缺口。未来 GREEN 使用相同命令，由 T103-P11 执行。
同时确认现有 storage test binaries 未被修改或弱化。

清理 test-owned temp DB/handle；运行 git diff --check 和 status，确认 forbidden path 零改动。
预期 RED 准确成立后只 stage allowed paths，提交：
test(storage): add effective config repository RED
不 amend/rebase/push。handoff 报告 base/result SHA、失败断言、changed files、cleanup、
diff-check；P10 可消费 P02，P11 仍需等待本 P03 和 T-102 G3。
```

## 5. GREEN Packets

### T103-P10 / S103-GREEN-01

- **Packet ID:** `T103-P10`
- **依赖:** `T103-P02` PASS
- **优先级:** `P0`
- **Owner:** `S103-storage-impl`，`homie-storage/src/lib.rs` 唯一 owner
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/src/lib.rs`
- **Forbidden paths:** storage tests/Cargo、全部 proto/runtime/client/app/CLI/docs；T-102 owner files
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration -- --nocapture`
- **GREEN command:**
  - 同一 focused command
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --lib --test diri_storage_indexing --test local_basic_v1 --test reference_parity_schema --test storage_bootstrap --test usage_scan_cache --test usage_transcript_import`
- **Deadline:** `4h`；每个 cargo command `10m`
- **Cleanup:** 关闭 DB；删除 packet test-owned temp DB；无 process/network
- **Expected result:** `SCHEMA_VERSION=4`，空库 `[1,2,3,4]`、真实 v3 `[4]`、repeat `[]`、
  fault rollback、too-new 全绿；v1-v3 语义不改
- **Handoff/commit contract:** commit `feat(storage): add ordered schema v4 migration`；继续将
  `lib.rs` 独占 ownership 交给 `T103-P11`

**完整可复制 prompt：**

```text
执行 T103-P10 / S103-GREEN-01，owner=S103-storage-impl，priority=P0，依赖 T103-P02 PASS。
固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts，deadline=4h，每条 cargo timeout=10m。完整读取 AGENTS.md、
T-103 PRD/OpenSpec/tasks/delegation/alignment，做 branch/status/base preflight。确认 worktree
clean 且没有其他 owner 编辑 crates/homie-storage/src/lib.rs；否则 BLOCKED。不得新增环境变量
配置。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/src/lib.rs
禁止 tests、Cargo、proto/runtime/client/app/CLI/docs 和 T-102 owner 文件。

先运行 RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration -- --nocapture
确认失败仍只指向缺少 v4。然后最小实现 ordered v4：保留 v1/v2/v3 migration 常量和语义，追加
SCHEMA_VERSION=4；同一 transaction 内完成 preferences revision、effective config additions、
session_runtime_recovery、lineage audit、handoff extensions、update_receipts 的 DDL/backfill/
indexes 和 schema_migrations(4)。提供测试所需 deterministic fault injection，但不得引入
production env/test mode、downgrade、双写或 fallback。

GREEN：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --lib --test diri_storage_indexing --test local_basic_v1 --test reference_parity_schema --test storage_bootstrap --test usage_scan_cache --test usage_transcript_import
空库 applied=[1,2,3,4]，真实 v3=[4]，repeat=[]，fault rollback 无半升级，too-new fail closed。
不得顺手实现 P11-P13 repository 或修改 tests。

关闭 DB 并清理 packet-owned temp DB。运行 git diff --check/status，确认本 packet 只改
lib.rs。PASS 后只 stage lib.rs，运行绝对 pre-commit hook，提交：
feat(storage): add ordered schema v4 migration
不得 amend/rebase/push。handoff 报告 base/result SHA、RED/GREEN 实际计数、migration
invariants、cleanup、diff-check，并将 lib.rs 独占 ownership 串行交给 T103-P11。
```

### T103-P11 / S103-GREEN-02

- **Packet ID:** `T103-P11`
- **依赖:** `T103-P03`、`T103-P10` PASS
- **外部门禁:** 仅 T-102 G3 `ResolvedEffectiveAgentConfig` contract handoff；不等待 T-102 完成
- **优先级:** `P0`
- **Owner:** `S103-storage-impl`，唯一 `lib.rs` owner
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/src/lib.rs`
- **Forbidden paths:** T-102 G3 owner path、storage tests、所有 proto/runtime/client/app/CLI/docs
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test effective_config_facts -- --nocapture`
- **GREEN command:**
  - 同一 focused command
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration -- --nocapture`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests`
- **Deadline:** `4h`；cargo command `10m`
- **Cleanup:** 清理 test-owned DB；不得启动 agent/holder
- **Expected result:** bounded/versioned safe snapshot、deterministic hash、atomic session/parent/
  config bind、immutable by-session readback 全绿
- **Handoff/commit contract:** 提交 H-02 milestone tip，subject 精确为
  `feat(storage): publish effective config repository milestone`；立即把 40 位 SHA/API 签名/
  transaction 语义和全绿 `homie-storage --tests` 交给 T-102 G5；T-102 只 exact merge 此 tip

**完整可复制 prompt：**

```text
执行 T103-P11 / S103-GREEN-02，owner=S103-storage-impl，priority=P0，依赖 T103-P03 和
T103-P10 PASS。
固定 worktree=/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，
branch=wave1c/diri-storage-core-facts，deadline=4h，cargo timeout=10m。读取 AGENTS.md、全部
T-103 approved docs，以及 T-102 G3 handoff。运行 /opt/homebrew/bin/bd show homie-t3u.1
--long，确认 G3 handoff 给出 40 位 commit SHA、ResolvedEffectiveAgentConfig owner 绝对路径、
完整字段/类型、safe/redaction 规则、commands 和 dirty files。这里只等待 G3 contract，不等待
T-102 G5、shared release 或 T-102 完成。用 /usr/bin/git -C
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts show --stat 加该实际 40
位 SHA 只读核对 G3 contract，不 merge T-102 branch。handoff 缺失/语义损失则 BLOCKED。
确认 worktree clean。禁止环境变量配置。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/src/lib.rs
禁止编辑 G3 owner path、storage tests、proto/runtime/client/app/CLI/docs。保持 lib.rs 唯一
owner。

先运行 RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test effective_config_facts -- --nocapture
然后最小实现 typed freeze/readback repository：versioned bounded runtime/managed-LLM/permission
safe snapshots；profile/runtime/LLM/provider/permission IDs、skill/MCP/workspace scope；
virtual_key_id 只能是引用；deterministic config_hash；session+parent+config 单 transaction
bind；每 session 最多一个 frozen config；无 update API；按 session safe readback；profile
mutation 不影响历史 snapshot；unknown version、oversize/corrupt JSON、invalid reference/
hash/bind fail closed 且不留半 row。不得持久化 raw/virtual key、Authorization、Cookie。

GREEN：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test effective_config_facts -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests
不得编辑 T-102 或 shared proto/runtime 文件。

清理 test-owned DB；运行 git diff --check/status。PASS 后只 stage lib.rs，运行绝对
pre-commit hook，提交 H-02 milestone tip：
feat(storage): publish effective config repository milestone
立即记录该 40 位 tip SHA，并向 master/T-102 G5 handoff：milestone SHA、base SHA、所有公开
type/method signature、hash canonicalization、transaction/rollback、RED/GREEN 命令实际结果、
完整 `cargo test -p homie-storage --tests` 全绿计数、changed file、cleanup、diff-check。确认
tip ancestry 包含 P10/P11，T-102 必须 `git merge --no-ff` 此 exact SHA、禁止 cherry-pick，
不得编辑 storage。不得 amend/rebase/push；packet 结束时 worktree clean。
```

### T103-P04 / S103-RED-04

- **Packet ID:** `T103-P04`
- **依赖:** `T103-P11` PASS；H-02 milestone tip 已发布
- **优先级:** `P0`
- **Owner:** `S103-storage-test`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/tests/runtime_recovery_facts.rs`
- **Forbidden paths:** `crates/homie-storage/src/lib.rs`、所有其他 source/test/doc/Cargo 文件
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test runtime_recovery_facts -- --nocapture`
- **GREEN command:** 同一 focused command，供 `T103-P12` 使用
- **Deadline:** `3h`；focused command `10m`
- **Cleanup:** 删除 test-owned temp DB；不启动 holder/process
- **Expected result:** checkpoint 多字段原子性、deterministic bounded candidates、invalid
  offset fail closed、reopen readback、失败保留旧 row；PID/status 只作为 hint
- **Handoff/commit contract:** 预期 RED 成立后提交独立 binary，commit
  `test(storage): add runtime recovery facts RED`；packet 结束时 worktree clean

**完整可复制 prompt：**

```text
执行 T103-P04 / S103-RED-04，owner=S103-storage-test，priority=P0，依赖 T103-P11 PASS 且
H-02 milestone tip 已发布。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts，deadline=3h，focused timeout=10m。完整读取 AGENTS.md 与 T-103
approved docs，做 branch/status/base preflight；worktree 非 clean 则 BLOCKED。禁止环境变量配置。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/tests/runtime_recovery_facts.rs
禁止 lib.rs 和其他全部路径。

添加独立 runtime_recovery_facts binary RED：checkpoint path/output offset/content sequence/
event sequence/durable status 必须原子更新；candidate query 稳定排序并有 hard limit；
negative/overflow/checkpoint offset 超过 output tail fail closed；关闭并 reopen 后 facts 不变；
失败 transaction 保留旧 row；output bytes/grid/checkpoint blob 不进 SQLite。测试名称和断言
必须明确 holder PID、instance id、last status 是 hint，不得断言 storage row 证明 live。
typed repository symbol 尚不存在时允许该独立 binary 合法 compile-fail，不得影响
ordered_v4_migration 或 effective_config_facts target。

RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test runtime_recovery_facts -- --nocapture
失败只归因 recovery repository contract 缺失。未来 GREEN 同命令由 T103-P12 执行。

不得启动 holder/process；只清理 test-owned temp DB。运行 git diff --check/status，确认只改
allowed test file。预期 RED 成立后 stage allowed path，运行绝对 pre-commit hook，提交：
test(storage): add runtime recovery facts RED
不得 amend/rebase/push。handoff 报告 base/result SHA、actual RED、changed file、cleanup、
diff-check；packet 结束时 worktree clean。
```

### T103-P12 / S103-GREEN-03

- **Packet ID:** `T103-P12`
- **依赖:** `T103-P04`、`T103-P11` PASS
- **外部门禁:** 无；明确不等待 T-102 G5/shared release
- **优先级:** `P0`
- **Owner:** `S103-storage-impl`，唯一 `lib.rs` owner
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/src/lib.rs`
- **Forbidden paths:** storage tests、全部 shared proto/runtime/client、app/CLI/docs
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test runtime_recovery_facts -- --nocapture`
- **GREEN command:**
  - 同一 focused command
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests`
- **Deadline:** `4h`
- **Cleanup:** 关闭 DB、删除 packet temp DB；无 holder/process
- **Expected result:** atomic upsert/assessment、by-session read、stable bounded candidates、
  validation、storage-owned flush 全绿；bytes/grid/blob 不入 SQLite
- **Handoff/commit contract:** commit
  `feat(storage): add runtime recovery fact repository`；串行交给 `T103-P13`

**完整可复制 prompt：**

```text
执行 T103-P12 / S103-GREEN-03，owner=S103-storage-impl，priority=P0，依赖 T103-P04 和
T103-P11 PASS。固定 worktree=/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，
branch=wave1c/diri-storage-core-facts，deadline=4h。读取 AGENTS.md 与全部 T-103 approved docs，
做 branch/status/base preflight并确认 lib.rs 独占 ownership。此 packet 不等待 T-102 G5、
shared release 或完成；不得因 T-102 尚未消费 milestone 而阻塞。禁止环境变量配置。

唯一允许写 homie-storage/src/lib.rs 的绝对路径；storage tests、shared proto/runtime/client、
app/CLI/docs 全部 forbidden。

先运行 RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test runtime_recovery_facts -- --nocapture
最小实现 RuntimeRecoveryFacts typed repository：每 session 一 row metadata 与既有 session
output path/tail offset join；atomic upsert/assessment；按 session read；stable deterministic
bounded candidate list；holder instance/PID/start、output epoch、checkpoint path/offset/content
sequence、event sequence、runtime instance、durable status、updated time validation；negative/
overflow/tail 越界 fail closed；失败保留旧 row；storage-owned flush/checkpoint API。PID/status
始终为 hint，不能作为 live proof；SQLite 不保存 output bytes、terminal grid、checkpoint blob。

GREEN：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test runtime_recovery_facts -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests
不得接 runtime handler。

关闭 DB 并清理 packet temp DB；运行 git diff --check/status。PASS 后只 stage lib.rs，提交：
feat(storage): add runtime recovery fact repository
不 amend/rebase/push。handoff 报告 base/result SHA、RED/GREEN、public repository/flush API、
changed file、cleanup、diff-check，并串行交给 T103-P13。
```

### T103-P05 / S103-RED-05

- **Packet ID:** `T103-P05`
- **依赖:** `T103-P12` PASS
- **优先级:** `P0`
- **Owner:** `S103-storage-test`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/tests/durable_metadata_foundation.rs`
- **Forbidden paths:** `crates/homie-storage/src/lib.rs`、所有其他 source/test/doc/Cargo 文件
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test durable_metadata_foundation -- --nocapture`
- **GREEN command:** 同一 focused command，供 `T103-P13` 使用
- **Deadline:** `3h`；focused command `10m`
- **Cleanup:** 关闭 DB 并删除 test-owned temp paths；无网络/安装/process
- **Expected result:** parent canonical、lineage operation id 幂等、handoff operation/lease CAS、
  update legal/illegal transition、secret scan 均 RED；不测试 workflow
- **Handoff/commit contract:** 预期 RED 成立后提交独立 binary，commit
  `test(storage): add durable metadata foundation RED`；packet 结束时 worktree clean

**完整可复制 prompt：**

```text
执行 T103-P05 / S103-RED-05，owner=S103-storage-test，priority=P0，依赖 T103-P12 PASS。
固定 worktree=/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，
branch=wave1c/diri-storage-core-facts，deadline=3h，focused timeout=10m。读取 AGENTS.md 与全部
T-103 approved docs，做 branch/status/base preflight；worktree 非 clean 则 BLOCKED。不得新增
环境变量配置。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/tests/durable_metadata_foundation.rs
禁止 homie-storage/src/lib.rs 及其他全部路径。

添加独立 durable_metadata_foundation binary RED：sessions.parent_session_id 仍是 direct
parent 单一事实源；lineage audit operation id 幂等且只含 safe actor/subject/relation/action/
decision/reason；复用 hosts/node_accounts/handoff_records 并验证 operation/checkpoint/phase/
lease/manifest hash、重复提交幂等或 stable conflict、非法 CAS fail closed；update receipt
operation id 唯一、合法 phase 前进、非法倒退失败；新增持久 facts 不含 prompt、tool args/
result、blob、provider home、token、raw/virtual key、Authorization、Cookie。typed repository
symbol 尚不存在时允许该独立 binary 合法 compile-fail，不得影响前三个 storage targets。不得
实现或断言 remote network/handoff、download/install/rollback workflow。

RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test durable_metadata_foundation -- --nocapture
失败只归因 metadata repository contract 缺失。未来 GREEN 同命令由 T103-P13 执行。

清理 test-owned DB，无网络/安装/process。运行 git diff --check/status，确认只改 allowed test
file。预期 RED 成立后 stage allowed path，运行绝对 pre-commit hook，提交：
test(storage): add durable metadata foundation RED
不得 amend/rebase/push。handoff 报告 base/result SHA、actual RED、changed file、cleanup、
diff-check；packet 结束时 worktree clean。
```

### T103-P13 / S103-GREEN-04

- **Packet ID:** `T103-P13`
- **依赖:** `T103-P05`、`T103-P12` PASS
- **外部门禁:** 无；不等待 T-102
- **优先级:** `P0`
- **Owner:** `S103-storage-impl`，唯一 `lib.rs` owner
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/src/lib.rs`
- **Forbidden paths:** storage tests、全部 proto/runtime/client/app/CLI/docs
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test durable_metadata_foundation -- --nocapture`
- **GREEN command:**
  - 同一 focused command
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests`
- **Deadline:** `4h`
- **Cleanup:** test DB only；无 network/install/process
- **Expected result:** lineage append/read、host/account/handoff typed APIs/CAS、update receipt
  create/read/CAS、stable conflicts 全绿；无 workflow
- **Handoff/commit contract:** commit
  `feat(storage): add durable metadata repositories`；释放 storage GREEN queue 给 shared integration

**完整可复制 prompt：**

```text
执行 T103-P13 / S103-GREEN-04，owner=S103-storage-impl，priority=P0，依赖 T103-P05 和
T103-P12 PASS。
固定 worktree=/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，
branch=wave1c/diri-storage-core-facts，deadline=4h。读取 AGENTS.md 与全部 T-103 approved docs，
做 branch/status/base preflight，确认 lib.rs 独占 owner。此 storage packet 不等待 T-102。
禁止环境变量配置。

唯一允许写 homie-storage/src/lib.rs 的绝对路径；tests、proto/runtime/client/app/CLI/docs
全部 forbidden。

先运行 RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test durable_metadata_foundation -- --nocapture
最小实现：lineage safe audit append/read，operation id 唯一幂等且 parent_session_id 仍是 direct
parent 单一事实源；复用 hosts/node_accounts/handoff_records，增加 typed CRUD、operation/
checkpoint/phase/lease/manifest hash validation 和 CAS/stable conflict；update receipt create/read/
CAS，合法 phase transition only，非法倒退 fail closed。只存 safe metadata/hash/ref/error code；
不存 blob、provider home、token、raw key、Authorization、Cookie、prompt/tool payload。不得添加
remote listener/transfer/resume/move/fork 或 updater feed/download/install/rollback workflow，
不得广告 host.* 或 update.* workflow method。

GREEN：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test durable_metadata_foundation -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests

清理 test-owned DB；运行 git diff --check/status。PASS 后只 stage lib.rs，提交：
feat(storage): add durable metadata repositories
不 amend/rebase/push。handoff 报告 base/result SHA、RED/GREEN、public APIs、stable conflict/
phase table、changed file、cleanup、diff-check。明确 storage GREEN-01..04 完成，但 shared
integration 仍须 T-102 shared-release milestone 已合并。
```

### T103-P14 / S103-GREEN-05

- **Packet ID:** `T103-P14`
- **依赖:** `T103-P06`、`T103-P13` PASS
- **外部门禁:** T-102 shared-release milestone 已合并且 released proto files 无 active owner
- **优先级:** `P0`
- **Owner:** `S103-proto-integration`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-proto/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-proto/src/model.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-proto/tests/diri_storage_core_facts_contract.rs`
- **Forbidden paths:** 其他 proto files、全部 storage/runtime/client/app/CLI/docs
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-proto --test diri_storage_core_facts_contract`
- **GREEN command:**
  - 同一 focused command
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-proto --tests`
- **Deadline:** `4h`
- **Cleanup:** 无 process；删除 test-only serialization artifacts
- **Expected result:** 六个 frozen methods/DTO 与 safe serde/bounds/validation 全绿；不广告
  remote/updater workflow
- **Handoff/commit contract:** commit
  `feat(proto): add durable storage service contracts`；将 exact DTO/method SHA 交 `T103-P15`

**完整可复制 prompt：**

```text
执行 T103-P14 / S103-GREEN-05，owner=S103-proto-integration，priority=P0，依赖 T103-P06 和
T103-P13 PASS，deadline=4h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 approved docs、T-102 shared-release
handoff。用 Bead 中实际 40 位 SHA 验证 shared-release 已是 HEAD ancestor，lib.rs/model.rs 已被
T-102 显式 release 且无 active owner；否则 BLOCKED。禁止环境变量配置。

只允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-proto/src/lib.rs
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-proto/src/model.rs
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-proto/tests/diri_storage_core_facts_contract.rs
其他全部 forbidden，尤其 storage/runtime/client/app/CLI。

先运行 RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-proto --test diri_storage_core_facts_contract
最小实现恰好六个 method 常量和 DTO：storage.health -> StorageHealthResult；settings.get ->
SettingsSnapshot；settings.update(SettingsUpdateRequest)->SettingsSnapshot；usage.summary
(UsageSummaryRequest)->UsageSummaryResult；session.effective_config(session id)->
EffectiveAgentConfigSnapshot；runtime.recovery.summary(bounded filter)->
RuntimeRecoverySummary。settings 带 monotonic revision/expectedRevision；snapshot versioned/
bounded/safe；recovery 区分 persisted hint/verified live；safe stable errors/serde。不得添加新的
lineage method、host.* handoff 或 update.* workflow method。仅定义协议，不提前声称 handler
capability。

GREEN：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-proto --test diri_storage_core_facts_contract
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-proto --tests

运行 git diff --check/status，确认仅 allowed paths。PASS 后提交：
feat(proto): add durable storage service contracts
不 amend/rebase/push。handoff 报告 T-102 release SHA、base/result SHA、exact method/DTO paths
和 names、RED/GREEN counts、changed files、cleanup、diff-check，交给 T103-P15。
```

### T103-P15 / S103-GREEN-06

- **Packet ID:** `T103-P15`
- **依赖:** `T103-P07`、`T103-P14` PASS
- **外部门禁:** T-102 shared-release milestone 已合并且 runtime actor/dispatcher/capabilities 已释放
- **优先级:** `P0`
- **Owner:** `S103-runtime-integration`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/src/runtime_actor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/src/dispatcher.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/src/capabilities.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/tests/diri_storage_core_facts_service.rs`
- **Forbidden paths:** `homie-runtime/src/lib.rs`、holder/lifecycle 其他文件、全部 storage/proto/
  client/app/CLI/docs
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service`
- **GREEN command:**
  - 同一 focused command
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --tests`
- **Deadline:** `6h`；focused `15m`，package `20m`
- **Cleanup:** exact fixture daemon/PID/socket/temp dirs，3s 有界退出；禁止 `pkill`
- **Expected result:** owning services/dispatcher 六 handlers、settings CAS、safe health/usage/config/
  recovery、verified-live gate、handler-driven discovery 全绿
- **Handoff/commit contract:** commit
  `feat(runtime): add durable storage service handlers`；交 exact handler SHA 给 `T103-P16`

**完整可复制 prompt：**

```text
执行 T103-P15 / S103-GREEN-06，owner=S103-runtime-integration，priority=P0，依赖 T103-P07、
T103-P14 PASS，deadline=6h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 approved docs、T-102 release handoff；
验证 shared-release 40 位 SHA 已是 HEAD ancestor，runtime_actor.rs/dispatcher.rs/
capabilities.rs 已显式 release 且无 active owner。否则 BLOCKED。禁止环境变量配置。

只允许写 runtime_actor.rs、dispatcher.rs、capabilities.rs 和
tests/diri_storage_core_facts_service.rs 的上述绝对路径。禁止 runtime/lib.rs、holder/其他
lifecycle、storage/proto/client/app/CLI/docs。确需越界先 BLOCKED，不自行扩 scope。

先运行 RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service
最小实现六个 runtime owning service handlers，调用 T103-P11..P13 typed storage repository：
settings get/update revision CAS；storage health schema/FK/journal/safe identity且无 live claim；
usage summary 复用既有 semantics；effective config safe readback；recovery bounded summary，
persisted row/PID/status 不能发布 running，必须调用 T-102 已释放 verifier 验证 holder/process/
output 后才标 verified live。storage failure 映射 safe stable error。只有 handler executable 且
integration test 通过的方法进入 registry/capability discovery。不得添加 remote/updater
workflow handler，不得直接 SQL/Connection。

GREEN：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --tests

fixture 以 exact PID/start time/socket/temp dir 归属，3s 内有界退出，禁止 pkill。运行 git diff
--check/status。PASS 后提交：
feat(runtime): add durable storage service handlers
不 amend/rebase/push。handoff 报告 T-102 release SHA、base/result SHA、handler registry、
RED/GREEN counts、fixture cleanup、changed files、diff-check，交 T103-P16。
```

### T103-P16 / S103-GREEN-07

- **Packet ID:** `T103-P16`
- **依赖:** `T103-P15` PASS
- **外部门禁:** T-102 shared-release milestone 已合并且 client file 已释放
- **优先级:** `P0`
- **Owner:** `S103-client-integration`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-client/src/client.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-client/tests/diri_storage_core_facts_client.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-client/tests/typed_facade.rs`
- **Forbidden paths:** 其他 client files、全部 storage/proto/runtime/app/CLI/docs
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client`
- **GREEN command:**
  - 同一 focused command
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --tests`
- **Deadline:** `3h`
- **Cleanup:** mock socket/connection/temp dirs；无遗留 task/socket
- **Expected result:** 六个 typed methods、capability check、stable error mapping 全绿；caller
  不见 SQL/repository types
- **Handoff/commit contract:** commit
  `feat(client): add durable storage service methods`；client API stable 后放行 P17/P18 并行

**完整可复制 prompt：**

```text
执行 T103-P16 / S103-GREEN-07，owner=S103-client-integration，priority=P0，依赖 T103-P15
PASS，deadline=3h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 approved docs、T-102 release handoff；
确认 shared-release 已合并，client.rs 已释放且无 active owner。否则 BLOCKED。禁止环境变量配置。

只允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-client/src/client.rs
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-client/tests/diri_storage_core_facts_client.rs
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-client/tests/typed_facade.rs
其他 client files 及 storage/proto/runtime/app/CLI/docs forbidden。

先运行 RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client
最小添加 HomieClient typed methods，精确使用六个 frozen Method/DTO，复用 request/
typed_request 机制，做 executable capability check 和 stable ClientError mapping。不得暴露
Storage、Connection、transaction、SQL 或 storage crate type；不得复制 DTO；不得添加 remote/
updater workflow method。

GREEN：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --tests

关闭 mock socket/connection，确认无遗留 Tokio task/socket。运行 git diff --check/status。
PASS 后提交：
feat(client): add durable storage service methods
不 amend/rebase/push。handoff 报告 release/base/result SHA、exact typed APIs、RED/GREEN counts、
changed files、cleanup、diff-check。明确 client API stable，T103-P17 与 T103-P18 可并行且不得
互改文件。
```

### T103-P17 / S103-GREEN-08

- **Packet ID:** `T103-P17`
- **依赖:** `T103-P08`、`T103-P16` PASS
- **优先级:** `P0`
- **Owner:** `S103-app-integration`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-app/Cargo.toml`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-app/src/main.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-app/src/runtime_bridge.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-app/tests/diri_storage_service_boundary.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-app/tests/runtime_bridge.rs`
- **Forbidden paths:** workspace Cargo.lock/Cargo.toml、其他 app files、全部 storage/proto/runtime/
  client/CLI/docs
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary`
- **GREEN command:**
  - 同一 focused command
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --tests`
  - `/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app -e normal`
- **Deadline:** `4h`；app tests `20m`
- **Cleanup:** bridge worker/client 正常 shutdown；无 socket/task 泄漏
- **Expected result:** app settings 异步经 bridge/client load/save，revision authoritative，失败无
  假成功；normal tree/source 无 storage/direct-open/fallback
- **Handoff/commit contract:** commit
  `refactor(app): route settings through runtime service`

**完整可复制 prompt：**

```text
执行 T103-P17 / S103-GREEN-08，owner=S103-app-integration，priority=P0，依赖 T103-P08、
T103-P16 PASS，deadline=4h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md 和 T-103 approved docs，做 branch/status/base
preflight，确认 app allowed files 无 active owner；P18 可并行但只能写 CLI。禁止环境变量配置。

只允许写 app/Cargo.toml、src/main.rs、src/runtime_bridge.rs、
tests/diri_storage_service_boundary.rs、tests/runtime_bridge.rs 的上述绝对路径。禁止 workspace
Cargo.toml/Cargo.lock、其他 app、storage/proto/runtime/client/CLI/docs。

先运行 RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary
最小把 settings load/save 改为 runtime_bridge 调 HomieClient settings.get/settings.update；
bridge 使用既有 async worker，不阻塞 GPUI 主线程；保存携带 expected revision，以 daemon 返回
snapshot 为 authoritative；conflict/unavailable 显示 safe failure，不保留假成功。删除 app
normal homie-storage dependency、homie_storage imports、StorageConfig/open_or_create/
open_ready_storage 和 direct fallback；使用 proto-owned settings DTO。不得增加临时 adapter、
双路径或 env config。

GREEN：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --tests
/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app -e normal
tree 和 source scan 必须无 production homie-storage/direct open。

正常 shutdown bridge worker/client，确认无 task/socket 泄漏。运行 git diff --check/status。
PASS 后只 stage allowed paths，提交：
refactor(app): route settings through runtime service
不 amend/rebase/push。handoff 报告 base/result SHA、RED/GREEN/tree、changed files、failure UX、
cleanup、diff-check 和 forbidden path 零改动。
```

### T103-P18 / S103-GREEN-09

- **Packet ID:** `T103-P18`
- **依赖:** `T103-P09`、`T103-P16` PASS
- **优先级:** `P0`
- **Owner:** `S103-cli-integration`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/Cargo.toml`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/src/main.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/tests/diri_storage_service_boundary.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/tests/runtime_daemon_cli.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/tests/usage_summary_cli.rs`
- **Forbidden paths:** workspace Cargo.lock/Cargo.toml、其他 CLI files、全部 storage/proto/runtime/
  client/app/docs
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary`
- **GREEN command:**
  - 同一 focused command
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --tests`
  - `/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli -e normal`
- **Deadline:** `4h`；CLI tests `20m`
- **Cleanup:** launcher/daemon exact PID/socket/temp dir，3s 有界退出；禁止 `pkill`
- **Expected result:** doctor 经 `storage.health`、usage 经 `usage.summary`，safe unavailable，
  normal tree/source 无 direct storage/fallback
- **Handoff/commit contract:** commit
  `refactor(cli): route durable reads through runtime service`

**完整可复制 prompt：**

```text
执行 T103-P18 / S103-GREEN-09，owner=S103-cli-integration，priority=P0，依赖 T103-P09、
T103-P16 PASS，deadline=4h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md 和 T-103 approved docs，做 branch/status/base
preflight；P17 可并行但只写 app。禁止环境变量配置。

只允许写 CLI Cargo.toml、src/main.rs、tests/diri_storage_service_boundary.rs、
tests/runtime_daemon_cli.rs、tests/usage_summary_cli.rs 的上述绝对路径。禁止 workspace
Cargo.toml/Cargo.lock、其他 CLI、storage/proto/runtime/client/app/docs。

先运行 RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary
最小把 doctor 改为 ensure/connect daemon 后调用 HomieClient storage.health；把 usage summary
改为 usage.summary typed method并保持既有 safe aggregate 输出语义。删除 production
homie-storage normal dependency/import、StorageConfig/open_or_create/open_ready_storage 和
direct fallback。测试如需 seed storage，只能使用明确 dev-dependency/test harness，normal
tree 不得含 storage。daemon/storage unavailable 返回 stable safe diagnostic，不伪造 live。

GREEN：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --tests
/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli -e normal
normal tree/source 必须无 production storage/direct open/fallback。

launcher/daemon fixture 记录 exact PID/start time/socket/temp dir，3s 内有界退出，禁止 pkill。
运行 git diff --check/status。PASS 后只 stage allowed paths，提交：
refactor(cli): route durable reads through runtime service
不 amend/rebase/push。handoff 报告 base/result SHA、RED/GREEN/tree、changed files、safe errors、
cleanup、diff-check 和 forbidden path 零改动。
```

### T103-P19 / S103-GREEN-10

- **Packet ID:** `T103-P19`
- **依赖:** `T103-P17`、`T103-P18` PASS
- **优先级:** `P0`
- **Owner:** `S103-verification`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/focused-green-matrix.md`
- **Forbidden paths:** 全部 product/test/spec/plan 及其他 evidence
- **RED command:** 依次运行所有 focused targets；任一未绿即 matrix RED，不写源码修复
- **GREEN command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-proto --tests`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary`
- **Deadline:** `2h`；单 command `20m`
- **Cleanup:** 所有 fixture-owned PID/socket/temp dir 归零；禁止按名称清理
- **Expected result:** 所有 RED 现为 GREEN，storage 既有 suites 保持 GREEN，记录实际计数
- **Handoff/commit contract:** commit
  `test(storage): record focused T-103 GREEN matrix`

**完整可复制 prompt：**

```text
执行 T103-P19 / S103-GREEN-10，owner=S103-verification，priority=P0，依赖 T103-P17/P18
PASS，deadline=2h，每条 command timeout=20m。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 approved docs 和 P01-P18 handoffs，做
branch/status/base preflight。禁止环境变量配置，不修改源码/测试。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/focused-green-matrix.md
全部 product/test/spec/plan 和其他 evidence forbidden。

把第一次完整 focused matrix 视为 RED gate：任一失败则如实记录 FAIL/BLOCKED，并把修复退回
原 exclusive owner，不自行修。依次运行：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-proto --tests
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary

全部通过后按同顺序重跑 flaky-sensitive focused commands 或记录无需重跑的确定性依据，形成
GREEN matrix。evidence 记录每个 binary/case 的实际 passed/failed/ignored、duration、SHA、
T-102 release SHA、cleanup。fixture-owned PID/socket/temp dir 必须归零，禁止 pkill。

运行 git diff --check/status，确认仅 allowed evidence。PASS 后提交：
test(storage): record focused T-103 GREEN matrix
不 amend/rebase/push。handoff 报告 base/result SHA、matrix counts、changed file、cleanup、
diff-check 和任何 residual risk。
```

### T103-P06 / S103-RED-06

- **Packet ID:** `T103-P06`
- **依赖:** SPEC approved
- **外部门禁:** T-102 shared-release milestone 已通过 `merge --no-ff` 合并、其 SHA 是当前
  `HEAD` ancestor，且 T-102 regressions 已重跑全绿
- **优先级:** `P0`
- **Owner:** `S103-proto-test`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-proto/tests/diri_storage_core_facts_contract.rs`
- **Forbidden paths:** `crates/homie-proto/src/**`、全部 runtime/client/storage/app/CLI 产品路径
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-proto --test diri_storage_core_facts_contract`
- **GREEN command:** 同一 command，供 `T103-P14` 使用
- **Deadline:** `3h`；T-102 regression suite `30m`
- **Cleanup:** 无 process；删除 test-owned serialization artifacts
- **Expected result:** 六个 frozen method/DTO 的 serde、安全、bounds、unknown version/phase
  rejection RED；不修改 production proto
- **Handoff/commit contract:** commit
  `test(proto): add durable service contract RED`

**完整可复制 prompt：**

```text
执行 T103-P06 / S103-RED-06，owner=S103-proto-test，priority=P0，deadline=3h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。先完整读取 AGENTS.md、T-103 approved docs、T-102 tasks/
delegation。运行 /opt/homebrew/bin/bd show homie-t3u.1 --long，取得明确 shared-release 40 位
SHA、released 文件和 focused tests；用 /usr/bin/git -C
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts merge-base
--is-ancestor 加该实际 40 位 SHA 验证它已通过 merge --no-ff 合并到当前 HEAD，并确认 H-02
milestone 是其真实祖先。禁止 cherry-pick。先运行：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-agents
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --tests
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-proto --tests
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --tests
regression suite timeout=30m。SHA 未记录、未合并、regression 失败、文件未释放或 dirty
ownership 不清时立即 BLOCKED，不得写文件。不得新增环境变量配置。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-proto/tests/diri_storage_core_facts_contract.rs
禁止 homie-proto/src/** 和全部 runtime/client/storage/app/CLI 产品路径。

添加 compile-fail 或 behavior-fail RED fixtures，精确覆盖六个 method：
storage.health；settings.get；settings.update；usage.summary；
session.effective_config；runtime.recovery.summary。
覆盖 SettingsSnapshot revision、expectedRevision conflict、bounded usage/recovery query、
versioned immutable config snapshot/hash、persisted hint 与 verified live 分离、safe errors、
camelCase serde round-trip、unknown snapshot version/phase rejection。不得添加 host.* handoff、
update.* workflow 或 lineage 同义 method。production method 不存在时 compile failure 是合法
RED，但必须只指向 frozen contract 缺失。

RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-proto --test diri_storage_core_facts_contract
未来 GREEN 同命令由 T103-P14 执行；本 packet 不改 production proto。

运行 git diff --check/status；确认 forbidden path 零改动。预期 RED 准确成立后提交：
test(proto): add durable service contract RED
不 amend/rebase/push。handoff 必须附 T-102 shared-release SHA/ancestor 验证、base/result SHA、
actual RED、changed files、cleanup、diff-check。
```

### T103-P07 / S103-RED-07

- **Packet ID:** `T103-P07`
- **依赖:** `T103-P06` PASS
- **外部门禁:** 同一 T-102 shared-release milestone 已合并
- **优先级:** `P0`
- **Owner:** `S103-runtime-test`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/tests/diri_storage_core_facts_service.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-client/tests/diri_storage_core_facts_client.rs`
- **Forbidden paths:** 所有 production `src/**`、storage/app/CLI 路径和其他 tests
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client`
- **GREEN command:** 同一组 commands，供 `T103-P15/P16` 使用
- **Deadline:** `4h`
- **Cleanup:** mock socket/temp dir 必须 RAII；若启动 daemon，只按 exact PID/start time 在 3s
  内退出；禁止 `pkill`
- **Expected result:** discovery、CAS、health/usage/config/recovery、安全错误均因 handler/client
  缺失 RED；未知 method 仍 `method_not_found`
- **Handoff/commit contract:** commit
  `test(runtime): add durable service integration RED`

**完整可复制 prompt：**

```text
执行 T103-P07 / S103-RED-07，owner=S103-runtime-test，priority=P0，依赖 T103-P06 PASS，
deadline=4h。worktree 固定
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch 固定
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 approved docs 和 T-102 release handoff；
再次用 Bead 记录的实际 40 位 SHA 验证 shared-release 已是 HEAD ancestor，released runtime/
client files 无 active owner。门禁不满足立即 BLOCKED。禁止环境变量配置。

只允许写两个新 test：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/tests/diri_storage_core_facts_service.rs
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-client/tests/diri_storage_core_facts_client.rs
所有 production src、storage/app/CLI 和其他路径 forbidden。

先写 RED：capability discovery 只有 handler 存在时才广告六个方法；settings get/update CAS
冲突不覆盖；storage health 不声称 live；usage summary bounded safe aggregate；effective config
safe readback；recovery summary 区分 durable hints/verified live，row-only 不得报告 running；
storage unavailable/corrupt/invalid phase 映射 stable safe errors；直接未知 method 仍返回
method_not_found。不得修改 production files。

RED commands：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client
必须只因 T-103 handler/client 尚未实现而失败。未来 GREEN 由 P15/P16 使用相同命令。

fixture 用 RAII；若测试确需 daemon，记录 exact PID/start time 并在 3s 内有界退出，禁止 pkill
和清理预存进程。运行 git diff --check/status，确认 forbidden path 零改动。预期 RED 成立后
提交：
test(runtime): add durable service integration RED
不 amend/rebase/push。handoff 附 release SHA、base/result SHA、两个命令实际失败、changed
files、cleanup、diff-check。
```

### T103-P08 / S103-RED-08

- **Packet ID:** `T103-P08`
- **依赖:** `T103-P07` PASS
- **优先级:** `P0`
- **Owner:** `S103-app-test`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-app/tests/diri_storage_service_boundary.rs`
- **Forbidden paths:** app production/Cargo、全部其他 crate/doc
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary`
- **GREEN command:**
  - 同一 command，供 `T103-P17` 使用
  - `/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app -e normal`
- **Deadline:** `2h`
- **Cleanup:** 无 daemon/process；source/dependency scan 只读
- **Expected result:** 测试真实证明当前 app normal dependency 和 direct-open helper 存在，并因
  尚未走 bridge/client RED
- **Handoff/commit contract:** commit
  `test(app): add direct storage removal RED`

**完整可复制 prompt：**

```text
执行 T103-P08 / S103-RED-08，owner=S103-app-test，priority=P0，依赖 T103-P07 PASS，
deadline=2h。固定 worktree=/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，
branch=wave1c/diri-storage-core-facts。读取 AGENTS.md 与 T-103 approved docs，做 branch/status/
base preflight。禁止新增环境变量配置。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-app/tests/diri_storage_service_boundary.rs
禁止 app Cargo.toml/src/** 和其他全部路径。

写 source/dependency/bridge behavior RED，明确当前 app normal dependency 含 homie-storage，
main.rs 使用 homie_storage、StorageConfig、open_or_create、open_ready_storage 和 storage-owned
SettingsPreferences；目标行为是 settings load/save 经 runtime_bridge/HomieClient，带 revision
authoritative response，失败不显示假成功，且无 direct fallback。测试须能编译并以行为/扫描
断言失败，不改 production。

RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary
记录当前 dependency tree：
/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app -e normal
未来 GREEN 使用相同命令并要求 tree 无 homie-storage，由 T103-P17 执行。

不启动 daemon/process。运行 git diff --check/status，确认 forbidden path 零改动。预期 RED
成立后提交：
test(app): add direct storage removal RED
不 amend/rebase/push。handoff 报告 base/result SHA、actual RED、tree 证据、changed files、
cleanup、diff-check。
```

### T103-P09 / S103-RED-09

- **Packet ID:** `T103-P09`
- **依赖:** `T103-P07` PASS
- **优先级:** `P0`
- **Owner:** `S103-cli-test`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/tests/diri_storage_service_boundary.rs`
- **Forbidden paths:** CLI production/Cargo、现有 tests、全部其他路径
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary`
- **GREEN command:**
  - 同一 command，供 `T103-P18` 使用
  - `/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli -e normal`
- **Deadline:** `2h`
- **Cleanup:** 无 daemon/process；只读 source/dependency scan
- **Expected result:** doctor/usage direct open 和 normal storage dependency 被准确 RED；目标
  service path/no fallback 明确
- **Handoff/commit contract:** commit
  `test(cli): add direct storage removal RED`

**完整可复制 prompt：**

```text
执行 T103-P09 / S103-RED-09，owner=S103-cli-test，priority=P0，依赖 T103-P07 PASS，
deadline=2h。固定 worktree=/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，
branch=wave1c/diri-storage-core-facts。读取 AGENTS.md 与全部 T-103 approved docs，做 branch/
status/base preflight。禁止新增环境变量配置。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/tests/diri_storage_service_boundary.rs
禁止 CLI Cargo.toml/src/**、现有 tests 和其他全部路径。

添加可编译的 source/dependency/CLI contract RED：doctor 当前通过 open_or_create 直接读取
storage；usage summary 当前通过 open_ready_storage/query_usage_totals 直接读取；normal
dependency 含 homie-storage。目标是 doctor 调 storage.health，usage 调 usage.summary，
storage unavailable 返回 safe error 且绝不 direct fallback。不要改写既有 CLI tests。

RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary
记录：
/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli -e normal
未来 GREEN 使用同命令并要求 tree 无 homie-storage，由 T103-P18 执行。

不启动 daemon/process。运行 git diff --check/status，确认 forbidden path 零改动。预期 RED
成立后提交：
test(cli): add direct storage removal RED
不 amend/rebase/push。handoff 报告 base/result SHA、actual RED、tree、changed files、cleanup、
diff-check。
```

## 6. REFACTOR Packets

### T103-P20 / S103-REFACTOR-01

- **Packet ID:** `T103-P20`
- **依赖:** `T103-P19` PASS
- **优先级:** `P0`
- **Owner:** `S103-runtime-integration`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/src/runtime_actor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/src/history.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/tests/diri_storage_core_facts_service.rs`
- **Forbidden paths:** `crates/homie-storage/src/lib.rs`、全部 proto/client/app/CLI/docs/其他 runtime
- **RED command:**
  - `/opt/homebrew/bin/rg -n 'RuntimeSupervisor::storage|\\.storage\\(\\)|\\.connection\\(\\)' /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/src`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service`
- **GREEN command:**
  - 同一 `rg` 返回零个 production consumer direct-access match
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --tests`
- **Deadline:** `3h`；package test `20m`
- **Cleanup:** exact runtime fixture PID/socket/temp dir 归零；禁止 `pkill`
- **Expected result:** touched runtime consumer 不再把 `Storage::connection()` 或 raw
  `RuntimeSupervisor::storage()` 当领域 API；shutdown 调 storage-owned flush
- **Handoff/commit contract:** commit
  `refactor(runtime): close raw storage access`

**完整可复制 prompt：**

```text
执行 T103-P20 / S103-REFACTOR-01，owner=S103-runtime-integration，priority=P0，依赖
T103-P19 PASS，deadline=3h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 approved docs 和 P15/P19 handoff，做
branch/status/base preflight。确认 T-102 shared files 已释放且 allowed runtime files 无 active
owner。禁止环境变量配置。

只允许写 runtime/src/lib.rs、runtime_actor.rs、history.rs 和
tests/diri_storage_core_facts_service.rs 的上述绝对路径。严禁写
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/src/lib.rs
以及 proto/client/app/CLI/docs/其他 runtime 文件；storage API 如不足则 BLOCKED 并退回
S103-storage-impl，不跨 owner 修。

RED scan：
/opt/homebrew/bin/rg -n 'RuntimeSupervisor::storage|\.storage\(\)|\.connection\(\)' /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/src
并运行：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service
记录 raw access 现状。最小收口 touched runtime：actor/service 调 narrow typed supervisor/
domain methods；prepare_shutdown 使用 P12 storage-owned flush，不直接取得 rusqlite
Connection；删除不再需要的 public storage getter，不改变 T-102 live lifecycle 语义。

GREEN：
/opt/homebrew/bin/rg -n 'RuntimeSupervisor::storage|\.storage\(\)|\.connection\(\)' /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/src
production consumer direct access 必须零匹配；若测试 harness 有合法 match，需用精确路径解释，
不得放宽全局规则。
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --tests

清理 exact fixture PID/socket/temp dir，禁止 pkill。运行 git diff --check/status。PASS 后只提交
allowed paths：
refactor(runtime): close raw storage access
不 amend/rebase/push。handoff 报告 base/result SHA、scan before/after、tests、changed files、
cleanup、diff-check 和 forbidden path 零改动。
```

### T103-P21 / S103-REFACTOR-02

- **Packet ID:** `T103-P21`
- **依赖:** `T103-P20` PASS
- **优先级:** `P0`
- **Owner:** `S103-storage-impl`，唯一 `homie-storage/src/lib.rs` owner
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/src/lib.rs`
- **Forbidden paths:** storage tests/Cargo、全部 runtime/proto/client/app/CLI/docs
- **RED command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration --test effective_config_facts --test runtime_recovery_facts --test durable_metadata_foundation -- --nocapture`
- **GREEN command:**
  - 同一 focused command
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests`
- **Deadline:** `2h`
- **Cleanup:** test-owned DB only
- **Expected result:** 仅在已证明重复时合并 validation/error/phase-transition helper；不引入
  framework，行为/公开 contract 不变
- **Handoff/commit contract:** 有实际最小 refactor 才 commit
  `refactor(storage): consolidate durable fact validation`；无重复则 no-op PASS、不制造提交

**完整可复制 prompt：**

```text
执行 T103-P21 / S103-REFACTOR-02，owner=S103-storage-impl，priority=P0，依赖 T103-P20 PASS，
deadline=2h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 approved docs 和 P10-P13/P20 handoff，
做 branch/status/base preflight，重新取得 lib.rs 唯一 ownership。禁止环境变量配置。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-storage/src/lib.rs
tests/Cargo/runtime/proto/client/app/CLI/docs 全部 forbidden。

先运行 focused tests：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration --test effective_config_facts --test runtime_recovery_facts --test durable_metadata_foundation -- --nocapture
审计 P10-P13 新代码，只在至少两处真实重复且合并能减少复杂度时，最小合并 bounded snapshot
validation、stable conflict mapping 或 phase-transition table。不得新增 repository trait/
framework、拆文件、改变 public API/schema/migration、做无关格式化。没有真实重复时明确 no-op
PASS，不为满足任务制造改动。

GREEN：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration --test effective_config_facts --test runtime_recovery_facts --test durable_metadata_foundation -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests

清理 test-owned DB，运行 git diff --check/status。若有最小改动，只 stage lib.rs 并提交：
refactor(storage): consolidate durable fact validation
若 no-op，不提交。不得 amend/rebase/push。handoff 报告 base/result SHA 或 no-op、重复证据、
tests、changed files、cleanup、diff-check。
```

### T103-P22 / S103-REFACTOR-03

- **Packet ID:** `T103-P22`
- **依赖:** `T103-P21` PASS/no-op
- **优先级:** `P0`
- **Owner:** `S103-verification`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/refactor-quality-gates.md`
- **Forbidden paths:** 全部 product/test/spec/plan 和其他 evidence；verification owner 不修源码
- **RED command:** fmt/check/clippy/dependency/direct-open/sensitive scan 第一轮；任一失败即 RED
- **GREEN command:** 原样重跑通过；见完整 prompt
- **Deadline:** `3h`；workspace clippy `30m`
- **Cleanup:** command/process/temp resources 归零；不执行 formatter 写模式
- **Expected result:** fmt/check/clippy pass，app/CLI normal tree 无 storage，direct-open 零，
  sensitive persisted result 零；失败退回原 owner
- **Handoff/commit contract:** commit
  `test(storage): record T-103 refactor quality gates`

**完整可复制 prompt：**

```text
执行 T103-P22 / S103-REFACTOR-03，owner=S103-verification，priority=P0，依赖 T103-P21
PASS/no-op，deadline=3h，workspace clippy timeout=30m。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 approved docs 和 P19-P21 handoff，做
branch/status/base preflight。禁止环境变量配置；verification owner 不修源码。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/refactor-quality-gates.md
全部 product/test/spec/plan 和其他 evidence forbidden。不得运行 cargo fmt 写模式。

第一轮作为 RED gate，逐条运行并记录退出码/摘要：
/opt/homebrew/bin/cargo fmt --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml --all -- --check
/opt/homebrew/bin/cargo check --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml --workspace --all-targets
/opt/homebrew/bin/cargo clippy --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml --workspace --all-targets -- -D warnings
/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app -e normal
/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli -e normal
/opt/homebrew/bin/rg -n 'open_ready_storage|open_or_create|StorageConfig|homie_storage|Storage::connection|\.connection\(\)' /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-app/src /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/src /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-runtime/src

app/CLI normal tree 不得含 homie-storage；production direct-open/raw connection 零匹配。合法
test harness match 必须按绝对文件/行解释。扫描 evidence/fixture 中实际 secret value marker，
不得把字段名断言误报为 secret；raw provider/virtual key、Bearer value、Cookie value、private
key material 必须零。任一 gate 失败则记录 RED 并退回原 exclusive owner，不自行编辑。

全部修复经原 owner 新 packet handoff 后，原样重跑形成 GREEN。清理 command resources，运行
git diff --check/status。PASS 后只提交 evidence：
test(storage): record T-103 refactor quality gates
不 amend/rebase/push。handoff 报告 base/result SHA、每条命令状态、tree/scan、changed file、
cleanup、diff-check。
```

## 7. EVIDENCE Packets

### T103-P23 / S103-EVIDENCE-01

- **Packet ID:** `T103-P23`
- **依赖:** `T103-P22` PASS
- **优先级:** `P0`
- **Owner:** `S103-verification`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/migration-repository-evidence.md`
- **Forbidden paths:** 全部 product/test/spec/plan 和其他 evidence
- **RED command:** focused migration/repository matrix 首轮；缺项或失败即 RED
- **GREEN command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration --test effective_config_facts --test runtime_recovery_facts --test durable_metadata_foundation -- --nocapture`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests`
- **Deadline:** `2h`
- **Cleanup:** test DB/handles 归零；fixture SHA256 保留
- **Expected result:** baseline/final counts、v3 fixture SHA256、v4 applied/rollback、repository
  idempotency/CAS/security 全有可复验证据
- **Handoff/commit contract:** commit
  `test(storage): record migration repository evidence`

**完整可复制 prompt：**

```text
执行 T103-P23 / S103-EVIDENCE-01，owner=S103-verification，priority=P0，依赖 T103-P22 PASS，
deadline=2h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 approved docs、storage-baseline 和
P10-P13/P19/P22 handoff，做 branch/status/base preflight。禁止环境变量配置，不修改源码/测试。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/migration-repository-evidence.md
其他全部 forbidden。

首轮 focused matrix 作为 RED completeness gate：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --test ordered_v4_migration --test effective_config_facts --test runtime_recovery_facts --test durable_metadata_foundation -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests
任何 missing case/failure 都如实记录并退回 owner。GREEN evidence 必须记录：baseline/final 每个
binary 实际计数；真实 v3 fixture 绝对路径及 /usr/bin/shasum -a 256；empty applied
[1,2,3,4]、v3 [4]、repeat []；fault rollback 后 version/DDL/DML 不变；too-new；settings
revision；effective config immutable/atomic/hash；recovery atomic/bounded；lineage/handoff/update
idempotency/CAS；secret persisted-result scan。

关闭所有 DB handle并删除 packet-owned temp DB；保留规范 fixture。运行 git diff --check/status，
确认仅 evidence。PASS 后提交：
test(storage): record migration repository evidence
不 amend/rebase/push。handoff 报告 base/result SHA、commands/counts、fixture SHA256、changed
file、cleanup、diff-check。
```

### T103-P24 / S103-EVIDENCE-02

- **Packet ID:** `T103-P24`
- **依赖:** `T103-P23` PASS
- **优先级:** `P0`
- **Owner:** `S103-e2e`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/tests/diri_storage_recovery_e2e.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/runtime-recovery-e2e.md`
- **Forbidden paths:** 除上述专用 E2E test/evidence 外的全部 product/test/spec/plan 和其他
  evidence；禁止 production test mode
- **RED command:** daemon replacement/effective-config/recovery E2E 首轮；失败即 RED evidence
- **GREEN command:**
  - `/opt/homebrew/bin/cargo build --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --bins`
  - `/opt/homebrew/bin/cargo build --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_recovery_e2e -- --test-threads=1 --nocapture`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service -- --nocapture`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client -- --nocapture`
- **Deadline:** `4h`；E2E command `20m`
- **Cleanup:** exact daemon/holder/child PID+start time、socket、temp dir 归零；3s 有界；禁止 `pkill`
- **Expected result:** frozen config 跨 daemon replacement hash 不变，holder/output 重验证，
  row-only fake running 被拒绝
- **Handoff/commit contract:** commit
  `test(runtime): record recovery and config E2E`

**完整可复制 prompt：**

```text
执行 T103-P24 / S103-EVIDENCE-02，owner=S103-e2e，priority=P0，依赖 T103-P23 PASS，
deadline=4h，每条 E2E timeout=20m。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103/T-102 approved docs 和 P11/P12/P15/P16/
P23 handoff，做 branch/status/base preflight。禁止环境变量配置；只可新增本 packet 声明的
专用 E2E test 和 evidence，不得修改 production 或其他 tests。

只允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/tests/diri_storage_recovery_e2e.rs
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/runtime-recovery-e2e.md
其他全部 forbidden；禁止 production test mode、mock holder 或 mock PTY。

写文件前先运行以下命令，证明 dedicated target 尚不存在，作为 E2E coverage RED：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_recovery_e2e -- --test-threads=1 --nocapture
然后新增 dedicated real-process E2E。
测试必须启动 build 产物中的真实 daemon、真实 holder、真实 PTY 和真实 local fake
executable；若只覆盖 mock/service unit path，则不接受为 E2E。随后运行 GREEN：
/opt/homebrew/bin/cargo build --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --bins
/opt/homebrew/bin/cargo build --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_recovery_e2e -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client -- --nocapture
证据必须覆盖：创建 session 并原子 freeze config；记录 config hash；替换 daemon；重读相同 hash
和 safe snapshot；从 durable candidate 读取 holder/output/checkpoint hints；用 T-102 verifier
重验证 holder/process/output 后才可 running；构造 row-only stale PID/status 时必须拒绝 fake
running；失败/blocked 写 exact reason，不伪造 pass。

每个 fixture 记录 exact daemon/holder/child PID 和 start time、socket、temp dir；3s 内有界退出并
证明新增资源集合为零，禁止 pkill/按名称清理。运行 git diff --check/status。PASS 后只提交
专用 E2E test 和 evidence：
test(runtime): record recovery and config E2E
不 amend/rebase/push。handoff 报告 base/result SHA、commands/counts、hash/restart proof、
changed file、cleanup、diff-check。
```

### T103-P25 / S103-EVIDENCE-03

- **Packet ID:** `T103-P25`
- **依赖:** `T103-P23` PASS
- **优先级:** `P0`
- **Owner:** `S103-e2e`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/service-boundary-e2e.md`
- **Forbidden paths:** 全部 product/test/spec/plan 和其他 evidence
- **RED command:** app/CLI service-boundary E2E 首轮；任一 direct path/失败即 RED
- **GREEN command:**
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary -- --nocapture`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary -- --nocapture`
  - `/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app -e normal`
  - `/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli -e normal`
  - `/opt/homebrew/bin/rg -n 'open_ready_storage|open_or_create|StorageConfig|homie_storage' /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-app/src /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/src`
- **Deadline:** `4h`
- **Cleanup:** exact daemon/client/socket/temp resources 归零
- **Expected result:** app settings read/write/revision conflict、CLI doctor/usage、dependency/source
  scans 均证明 daemon authoritative 且无 fallback
- **Handoff/commit contract:** commit
  `test(app): record durable service boundary E2E`

**完整可复制 prompt：**

```text
执行 T103-P25 / S103-EVIDENCE-03，owner=S103-e2e，priority=P0，依赖 T103-P23 PASS，
deadline=4h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 approved docs 和 P17/P18/P23 handoff，
做 branch/status/base preflight。禁止环境变量配置，不修改源码/测试。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/service-boundary-e2e.md
其他全部 forbidden。

首轮作为 RED boundary gate，运行：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary -- --nocapture
/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app -e normal
/opt/homebrew/bin/cargo tree --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli -e normal
/opt/homebrew/bin/rg -n 'open_ready_storage|open_or_create|StorageConfig|homie_storage' /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-app/src /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/crates/homie-cli/src
证据覆盖 app settings get/update、revision conflict 不覆盖且 UI 不
假成功；CLI doctor 返回 storage.health；usage 返回 usage.summary safe aggregate；service
unavailable 无 direct fallback；两个 normal tree 不含 homie-storage；source scan 零。

清理 exact daemon/client/socket/temp resources，证明零新增遗留。运行 git diff --check/status。
PASS 后只提交 evidence：
test(app): record durable service boundary E2E
不 amend/rebase/push。handoff 报告 base/result SHA、commands/counts/tree/source scan、changed
file、cleanup、diff-check。
```

### T103-P26 / S103-EVIDENCE-04

- **Packet ID:** `T103-P26`
- **依赖:** `T103-P24`、`T103-P25` PASS
- **优先级:** `P0`
- **Owner:** `S103-review`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/code-review-report.md`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/security-review-report.md`
- **Forbidden paths:** 全部 product/test/spec/plan 和其他 evidence；reviewer 不直接修跨 owner 代码
- **RED command:** 两轮 review 首轮 finding scan；P0/P1 任一未解决即 RED/BLOCKED
- **GREEN command:**
  - `/usr/bin/git -C /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts diff --check`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service -- --nocapture`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client -- --nocapture`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary -- --nocapture`
  - `/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary -- --nocapture`
- **Deadline:** `3h`
- **Cleanup:** 无运行资源；review temp output 删除
- **Expected result:** 显性语法/运行/API/逻辑与隐性边界/资源/语义/安全两轮完成；P0/P1 归零，
  residual risk 明确
- **Handoff/commit contract:** findings 修复必须退回原 owner 单独提交；review PASS 后 commit
  `docs(verification): record T-103 code and security review`

**完整可复制 prompt：**

```text
执行 T103-P26 / S103-EVIDENCE-04，owner=S103-review，priority=P0，依赖 T103-P24/P25 PASS，
deadline=3h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 PRD/OpenSpec/alignment/delegation、从
T103-P02 到 P25 的 commits/evidence，做 branch/status/base preflight。禁止环境变量配置。

只允许写 code-review-report.md 和 security-review-report.md 的上述绝对路径。全部 product/
test/spec/plan/其他 evidence forbidden。reviewer 不直接修跨 owner 文件。

第一轮 RED review：语法、编译、运行、逻辑、API/DTO、migration、transaction、CAS、命名、
capability discovery、app/CLI removal。第二轮 adversarial review：边界条件、并发/revision、
rollback、resource cleanup、stale PID/live proof、bounded query/JSON、secret/path/error 泄漏、
foundation 被误报 workflow/parity。每个 finding 给 severity、confidence、绝对 file/line、
复现命令、影响和最小修复 owner。P0/P1 必须退回原 exclusive owner 修复并形成独立 commit；
reviewer 不越权编辑。未修复 P0/P1 时结果 RED/BLOCKED。

修复合并后运行：
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-storage --tests
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-runtime --test diri_storage_core_facts_service -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-client --test diri_storage_core_facts_client -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-app --test diri_storage_service_boundary -- --nocapture
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml -p homie-cli --test diri_storage_service_boundary -- --nocapture
/usr/bin/git -C /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts diff --check
GREEN 条件：P0/P1=0，P2/残余风险有明确 disposition，forbidden secret/raw payload 零。

删除 review temp output，运行 status。PASS 后只提交两份 report：
docs(verification): record T-103 code and security review
不 amend/rebase/push。handoff 报告 base/result SHA、findings/resolution commits、commands、
changed files、cleanup、diff-check。
```

### T103-P27 / S103-EVIDENCE-05

- **Packet ID:** `T103-P27`
- **依赖:** `T103-P26` PASS
- **优先级:** `P0`
- **Owner:** `S103-verification`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/implementation-quality-gates.md`
- **Forbidden paths:** 全部 product/test/spec/plan 和其他 evidence
- **RED command:** workspace/OpenSpec/parity/secret/diff gate 首轮；任一失败即 RED/BLOCKED
- **GREEN command:** 原样重跑全部 commands；见完整 prompt
- **Deadline:** `4h`；workspace tests `45m`
- **Cleanup:** 所有 test fixture resources 归零；hook temp output 删除
- **Expected result:** fmt/check/clippy/tests、OpenSpec 4/4/strict、parity consistency、secret hook、
  diff check 有真实状态；无虚假 pass
- **Handoff/commit contract:** commit
  `test(storage): record T-103 workspace quality gates`

**完整可复制 prompt：**

```text
执行 T103-P27 / S103-EVIDENCE-05，owner=S103-verification，priority=P0，依赖 T103-P26 PASS，
deadline=4h，workspace tests timeout=45m。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、quality-gates 文档、T-103 approved docs 和全部
implementation evidence，做 branch/status/base preflight。禁止环境变量配置，不修改源码/测试/
spec。

唯一允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/implementation-quality-gates.md
其他全部 forbidden。

首轮作为 RED gate，逐条运行并记录实际状态：
/opt/homebrew/bin/cargo fmt --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml --all -- --check
/opt/homebrew/bin/cargo check --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml --workspace --all-targets
/opt/homebrew/bin/cargo clippy --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml --workspace --all-targets -- -D warnings
/opt/homebrew/bin/cargo test --manifest-path /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/Cargo.toml --workspace
/bin/zsh -lc 'cd /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts && /opt/homebrew/bin/openspec status --change diri-storage-core-facts'
/bin/zsh -lc 'cd /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts && /opt/homebrew/bin/openspec validate diri-storage-core-facts --strict'
/usr/bin/make -C /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts parity-lock
/bin/zsh -lc 'cd /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts && /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/.githooks/pre-commit'
/usr/bin/git -C /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts diff --check

命令因工具要求必须从 repo root 时，使用 /bin/zsh -lc 加绝对 cd，不设置/导出环境变量。
OpenSpec 必须 4/4 且 strict valid；parity 不得因 foundation 提前关闭 downstream rows；secret
hook 和 diff-check pass。无关既存 blocker 必须记录 exact command/reason/status，不能报 pass。
修复必须退回原 owner；本 packet 不编辑 forbidden files。修复合并后原样重跑形成 GREEN。

清理所有 fixture-owned resources/hook temp output。运行 status，确认仅 evidence。PASS 后提交：
test(storage): record T-103 workspace quality gates
不 amend/rebase/push。handoff 报告 base/result SHA、每条 gate 状态/count、changed file、
cleanup、diff-check、blocked unrelated gates。
```

### T103-P28 / S103-EVIDENCE-06

- **Packet ID:** `T103-P28`
- **依赖:** `T103-P27` PASS
- **优先级:** `P0`
- **Owner:** `S103-release`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/release-readiness-report.md`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/checksums.sha256`
- **Forbidden paths:** 全部 product/test/spec/plan、parity lock、master tasks 和其他 evidence
- **RED command:** requirement/evidence completeness scan；缺映射/SHA/限制即 RED
- **GREEN command:**
  - `/usr/bin/shasum -a 256 /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/*.md`
  - `/usr/bin/git -C /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts diff --check`
- **Deadline:** `2h`
- **Cleanup:** checksum temp file 原子替换；无 process
- **Expected result:** FR-to-test/evidence matrix、changed inventory、commit chain、SHA256、known
  limitations、truthful parity boundary 完整，release readiness 明确 PASS/BLOCKED
- **Handoff/commit contract:** PASS 后 commit
  `docs(verification): publish T-103 release readiness`；该 SHA 交 P29

**完整可复制 prompt：**

```text
执行 T103-P28 / S103-EVIDENCE-06，owner=S103-release，priority=P0，依赖 T103-P27 PASS，
deadline=2h。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 PRD/OpenSpec/tasks/alignment/delegation、
P01-P27 commits 和 docs/verification/diri-storage-core-facts 全部 evidence，做 branch/status/
base preflight。禁止环境变量配置。

只允许写：
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/release-readiness-report.md
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/checksums.sha256
禁止 product/test/spec/plan、parity lock、master tasks 和其他 evidence。

先做 RED completeness scan：FR-01..FR-12 每项必须映射 RED、GREEN、actual command、evidence；
列出完整 changed-file inventory、P11/T-102 cross-handoff SHA、implementation commit chain、
baseline/final counts、migration/repository/restart/service/security/review/quality gates、known
limitations。任一 required evidence 缺失即 BLOCKED，不写 readiness PASS。

明确 truthful boundary：本 change 只完成 storage core facts foundation；不得把 UI-005/UI-006/
API-005/REM-001..003/USAGE-001/UPDATE-001/PKG-001/PERF-001 或 remote/updater workflow 标为
implemented。生成所有 markdown evidence 的 SHA256：
/usr/bin/shasum -a 256 /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts/docs/verification/diri-storage-core-facts/*.md
把稳定结果写入 checksums.sha256，不包含 checksums 文件自身。运行：
/usr/bin/git -C /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts diff --check

无 process 资源。PASS 后只 stage 两个 allowed files，提交：
docs(verification): publish T-103 release readiness
不 amend/rebase/push。handoff 报告 base/result 40 位 SHA、readiness PASS/BLOCKED、FR matrix、
checksums、changed files、cleanup、diff-check，交 T103-P29。
```

### T103-P29 / S103-EVIDENCE-07

- **Packet ID:** `T103-P29`
- **依赖:** `T103-P28` release readiness PASS 且 commit 已存在
- **优先级:** `P0`
- **Owner:** `S103-release`
- **Worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts`
- **Branch:** `wave1c/diri-storage-core-facts`
- **Allowed write paths:** 无
- **Allowed non-file side effect:** 仅通过 `/opt/homebrew/bin/bd` 更新/关闭 Bead
  `homie-t3u.2`
- **Forbidden paths:** worktree 内全部文件；不得修改 report/checksum/product/spec/parity
- **RED command:**
  - `/opt/homebrew/bin/bd show homie-t3u.2 --long`
  - `/usr/bin/git -C /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts status --short`
- **GREEN command:**
  - `/opt/homebrew/bin/bd close homie-t3u.2 --reason "Implemented and verified. See docs/verification/diri-storage-core-facts/release-readiness-report.md."`
  - `/opt/homebrew/bin/bd show homie-t3u.2 --long`
- **Deadline:** `30m`
- **Cleanup:** 无文件/process；确认 worktree 状态未因本 packet 改变
- **Expected result:** 仅当 evidence 完整且 readiness PASS 时关闭 Bead；否则保持 open/blocked 并
  报 exact reason
- **Handoff/commit contract:** 本 packet 不创建 Git commit、不 push；报告最终 Bead 状态和
  P28 release SHA

**完整可复制 prompt：**

```text
执行 T103-P29 / S103-EVIDENCE-07，owner=S103-release，priority=P0，依赖 T103-P28 release
readiness PASS 且 commit 已存在，deadline=30m。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts，branch
wave1c/diri-storage-core-facts。读取 AGENTS.md、T-103 tasks、release-readiness-report.md、
checksums.sha256 和 P28 40 位 commit SHA，做 branch/status/base preflight。禁止环境变量配置。

allowed write paths：无。worktree 全部文件 forbidden。唯一允许副作用是通过
/opt/homebrew/bin/bd 更新 Bead homie-t3u.2；不得直接编辑 Beads 存储文件。

RED/precondition：
/opt/homebrew/bin/bd show homie-t3u.2 --long
/usr/bin/git -C /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts status --short
逐项确认 P28 report 明确 PASS、FR-01..FR-12 evidence 完整、checksums 可复验、P0/P1=0、所需
commands pass、known limitations/parity boundary 真实。任一缺失则不得 close；保持
IN_PROGRESS 或用 bd update 标 blocked 并写 exact reason。

全部满足时执行：
/opt/homebrew/bin/bd close homie-t3u.2 --reason "Implemented and verified. See docs/verification/diri-storage-core-facts/release-readiness-report.md."
/opt/homebrew/bin/bd show homie-t3u.2 --long
确认状态 CLOSED 且 reason 正确。再次运行 git status，必须与 packet 开始一致。

本 packet 不创建 Git commit、不 amend/rebase/push，不修改任何文件。handoff 报告 P28 release
SHA、最终 Bead 状态、执行命令、worktree status before/after、cleanup=none。
```

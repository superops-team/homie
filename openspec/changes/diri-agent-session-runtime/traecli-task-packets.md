# T-102 TraeCLI Task Packets

```yaml
change_id: diri-agent-session-runtime
master_task: T-102
bead: homie-t3u.1
checkpoint_commit: 48f522b
worktree: /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
branch: wave1b/diri-agent-session-runtime
status: ready_for_traecli
product_implementation_started: false
packet_count: 21
```

## 1. 使用约束

- 本文件只定义后续 TraeCLI 执行包，不代表已启动实现。
- 每次 TraeCLI 只执行一个 packet，且只有该 packet 声明的单一 owner 可以写入。
- 所有 packet 从上述绝对 worktree 和 branch 执行；不得在 coordinator worktree
  `/Users/bytedance/workspace/github/homie` 写产品代码。
- 所有 fixture data dir、cwd、executable、socket、pid/status file 路径必须是绝对路径。
- 不新增或使用环境变量配置，不使用 `HOMIE_*` override，不以环境变量注入 manifest、
  holder path、agent binary、timeout、governor 或 test mode。
- 禁止 `pkill`、按进程名 kill、清理用户 data dir、清理测试前已存在的 holder。
- 进程名只可用于测试前后 holder PID+start-time 集合观测。
- T-102 不直接编辑 `crates/homie-storage/**`。T-103
  `S103-GREEN-02` 是 effective-config schema/repository 的唯一 owner。
- shared file 按 `G1 -> G5 -> G6 -> G7 -> G9 -> G10 -> G11 -> F1`
  串行交接；发现前 owner 未提交或存在其他 owner dirty hunk 时立即 `blocked`。
- 每个 pass packet 只 stage allowed write paths，运行
  `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/.githooks/pre-commit`
  后创建声明的 conventional commit；不 amend、不 rebase shared milestone、不 push。
- 任一命令超时、RED 语义不匹配、需要 forbidden path、fixture residual 非零或 holder
  after-minus-before 非空时，不提交，返回 `blocked` 或 `fail`。
- 每次 completion report 固定包含：

```text
packet_id:
owner:
status: pass | blocked | fail
base_commit:
commit_sha:
files_changed:
commands_and_exit_codes:
test_counts:
deadline_result:
cleanup_residual_count:
holder_baseline_count:
holder_after_count:
holder_added_set:
forbidden_path_check:
remaining_risks:
handoff:
```

## 2. RED Packets

### Packet T102-R1-RED-BASELINE

- **Packet ID:** `T102-R1-RED-BASELINE`
- **Owner:** `trae-r-base` / `R-BASE`
- **依赖:** coordinator `C-00/C-01/C-02` 完成；worktree clean；规格基线已提交
- **优先级:** `P0 / Wave A`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；integration test binary `120s`；cleanup `3s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/session_lifecycle.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/process_fixture_cleanup.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/support/process_fixture.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/support/mod.rs`
- **Forbidden paths:** 所有未列路径；尤其 `crates/homie-runtime/src/**`、
  `crates/homie-storage/**`、`prd-spec/**`、`openspec/**`、`specs/**`、
  `docs/verification/**`、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --nocapture
```

  Expected: exit non-zero；精确 `14 tests: 12 passed, 2 failed`；仅
  `runtime_reopen_can_adopt_holder_and_continue_session` 和
  `runtime_spawn_shell_uses_live_pty` 为历史 RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle runtime_holder_stat_tracks_resize_and_log_offsets -- --exact --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test process_fixture_cleanup -- --nocapture
```

- **Cleanup:** panic-safe ledger 记录 absolute temp dir、daemon/holder/root PID+start-time、
  socket/pid/status paths；pass、RED assertion、panic、timeout 均只清 ledger；最终 residual
  `0` 且 holder after-minus-before 为空。
- **Expected result:** 固化 2 RED / 1 retained GREEN；新增 cleanup self-test；不改 production。
- **Handoff/commit contract:** pass 后 commit
  `test(runtime): harden lifecycle process fixture`；报告 exact counts、cleanup 和 SHA；R2/R5/R6
  只接受该 commit 的 clean handoff。
- **完整可复制 prompt:**

```text
执行 packet T102-R1-RED-BASELINE。唯一 owner 是 trae-r-base/R-BASE。只在
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime 的
wave1b/diri-agent-session-runtime 分支工作，active deadline 4h，单 test binary 120s，
cleanup 3s。开始时用 /usr/bin/git branch --show-current 和 /usr/bin/git status --short
确认 branch 正确且 clean；否则 blocked，不 stash/reset/checkout 覆盖。

先完整读取 AGENTS.md、docs/architecture/project-layout.md、docs/development/standards.md、
docs/development/quality-gates.md、docs/research/rust-package-selection.md、T-102 PRD、
openspec/changes/diri-agent-session-runtime/{design.md,plan.md,tasks.md,delegation-plan.md} 和
specs/holder-pty-continuity/spec.md。只可写
crates/homie-runtime/tests/session_lifecycle.rs、
crates/homie-runtime/tests/process_fixture_cleanup.rs、
crates/homie-runtime/tests/support/process_fixture.rs、
crates/homie-runtime/tests/support/mod.rs；禁止所有其他路径，尤其 production source、
homie-storage、规格、evidence、.beads 和 diri。

严格 TDD。保留两个既有 detached != running assertion 的语义和名称，保持
runtime_holder_stat_tracks_resize_and_log_offsets GREEN；`session_lifecycle` 必须始终保持
14 tests / 12 passed / 2 failed 的 RED 基线，不得在该 binary 新增 cleanup self-test。
cleanup self-test 只放在独立的 `process_fixture_cleanup` test binary。建立 panic-safe fixture ledger：
测试前采样 holder PID+start-time；记录 absolute temp data dir、daemon/holder/root
PID+start-time、socket/pid/status paths；正常返回、RED assertion failure、panic、timeout
都进入 Drop/guard cleanup；最多 3s 后只对 start-time 仍匹配的 ledger PID 做兜底并 reap；
只删除 fixture temp dir。进程名可观测，不可 kill；禁止 pkill 和用户目录。

RED 运行：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --nocapture
结果必须精确为 12 passed / 2 failed，失败只能是
runtime_reopen_can_adopt_holder_and_continue_session 与
runtime_spawn_shell_uses_live_pty。GREEN 回归运行：
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle runtime_holder_stat_tracks_resize_and_log_offsets -- --exact --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test process_fixture_cleanup -- --nocapture
每次结束必须 residual=0 且 holder after-minus-before 为空。若计数、失败语义或 cleanup
不符，fail，不写 GREEN production。

完成后运行 /usr/bin/git diff --check 和 /usr/bin/git diff --name-only，确认只有 allowed
paths。只 stage allowed paths，运行
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/.githooks/pre-commit，
pass 后提交 test(runtime): harden lifecycle process fixture；不 amend/rebase/push。按固定
completion report 返回 commit SHA、命令 exit/count、deadline、cleanup 和 R2/R5/R6 handoff。
```

### Packet T102-R2-RED-RECONCILIATION

- **Packet ID:** `T102-R2-RED-RECONCILIATION`
- **Owner:** `trae-r-base` / `R-BASE`
- **依赖:** `T102-R1-RED-BASELINE` pass commit
- **优先级:** `P0 / Wave B foundation`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；integration test binary `120s`；cleanup `3s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/startup_reconciliation.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/support/process_fixture.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/support/mod.rs`
- **Forbidden paths:** 所有未列路径；尤其全部 production source、`homie-storage/**`、
  `session_lifecycle.rs` 的既有 assertion、规格/evidence/tracking、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test startup_reconciliation -- --nocapture
```

  Expected: truth-table cases 因 bulk-detach-before-adopt 等缺口定向失败。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle runtime_holder_stat_tracks_resize_and_log_offsets -- --exact --nocapture
```

- **Cleanup:** 复用 R1 ledger；每个 case residual `0`；after-minus-before holder set 为空。
- **Expected result:** created/starting/running/detached、idle/needs-input、missing、exited、
  hibernated+stopped、archived+live contradiction、duplicate prevention 均有精确 RED。
- **Handoff/commit contract:** pass 后 commit
  `test(runtime): add startup reconciliation red cases`；handoff 给 G1，包含每个 case 的失败原因。
- **完整可复制 prompt:**

```text
执行 packet T102-R2-RED-RECONCILIATION。唯一 owner 是 trae-r-base/R-BASE。worktree 固定
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，branch 固定
wave1b/diri-agent-session-runtime，active deadline 4h，test 120s，cleanup 3s。确认 R1 pass
commit 已在 HEAD ancestry 且 worktree clean，否则 blocked。

读取 AGENTS.md、T-102 PRD、design.md、plan.md 的 R2/G1、tasks.md 1.2、
delegation-plan.md、specs/holder-pty-continuity/spec.md 和 R1 fixture。只可写
crates/homie-runtime/tests/startup_reconciliation.rs，并仅在必要时复用/扩展
tests/support/process_fixture.rs 与 mod.rs。禁止 production source、homie-storage、既有
session_lifecycle assertion、规格/evidence/tracking、.beads、diri。

只写 RED，不改 production。用表驱动 case 精确覆盖：
created/starting/running/detached + verified running holder -> adopted/running；
idle/needs_input + live holder -> adopted 且保留行为状态；
live storage + missing holder -> detached；
explicit holder exit -> exited；
hibernated + stopped holder -> adopted/hibernated；
archived + live holder -> recovery contradiction；
任何 adoption/relaunch 都不产生第二 holder/child。
所有 fixture 使用 absolute paths、R1 panic-safe PID+start-time ledger 和 bounded cleanup。

RED 命令：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test startup_reconciliation -- --nocapture
必须因当前缺失行为定向失败，不允许 compile-only 假 RED。retained GREEN：
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle runtime_holder_stat_tracks_resize_and_log_offsets -- --exact --nocapture
必须通过。每个命令后 residual=0、holder added set 为空。RED 不符合预期时停止并报告。

运行 git diff --check/name-only，确认只含 allowed paths；stage allowed paths，运行绝对
pre-commit hook；提交 test(runtime): add startup reconciliation red cases。禁止 amend、
rebase、push。返回固定 report，handoff 必须逐项列出 G1 应关闭的 RED 和 cleanup 证据。
```

### Packet T102-R3-RED-MANIFEST

- **Packet ID:** `T102-R3-RED-MANIFEST`
- **Owner:** `trae-r-manifest` / `G-AGENT-PLAN`
- **依赖:** coordinator preflight；可与 R1/R4 并行，但不得共享文件
- **优先级:** `P0 / Wave A / cross-change critical path`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；每个 test binary `120s`；cleanup `3s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-agents/tests/runtime_launch_plan.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/manifest_spawn.rs`
- **Forbidden paths:** 所有未列路径；尤其 `crates/**/src/**`、`assets/agent-descriptors/**`、
  `crates/homie-storage/**`、规格/evidence/tracking、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-agents --test runtime_launch_plan -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test manifest_spawn -- --nocapture
```

  Expected: immutable resolved contract 和 real manifest holder spawn 缺失导致目标 RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-agents --test manifest_catalog
/opt/homebrew/bin/cargo test -p homie-agents --test hook_parser
```

- **Cleanup:** test temp executable、holder/child/socket ledger 全部归零；holder added set 为空。
- **Expected result:** compiled catalog closure、absolute executable、argv boundary、env scrub、
  resolved fields、explicit shell、unknown no-fallback、real fake executable/PTY 均有 RED。
- **Handoff/commit contract:** pass 后 commit
  `test(runtime): add manifest launch red cases`；handoff 给 G3，并明确哪些 runtime cases 留给 G5。
- **完整可复制 prompt:**

```text
执行 packet T102-R3-RED-MANIFEST。唯一 owner 是 trae-r-manifest/G-AGENT-PLAN。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch
wave1b/diri-agent-session-runtime，active deadline 4h，每个 test 120s，cleanup 3s。确认 clean。

完整读取 AGENTS.md、开发标准/质量门禁、T-102 PRD FR-04/FR-10/FR-11、design Decision 4-6、
plan R3/G3/G5、tasks 1.3、delegation plan、specs/manifest-agent-runtime/spec.md 和
holder-pty-continuity 的 structured launch/cleanup requirements。只可写两个新测试文件：
crates/homie-agents/tests/runtime_launch_plan.rs 和
crates/homie-runtime/tests/manifest_spawn.rs。禁止任何 production source、descriptor JSON、
homie-storage、规格/evidence/tracking。

只写 RED。覆盖显式 include_str compiled catalog inventory closure；packaged binary 不从
cwd/PATH/external resource 找 manifest；readiness 返回 absolute executable 但不执行 agent；
profile/runtime/LLM/permission ids、manifest id/version/status authority、absolute executable、
final argv、sanitized env、injection、resume、geometry/cwd/parent 的 resolved contract；
argv boundary；敏感 parent env scrub；explicit shell；unknown/disabled/unavailable 无 shell
fallback；real local fake executable 经真实 holder/PTY 输出 exact argv/env。测试注入只走 Rust
constructor/fixture，不新增环境配置或 production test mode。所有路径绝对，fixture panic-safe。

运行：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-agents --test runtime_launch_plan -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test manifest_spawn -- --nocapture
必须因缺失 contract/fixed shell 行为定向 RED。再运行既有 GREEN：
/opt/homebrew/bin/cargo test -p homie-agents --test manifest_catalog
/opt/homebrew/bin/cargo test -p homie-agents --test hook_parser
不得退化。每轮 cleanup residual=0、holder added set 为空。

确认 diff 只有 allowed paths，git diff --check 通过；stage 后运行绝对 pre-commit hook；提交
test(runtime): add manifest launch red cases。返回 SHA、每个 RED 的期望缺口、retained GREEN、
cleanup；handoff 明确 agent contract cases 由 G3 关闭，actor/real holder spawn cases 保持 RED
直到 G5。
```

### Packet T102-R4-RED-STATUS

- **Packet ID:** `T102-R4-RED-STATUS`
- **Owner:** `trae-r-status` / `G-STATUS`
- **依赖:** coordinator preflight；可与 R1/R3 并行
- **优先级:** `P0 / Wave A`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；test binary `120s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/runtime_status_engine.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-agents/tests/status_reducer.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-agents/tests/hook_parser.rs`
- **Forbidden paths:** 所有未列路径；尤其全部 production source、`homie-storage/**`、
  规格/evidence/tracking、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine -- --nocapture
```

- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-agents --test status_reducer
/opt/homebrew/bin/cargo test -p homie-agents --test hook_parser
```

- **Cleanup:** temp event/storage dirs 删除；无 worker/process/socket residual。
- **Expected result:** stateful reducer、side-effect-free read、signal convergence、
  commit-before-event、subagent isolation、restart reconstruction 形成目标 RED。
- **Handoff/commit contract:** pass 后 commit
  `test(runtime): add status reducer red cases`；handoff 给 G6/G7，区分 core 与 external ingress。
- **完整可复制 prompt:**

```text
执行 packet T102-R4-RED-STATUS。唯一 owner 是 trae-r-status/G-STATUS。固定 worktree/branch 为
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime 和
wave1b/diri-agent-session-runtime，active deadline 4h，test 120s。worktree 非 clean 则 blocked。

读取 AGENTS.md、T-102 PRD FR-05、design Decision 7-8、plan R4/G6/G7、tasks 1.4、
delegation-plan、specs/runtime-status-governor/spec.md 的 reducer/hook/screen requirements。
只可写 runtime_status_engine.rs、现有 status_reducer.rs 和 hook_parser.rs 测试；禁止 production、
storage、规格/evidence。

只写 RED，覆盖每 live session 一个由 frozen manifest authority 创建的 reducer；重复 read
无副作用且不重放完整 output；holder process、PTY output、manifest screen、hook、notify、
user input、tick 汇入同一 reducer；persist-before-event；subagent 不覆盖 parent；
restart 由 holder + persisted behavior + checkpoint + bounded output 重建。测试不得通过
source string matching 或 direct storage write 伪证。core reducer case 使用 `core_` test
name prefix，external hook/notify ingress case 使用 `external_` prefix，供 G6/G7 精确分段。

RED：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine -- --nocapture
必须对当前 fresh ScreenPrimary/direct status write 定向失败。GREEN 回归：
/opt/homebrew/bin/cargo test -p homie-agents --test status_reducer
/opt/homebrew/bin/cargo test -p homie-agents --test hook_parser
必须通过。删除 temp facts，确认无 process/worker leak。

git diff --check/name-only 只允许声明路径；stage 后运行绝对 pre-commit hook；提交
test(runtime): add status reducer red cases。completion report 将 core reducer RED 交 G6，
external hook/notify ingress RED 交 G7，并附命令 counts/cleanup/SHA。
```

### Packet T102-R5-RED-PROCESS

- **Packet ID:** `T102-R5-RED-PROCESS`
- **Owner:** `trae-r-process` / `G-PROCESS`
- **依赖:** `T102-R1-RED-BASELINE` fixture handoff
- **优先级:** `P0 / Wave B foundation`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；每个 process case `60s`；cleanup `3s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/process_tree.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/resource_governor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/support/process_fixture.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/support/mod.rs`
- **Forbidden paths:** 所有未列路径；尤其 production source、`homie-storage/**`、规格/evidence、
  `.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test process_tree -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test resource_governor -- --test-threads=1 --nocapture
```

- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle runtime_terminate_kills_detached_child_tree -- --exact --nocapture
```

- **Cleanup:** PID start-time checked exact tree cleanup；不 global signal；residual `0`。
- **Expected result:** STOP/CONT/PID reuse/sample/governor/hibernate cases RED；terminate retained GREEN。
- **Handoff/commit contract:** pass 后 commit
  `test(runtime): add process governor red cases`；handoff G8/G9。
- **完整可复制 prompt:**

```text
执行 packet T102-R5-RED-PROCESS。唯一 owner 是 trae-r-process/G-PROCESS。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime 和 branch
wave1b/diri-agent-session-runtime。active 4h，每 process case 60s，cleanup 3s。确认 R1 fixture
commit 已合入且 clean。

读取 AGENTS.md、T-102 PRD FR-06/FR-07/FR-11、design Decision 9-11/15、plan R5/G8/G9、
tasks 1.5、delegation-plan、specs/runtime-status-governor/spec.md 的 process/resource/
hibernate requirements。只可写 tests/process_tree.rs、tests/resource_governor.rs 和 R1 test
support；禁止 production/storage/spec/evidence。

只写 RED：真实 child tree STOP 且验证 stopped；leaves-first CONT 且 child identity 不变；
PID start-time reuse 防护；tree size/memory footprint；race/unsupported 为 unknown 且不 kill；
仅 idle+unattached+unpinned eligible；running/needs_input protected；same holder/child/PTY/
offset hibernate-wake；hibernated input stable error。fixture 全部 absolute path 和 panic-safe
ledger；禁止 pkill/global process-group guessing。

RED 依次运行：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test process_tree -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test resource_governor -- --test-threads=1 --nocapture
应定向失败。retained GREEN：
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle runtime_terminate_kills_detached_child_tree -- --exact --nocapture
必须通过。每轮 residual=0、holder added set 为空。

只 stage allowed paths，git diff --check 和绝对 pre-commit hook 通过后提交
test(runtime): add process governor red cases。不 amend/rebase/push。report 将 process
primitive RED 交 G8、governor/hibernate RED 交 G9，并附 exact cleanup。
```

### Packet T102-R6-RED-RECOVERY

- **Packet ID:** `T102-R6-RED-RECOVERY`
- **Owner:** `trae-r-recovery` / `G-RECOVERY`
- **依赖:** `T102-R1-RED-BASELINE` fixture handoff
- **优先级:** `P0 / Wave B foundation`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；每 process case `60s`；cleanup `3s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/session_recovery.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/runtime_dispatcher.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/server_control.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/daemon_process.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/support/process_fixture.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/support/mod.rs`
- **Forbidden paths:** 所有未列路径；尤其 production source、`homie-storage/**`、规格/evidence、
  `.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1 --nocapture
```

- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test daemon_process live_holder_survives_daemon_sigterm_and_is_cleaned_up_explicitly_after_restart -- --exact --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test server_control successful_daemon_shutdown_flushes_response_before_eof_and_server_exit -- --exact --nocapture
```

- **Cleanup:** ledger daemon/holder/child/socket/pid/status 全部归零；用户 holder 不受影响。
- **Expected result:** direct resume/relaunch/unarchive/prepare/flush/restart continuity 形成目标 RED。
- **Handoff/commit contract:** pass 后 commit
  `test(runtime): add recovery shutdown red cases`；handoff G10/G11。
- **完整可复制 prompt:**

```text
执行 packet T102-R6-RED-RECOVERY。唯一 owner trae-r-recovery/G-RECOVERY。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，branch
wave1b/diri-agent-session-runtime，active 4h，每 process case 60s，cleanup 3s。确认 R1 已合入且 clean。

读取 AGENTS.md、T-102 PRD FR-08/FR-09/FR-11、design Decision 12-15、plan R6/G10/G11、
tasks 1.6、delegation-plan、specs/local-session-recovery/spec.md。只可写声明的 recovery 和
focused shutdown tests/test support；禁止 production/storage/spec/evidence。

只写 RED：manifest ID/latest direct argv；missing ID fail closed；adopt existing live holder
before relaunch；same Homie ID/new epoch；preserve title/parent/profile/permission/checkpoint；
failed readiness retryable；unarchive no spawn；prepare rejects new lifecycle mutation and flushes
canonical facts；graceful shutdown preserves live/hibernated holder；hard restart adopts and
continues。禁止添加 remote session.migrate placeholder。所有进程使用 absolute paths 和 R1
panic-safe ledger。

RED：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1 --nocapture
必须因当前 shell-text resume/incomplete lifecycle flush 定向失败。retained GREEN：
/opt/homebrew/bin/cargo test -p homie-runtime --test daemon_process live_holder_survives_daemon_sigterm_and_is_cleaned_up_explicitly_after_restart -- --exact --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test server_control successful_daemon_shutdown_flushes_response_before_eof_and_server_exit -- --exact --nocapture
必须通过。每轮 residual=0、holder added set 空。

diff 只含 allowed paths；git diff --check、staged secret hook 通过后提交
test(runtime): add recovery shutdown red cases。report 将 resume/relaunch RED 交 G10，
prepare/shutdown RED 交 G11，并附 SHA/counts/cleanup。
```

## 3. GREEN Packets

### Packet T102-G1-GREEN-RECONCILIATION

- **Packet ID:** `T102-G1-GREEN-RECONCILIATION`
- **Owner:** `trae-g-reconcile` / `G-RECONCILE`
- **依赖:** R1、R2 pass commits
- **优先级:** `P0 / Wave B`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；focused suite `10m`；test binary `120s`；cleanup `3s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/reconciliation.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/startup_reconciliation.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/session_lifecycle.rs`
- **Forbidden paths:** 所有未列路径；尤其 holder binary/client files、`runtime_actor.rs`、
  `homie-storage/**`、proto/client/CLI、规格/evidence/tracking、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test startup_reconciliation -- --test-threads=1 --nocapture
```

  修改 production 前仍应看到 exact historical RED 和 reconciliation RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test startup_reconciliation -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
```

- **Cleanup:** 每 suite residual `0`；holder added set 为空；不启动 duplicate child。
- **Expected result:** persisted fact -> holder probe -> one outcome -> projection -> registry；
  lifecycle `14/14`，retained stat GREEN。
- **Handoff/commit contract:** pass 后 commit
  `fix(runtime): reconcile live holders before projection`；冻结 adoption interface，handoff G2/G5。
- **完整可复制 prompt:**

```text
执行 T102-G1-GREEN-RECONCILIATION。唯一 owner trae-g-reconcile/G-RECONCILE。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，branch
wave1b/diri-agent-session-runtime，active 4h，suite 10m，test 120s，cleanup 3s。确认 R1/R2 commits
在 ancestry、worktree clean、lib.rs 无其他 owner dirty hunk。

读取 AGENTS.md、T-102 PRD FR-01..03、design Decision 1-3、plan G1、tasks 2.1、
delegation-plan、holder continuity spec 和 R1/R2 tests。只可写 reconciliation.rs、lib.rs 的
focused startup/adoption hunk，以及两项测试文件；禁止 runtime_actor、holder binary、
storage、proto/client/CLI、规格/evidence。

先运行 R1/R2 RED 并记录。然后最小实现 persisted fact -> probe expected holder -> classify one
ReconciliationOutcome -> persist projection -> insert live registry。startup 不得先 bulk detach；
verified created/starting/running/detached -> running；verified idle/needs_input 保留行为状态；
missing evidence -> detached；explicit exit -> exited；hibernated stopped holder 保留；
archived+live 报 contradiction。holder Stat/live response 是 liveness authority，storage row
不是。只删除 startup call，不删除可能仍被其他 owner 使用的 storage method。不得启动第二
holder/child。

GREEN：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test startup_reconciliation -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
必须全部通过、lifecycle 14/14、holder stat 保持 GREEN、residual=0、holder added set 空。
不弱化/改名 RED assertion。

检查 diff 仅 allowed paths，git diff --check；stage 后跑绝对 pre-commit hook；提交
fix(runtime): reconcile live holders before projection。返回 frozen reconciliation/adoption
API、SHA、counts、cleanup，明确释放 lib.rs 给后续 G5 且 G2 可开始。
```

### Packet T102-G2-GREEN-HOLDER

- **Packet ID:** `T102-G2-GREEN-HOLDER`
- **Owner:** `trae-g-holder` / `G-HOLDER`
- **依赖:** G1 pass/frozen adoption interface
- **优先级:** `P0 / Wave B`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `6h`；holder IPC `350ms`；readiness `3s`；cleanup `3s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/holder.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/bin/homie-runtime-holder.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/holder_protocol.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/session_lifecycle.rs`
- **Forbidden paths:** 所有未列路径；尤其 `lib.rs`、`runtime_actor.rs`、`process_tree.rs`、
  `homie-storage/**`、proto/client/CLI、规格/evidence、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test holder_protocol -- --test-threads=1 --nocapture
```

  先创建/确认 holder protocol tests；必须在 structured launch/STOP/CONT/sample request
  缺口上 RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test holder_protocol -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle runtime_holder_stat_tracks_resize_and_log_offsets -- --exact --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime holder --lib
```

- **Cleanup:** terminate <=`3s`；control files/residual `0`；holder added set 空。
- **Expected result:** structured argv/cwd/env/geometry、additive control/stat、owner-only one-shot
  transport；旧 holder stat/adoption 可解析；不记录 argv/env/raw secret。
- **Handoff/commit contract:** pass 后 commit
  `feat(runtime): add structured holder control`；handoff request/response shape 给 G8/G5。
- **完整可复制 prompt:**

```text
执行 T102-G2-GREEN-HOLDER。唯一 owner trae-g-holder/G-HOLDER。固定 worktree/branch：
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime /
wave1b/diri-agent-session-runtime。active 6h，IPC 350ms，readiness 3s，cleanup 3s。确认 G1 pass、
worktree clean、holder files 无其他 owner dirty hunk。

读取 AGENTS.md、T-102 PRD FR-03/FR-06/FR-10/FR-11、design Decision 3/4/9/15、plan G2、
tasks 2.2、delegation plan、holder continuity spec。只可写 holder.rs、holder binary、
holder_protocol.rs 和 holder-stat focused test hunk；禁止 lib/runtime_actor/process_tree、
storage、proto/client/CLI。

严格 TDD：先用 holder_protocol.rs 证明 structured absolute executable + argv boundaries +
absolute cwd + sanitized env + geometry、invalid plan reject-before-child、additive STOP/CONT/
sample request、bounded timeout、owner-only one-shot launch transport。RED 后最小实现。不得
记录 argv/env/raw key；不得加入 fixed-agent fallback；现存 live holder stat/adoption 必须通过
additive fields 保持可解析。

GREEN 运行：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test holder_protocol -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle runtime_holder_stat_tracks_resize_and_log_offsets -- --exact --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime holder --lib
IPC/readiness/cleanup 均有界，residual=0，holder added set 空。不要实现 G8 的 process-tree
算法，只冻结 holder request/response shape。

diff 只含 allowed paths；git diff --check；stage + absolute pre-commit hook；提交
feat(runtime): add structured holder control。report 给出 request/response schema、安全字段、
timeout、SHA、cleanup，并释放 holder.rs 给 G8 integration；holder binary 后续不再共享。
```

### Packet T102-G3-GREEN-AGENT-PLAN

- **Packet ID:** `T102-G3-GREEN-AGENT-PLAN`
- **Owner:** `trae-g-agent-plan` / `G-AGENT-PLAN`
- **依赖:** R3 pass commit
- **优先级:** `P0 / Wave B / H-01 milestone`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `6h`；package suite `10m`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-agents/src/launch.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-agents/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-agents/tests/runtime_launch_plan.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-agents/tests/manifest_catalog.rs`
- **Forbidden paths:** 所有未列路径；尤其 `assets/agent-descriptors/**`、
  `crates/homie-runtime/src/**`、`crates/homie-storage/**`、proto/client/CLI、规格/evidence、
  `.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-agents --test runtime_launch_plan -- --nocapture
```

  实现前必须为 contract RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-agents --test runtime_launch_plan -- --nocapture
/opt/homebrew/bin/cargo test -p homie-agents
/opt/homebrew/bin/rg -n "Authorization|Bearer|api[_-]?key|provider[_-]?key|cookie" crates/homie-agents/src/launch.rs crates/homie-agents/src/lib.rs
```

  Last scan must have no secret-bearing value/log path finding；identifier/reference names alone
  must be reviewed, not hidden.
- **Cleanup:** readiness 不启动 agent；temp executable 删除；无 process residual。
- **Expected result:** immutable compiled catalog 和 exact `ResolvedEffectiveAgentConfig` contract
  GREEN；runtime `manifest_spawn` 的 actor cases 继续等待 G5。
- **Handoff/commit contract:** 创建单独 H-01 commit
  `feat(agents): freeze manifest launch contracts`；报告 exact SHA、字段、安全约束、tests、
  no-secret scan，明确无 storage 改动；T-103 通过 `git show` 读取，不 merge T-102 branch。
- **完整可复制 prompt:**

```text
执行 T102-G3-GREEN-AGENT-PLAN。唯一 owner trae-g-agent-plan/G-AGENT-PLAN。这是 H-01
cross-change milestone。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime、branch
wave1b/diri-agent-session-runtime，active 6h、package 10m。确认 R3 pass、clean。

读取 AGENTS.md、T-102 PRD FR-04/FR-10、design Decision 4-6、plan G3、tasks 2.3、
delegation-plan、manifest-agent-runtime spec、T-103 effective-agent-config-facts spec 和
wave-1bc-traecli-orchestration H-01/H-02。只可写 homie-agents 的 launch.rs、lib.rs、
runtime_launch_plan.rs、manifest_catalog.rs。descriptor JSON 只读；禁止 runtime source、
homie-storage、proto/client/CLI、规格/evidence。

先运行 R3 agent RED。最小实现 EffectiveAgentConfig、ResolvedAgentExecutable、
AgentLaunchPlan、AgentResumePlan、LaunchPlanError 以及对 T-103 冻结的
ResolvedEffectiveAgentConfig。后者字段必须完整覆盖 profile/runtime/LLM/permission ids、
manifest id/version/status authority、absolute executable、final argv、sanitized non-secret
env key/value、injection decision、resume semantics、initial geometry、absolute cwd、parent
session。所有 committed assets/agent-descriptors/*.json 通过显式 include_str! table 编译进
immutable production catalog，并有 inventory completeness test；packaged daemon/CLI 不依赖
cwd/PATH/external resource 找 manifest。readiness 可用 bounded login-shell resolver 解析
binary，但只返回唯一 absolute executable regular file，不执行 agent，不接受 caller shell
fragment。测试 catalog 只经 constructor 注入。explicit shell 是唯一 shell fallback。
Debug/Display 必须 redacted。

GREEN：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-agents --test runtime_launch_plan -- --nocapture
/opt/homebrew/bin/cargo test -p homie-agents
/opt/homebrew/bin/rg -n "Authorization|Bearer|api[_-]?key|provider[_-]?key|cookie" crates/homie-agents/src/launch.rs crates/homie-agents/src/lib.rs
测试全过；scan 不得发现 secret-bearing value/logging。temp executable 清理。不要修改 R3
runtime manifest_spawn assertion，它留给 G5。

确认 diff 无 storage/runtime，git diff --check；stage allowed paths，跑绝对 pre-commit hook；
提交且仅提交本 packet 的独立 commit：feat(agents): freeze manifest launch contracts。
completion report 必须给 exact 40-char SHA、完整字段表、安全/size/version invariant、测试
counts、no-secret scan、no-storage confirmation。handoff 给 coordinator/T-103
S103-GREEN-02；T-103 只 git show 此 SHA，不 merge 当前 T-102 branch。
```

### Packet T102-G5-GREEN-MANIFEST-SPAWN

- **Packet ID:** `T102-G5-GREEN-MANIFEST-SPAWN`
- **Owner:** `trae-g-spawn` / `G-SPAWN`
- **依赖:** G1、G2、G3 pass；T-103 `S103-GREEN-01` + `S103-GREEN-02` milestone exact SHA 已发布
- **优先级:** `P0 / Wave D / cross-change critical path`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `6h`；readiness `3s`；integration `120s`；cleanup `3s`
- **Allowed direct write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/agent_launch.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/dispatcher.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/runtime_actor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/manifest_spawn.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/daemon_process.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/runtime_dispatcher.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-proto/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-proto/src/model.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-proto/tests/protocol_contract.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-proto/tests/runtime_transport_contract.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-client/src/client.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-client/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-client/tests/typed_facade.rs`
- **Merge-only imported paths:** exact T-103 milestone 可通过 `git merge --no-ff` 引入
  `crates/homie-storage/src/lib.rs`、focused storage tests 和 handoff evidence；G5 不得直接编辑或
  解决其语义。
- **Forbidden paths:** 所有其他路径；尤其直接编辑 `crates/homie-storage/**`、holder/process
  files、CLI/UI/remote、规格、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test manifest_spawn -- --test-threads=1 --nocapture
```

  merge 后、G5 implementation 前必须仍是 actor/holder spawn RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-storage --tests
/opt/homebrew/bin/cargo test -p homie-agents
/opt/homebrew/bin/cargo test -p homie-runtime --test manifest_spawn -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test daemon_process --no-run
/opt/homebrew/bin/cargo test -p homie-runtime --lib dispatcher::tests::every_registered_handler_decodes_to_its_declared_execution_class -- --exact
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_dispatcher
/opt/homebrew/bin/cargo test -p homie-proto --test runtime_transport_contract
/opt/homebrew/bin/cargo test -p homie-client --test typed_facade
/usr/bin/git diff --check
```

- **Cleanup:** partial launch reverse rollback session/config/holder/child；residual `0`；added holder
  set 空。
- **Expected result:** 先 exact merge T-103 repository milestone，再完成 typed profile/explicit
  shell -> freeze/bind/readback -> real holder readiness -> running/event；无 fallback。
- **Handoff/commit contract:** 无 milestone SHA 则 `blocked`。禁止 cherry-pick。允许一个
  `--no-ff` H-02 merge commit，随后一个 packet commit
  `feat(runtime): launch manifest agents through holder`；merge conflict 不手工选边。
- **完整可复制 prompt:**

```text
执行 T102-G5-GREEN-MANIFEST-SPAWN。唯一 owner trae-g-spawn/G-SPAWN。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，branch
wave1b/diri-agent-session-runtime，active 6h，readiness 3s，integration 120s，cleanup 3s。
确认 G1/G2/G3 pass、shared lib.rs/runtime_actor.rs clean。

先读取 AGENTS.md、T-102 PRD FR-04/10/11、design Decision 4-6、plan G5、tasks 2.5、
delegation-plan、manifest/holder specs、T-103 effective-config spec、T-103 delegation plan
§5，以及 wave orchestration H-02。用
/opt/homebrew/bin/bd show homie-t3u.2 --long
读取 coordinator 记录的 S103-GREEN-01+02 exact 40-char milestone SHA。若 notes 中没有 exact
SHA、没有 storage test evidence、或 milestone 未同时包含 GREEN-01/02，立即 blocked，不实现。

在 clean worktree 中先用 /usr/bin/git show --stat 查看该 exact SHA，确认只含 v4 migration、
effective-config repository、focused storage tests、必要 handoff。将该 40-char SHA 直接作为
参数执行 /usr/bin/git merge --no-ff。禁止 cherry-pick。出现 conflict 时停止，不手工选边：
storage conflict 退回 T-103 owner，resolved-contract conflict 退回 G3 owner。merge 成功后先跑：
/opt/homebrew/bin/cargo test -p homie-storage --tests
/opt/homebrew/bin/cargo test -p homie-agents
/usr/bin/git diff --check
任何失败都 blocked。T-102 不直接编辑 crates/homie-storage。

允许直接写的文件仅为 agent_launch.rs、dispatcher.rs、runtime_actor.rs、lib.rs、
manifest_spawn.rs、daemon_process.rs、runtime_dispatcher.rs，以及声明的 homie-proto/
homie-client source/tests。dispatcher.rs、daemon_process.rs、runtime_dispatcher.rs 仅可更新
受 typed SessionSpawnRequest 影响的既有 request literal。其他路径 forbidden。
先运行 manifest_spawn 记录仍缺 actor integration 的 RED。最小实现 typed profile 或
explicit shell selection；用 G3 contract resolve；在 launch 前调用已 merge 的 T-103
repository freeze/hash/atomic bind/readback；真实 holder Stat readiness 后才 commit running
和发布 event/capability；失败按 reverse order rollback holder/config/session。禁止 shell
command string、unknown fallback、环境配置、provider raw key、remote/UI scope。

GREEN：
/opt/homebrew/bin/cargo test -p homie-storage --tests
/opt/homebrew/bin/cargo test -p homie-agents
/opt/homebrew/bin/cargo test -p homie-runtime --test manifest_spawn -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test daemon_process --no-run
/opt/homebrew/bin/cargo test -p homie-runtime --lib dispatcher::tests::every_registered_handler_decodes_to_its_declared_execution_class -- --exact
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_dispatcher
/opt/homebrew/bin/cargo test -p homie-proto --test runtime_transport_contract
/opt/homebrew/bin/cargo test -p homie-client --test typed_facade
/usr/bin/git diff --check
real fake executable 必须经真实 holder/PTY 输出 exact argv/env；partial launch residual=0；
holder added set 空。

merge commit 是 H-02 必需 imported-history 例外。实现 diff 只能含 allowed direct paths；stage
这些路径，跑绝对 pre-commit hook，再提交
feat(runtime): launch manifest agents through holder。不 amend/rebase/push。report 同时给出
T-103 milestone SHA、merge commit SHA、G5 commit SHA、storage/agent/runtime/proto/client
counts、cleanup、repository API handoff，并释放 lib/runtime_actor 给 G6。
```

### Packet T102-G6-GREEN-STATUS-RUNTIME

- **Packet ID:** `T102-G6-GREEN-STATUS-RUNTIME`
- **Owner:** `trae-g-status` / `G-STATUS`
- **依赖:** R4、G3、G5 pass
- **优先级:** `P0 / Wave D serial actor`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `6h`；sample/replay `10s`；test `120s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/status_runtime.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/runtime_actor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/runtime_status_engine.rs`
- **Forbidden paths:** 所有未列路径；尤其 hook CLI/proto ingress（G7）、holder/process/governor、
  `homie-storage/**`、规格/evidence、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine core_ -- --nocapture
```

  Core cases 在 G6 implementation 前必须 RED；`external_` cases 留给 G7。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine core_ -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --lib runtime_actor
```

  Core cases GREEN；明确标识仅 external hook ingress cases 由 G7 接手。
- **Cleanup:** 单一 bounded status worker path；无 per-client parser/task leak；temp facts 删除。
- **Expected result:** one reducer/manifest engine/cursor per live session；incremental signals；
  persist-before-event；read side-effect-free；restart reconstruction。
- **Handoff/commit contract:** pass 后 commit
  `feat(runtime): retain per-session status reducers`；handoff G7/G9/G10。
- **完整可复制 prompt:**

```text
执行 T102-G6-GREEN-STATUS-RUNTIME。唯一 owner trae-g-status/G-STATUS。固定 worktree 是
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch 是
wave1b/diri-agent-session-runtime。active 6h，sample/replay 10s，test 120s。确认
R4/G3/G5 pass、runtime_actor/lib clean。

读取 AGENTS.md、T-102 PRD FR-05、design Decision 7-8、plan G6、tasks 2.6、
delegation-plan、runtime-status-governor spec 的 reducer/signal/screen requirements。只可写
status_runtime.rs、runtime_actor.rs、lib.rs focused hunk、runtime_status_engine.rs。禁止 G7
external ingress files、holder/process/governor/storage/spec/evidence。

先跑 R4 RED。最小实现每 live session 一个由 frozen manifest authority/timing 创建的
reducer + ManifestEngine + screen/output cursor；holder readiness/process exit、bounded
incremental PTY output、manifest screen、accepted input、tick 汇入同一 reducer；canonical
status/needs-input/screen cursor 先 persist 再 event；read 只 projection，不重建/重放/写入；
startup 从 verified holder + frozen authority + persisted behavior + checkpoint + bounded output
重建。不得创建 per-client parser 或 unbounded per-session task。

GREEN：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine core_ -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --lib runtime_actor
core_ cases 必须 GREEN；external_ hook ingress 仍 RED 并完整 handoff G7，不得 ignore、删除或
弱化。cleanup 必须删除 temp facts，并确认无 worker/process residual。

diff 只含 allowed，git diff --check，stage + absolute hook，提交
feat(runtime): retain per-session status reducers。report 包含 reducer ownership/API、bounded
replay、counts、SHA；明确释放 actor/lib 给 G7，signal API 给 G9/G10。
```

### Packet T102-G7-GREEN-HOOK-INGRESS

- **Packet ID:** `T102-G7-GREEN-HOOK-INGRESS`
- **Owner:** `trae-g-status` / `G-STATUS`
- **依赖:** G6 pass
- **优先级:** `P0 / Wave D serial actor`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；focused suites `120s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/status_runtime.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/runtime_actor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/dispatcher.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/runtime_status_engine.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/runtime_dispatcher.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-proto/src/model.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-proto/tests/runtime_transport_contract.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-client/src/client.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-client/tests/typed_facade.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-cli/src/main.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-cli/tests/events_cli.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-cli/tests/hook_report_runtime_cli.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-cli/tests/notify_runtime_cli.rs`
- **Forbidden paths:** 所有未列路径；尤其 raw payload persistence、`homie-storage/**`、
  holder/process/governor/recovery、规格/evidence、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine external_ -- --nocapture
```

  G7 implementation 前 external hook/notify cases 必须 RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --lib dispatcher::tests::every_registered_handler_decodes_to_its_declared_execution_class -- --exact
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_dispatcher production_actor_handlers_execute_runtime_lifecycle_and_shutdown -- --exact --nocapture
/opt/homebrew/bin/cargo test -p homie-proto --test runtime_transport_contract
/opt/homebrew/bin/cargo test -p homie-client --test typed_facade
/opt/homebrew/bin/cargo test -p homie-cli --test events_cli -- --nocapture
/opt/homebrew/bin/cargo test -p homie-cli --test hook_report_runtime_cli -- --nocapture
/opt/homebrew/bin/cargo test -p homie-cli --test notify_runtime_cli -- --nocapture
```

- **Cleanup:** raw payload/temp event/storage facts 不残留；无 process leak。
- **Expected result:** allowlisted structured signals、redaction、subagent isolation、
  invalid stable error、commit-before-event，全 status/hook suite GREEN。
- **Handoff/commit contract:** pass 后 commit
  `feat(runtime): route hooks through status reducer`；释放 actor 给 G9。
- **完整可复制 prompt:**

```text
执行 T102-G7-GREEN-HOOK-INGRESS。唯一 owner trae-g-status/G-STATUS。固定 worktree 是
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch 是
wave1b/diri-agent-session-runtime。active 4h，每 suite 120s。确认 G6 pass、shared actor clean。

读取 AGENTS.md、T-102 PRD FR-05/10、design Decision 7-8/16、plan G7、tasks 2.7、
delegation-plan、runtime-status-governor hook requirements。只写声明的 status/actor、
dispatcher contract fixture、proto/client/CLI focused hook/event files/tests。禁止 storage、
holder/process/governor/recovery，禁止持久化或 event/log 输出 raw payload。

先跑 external ingress RED。最小实现严格 allowlisted Hook/Notify DTO 和 parser mapping；
敏感/free-form 数据 reject 或 reduce；subagent signal 不覆盖 parent status/title/needs-input；
所有 accepted signal 进入 G6 同一 reducer；canonical facts commit 后再 event；invalid payload
返回 stable safe error。不要创建第二 status path 或 direct storage write。

运行全部 GREEN commands：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --lib dispatcher::tests::every_registered_handler_decodes_to_its_declared_execution_class -- --exact
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_dispatcher production_actor_handlers_execute_runtime_lifecycle_and_shutdown -- --exact --nocapture
/opt/homebrew/bin/cargo test -p homie-proto --test runtime_transport_contract
/opt/homebrew/bin/cargo test -p homie-client --test typed_facade
/opt/homebrew/bin/cargo test -p homie-cli --test events_cli -- --nocapture
/opt/homebrew/bin/cargo test -p homie-cli --test hook_report_runtime_cli -- --nocapture
/opt/homebrew/bin/cargo test -p homie-cli --test notify_runtime_cli -- --nocapture
全过且输出/事件/持久化无 raw payload。清理 temp facts。

git diff --check/name-only；stage only allowed；absolute hook；提交
feat(runtime): route hooks through status reducer。不 amend/rebase/push。report 含 DTO allowlist、
redaction、counts、SHA、cleanup，并释放 runtime_actor 给 G9。
```

### Packet T102-G8-GREEN-PROCESS-TREE

- **Packet ID:** `T102-G8-GREEN-PROCESS-TREE`
- **Owner:** `trae-g-process` / `G-PROCESS`
- **依赖:** R5、G2 request-shape handoff
- **优先级:** `P0 / Wave B`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `6h`；STOP/CONT `2s`；cleanup `3s`；case `60s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/process_tree.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/holder.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/process_tree.rs`
- **Forbidden paths:** 所有未列路径；尤其 holder binary、lib/runtime_actor/governor、
  `homie-storage/**`、proto/client/CLI、规格/evidence、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test process_tree -- --test-threads=1 --nocapture
```

  必须在 STOP/CONT/sample/PID reuse 上 RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test process_tree -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test process_tree -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test process_tree -- --test-threads=1 --nocapture
```

- **Cleanup:** exact verified tree only；TERM+CONT -> `500ms` -> KILL+CONT；reap；residual `0`。
- **Expected result:** identity-safe enumerate/signal/sample repeated serial GREEN；race -> unknown。
- **Handoff/commit contract:** pass 后 commit
  `feat(runtime): add identity-safe process controls`；handoff holder call API 给 G9。
- **完整可复制 prompt:**

```text
执行 T102-G8-GREEN-PROCESS-TREE。唯一 owner trae-g-process/G-PROCESS。固定 worktree 是
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch 是
wave1b/diri-agent-session-runtime。active 6h，STOP/CONT 2s，cleanup 3s，case 60s。确认
R5/G2 pass、holder.rs 已由 G2 clean release。

读取 AGENTS.md、T-102 PRD FR-06/07/11、design Decision 9/15、plan G8、tasks 2.8、
delegation-plan、runtime-status-governor process requirements 和 G2 request schema。只可写
process_tree.rs、holder.rs integration hunk、process_tree test；holder binary 仍 forbidden。

先跑 R5 process RED。最小实现 enumerate root/descendants/required process-group peers；采集并
每次 signal 前验证 PID start-time；STOP 后验证 stopped；CONT leaves-first 并验证原 tree live；
terminate 为 TERM+CONT、bounded 500ms grace、KILL+CONT survivors，并包含从 original tree
发现的 detached descendants；sample 只返回 tree size/memory footprint，不泄漏 cmd/env/
prompt/tool args；race/unsupported/timeout 为 unknown，不误判 exited、不 kill。阻塞检查不得
占 socket task。

同一命令连续串行运行三次：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test process_tree -- --test-threads=1 --nocapture
每次全部 GREEN、residual=0、holder/process added set 空。禁止 global signal/pkill。

diff 仅 allowed，git diff --check，stage + absolute hook，提交
feat(runtime): add identity-safe process controls。report 给出 identity invariant、timeout、
3 次 counts、cleanup、SHA，handoff G9。
```

### Packet T102-G9-GREEN-GOVERNOR

- **Packet ID:** `T102-G9-GREEN-GOVERNOR`
- **Owner:** `trae-g-governor` / `G-GOVERNOR`
- **依赖:** G6、G8 pass；G7 已释放 actor
- **优先级:** `P0 / Wave D serial actor`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `6h`；每 case `60s`；STOP/CONT `2s`；cleanup `3s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/resource_governor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/runtime_actor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/daemon.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/bin/homie-runtime-daemon.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/resource_governor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/session_lifecycle.rs`
- **Forbidden paths:** 所有未列路径；尤其 `holder.rs`、holder binary/process_tree algorithm、storage、
  proto/client/CLI、规格/evidence、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test resource_governor -- --test-threads=1 --nocapture
```

  G9 implementation 前 governor/hibernate cases 必须 RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test resource_governor -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle runtime_hibernate_stops_holder_and_wake_restarts_it -- --exact --nocapture
```

  Existing test may be renamed only if semantics are strengthened to same-holder continuity；不得
  preserve terminate-and-respawn expectation。
- **Cleanup:** timer task join；exact fixture ledger `0`；same holder/child/PTY/offset continuity。
- **Expected result:** one daemon timer；conservative eligibility；unknown no-op；STOP/CONT
  hibernate/wake；archive terminate；prepare stops ticks。
- **Handoff/commit contract:** pass 后 commit
  `feat(runtime): preserve sessions across hibernation`；handoff G10/G11。
- **完整可复制 prompt:**

```text
执行 T102-G9-GREEN-GOVERNOR。唯一 owner trae-g-governor/G-GOVERNOR。固定 worktree 是
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch 是
wave1b/diri-agent-session-runtime。active 6h，case 60s，STOP/CONT 2s，cleanup 3s。确认
G6/G8 pass、G7 actor release、clean。

读取 AGENTS.md、T-102 PRD FR-06/07/09、design Decision 9-11/14-15、plan G9、tasks 2.9、
delegation-plan、runtime-status-governor governor/hibernate requirements。只可写声明的
governor/actor/lib/daemon focused files和 tests；G8 已冻结的 holder/process API 只读；
禁止改 holder.rs、holder binary、G8 process algorithm、storage/proto/client/CLI。

先跑 R5 governor RED。最小实现一个 daemon-scoped bounded timer；仅 idle+unattached+unpinned
且达到 reviewed threshold 的 session eligible；starting/running/needs_input/attached/pinned
保护；sample unknown/backpressure -> skip/defer no-op；hibernate 用 G8 verified STOP，wake 用
CONT，保持同 holder、child、PTY、output log、epoch/log offset、Homie session ID；hibernated
input stable error；archive 仍 terminate；prepare 后停止新 tick。不得 per-session unbounded
worker，不新增环境配置。

GREEN：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test resource_governor -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle runtime_hibernate_stops_holder_and_wake_restarts_it -- --exact --nocapture
若既有 test 名与旧 terminate semantics 冲突，只可在同文件改为更强的 same-holder assertion，
不得削弱。timer join，residual=0，holder added set 空。

diff 仅 allowed，git diff --check，stage + absolute hook，提交
feat(runtime): preserve sessions across hibernation。report 含 policy、same-identity evidence、
counts、cleanup、SHA，释放 actor/lib 给 G10 并提供 prepare tick handoff 给 G11。
```

### Packet T102-G10-GREEN-RECOVERY

- **Packet ID:** `T102-G10-GREEN-RECOVERY`
- **Owner:** `trae-g-recovery` / `G-RECOVERY`
- **依赖:** R6、G3、G5、G6、G9 pass
- **优先级:** `P0 / Wave D serial actor`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `6h`；resume readiness `3s`；case `60s`；cleanup `3s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/session_recovery.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/runtime_actor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/dispatcher.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/history.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/session_recovery.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/runtime_dispatcher.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-proto/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-proto/src/model.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-proto/tests/runtime_transport_contract.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-client/src/client.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-client/tests/typed_facade.rs`
- **Forbidden paths:** 所有未列路径；尤其 `homie-storage/**`、remote node/migration、UI、
  holder/process/governor、规格/evidence、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1 --nocapture
```

  G10 implementation 前 direct resume/relaunch cases 必须 RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_dispatcher production_resume_from_history_uses_actor_owned_runtime_and_storage -- --exact --nocapture
/opt/homebrew/bin/cargo test -p homie-proto --test runtime_transport_contract
/opt/homebrew/bin/cargo test -p homie-client --test typed_facade
/opt/homebrew/bin/cargo test -p homie-runtime capabilities --lib
```

- **Cleanup:** failed incarnation 清除；prior record/checkpoint/output 保留；residual `0`。
- **Expected result:** direct ID/latest manifest resume、same session/new epoch、adopt-before-launch、
  unarchive-no-spawn、retryable failure；无 remote capability。
- **Handoff/commit contract:** pass 后 commit
  `feat(runtime): add manifest session recovery`；handoff G11。
- **完整可复制 prompt:**

```text
执行 T102-G10-GREEN-RECOVERY。唯一 owner trae-g-recovery/G-RECOVERY。固定 worktree 是
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch 是
wave1b/diri-agent-session-runtime。active 6h，readiness 3s，case 60s，cleanup 3s。确认
R6/G3/G5/G6/G9 pass、shared files clean。

读取 AGENTS.md、T-102 PRD FR-08/09/10、design Decision 12-13/15、plan G10、tasks 2.10、
delegation-plan、local-session-recovery spec 和 manifest resume requirements。只写声明的
session_recovery、focused actor/lib/dispatcher/history/proto/client/tests。禁止 storage、
remote node/transfer/UI 和未列文件。

先跑 R6 recovery RED。最小实现从 T-103 safe frozen config readback 构建 manifest ID/latest
resume argv；same Homie session ID + new output epoch；保留 title/parent/profile/permission/
checkpoint/output；先 probe/adopt existing verified holder，再决定 launch；unarchive 仅改 offline
visibility 不 spawn；readiness failure 清 failed incarnation 但 prior record/checkpoint/output
retryable；local checkpoint/relaunch API 保持 internal。禁止 shell-text injection，禁止公开
remote session.migrate/move/fork/handoff capability 或 placeholder。

GREEN：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_dispatcher production_resume_from_history_uses_actor_owned_runtime_and_storage -- --exact --nocapture
/opt/homebrew/bin/cargo test -p homie-proto --test runtime_transport_contract
/opt/homebrew/bin/cargo test -p homie-client --test typed_facade
/opt/homebrew/bin/cargo test -p homie-runtime capabilities --lib
remote capability 必须 absent，failed fixture residual=0，prior facts 可读。

diff 仅 allowed，git diff --check，stage + absolute hook，提交
feat(runtime): add manifest session recovery。report 含 resume semantics、capability absence、
counts、cleanup、SHA，释放 actor/lib/dispatcher 给 G11。
```

### Packet T102-G11-GREEN-SHUTDOWN

- **Packet ID:** `T102-G11-GREEN-SHUTDOWN`
- **Owner:** `trae-g-shutdown` / `G-SHUTDOWN`
- **依赖:** G6、G9、G10 pass
- **优先级:** `P0 / Wave D serial actor`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；existing shutdown deadlines；process test `120s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/runtime_actor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/daemon.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/bin/homie-runtime-daemon.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/session_recovery.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/runtime_dispatcher.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/server_control.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/daemon_process.rs`
- **Forbidden paths:** 所有未列路径；尤其 Wave 1A wire framing/server ownership、holder
  termination behavior、`homie-storage/**`、proto/client/CLI、规格/evidence、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery shutdown_ -- --test-threads=1 --nocapture
```

  G11 implementation 前 `shutdown_` prepare/flush/continuity cases 必须 RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_dispatcher -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test server_control -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test daemon_process -- --test-threads=1 --nocapture
```

- **Cleanup:** continuity assertion 后测试显式清自己的 adopted holder；residual `0`。
- **Expected result:** quiesce、stop ticks、bounded drain、flush reducer/needs-input/screen/output/
  event/WAL、ACK-before-teardown、holders survive。
- **Handoff/commit contract:** pass 后 commit
  `fix(runtime): preserve holders during shutdown`；释放 shared runtime files 给 F1。
- **完整可复制 prompt:**

```text
执行 T102-G11-GREEN-SHUTDOWN。唯一 owner trae-g-shutdown/G-SHUTDOWN。固定 worktree 是
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch 是
wave1b/diri-agent-session-runtime。active 4h，沿用 Wave 1A shutdown deadline，test 120s。
确认 G6/G9/G10 pass、shared files clean。

读取 AGENTS.md、T-102 PRD FR-09/11、design Decision 14-15、plan G11、tasks 2.11、
delegation-plan、local-session-recovery prepare/shutdown requirements，以及 Wave 1A shutdown
tests。只写声明的 actor/lib/daemon focused paths和 tests。禁止改变 wire framing/server
ownership，禁止 terminate live/hibernated holder，禁止 storage/proto/client/CLI。

先跑 R6 shutdown RED。最小扩展 prepare：拒绝新的 spawn/resume/archive/hibernate mutation；
停止新 governor tick；bounded drain 已接受 work；flush reducer status、needs-input、screen/
output cursor、event store、SQLite WAL；timeout 返回 bounded outcome。shutdown 必须保持 Wave 1A
ACK-before-transport/actor teardown；running/hibernated holder 留给 replacement daemon，hard
restart 走 G1 reconciliation。不要把测试 cleanup 误放进 production shutdown。

GREEN：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_dispatcher -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test server_control -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test daemon_process -- --test-threads=1 --nocapture
continuity assertion 后测试才显式 kill 自己 ledger holder；residual=0、added set 空。

diff 仅 allowed，git diff --check，stage + absolute hook，提交
fix(runtime): preserve holders during shutdown。report 含 ACK order、flush facts、holder survival、
counts、cleanup、SHA，并明确 G1-G11 shared runtime files clean release 给 F1。
```

## 4. REFACTOR Packets

### Packet T102-F1-REFACTOR-PATHS

- **Packet ID:** `T102-F1-REFACTOR-PATHS`
- **Owner:** `trae-refactor` / `R-CLEANUP`
- **依赖:** G1、G2、G3、G5、G6、G7、G8、G9、G10、G11 全部 pass
- **优先级:** `P0 / Wave F`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；affected suites `10m`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/lib.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/runtime_actor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/history.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/agent_launch.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/status_runtime.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/resource_governor.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/src/session_recovery.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/session_lifecycle.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/runtime_status_engine.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/session_recovery.rs`
- **Forbidden paths:** 所有未列路径；尤其 holder/process algorithms、`homie-storage/**`、
  proto/client/CLI、规格/evidence/tracking、`.beads/**`、`diri/**`
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/rg -n "mark_interrupted_sessions_detached|resume_command|StatusReducer::new|spawn_shell|hibernate" crates/homie-runtime/src/lib.rs crates/homie-runtime/src/runtime_actor.rs crates/homie-runtime/src/history.rs
```

  Scan 必须只定位已知 superseded implementations；先确认全部 GREEN suites pass。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo fmt --all -- --check
/opt/homebrew/bin/cargo clippy --workspace --all-targets -- -D warnings
/opt/homebrew/bin/cargo test -p homie-runtime --lib
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1
```

- **Cleanup:** 不启动额外进程；所有 test fixture residual `0`。
- **Expected result:** 删除 bulk-detach call、fixed-shell profile spawn、shell-text resume、fresh
  reducer read、agent-agnostic full classifier、respawn hibernate、duplicate persistence/event；
  保留 explicit shell manifest。
- **Handoff/commit contract:** pass 后 commit
  `refactor(runtime): remove superseded session paths`；handoff F2。
- **完整可复制 prompt:**

```text
执行 T102-F1-REFACTOR-PATHS。唯一 owner trae-refactor/R-CLEANUP。固定 worktree 是
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch 是
wave1b/diri-agent-session-runtime。active 4h，affected suite 10m。开始前逐个验证 G1-G11
pass commits 在 ancestry，worktree clean。

读取 AGENTS.md、T-102 plan F1、tasks 3.1、delegation plan、全部 GREEN handoff。只可写声明的
runtime modules/tests。禁止 storage、proto/client/CLI、规格/evidence。先用
/opt/homebrew/bin/rg 定位并证明 caller，再删除且只删除：startup bulk-detach-before-adopt
call/path、fixed-shell agent-profile spawn、shell-text history resume、status read 中 fresh
reducer、agent-agnostic complete classifier、terminate-and-new-shell hibernate、duplicate
persistence/event。保留 explicit shell manifest 和 Wave 1A public contracts。若 finding 属于
其他 owner 或需要未列文件，blocked，退回原 owner，不趁 refactor 改行为。

删除前把 rg finding 作为 RED/precondition 记录；删除后运行：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo fmt --all -- --check
/opt/homebrew/bin/cargo clippy --workspace --all-targets -- -D warnings
/opt/homebrew/bin/cargo test -p homie-runtime --lib
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1
并重复 exact rg negative scans。测试 residual=0。

diff 仅 allowed，git diff --check，stage + absolute hook，提交
refactor(runtime): remove superseded session paths。report 提供每个 deleted path 的 before/
after rg、tests、cleanup、SHA，handoff F2。
```

### Packet T102-F2-REFACTOR-NEGATIVE-SCANS

- **Packet ID:** `T102-F2-REFACTOR-NEGATIVE-SCANS`
- **Owner:** `trae-review-scan` / `R-CLEANUP`
- **依赖:** F1 pass
- **优先级:** `P0 / Wave F`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；scan/focused suite `120s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-runtime/tests/security_consistency.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-agents/tests/runtime_launch_plan.rs`
- **Forbidden paths:** 所有 production source、`homie-storage/**`、规格/evidence/tracking、
  `.beads/**`、`diri/**`；finding 必须退回原 GREEN owner 修复
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/rg -n "pkill|session\.migrate|HOMIE_.*(MANIFEST|HOLDER|RUNTIME|TIMEOUT|GOVERNOR)" crates
/opt/homebrew/bin/cargo test -p homie-runtime --test security_consistency -- --nocapture
```

  新 negative assertions 在 unresolved finding 上必须 RED；若无 production finding，先证明
  scanner 会捕获 controlled fixture，不制造假 RED。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test security_consistency -- --nocapture
/opt/homebrew/bin/cargo test -p homie-agents --test runtime_launch_plan
/opt/homebrew/bin/rg -n "pkill|session\.migrate|HOMIE_.*(MANIFEST|HOLDER|RUNTIME|TIMEOUT|GOVERNOR)" crates
```

- **Cleanup:** scanner 不启动用户进程；无 generated/temp residual。
- **Expected result:** raw secret、production env override、embedded/fake runtime、shell fallback、
  remote migrate、storage-only running、global pkill、unbounded governor findings 为零。
- **Handoff/commit contract:** findings 非零则不跨 owner 修复，packet `blocked` 并退回；全部零后
  commit `test(runtime): add lifecycle security scans`。
- **完整可复制 prompt:**

```text
执行 T102-F2-REFACTOR-NEGATIVE-SCANS。唯一 owner trae-review-scan/R-CLEANUP。固定 worktree 是
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch 是
wave1b/diri-agent-session-runtime。active 4h，scan/test 120s。确认 F1 pass、clean。

读取 AGENTS.md、安全 baseline、T-102 PRD FR-10/11、design Decision 16、plan F2、tasks 3.2、
delegation plan。只可写 security_consistency.rs 和 runtime_launch_plan.rs tests。禁止任何
production source/storage/spec/evidence。建立行为/AST或结构化测试优先的 negative gates，
不以脆弱的任意源码字符串测试替代行为；rg 仅作辅助 inventory。

扫描并断言：argv/metadata/log/event/snapshot/evidence 无 provider raw key、Authorization、
cookie；无 production HOMIE manifest/holder/runtime/timeout/governor override；无 embedded
runtime/fake backend；unavailable agent 无 shell fallback；无 public remote session.migrate；
无 storage-only running inference；无 global pkill；无 unbounded per-session governor worker。
若发现 production 问题，不编辑该文件，blocked 并按原 owner G2/G3/G5/G6/G7/G9/G10 返回。

GREEN：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-runtime --test security_consistency -- --nocapture
/opt/homebrew/bin/cargo test -p homie-agents --test runtime_launch_plan
/opt/homebrew/bin/rg -n "pkill|session\.migrate|HOMIE_.*(MANIFEST|HOLDER|RUNTIME|TIMEOUT|GOVERNOR)" crates
对 rg 命中逐项分类，允许测试中的禁止项断言，不允许 production finding。scanner 不启动用户
进程，无 temp residual。

零 unresolved finding 时 diff 仅 allowed，git diff --check，stage + absolute hook，提交
test(runtime): add lifecycle security scans。report 含 scan matrix、classified rg output、
tests、SHA；否则不提交并返回 owner-specific blocker。
```

## 5. EVIDENCE Packets

### Packet T102-E1-EVIDENCE-FOCUSED

- **Packet ID:** `T102-E1-EVIDENCE-FOCUSED`
- **Owner:** `trae-e-focused` / `E-E2E`
- **依赖:** F1、F2 pass
- **优先级:** `P0 / Wave F`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；package suite `10m`；每 integration `120s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/docs/verification/diri-agent-session-runtime/raw/e1-focused-results.md`
- **Forbidden paths:** 所有 production/test/spec/tracking/Bead 文件、`homie-storage/**`、
  `diri/**`；本 packet 不修行为
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/usr/bin/git log --oneline --decorate -40
/usr/bin/git diff 48f522b..HEAD -- crates/homie-runtime/tests/session_lifecycle.rs
```

  读取 R1-R6 原始 handoff，核对 2 historical RED 和新增 RED 均有对应 GREEN commit；缺任一
  证据即 blocked。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-agents
/opt/homebrew/bin/cargo test -p homie-runtime --lib
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test startup_reconciliation -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test manifest_spawn -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test process_tree -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test resource_governor -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1 --nocapture
```

  五条相同 `session_lifecycle` 命令是连续 serial 五轮，不复制测试逻辑。
- **Cleanup:** 每命令 residual `0`；每轮 holder after-minus-before 为空。
- **Expected result:** 全 GREEN；2 historical RED assertion unchanged；holder stat retained GREEN；
  raw report 记录 command/exit/count/duration/cleanup。
- **Handoff/commit contract:** 不改测试/产品；pass 后 commit
  `test(runtime): verify repeated session lifecycle`，只含 raw result；handoff E2/E3。
- **完整可复制 prompt:**

```text
执行 T102-E1-EVIDENCE-FOCUSED。唯一 owner trae-e-focused/E-E2E。固定 worktree 是
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch 是
wave1b/diri-agent-session-runtime。active 4h，package 10m，integration 120s。确认 F1/F2
pass、clean。只可写
docs/verification/diri-agent-session-runtime/raw/e1-focused-results.md；禁止所有 source/test/
spec/tracking 修改。本 packet 不修行为，失败返回原 owner。

读取 AGENTS.md、T-102 tasks 4.1、plan E1、delegation evidence contract、R1-R6 和 G1-G11/F1/F2
handoffs。先核对 exact RED provenance：checkpoint 14 tests = 12 pass/2 fail；两个 historical
assertion 未改；holder stat 原本 GREEN；每个新增 RED 有对应 GREEN commit。缺证据 blocked。

按以下绝对命令逐一运行并记录 start/end duration、exit、pass/fail/ignored、cleanup
residual、holder baseline/after/added set：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-agents
/opt/homebrew/bin/cargo test -p homie-runtime --lib
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test startup_reconciliation -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test manifest_spawn -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test runtime_status_engine -- --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test process_tree -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test resource_governor -- --test-threads=1 --nocapture
/opt/homebrew/bin/cargo test -p homie-runtime --test session_recovery -- --test-threads=1 --nocapture
五条相同 session_lifecycle 命令即连续 serial 五次，禁止复制测试逻辑或用环境变量控制。
所有 suite 必须 GREEN；每轮 residual=0、added set 空。

将事实写入唯一 allowed raw report，不写 aspirational pass。git diff --check/name-only；只
stage raw report，跑绝对 pre-commit hook，提交
test(runtime): verify repeated session lifecycle。返回 SHA、完整 command table、五轮 counts、
cleanup 和 E2/E3 handoff。任一失败不提交，原样报告 owner。
```

### Packet T102-E2-EVIDENCE-CROSS-ENTRY

- **Packet ID:** `T102-E2-EVIDENCE-CROSS-ENTRY`
- **Owner:** `trae-e-process` / `E-E2E`
- **依赖:** E1 pass
- **优先级:** `P0 / Wave F blocking E2E`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `6h`；每 E2E case `60s`；cleanup `3s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-cli/tests/diri_agent_session_runtime_e2e.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/crates/homie-cli/tests/support/mod.rs`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/docs/verification/diri-agent-session-runtime/raw/e2-cross-entry-results.md`
- **Forbidden paths:** 所有 production source、其他 tests、`homie-storage/**`、规格/tracking、
  `.beads/**`、`diri/**`；禁止 production test mode
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo test -p homie-cli --test diri_agent_session_runtime_e2e --no-run
/opt/homebrew/bin/cargo test -p homie-cli --test diri_agent_session_runtime_e2e -- --test-threads=1 --nocapture
```

  首次运行必须真实触达 packaged daemon/holder/PTY/fake executable；若仅 mock 或未触达
  cross-entry flow，不接受为 RED/evidence。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo build -p homie-runtime --bins
/opt/homebrew/bin/cargo build -p homie-cli
/opt/homebrew/bin/cargo test -p homie-cli --test diri_agent_session_runtime_e2e -- --test-threads=1 --nocapture
```

- **Cleanup:** panic-safe exact daemon/holder/process-group/socket ledger；after-minus-before 空；
  不碰 pre-existing holder。
- **Expected result:** full packaged cross-entry flow pass；storage/registry/snapshot 一致；需要时
  same holder/PTY；无 duplicate child。
- **Handoff/commit contract:** pass 后 commit
  `test(runtime): cover daemon holder cross-entry lifecycle`；handoff E3；失败不改 production。
- **完整可复制 prompt:**

```text
执行 T102-E2-EVIDENCE-CROSS-ENTRY。唯一 owner trae-e-process/E-E2E。固定 worktree 是
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime，固定 branch 是
wave1b/diri-agent-session-runtime。active 6h，每 case 60s，cleanup 3s。确认 E1 pass、clean。
只可写专用 CLI E2E、现有 CLI test support focused hunk、raw E2 report；禁止任何
production、storage、规格/tracking，禁止 production test mode/env override。

读取 AGENTS.md、T-102 PRD FR-03..11、plan E2、tasks 4.2、delegation cleanup contract、四个
capability specs 和 E1 raw report。测试必须 build/启动 packaged homie-runtime-daemon、
packaged homie-runtime-holder、packaged homie CLI，使用 absolute fixture data dir、真实 PTY、
真实 local fake executable 和 constructor/typed request；不能 mock holder/PTY。

单一 cross-entry flow：typed manifest spawn -> exact argv/env/output/status -> resize/stat ->
SIGKILL daemon 且 holder survives -> replacement daemon adopts -> input/output continues ->
structured hook/notify reducer -> hibernate/wake same holder/child/PTY/offset -> archive terminates ->
unarchive no spawn -> direct manifest resume same Homie ID/new epoch -> prepare/shutdown ACK and
holder survival -> explicit fixture session cleanup。每阶段校验 storage/registry/snapshot 一致，
无 duplicate child。

运行：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/cargo build -p homie-runtime --bins
/opt/homebrew/bin/cargo build -p homie-cli
/opt/homebrew/bin/cargo test -p homie-cli --test diri_agent_session_runtime_e2e -- --test-threads=1 --nocapture
fixture guard 在 pass/assertion/panic/timeout 都只清 ledger PID+start-time/socket/temp dir，3s 后
仅 matched PID 兜底并 reap；holder after-minus-before 必须空，baseline holders 不动。

把 command/phase/duration/count/cleanup 写 raw report。diff 只含 allowed，git diff --check，
stage + absolute hook，提交 test(runtime): cover daemon holder cross-entry lifecycle。失败时
不改 production、不提交，报告最后完成 phase 和原 owner。pass report 给 E3 SHA、全 flow
evidence、cleanup。
```

### Packet T102-E3-EVIDENCE-READINESS

- **Packet ID:** `T102-E3-EVIDENCE-READINESS`
- **Owner:** `trae-e-docs` / `E-DOCS`
- **依赖:** E1、E2 pass
- **优先级:** `P0 / Wave F release gate`
- **绝对 worktree:** `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime`
- **Branch:** `wave1b/diri-agent-session-runtime`
- **Deadline:** active work `4h`；docs/OpenSpec gates `120s`
- **Allowed write paths:**
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/docs/verification/diri-agent-session-runtime/spec-review-report.md`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/docs/verification/diri-agent-session-runtime/test-report.md`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/docs/verification/diri-agent-session-runtime/e2e-report.md`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/docs/verification/diri-agent-session-runtime/security-review-report.md`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/docs/verification/diri-agent-session-runtime/code-review-report.md`
  - `/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/docs/verification/diri-agent-session-runtime/release-readiness-report.md`
- **Forbidden paths:** 所有 source/test/Cargo/Makefile、PRD/OpenSpec/component specs、parity/master
  tasks、`.beads/**`、`homie-storage/**`、`diri/**`；本 packet 只产生 tracking handoff，不直接
  更新 parity/Bead
- **RED commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/usr/bin/test -s docs/verification/diri-agent-session-runtime/raw/e1-focused-results.md
/usr/bin/test -s docs/verification/diri-agent-session-runtime/raw/e2-cross-entry-results.md
/usr/bin/test -s docs/verification/diri-agent-session-runtime/release-readiness-report.md
```

  前两项必须存在；最终 readiness report 在本 packet 写入前应缺失或不完整。任一 source
  evidence 缺失/失败时 readiness 必须 `blocked`/`fail`。
- **GREEN commands:**

```text
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/openspec validate diri-agent-session-runtime --strict
/usr/bin/make parity-lock
/usr/bin/git diff --check
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/.githooks/pre-commit
```

- **Cleanup:** 不启动新 runtime process；确认 E1/E2 residual `0` 和 holder added set 空。
- **Expected result:** 六份事实 evidence；release-readiness 仅在全 blocking gates pass 时为
  `pass`；RT-010 remote/UI/remote-node/provider 保持 partial/deferred。
- **Handoff/commit contract:** pass 后 commit
  `docs(runtime): record T-102 release evidence`；报告 coordinator 创建 H-03 shared-release
  milestone 所需 exact SHA/ancestry/clean status；本 packet 不更新 Bead/parity，不 push。
- **完整可复制 prompt:**

```text
执行 T102-E3-EVIDENCE-READINESS。唯一 owner trae-e-docs/E-DOCS。固定 worktree
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime、branch
wave1b/diri-agent-session-runtime，active 4h，docs/OpenSpec 120s。确认 E1/E2 pass、clean。

读取 AGENTS.md、T-102 PRD acceptance、alignment-report.md、plan E3、tasks 4.3、
delegation evidence minimum、spec review、E1/E2 raw reports、所有 packet handoffs 和 Wave
orchestration H-03。只可写六个声明 evidence reports。禁止 source/test/Cargo、PRD/OpenSpec/
component specs、parity/master tasks、.beads、storage。代码/安全 review 发现行为问题时退回
原 owner，不自行跨 scope 修复。

报告必须记录 exact base/checkpoint/HEAD/packet commit SHAs；每条 command/exit/count/duration；
2 RED -> GREEN 与 retained holder-stat GREEN；五轮 lifecycle；real cross-entry phase；
timeout/cleanup ledger 和 pre/post holder set；no-fallback/no-secret scan；两轮 code review；
OpenSpec strict/status/alignment；release blocker。不得写 aspirational pass。RT-010 remote
migration、UI/terminal、remote-node、provider proxy/virtual-key 明确 partial/deferred。

运行：
cd /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime
/opt/homebrew/bin/openspec validate diri-agent-session-runtime --strict
/usr/bin/make parity-lock
/usr/bin/git diff --check
先 stage 六份 evidence，再运行
/Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime/.githooks/pre-commit
只有全部 blocking evidence pass 时 release-readiness 写 pass；否则写 explicit blocker 且不提交
pass 声明。

pass 时提交 docs(runtime): record T-102 release evidence；不 amend/rebase/push，不直接更新
Bead/parity/master tasks。completion report 给 exact evidence commit SHA、H-02 ancestry
confirmation、git status clean、strict/parity/hook results，并 handoff coordinator 创建 H-03
T-102 shared-release milestone。H-03 必须包含 H-02 真实祖先且不得有 T-102-authored
homie-storage edit。
```

## 6. Cross-Change Handoff Summary

```text
T102-G3-GREEN-AGENT-PLAN
  -> 独立 H-01 contract commit
  -> T-103 只通过 git show 读取 exact SHA
  -> T-103 S103-GREEN-01 + S103-GREEN-02 milestone commit
  -> coordinator 在 homie-t3u.2 notes 记录 exact SHA 和 tests
  -> T102-G5-GREEN-MANIFEST-SPAWN 等待该 SHA
  -> G5 使用 git merge --no-ff exact SHA，禁止 cherry-pick
  -> storage/agent gates GREEN 后才实现 G5
  -> G5-G11/F1/F2/E1/E2/E3
  -> coordinator 创建 H-03 shared-release milestone
```

若 T-103 repository 无法无损表达 G3 contract，G3/T-103/G5 全部保持 `blocked`，返回
cross-spec review。T-102 不以临时 schema、repository wrapper、compatibility fallback 或直接
编辑 `homie-storage` 绕过该 gate。

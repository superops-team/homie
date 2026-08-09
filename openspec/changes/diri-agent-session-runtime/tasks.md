## 1. RED

- [ ] 1.1 **R1 - 固化 2 RED / 1 GREEN 与进程 cleanup fixture**
  - Owner: `R-BASE`
  - Files: `crates/homie-runtime/tests/session_lifecycle.rs`,
    create `crates/homie-runtime/tests/process_fixture_cleanup.rs`,
    `crates/homie-runtime/tests/support/process_fixture.rs`
  - Budget/timeout: one TraeCLI, 4h; test binary 120s
  - Work: 保留两个 `detached != running` 失败；保持
    `runtime_holder_stat_tracks_resize_and_log_offsets` GREEN；测试前记录 holder
    PID+start-time baseline；fixture 记录 absolute temp dir、daemon/holder/root
    PID+start-time、socket/pid/status path，并用 panic-safe guard 在 RED assertion
    failure、panic、timeout 和正常返回时执行有界 process-group 清理。
  - Verify: `cargo test -p homie-runtime --test session_lifecycle -- --nocapture`
    精确得到 `12 passed, 2 failed`；独立 `process_fixture_cleanup` test binary
    验证 panic-safe cleanup；fixture residual 为零，且测试后 holder PID+start-time
    集合减 baseline 为空。
  - Cleanup: 只处理 ledger 资源；进程名只用于 before/after count，不用于 kill；禁止
    `pkill`、用户 data dir 和 baseline holder。

- [ ] 1.2 **R2 - 添加 startup reconciliation truth-table RED**
  - Owner: `R-BASE`
  - Files: create `crates/homie-runtime/tests/startup_reconciliation.rs`; reuse test support only
  - Budget/timeout: one TraeCLI, 4h; test binary 120s
  - Work: 覆盖 live+created/starting/running/detached、live+idle/needs-input、missing、exited、
    hibernated+stopped、archived+live contradiction 和 duplicate-holder prevention。
  - Verify: 新 case 在当前 bulk-detach-before-adopt 上按预期失败；holder stat 仍 GREEN。
  - Cleanup: fixture ledger zero。

- [ ] 1.3 **R3 - 添加 manifest launch/resolved-config contract RED**
  - Owner: `G-AGENT-PLAN`
  - Files: create `crates/homie-agents/tests/runtime_launch_plan.rs`,
    `crates/homie-runtime/tests/manifest_spawn.rs`
  - Budget/timeout: one TraeCLI, 4h; each test binary 120s
  - Work: 覆盖显式 `include_str!` compiled catalog closure、packaged binary 无 cwd/PATH/resource
    manifest lookup、absolute executable、argv boundary、env scrub、resolved config field
    contract、explicit shell、unknown-agent no-fallback、真实 fake executable + holder/PTY。
  - Verify: 当前 fixed shell path 不能满足 agent cases，产生目标 RED。
  - Cleanup: 删除 test temp executable；清理 ledger holder/child。

- [ ] 1.4 **R4 - 添加 stateful status/hook RED**
  - Owner: `G-STATUS`
  - Files: create `crates/homie-runtime/tests/runtime_status_engine.rs`; agent test modules only
  - Budget/timeout: one TraeCLI, 4h; test binary 120s
  - Work: 覆盖 manifest authority、read side-effect-free、process/output/screen/hook/notify/
    input/tick convergence、commit-before-event、subagent isolation、restart reconstruction。
  - Verify: 当前 fresh `ScreenPrimary` reducer/direct storage write 产生目标 RED。
  - Cleanup: no process leak; temp event/storage dir removed。

- [ ] 1.5 **R5 - 添加 process/resource/hibernate RED**
  - Owner: `G-PROCESS`
  - Files: create `crates/homie-runtime/tests/process_tree.rs` and
    `crates/homie-runtime/tests/resource_governor.rs`
  - Budget/timeout: one TraeCLI, 4h; each process case 60s
  - Work: 覆盖 STOP verification、leaves-first CONT、PID reuse、tree/footprint sample、safe
    unknown、idle eligibility、same-holder/PTY wake、hibernated input error。
  - Verify: 新 stop/continue/governor cases RED；现有 terminate cases GREEN。
  - Cleanup: PID start-time checked fixture cleanup。

- [ ] 1.6 **R6 - 添加 resume/relaunch/shutdown RED**
  - Owner: `G-RECOVERY`
  - Files: create `crates/homie-runtime/tests/session_recovery.rs`; focused shutdown tests
  - Budget/timeout: one TraeCLI, 4h; each process case 60s
  - Work: 覆盖 ID/latest direct resume、missing ID、adopt-before-relaunch、new epoch、failed
    readiness retry、unarchive-no-spawn、prepare quiesce/flush、graceful/hard restart continuity。
  - Verify: 当前 shell-text resume 与 incomplete shutdown facts 产生目标 RED。
  - Cleanup: ledger daemon/holder/child/socket zero。

## 2. GREEN

- [ ] 2.1 **G1 - 实现 holder-first startup reconciliation**
  - Owner: `G-RECONCILE`
  - Depends: 1.1, 1.2
  - Files: create `crates/homie-runtime/src/reconciliation.rs`; focused startup/adoption edits in
    `crates/homie-runtime/src/lib.rs`
  - Budget/timeout: one TraeCLI, 4h; focused suite 10m
  - Work: persisted fact -> holder probe -> one outcome -> storage projection -> registry insert；
    startup 不再调用 bulk detach；preserve verified idle/needs-input；missing -> detached；
    explicit exit -> exited。
  - Verify: 两个现有 RED 不改断言转 GREEN；holder stat GREEN；reconciliation table GREEN；
    no duplicate child。
  - Cleanup: fixture ledger zero。

- [ ] 2.2 **G2 - 实现 structured holder launch/control/stat**
  - Owner: `G-HOLDER`
  - Depends: 2.1 interface freeze
  - Files: `crates/homie-runtime/src/holder.rs`,
    `crates/homie-runtime/src/bin/homie-runtime-holder.rs`, holder tests
  - Budget/timeout: one TraeCLI, 6h; holder IPC 350ms, readiness 3s
  - Work: structured argv/cwd/sanitized env/geometry；additive STOP/CONT/sample；owner-only
    one-shot launch transport；不记录 argv/env/raw key；保留现有 holder adoption/stat。
  - Verify: holder protocol/process tests、retained stat gate、no fixed-agent fallback。
  - Cleanup: terminate <=3s；fixture control files zero。

- [ ] 2.3 **G3 - 实现 immutable manifest launch/resume plan**
  - Owner: `G-AGENT-PLAN`
  - Depends: 1.3
  - Files: create `crates/homie-agents/src/launch.rs`; modify
    `crates/homie-agents/src/lib.rs`; focused tests
  - Budget/timeout: one TraeCLI, 6h; package suite 10m
  - Work: `EffectiveAgentConfig`、resolved absolute executable、launch/resume plan、safe env、
    manifest injection/authority；所有 committed descriptor 通过显式 `include_str!` table 编译
    进 immutable production catalog；packaged daemon/standalone CLI 无 cwd/PATH/external
    resource manifest lookup；test constructor catalog；explicit shell only；redacted Debug。
  - Verify: `cargo test -p homie-agents`; runtime launch-plan/compiled-catalog contract GREEN；
    输出 exact `ResolvedEffectiveAgentConfig` 类型/字段 handoff，供 T-103
    `S103-GREEN-02` 实现 v4 freeze/hash/bind/readback。
  - Cleanup: readiness 不启动 agent；temp executables removed。

- [ ] 2.5 **G5 - 将 actor spawn 接到 manifest plan 和真实 holder**
  - Owner: `G-SPAWN`
  - Depends: 2.2, 2.3, T-103 `S103-GREEN-02` effective-config repository GREEN handoff
  - Files: create `crates/homie-runtime/src/agent_launch.rs`; focused
    `runtime_actor.rs`/`lib.rs`; focused DTO/client tests in `homie-proto`/`homie-client`
  - Budget/timeout: one TraeCLI, 6h; readiness 3s, integration 120s
  - Work: typed profile/explicit-shell selection；T-102 resolve 后调用 T-103 repository
    freeze/hash/atomic bind/readback；holder readiness before running/event；reverse rollback；
    handler 完成后才发布 capability。T-102 不编辑 `homie-storage`。
  - Verify: real fake executable 在真实 holder/PTY 输出 exact argv/env。
  - Cleanup: partial launch leaves no session/config/holder/child。

- [ ] 2.6 **G6 - 实现 actor-owned per-session status runtime**
  - Owner: `G-STATUS`
  - Depends: 2.3, 2.5
  - Files: create `crates/homie-runtime/src/status_runtime.rs`; focused actor/status paths
  - Budget/timeout: one TraeCLI, 6h; sample/replay 10s
  - Work: reducer+manifest engine+screen cursor per live session；incremental output；process/
    output/screen/input/tick；persist-before-event；read side-effect-free；startup reconstruction。
  - Verify: status suite（除 external hook ingress）GREEN。
  - Cleanup: one bounded status worker path, no per-client parser leak。

- [ ] 2.7 **G7 - 将 structured hook/notify 接入同一 reducer**
  - Owner: `G-STATUS`
  - Depends: 2.6
  - Files: focused hook DTO/CLI/runtime handler tests; `status_runtime.rs`
  - Budget/timeout: one TraeCLI, 4h; focused suite 120s
  - Work: allowlisted signal DTO、redaction、subagent isolation、invalid payload stable error、
    commit-before-event；不持久化 raw payload。
  - Verify: complete runtime status/hook suite GREEN。
  - Cleanup: temp event/storage facts removed。

- [ ] 2.8 **G8 - 实现 identity-safe tree signal/sample**
  - Owner: `G-PROCESS`
  - Depends: 1.5, 2.2 request shape
  - Files: `crates/homie-runtime/src/process_tree.rs`, process tests
  - Budget/timeout: one TraeCLI, 6h; STOP/CONT 2s, cleanup 3s
  - Work: enumerate root/descendants/group peers；PID start-time；STOP verify；leaves-first CONT；
    TERM+CONT -> 500ms -> KILL+CONT；tree size/footprint；races -> unknown。
  - Verify: process suite repeated serial GREEN。
  - Cleanup: exact tree only；no global signal。

- [ ] 2.9 **G9 - 实现 conservative governor 与连续 hibernate/wake**
  - Owner: `G-GOVERNOR`
  - Depends: 2.6, 2.8
  - Files: create `crates/homie-runtime/src/resource_governor.rs`; focused actor/daemon/holder wiring
  - Budget/timeout: one TraeCLI, 6h; each governor case 60s
  - Work: one daemon timer；idle+unattached+unpinned eligibility；protect
    running/needs-input；unknown sample no-op；STOP/CONT same tree；hibernated input error；
    archive terminate；prepare stops ticks。
  - Verify: resource/governor suite GREEN；holder/child/PTY/offset identity unchanged。
  - Cleanup: timer task joins；fixture ledger zero。

- [ ] 2.10 **G10 - 实现 direct manifest resume/local relaunch substrate**
  - Owner: `G-RECOVERY`
  - Depends: 2.3, 2.5, 2.6, 2.9
  - Files: create `crates/homie-runtime/src/session_recovery.rs`; focused
    actor/dispatcher/proto/client/history paths
  - Budget/timeout: one TraeCLI, 6h; resume readiness 3s
  - Work: direct ID/latest argv；same Homie ID/new epoch；preserve metadata/checkpoint；adopt before
    launch；unarchive no spawn；failure retryable；local substrate internal；无 remote migrate
    capability/placeholder。
  - Verify: recovery suite GREEN；Hello capabilities 不含 remote migration。
  - Cleanup: failed incarnation removed; prior record/output retained。

- [ ] 2.11 **G11 - 扩展 prepare/shutdown flush 并保留 holder**
  - Owner: `G-SHUTDOWN`
  - Depends: 2.6, 2.9, 2.10
  - Files: focused runtime actor/daemon shutdown paths and tests
  - Budget/timeout: one TraeCLI, 4h; existing shutdown deadlines
  - Work: reject new lifecycle mutations；stop governor ticks；drain accepted work；flush reducer/
    needs-input/screen/output/event/WAL；ACK-before-teardown；不 terminate live/hibernated holder。
  - Verify: shutdown/restart + Wave 1A daemon lifecycle suites GREEN。
  - Cleanup: tests explicitly kill adopted holder only after continuity assertion。

## 3. REFACTOR

- [ ] 3.1 **F1 - 删除 superseded production paths 并收敛模块边界**
  - Owner: `R-CLEANUP`
  - Depends: 2.1-2.11
  - Files: `crates/homie-runtime/src/lib.rs`, `runtime_actor.rs`, obsolete helpers proved by `rg`
  - Budget/timeout: one TraeCLI, 4h; affected suites 10m
  - Delete: bulk-detach-before-adopt call、fixed-shell agent spawn、shell-text history resume、fresh
    reducer status read、agent-agnostic full classifier、terminate-and-respawn hibernate、duplicate
    persistence/event paths。
  - Keep: explicit shell manifest。
  - Verify: fmt/clippy/affected suites/negative `rg` GREEN。
  - Cleanup: no generated/test process residual。

- [ ] 3.2 **F2 - 执行 security/consistency negative scans**
  - Owner: `R-CLEANUP`
  - Depends: 3.1
  - Files: tests/scanners only；finding 返回原 owner 修复
  - Budget/timeout: one TraeCLI, 4h; scans 120s
  - Scan: raw provider keys/Authorization/cookies、production manifest env override、embedded/fake
    runtime、unavailable-agent shell fallback、remote migrate capability、storage-only running、
    global `pkill`、unbounded governor workers。
  - Verify: zero unresolved findings。
  - Cleanup: scanners 不启动用户进程。

## 4. EVIDENCE

- [ ] 4.1 **E1 - 运行 focused、package、repeated lifecycle gates**
  - Owner: `E-E2E`
  - Depends: 3.1, 3.2
  - Files: test harness/results only
  - Budget/timeout: one TraeCLI, 4h; package 10m, workspace 20m
  - Run: `homie-agents`、runtime lib、session lifecycle、reconciliation、manifest spawn、status、
    process tree、governor、recovery；`session_lifecycle` serial 连续 5 次。
  - Verify: 全 GREEN；当前两个 RED 不改断言通过；holder stat 保持 GREEN；每轮 residual
    0；每个 suite 的 holder PID+start-time after-minus-before 为空。
  - Cleanup: each iteration ledger zero。

- [ ] 4.2 **E2 - 运行真实 daemon/holder cross-entry E2E**
  - Owner: `E-E2E`
  - Depends: 4.1
  - Files: dedicated process E2E only；禁止 production test-mode edit
  - Budget/timeout: one TraeCLI, 6h; each E2E 60s
  - Flow: packaged daemon -> typed manifest spawn -> argv/env/output/status -> resize/stat ->
    SIGKILL/restart/adopt -> input/output -> hook/notify -> hibernate/wake -> archive/unarchive/
    resume -> prepare/shutdown holder survival -> explicit cleanup。
  - Verify: storage/registry/snapshot 一致；same holder/PTY where required；no duplicate child。
  - Cleanup: panic-safe guard 回收 exact daemon/holder/process-group/socket ledger；
    holder PID+start-time after-minus-before 为空；不碰 pre-existing user holders。

- [ ] 4.3 **E3 - 记录 evidence、review 和 scoped parity handoff**
  - Owner: `E-DOCS`
  - Depends: 4.1, 4.2
  - Files: `docs/verification/diri-agent-session-runtime/**`; tracking files only by their owner
  - Budget/timeout: one TraeCLI, 4h; docs checks 120s
  - Record: commit/checkpoint、commands/exit、2 RED -> GREEN + retained GREEN、cleanup、security、
    two-round review、OpenSpec strict/status、alignment、release readiness。
  - Tracking: 仅 advancement evidence 对应的 local parity rows；RT-010 remote/UI/remote/provider
    保持 partial/deferred；Bead 仅在 evidence 匹配 delivered state 后更新。
  - Verify: release readiness `pass` or explicit blocker；不得写 aspirational pass。

## 5. Specification Gates

- [x] 5.1 OpenSpec `status` 显示 proposal/specs/design/tasks 4/4 complete。
- [x] 5.2 `openspec validate diri-agent-session-runtime --strict` 通过。
- [x] 5.3 PRD FR、OpenSpec requirements/scenarios、tasks 和 alignment report 无 orphan。
- [x] 5.4 16 维 spec review 无 blocker，且 2 RED / 1 GREEN、holder authority、无 remote/UI
  承诺保持一致。

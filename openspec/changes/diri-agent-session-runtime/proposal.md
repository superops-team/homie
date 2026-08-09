## Why

Wave 1A 已建立独立 runtime daemon 和 async client，但 agent session runtime 仍固定启动
`/bin/sh`，startup reconciliation 会把已成功 adoption 的 live holder 留在 `detached`
projection，status reducer、hooks、resource lifecycle 和 resume 也未形成同一条真实生产路径。

checkpoint `48f522b` 的当前事实是：

- `runtime_reopen_can_adopt_holder_and_continue_session` 为 RED：
  `detached != running`；
- `runtime_spawn_shell_uses_live_pty` 为 RED：reopen 后
  `detached != running`；
- `runtime_holder_stat_tracks_resize_and_log_offsets` 已为 GREEN，必须保持。

根因是 `RuntimeSupervisor::open_inner` 先执行
`mark_interrupted_sessions_detached()`，再执行 `adopt_live_holders()`；adoption 的 running
回写又只接受 `created|starting|running`，因此 live holder 被加入 registry 后 storage 和
projection 仍是 `detached`。

T-102 需要在不改变 Wave 1A transport、不过度承诺 remote/UI 的前提下，让 holder live
evidence、manifest agent launch、canonical status、resource governor、resume 和 shutdown
成为一个可验证的本地 runtime vertical slice。

## What Changes

- Replace bulk-detach-before-adopt with per-session startup reconciliation driven by holder evidence.
- Preserve holder-owned PTY, output, geometry, epoch, and log-offset continuity across daemon restart.
- Replace fixed shell production spawn with manifest-driven binary/argv/sanitized-env launch and
  a resolved effective-config contract persisted by T-103.
- Compile committed `assets/agent-descriptors/*.json` into the immutable `homie-agents` catalog so
  packaged binaries do not discover manifests from cwd, PATH, or external resources.
- Wire process, PTY, manifest screen, hook, notify, user-input, and tick signals through one per-session status reducer.
- Extend holder process-tree control with verified stop/continue and bounded memory sampling.
- Change hibernate/wake to preserve the same holder, PTY, and process tree; keep archive/kill as terminating operations.
- Add direct manifest resume/relaunch under the same Homie session identity and local migration substrate based on checkpoints.
- Preserve Wave 1A prepare/shutdown ACK ordering while flushing T-102 lifecycle facts and leaving holder sessions alive.
- Add real daemon/holder/PTY/fake-agent E2E, bounded timeouts, fixture-owned cleanup, and negative fallback scans.
- Keep remote `session.migrate`, move/fork handoff, terminal UI, remote node, and provider credential issuance out of scope.

## Capabilities

### New Capabilities

- `holder-pty-continuity`: Startup reconciliation, authoritative holder adoption, real PTY continuity, holder stat, and fail-closed detached/exited recovery.
- `manifest-agent-runtime`: Manifest-driven readiness, effective config freeze, structured agent launch, explicit shell kind, and direct resume argv.
- `runtime-status-governor`: Per-session reducer wiring, hook/notify integration, process-tree signal/sample, resource policy, and continuous hibernate/wake.
- `local-session-recovery`: Archive/unarchive/resume, local checkpoint/relaunch substrate, bounded shutdown flush, and real daemon recovery E2E.

### Modified Capabilities

- None. This child change adds executable Wave 1B capabilities derived from the master
  `runtime-session-lifecycle` requirement. It does not modify the already accepted Wave 1A
  transport wire contract.

## Impact

- Product implementation owners after approval:
  - `crates/homie-runtime/**`
  - `crates/homie-agents/**`
  - focused lifecycle DTO/client methods in `crates/homie-proto/**` and `crates/homie-client/**`
  - runtime/agent/process E2E tests
- Cross-change dependency:
  - T-102 G3 owns the resolved launch/effective-config type and field contract.
  - T-103 `homie-t3u.2` / `S103-GREEN-02` exclusively owns schema v4, repository persistence,
    deterministic hashing, atomic session binding, and readback.
  - T-102 G5 waits for the T-103 repository GREEN handoff.
  - T-103 shared proto/runtime integration still waits for T-102 release; this ordering has no
    dependency cycle.
- Long-lived contracts:
  - `specs/runtime-supervisor/README.md`
  - `specs/agent-adapter-contract/README.md`
- Tracking:
  - Bead `homie-t3u.1`
  - parent change `diri-7ba3407-parity-rebaseline`
  - master task T-102
  - baseline Diri `7ba3407`
  - checkpoint `48f522b`
- Explicitly unchanged in this specification task:
  - product code
  - parity lock
  - master tasks
  - other component specs
  - Bead state
- Explicitly deferred:
  - RT-010 remote migration and handoff
  - T-202 terminal/UI interaction
  - T-401 remote node
  - T-402 provider proxy and virtual-key issuance

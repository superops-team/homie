# Diri 7ba3407 Capability Matrix

```yaml
change_id: diri-7ba3407-parity-rebaseline
baseline_repository: diri/
baseline_commit: 7ba3407
status: locked
overall_result: not_parity_complete
beads: homie-t3u
source_prd: prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md
```

## 1. Purpose

This document is the canonical capability inventory for comparing Homie with the embedded Diri repository at commit `7ba3407`.

It supersedes `docs/research/reference-feature-coverage.md` and `docs/research/diri-module-inventory.md` as the planning baseline. Those documents remain useful historical inventories, but a status or mapping in them does not prove current parity.

## 2. Status Rules

| Status | Meaning |
|--------|---------|
| `implemented` | Real Homie code is wired through the production ownership boundary and current automated/e2e evidence passes. |
| `partial` | Some code or tests exist, but behavior, integration, persistence, UI, recovery, or evidence is incomplete. |
| `missing` | No meaningful implementation of the required behavior exists. |
| `blocked` | Implementation cannot proceed until a named prerequisite is delivered. |

The following never qualify as `implemented` by themselves:

- protocol constants or DTOs without a runtime handler;
- MCP descriptors without an executable tool handler;
- source-text tests;
- static or fixture-only UI;
- local UI state that bypasses the owning service;
- parser/model tests without product-path integration;
- release scripts that skip signing, notarization, installation, rollback, or measurement;
- documentation or Beads coverage.

## 3. Frozen Diri Module Inventory

| ID | Diri module | Diri source | Required capability | Current Homie evidence | Status | Primary remaining gap | Target wave |
|----|-------------|-------------|---------------------|------------------------|--------|-----------------------|-------------|
| M01 | Desktop Workbench | `diri-app/src/root.rs`, `workbench.rs` | Runtime-backed window/workbench/session projection | GPUI shell and live shell attach paths exist | partial | Complete projection lifecycle, disconnected/degraded state, real actions, screenshot E2E | Wave 2A |
| M02 | Sidebar | `diri-app/src/sidebar/*` | Projects/sessions, status, hover, rename, pin/archive, drag, multi-select | Sidebar model and visible controls exist | partial | Persist and dispatch actions through client; hover/drag/interaction E2E | Wave 2A |
| M03 | Terminal Pane | `diri-app/src/terminal_pane.rs` | Header/grid/input/paste/copy/resize/find/scrollback/selection | Grid/find/scrollback models and app surface exist | partial | Real grid stream, selection/copy, row fetch, resize/repaint E2E | Wave 2B |
| M04 | Inspector/Diff/Artifacts | `diri-app/src/inspector.rs`, `diff.rs` | Real Info/Changes/Artifacts tabs, diff, artifacts | Artifact scanner and git diff parser/tests exist | partial | Tab state, live data, large diff behavior, browser/PR/port integration | Wave 2D |
| M05 | Navigation/Quick Open/History | `navigation.rs`, `quick_open.rs`, `switcher.rs`, `history.rs` | Palette, file Quick Open, overview, switcher, history resume | Session/navigation quick open and transcript scanner exist | partial | File index/cache/ranking, overview, switcher, true resume UI/E2E | Wave 2C |
| M06 | Settings/Preferences | `settings.rs`, `surface_shell.rs` | General/Terminal/Resources/Remote and durable preferences | Settings surface and preferences repository exist | partial | Service-owned writes, full fields, apply/reload semantics, interaction E2E | Wave 2C |
| M07 | Worktrees/Git | `DirijorGit/*`, `worktrees.rs` | Locate/create/list/remove/cleanup/diff | Git/worktree runtime and CLI tests exist | partial | App workflow, remote ownership, cleanup safety, full e2e | Wave 2D |
| M08 | Notifications/Sounds/Menu Bar | `notifications.rs`, `sounds.rs`, `macos/*` | Native delivery, rollup, sounds, quick approve/deny | Rollup/descriptor models and visible modal exist | partial | Native bridge, menu, sound, real action dispatch and failure recovery | Wave 2C |
| M09 | Design System | `diri-ui/src/*`, `assets/icons/*` | Tokens, brand, icons, glyphs, floating surfaces, gallery | Homie tokens/glyph models and screenshot exist | partial | Complete asset rendering and side-by-side visual gate | Wave 2A/2C |
| M10 | Protocol | `diri-proto/src/*`, `DirijorProtocol/*` | Complete method/event/host/node/grid/data contracts | Broad DTO and method catalogs exist | partial | Wire fixture corpus, handler parity, version negotiation, attachment protocol | Wave 1A |
| M11 | Client | `diri-client/src/*`, `DirijorClient/*` | UDS client, reconnect, heartbeat, resume, attachment/backpressure | Typed in-process wrapper and event cursor tests exist | partial | Replace embedded supervisor with real transport and recovery | Wave 1A |
| M12 | CLI | `dirijor-cli/*` | Complete command grammar and streaming operations | create/list/snapshot/kill/diff/history/worktree/ports subsets exist | partial | get/read/send/wait/spawn/release/archive undo/status/artifacts/forward/subscribe | Wave 3A |
| M13 | MCP | `dirijor-mcp`, `DirijorMCP/*` | Exact schemas, executable tools, lineage and automation | Stdio and several runtime-backed tools exist | partial | Browser/test tools, two omitted tools, recursive lineage, error/schema parity | Wave 3B |
| M14 | Runtime Supervisor | `diri-engine/src/*`, `DirijorDaemonKit/*` | Daemon, session registry, event bus, screen, resource, migration, shutdown | Local supervisor, holder, log, screen and storage subsets exist | partial | Independent daemon, arbitrary agent spawn, resource/crash matrix, migration/shutdown | Wave 1A/1B |
| M15 | Holder/Process Tree | `DirijorHolderKit/*`, `dirijord-holder` | PTY survival, stat, adoption, process-tree control | Holder protocol and lifecycle tests exist but current tests regress | partial | Restore reliable live adoption, stop/continue/resource sampling and crash E2E | Wave 1B |
| M16 | Agent Detection | manifests, `DirijorDetection/*`, `diri-engine/src/detect/*` | 19-agent catalog, readiness, reducer, hook/notify, resume | Manifest, golden screen, parser and reducer tests pass | partial | Use descriptors in runtime spawn/profile/injection/resume product path | Wave 1B |
| M17 | Core Models | `DirijorCore/*`, `diri-proto` models | Stable IDs, session/project/status/attention/needs-input semantics | Rust DTO and storage subsets exist | partial | Single canonical model ownership and complete serialization fixtures | Wave 1C |
| M18 | Remote Node/Handoff | `diri-node/*`, `NODE.md` | Node server, accounts, remote spawn, checkpoint, move/fork, lease | DTO fixtures, prefs plan, repo locate and companion config exist | partial | Network server/auth, account runtime, transfer/restore/lease and remote E2E | Wave 4A |
| M19 | Usage | `diri-usage`, `diri-app/src/usage/*` | Transcript watcher/cache, pricing snapshots, local/fleet totals and UI | Parsers, pricing, import/cache repositories and CLI summary exist | partial | Incremental watcher, tail hash, snapshots, fleet merge, UI and proxy integration | Wave 4B |
| M20 | Updater/Packaging/Performance | `diri-updater`, `PACKAGING.md`, `UPDATING.md`, `PERF.md` | Feed/download/verify/install/rollback, universal signed package, perf | Trust decision model, ad-hoc package script, not-run perf script exist | partial | Real updater pipeline, Developer ID/notary/staple/DMG/feed and packaged metrics | Wave 5A |

No Diri module is currently complete at the module level. Some feature atoms inside M14, M16, M17 and terminal storage are implemented, but their owning user workflow remains partial.

## 4. Previously Omitted Baseline Capabilities

These capabilities exist at Diri commit `7ba3407` but were absent or incomplete in the previous parity planning documents.

| Capability | Diri source | Required Homie owner | Current state | Completion evidence |
|------------|-------------|----------------------|---------------|---------------------|
| `summarize_children` MCP tool | `DirijorMCP/Tools.swift` | MCP/orchestrator/context | missing | Exact schema, direct/recursive lineage fixtures, runtime-backed E2E |
| `report_to_parent` MCP tool | `DirijorMCP/Tools.swift` | MCP/orchestrator/context | missing | Parent permission, safe payload and transcript E2E |
| `session.migrate` | Diri protocol/runtime migration | proto/client/runtime | missing | Checkpoint/replay migration and rollback/failure E2E |
| `daemon.prepare_shutdown` | Diri daemon protocol | proto/client/runtime | missing | Flush order and active-session preservation tests |
| `daemon.shutdown` | Diri daemon protocol | proto/client/runtime | missing | Idempotent shutdown and reconnect behavior tests |
| Browser sidecar | `sidecar/browser.js`, `sidecar/server.js` | MCP/package/runtime | missing | Browser tool E2E and packaged dependency closure |
| Port forward data path | CLI/runtime forwarding | client/runtime/CLI | missing | Loopback forward, cancellation, collision and access-control E2E |
| Node service packaging | `infra/diri-node.service` | remote/package | missing | Install/start/upgrade/rollback service evidence |

## 5. Protocol Truthfulness Audit

| Surface | Current mismatch | Required correction |
|---------|------------------|---------------------|
| `homie-proto::Method::ALL` | Includes methods not implemented by `HomieClient::request` | Implement method through the owning service or remove it from the public catalog until implemented |
| `HomieClient` | `open` embeds `RuntimeSupervisor` in the caller process | Replace with endpoint-based client; keep direct supervisor construction test-only |
| MCP `tools/list` | Advertises `browser` and `test_run` while dispatch returns unsupported | Do not advertise unavailable tools; advertise only after executable handler and E2E |
| MCP schemas | Uses permissive `additionalProperties: true` | Define exact per-tool schemas, required fields, aliases and unknown-field behavior |
| CLI output | Some commands have local ad hoc JSON/human output | Freeze human/JSON/NDJSON grammar fixtures |
| UI actions | Several mutate local model or copy only | Dispatch through typed client command and update from authoritative event |

## 6. Current Verification Baseline

Commands were executed against the current Homie working tree during the rebaseline audit.

| Command | Result | Interpretation |
|---------|--------|----------------|
| `cargo test --workspace` | fail | App source-text regression fails before remaining suites run |
| `cargo test --workspace --exclude homie-app` | fail | MCP unsupported JSON-RPC code expected `-32601`, actual `-32000` |
| `cargo test --workspace --exclude homie-app --exclude homie-cli` | fail | Runtime live PTY and holder adoption report `detached`, expected `running` |
| `cargo fmt --all -- --check` | pass | Rust formatting is clean |
| `git diff --check` | pass | No whitespace errors in current diff |
| `make parity-lock` | pass with incomplete rows | Structural lock parser passes; it does not detect behavior regressions in rows marked implemented |
| `make module-inventory-check` | pass | Existing inventory has expected structure, not runtime completeness |
| `make spec-diri-mapping-check` | pass | Existing specs contain mapping sections, not implementation completeness |

### 6.1 Mandatory Status Corrections

The following rows in `docs/research/diri-parity-lock.md` cannot retain `implemented` under the frozen status rules:

| Row | Current evidence | Required status |
|-----|------------------|-----------------|
| RT-001 | `runtime_spawn_shell_uses_live_pty` currently fails | partial |
| RT-006 | holder adoption/registry test currently fails | partial |
| RT-007 | holder survival/adoption behavior currently fails | partial |
| API-002 | client is in-process and has no UDS reconnect/attachment implementation | partial |

These corrections are planning truth, not a claim that previously recorded scoped work never existed.

## 7. Component Ownership

| Capability group | Long-lived component specs |
|------------------|----------------------------|
| Protocol/runtime/client/session | `runtime-supervisor`, `storage-indexing`, `observability` |
| Agent/profile/detection | `agent-adapter-contract`, `virtual-key-credentials` |
| Desktop/terminal/navigation/native | `desktop-shell`, `runtime-supervisor` |
| CLI/MCP/browser/lineage | `mcp-automation`, `intent-orchestrator`, `session-context-store` |
| Remote/node/handoff | `remote-node-handoff`, `virtual-key-credentials`, `packaging-updater` |
| Usage/proxy | `llm-proxy`, `virtual-key-credentials`, `storage-indexing`, `observability` |
| Homie extensions | `session-context-store`, `memory-controller`, `task-controller`, `intent-orchestrator` |
| Release/update/performance | `packaging-updater`, `observability` |

## 8. Dependency Graph

```text
Wave 0: frozen requirements and contracts
  -> Wave 1A: runtime daemon + protocol + client transport
     -> Wave 1B: holder + agent session runtime
     -> Wave 1C: durable core facts
        -> Wave 2A/2B/2C/2D: desktop product surfaces
        -> Wave 3A: complete CLI
        -> Wave 3B: MCP/browser/test automation
        -> Wave 4A: remote node/handoff
        -> Wave 4B: usage/LLM proxy
        -> Wave 4C: Homie control-plane integration
           -> Wave 5A: packaging/updater/performance
              -> Wave 5B: final parity gate
```

Wave 2 and Wave 3 must not create new direct runtime/storage shortcuts while waiting for Wave 1. Their RED tests should target the agreed client/protocol contracts.

## 9. Final Completion Rule

`overall_result` may change to `parity_complete` only when:

1. every required M01-M20 capability and omitted capability is `implemented`;
2. every protocol method and advertised MCP/CLI operation has a real handler and current evidence;
3. no UI operation bypasses the owning service;
4. workspace, process E2E, remote E2E, package, updater, security, screenshot and performance gates pass;
5. no evidence uses an invalid status vocabulary;
6. the final Beads issue is closed with a release-readiness report that cites all required evidence.

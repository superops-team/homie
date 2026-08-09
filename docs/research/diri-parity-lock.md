# Homie Diri Parity Lock

```yaml
change_id: diri-parity-lock
status: locked
source_reference:
  repository: diri/
  commit: 7ba3407
  rust_gpui: diri/diri/crates
  swift_daemon: diri/Sources
purpose: prevent partial/static implementations from being reported as Diri parity complete
```

## 1. Lock Rules

Homie is not Diri-parity-complete until every row in this document is `implemented` with evidence.

Allowed statuses:

- `implemented`: real Homie code exists, uses the real runtime/data path, and has a passing verification case.
- `partial`: some Homie code exists, but behavior, UI, integration, persistence, or verification is incomplete.
- `missing`: no meaningful Homie implementation exists.
- `blocked`: implementation depends on a prerequisite not yet delivered.

Hard rules:

- A preview shell, static page, fixture-only UI, or source-text test alone is never `implemented`.
- A PRD/OpenSpec row marked as covered is not implementation evidence.
- A component port is not complete until it is wired into the user-facing workflow or explicitly scoped as a library-only task.
- Release readiness may pass for a scoped slice, but Diri parity remains incomplete while any row below is not `implemented`.
- Any future dev loop must update this file before implementation and update evidence paths after verification.

## 2. Product Surface Lock

| ID | Diri source | Required Homie behavior | Homie owner | Status | Required evidence |
|----|-------------|-------------------------|-------------|--------|-------------------|
| UI-001 | `diri-app/src/root.rs`, `workbench.rs` | Real app workbench with runtime-backed session state, not static preview | `homie-app`, `homie-client` | partial | `cargo test -p homie-client`; `cargo test -p homie-app`; `docs/verification/diri-ui-screenshot/visual-verification-report.md` cover client-backed live shell source/status, app snapshot attach path, command-palette runtime spawn, sidebar session projection/selection, runtime resize wiring, Diri-style light sidebar/inspector panel treatments, visible needs-input permission modal projection, and real app screenshot capture; Diri side-by-side interaction/screenshot E2E still pending |
| UI-002 | `diri-app/src/sidebar/*` | Sidebar projects/sessions, status glyphs, hover card, rename, pin/archive, drag reorder, multi-select | `homie-app`, `homie-ui` | partial | `cargo test -p homie-ui --tests`; `cargo test -p homie-app --tests`; `cargo clippy -p homie-ui --all-targets -- -D warnings`; `docs/verification/diri-ui-screenshot/visual-verification-report.md` cover sidebar session model selection, multi-select, rename, pin/archive, reorder, status glyph projection, and Diri-style light sidebar treatment; hover card/drag UI/screenshot E2E still pending |
| UI-003 | `diri-app/src/terminal_pane.rs` | Terminal header, chips, grid, input, paste/copy, resize, find, scrollback, selection | `homie-app`, `homie-term`, `homie-runtime` | partial | `cargo test -p homie-client` covers spawn/send/resize/snapshot; `cargo test -p homie-term --test grid_input_find`; `cargo test -p homie-app` verifies app source reads terminal state through client snapshots, sends paste through client, synchronizes terminal geometry through `HomieClient::resize_session`, and exposes model-backed terminal Find surface; `docs/verification/diri-ui-screenshot/visual-verification-report.md` records real app screenshot; full GPUI terminal interaction E2E still pending |
| UI-004 | `diri-app/src/inspector.rs`, `diff.rs` | Right inspector Info/Changes/Artifacts with real diff/artifact data | `homie-app`, `homie-runtime` | partial | `cargo test -p homie-runtime --test git_diff_loading`; `cargo test -p homie-cli --test session_diff_cli`; `cargo test -p homie-client --tests`; `cargo test -p homie-app --tests`; `cargo clippy -p homie-client -p homie-app --all-targets -- -D warnings`; `docs/verification/diri-ui-screenshot/visual-verification-report.md` cover runtime-output artifact scan, inspector artifact summary, Diri-style light inspector panel, and runtime/CLI git diff loading; GPUI Changes panel E2E still pending |
| UI-005 | `diri-app/src/navigation.rs`, `quick_open.rs`, `switcher.rs`, `history.rs` | Command palette, quick open, overview, switcher, history resume | `homie-app`, `homie-ui`, `homie-storage` | partial | `cargo test -p homie-app --tests`; `cargo test -p homie-ui --tests`; `cargo clippy -p homie-app -p homie-ui --all-targets -- -D warnings` cover real Quick Open surface, fuzzy-ranked session/navigation items, and runtime-backed session activation; file quick open, overview/switcher/history resume E2E still pending |
| UI-006 | `diri-app/src/settings.rs`, `surface_shell.rs` | Settings General/Terminal/Resources/Remote tabs with persisted preferences | `homie-app`, `homie-storage` | partial | `cargo test -p homie-storage --test storage_bootstrap`; `cargo test -p homie-app --tests`; `cargo clippy -p homie-storage -p homie-app --all-targets -- -D warnings` cover persisted settings preferences and real General/Terminal/Resources/Remote settings surface; full settings screenshot/interaction E2E still pending |
| UI-007 | `diri-app/src/worktrees.rs` | Worktree sheet, create/list/remove/cleanup suggestions | `homie-app`, `homie-runtime` | partial | `cargo test -p homie-runtime --test worktree_safety`; `cargo test -p homie-runtime --test worktree_git`; `cargo test -p homie-cli --test worktree_cli`; `cargo test -p homie-client --tests`; `cargo test -p homie-app --tests`; `cargo clippy -p homie-client -p homie-app --all-targets -- -D warnings` cover worktree overview projection, real git worktree create/list/remove, and app Worktrees surface with cleanup suggestion badge; full app interaction E2E still pending |
| UI-008 | `diri-app/src/notifications.rs`, `sounds.rs`, `macos/*` | Menu bar, native notifications, sounds, status rollup, quick approve/deny | `homie-app`, macOS bridge | partial | `cargo test -p homie-ui --tests`; `cargo test -p homie-app --tests`; `cargo clippy -p homie-ui -p homie-app --all-targets -- -D warnings` cover notification rollup, safe macOS notification command builder, quick action descriptors, app inspector notification summary, and visible needs-input permission modal projection; native delivery/menu bar/sounds/real quick approve-deny execution E2E still pending |
| UI-009 | `diri-ui/src/*`, `assets/icons/*` | Diri design tokens, brand marks, icons, status glyphs, floating surfaces, gallery parity | `homie-ui` | partial | `cargo test -p homie-ui --tests`; `cargo clippy -p homie-ui --all-targets -- -D warnings`; `docs/verification/diri-ui-screenshot/visual-verification-report.md` cover design tokens, brand mark, status glyph catalog, gallery entries, and real app screenshot capture; icon asset rendering and Diri side-by-side screenshot gate still pending |

## 3. Runtime And Session Lock

| ID | Diri source | Required Homie behavior | Homie owner | Status | Required evidence |
|----|-------------|-------------------------|-------------|--------|-------------------|
| RT-001 | `diri-engine/src/pty.rs`, `session.rs` | Spawn real PTY process, input, output pump, terminate | `homie-runtime` | partial | `cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1` currently fails `runtime_spawn_shell_uses_live_pty`: actual `detached`, expected `running`; restore and record passing process E2E under T-102 before returning to implemented |
| RT-002 | `diri-engine/src/log.rs`, Swift `OutputLog.swift` | Offset-addressed output log detach/replay | `homie-runtime` | implemented | `cargo test -p homie-runtime --test session_lifecycle` covers `read_output_range` replay |
| RT-003 | `diri-engine/src/screen.rs`, Swift `HeadlessScreen.swift` | Headless terminal emulator as status source | `homie-runtime` | implemented | `cargo test -p homie-runtime --test session_lifecycle` covers holder output log -> headless screen -> runtime status report |
| RT-004 | `diri-engine/src/status/*`, Swift `Reducer.swift` | Status reducer: hooks/screen/process authority, anti-flicker, subagent isolation | `homie-agents`, `homie-runtime` | partial | `cargo test -p homie-agents status_reducer`; `cargo test -p homie-runtime --test session_lifecycle` covers screen pipeline; hooks/process runtime injection still pending |
| RT-005 | `diri-engine/src/hooks.rs`, Swift `HookParsing.swift` | Claude/Codex hook/notify parsing, fail-open, redaction | `homie-agents`, `homie-cli`, `homie-runtime` | partial | `cargo test -p homie-agents hook_parser`; `cargo test -p homie-cli hook_command_outputs_redacted_structured_event`; `cargo test -p homie-cli notify_command_outputs_codex_turn_complete`; `cargo test -p homie-cli --test hook_report_runtime_cli`; `cargo test -p homie-cli --test notify_runtime_cli` cover CLI parsing/redaction, fail-open parse-only behavior, runtime-persisted PermissionRequest needs-input, and runtime-persisted Codex turn-complete idle status visible through session snapshot; full hook event bus/status matrix still pending |
| RT-006 | `diri-engine/src/registry.rs`, Swift `SessionRegistry.swift` | Durable session registry, restore from storage/logs | `homie-runtime`, `homie-storage` | partial | `cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1` currently fails holder adoption with actual `detached`, expected `running`; restore registry/adoption E2E under T-102 before returning to implemented |
| RT-007 | `DirijorHolderKit/*`, `dirijord-holder` | Holder-equivalent PTY survival across app/runtime crash | `homie-runtime` | partial | `cargo test -p homie-runtime --test daemon_process -- --test-threads=1` and `cargo test -p homie-cli --test shared_daemon_e2e -- --test-threads=1` prove holder survival across daemon replacement, but direct supervisor reopen/adoption remains blocked by T-102 `detached` failures |
| RT-008 | `ResourceGovernor.swift` | Resource governor, hibernate/wake/archive/reopen | `homie-runtime` | implemented | `cargo test -p homie-runtime --test session_lifecycle` covers hibernate stopping holder, wake restarting holder, and post-wake PTY interaction |
| RT-009 | `ProcessTree.swift`, `HolderProcessTree.swift` | Process tree tracking and group termination | `homie-runtime` | partial | `cargo test -p homie-runtime --test session_lifecycle` covers detached child tree kill; full stop/continue/resource sampling still pending |
| RT-010 | `ScreenCheckpoint.swift`, `SessionMigrator.swift` | Screen checkpoints and session migration | `homie-runtime` | partial | `cargo test -p homie-runtime --test session_lifecycle` covers screen checkpoint write/read after supervisor reopen; full session migration still pending |

## 4. Protocol, Client, And CLI Lock

| ID | Diri source | Required Homie behavior | Homie owner | Status | Required evidence |
|----|-------------|-------------------------|-------------|--------|-------------------|
| API-001 | `diri-proto/src/*`, `DirijorProtocol/*` | Full control methods/events, frame/grid codecs, host/remote models | `homie-proto` | partial | `cargo test -p homie-proto --tests` covers method/event catalog, session runtime DTOs, event cursor DTOs, control message roundtrip, Diri-compatible `host.locate_repo` DTO spelling with `originURL/sessionID`, Diri-compatible `host.sync_prefs` params/result DTOs, Diri-compatible host catalog and remote config DTOs, Diri-compatible node hello/status/usage DTO fixtures, Diri-compatible node checkpoint/blob/move DTO fixtures, Diri-compatible node account/login/provider-call DTO fixtures, and Diri-compatible `session.read_diff` base64 patch DTO; remaining full wire fixture corpus and runtime protocol E2E still pending |
| API-002 | `diri-client/src/*`, `DirijorClient/*` | Runtime client, reconnect, event resume, attachment | `homie-client` | implemented | `cargo test -p homie-client --tests -- --test-threads=1` covers pure async UDS Hello, correlation, cancellation, heartbeat, reconnect, bounded queues, event recovery, terminal offset/full-grid recovery, and shutdown; `cargo test -p homie-runtime --test server_streams -- --test-threads=1` proves the client over real UDS; `cargo test -p homie-cli --test shared_daemon_e2e -- --test-threads=1` proves cross-entry reconnect and event/terminal resume after daemon replacement |
| API-003 | `dirijor-cli/*` | CLI session/worktree/events/ports/forward/hook/notify/mcp bridge | `homie-cli` | partial | `cargo run -p homie-cli -- hook/notify ...` covers hook/notify parser entry; `cargo test -p homie-cli --test session_snapshot_cli` covers runtime-backed session create/snapshot/kill; `cargo test -p homie-cli --test control_stdio_cli` covers `homie control-stdio`; `cargo test -p homie-cli --test events_cli` covers `events list/wait`; `cargo test -p homie-cli --test host_locate_repo_cli` covers `homie host locate-repo`; `cargo test -p homie-cli --test mcp_stdio_runtime_cli` covers `homie mcp-stdio --data-dir` runtime-backed tool calls; `cargo test -p homie-cli --test worktree_cli` covers `homie worktree create/list/remove`; `cargo test -p homie-cli --test mcp_worktree_tools_cli` covers MCP bridge worktree create/list/remove against a real git repo; `cargo test -p homie-cli --test ports_cli` covers `homie ports`; forward E2E still pending |
| API-004 | `dirijor-mcp/src/main.rs`, `DirijorMCP/*` | MCP stdio server and tools for agent orchestration | `homie-cli`, `homie-runtime` | partial | `cargo test -p homie-cli --test mcp_stdio_cli` covers no-runtime JSON-RPC `tools/list/tools/call`; `cargo test -p homie-cli --test mcp_stdio_runtime_cli` covers runtime-backed `list_agents`, `whoami`, `get_status`, `read_output`, `send_prompt`, `spawn_agent`, and safe unsupported tool errors; `cargo test -p homie-cli --test mcp_wait_for_agent_cli` covers runtime-backed `wait_for_agent` for done/timeout/exited status waits; `cargo test -p homie-cli --test mcp_worktree_tools_cli` covers runtime-backed `create_worktree`, `list_worktrees`, and `remove_worktree`; `cargo test -p homie-cli --test mcp_get_artifacts_cli` covers runtime-backed `get_artifacts` over real session output with Diri `listeningPorts`; `cargo test -p homie-cli --test mcp_orchestration_transcript_cli` covers Diri-style spawn -> send -> wait -> read -> get_artifacts -> release transcript E2E; `cargo test -p homie-cli --test mcp_release_agent_cli`, `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli`, and `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli` cover release tool direct child, self, parent, ancestor, sibling, and unrelated safety paths; browser/test_run E2E still pending |
| API-005 | `MCPLineage*.swift` | MCP lineage, parent/child session tracking | `homie-orchestrator`, `homie-storage` | partial | `cargo test -p homie-cli --test mcp_stdio_runtime_cli` covers first-stage MCP identity flags `--session-id/--parent-session-id` in `whoami`; `cargo test -p homie-cli --test mcp_lineage_children_cli` covers MCP `spawn_agent` parent stamping and direct `list_children`; `cargo test -p homie-cli --test mcp_wait_for_agent_cli` covers single-session status wait used by the Diri spawn -> wait -> read flow; `cargo test -p homie-cli --test mcp_wait_children_cli` covers direct-child `wait_for_children`; `cargo test -p homie-cli --test mcp_release_agent_cli` covers direct-child release and self-release guard; `cargo test -p homie-cli --test mcp_release_ancestor_guard_cli` covers parent/ancestor release refusal; `cargo test -p homie-cli --test mcp_release_owned_child_guard_cli` covers sibling/unrelated release denial and target survival after refusal; `cargo test -p homie-cli --test mcp_send_prompt_lineage_cli` covers sibling provenance and self-send guard; recursive lineage and full permission enforcement E2E still pending |

## 5. Agent Catalog And Detection Lock

| ID | Diri source | Required Homie behavior | Homie owner | Status | Required evidence |
|----|-------------|-------------------------|-------------|--------|-------------------|
| AG-001 | `DirijorCore/Resources/manifests/*.json` | 19-agent catalog 1:1 with metadata, status authority, approve/deny, resume | `assets/agent-descriptors`, `homie-agents` | implemented | `cargo test -p homie-agents --test manifest_catalog` |
| AG-002 | `DirijorDetection/*`, `diri-engine/src/detect/*` | Manifest schema, regions, predicate engine, captures, risk | `homie-agents` | partial | golden screen parity tests |
| AG-003 | `AgentReadiness.swift`, `diri-engine/src/agent.rs` | Binary readiness, launch env, profile config | `homie-agents`, `homie-runtime` | partial | `cargo test -p homie-agents --test manifest_catalog`; `cargo test -p homie-cli --test agent_readiness_cli` cover manifest readiness projection and CLI binary resolver E2E with fake executables; app new-agent/readiness UI and real installed agent smoke still pending |
| AG-004 | `CodexTranscript.swift`, `CursorChatStore.swift` | Transcript/history scanning by agent type | `homie-runtime`, `homie-storage` | partial | `cargo test -p homie-runtime --test history_scanner` and `cargo test -p homie-cli --test session_history_cli` cover Claude/Codex transcript fixture scan, tracked-id dedupe, storage history upsert, resume command projection, client `session.history`, and CLI `session history`; app history surface and real resume E2E still pending |

## 6. Terminal Lock

| ID | Diri source | Required Homie behavior | Homie owner | Status | Required evidence |
|----|-------------|-------------------------|-------------|--------|-------------------|
| TERM-001 | `diri-term/src/buffer.rs` | Cell buffer, damage tracking, cursor, row generation | `homie-term` | implemented | workspace tests |
| TERM-002 | `diri-term/src/element.rs` | GPUI terminal element rendering real session grids | `homie-term`, `homie-app` | partial | live terminal screenshot/E2E |
| TERM-003 | `diri-term/src/scrollback.rs` | Scrollback viewport, fetch/cache/compose, wheel routing | `homie-term` | implemented | `cargo test -p homie-term --test scrollback` |
| TERM-004 | `diri-term/src/selection.rs`, `find.rs`, `keys.rs` | Selection, find, keyboard and paste encoding | `homie-term` | partial | `cargo test -p homie-term --test grid_input_find`; `cargo test -p homie-app --tests` cover terminal find/key model and app-visible Find surface; selection and real PTY interaction E2E still pending |
| TERM-005 | `diri-term/src/theme.rs`, `metrics.rs`, `repaint.rs` | Theme, metrics, repaint pacing | `homie-term` | partial | visual/perf tests |

## 7. Artifacts, Git, And Automation Lock

| ID | Diri source | Required Homie behavior | Homie owner | Status | Required evidence |
|----|-------------|-------------------------|-------------|--------|-------------------|
| ART-001 | `ArtifactScanner.swift`, `BrowserPool.swift` | Artifact scanner, browser pool, preview links | `homie-runtime`, `homie-app` | partial | `cargo test -p homie-runtime --test artifact_scanner`; `cargo test -p homie-cli --test mcp_get_artifacts_cli`; `cargo test -p homie-client --tests`; `cargo test -p homie-app --tests` cover scanner, MCP `get_artifacts` over real session output, and app inspector wiring; browser pool/preview E2E still pending |
| ART-002 | `PortForwarder.swift`, `Ports.swift` | Port detection/forward/listing | `homie-runtime`, `homie-cli` | partial | `cargo test -p homie-runtime --test artifact_scanner`; `cargo test -p homie-cli --test ports_cli`; `cargo test -p homie-cli --test mcp_get_artifacts_cli`; `cargo test -p homie-client --tests`; `cargo test -p homie-app --tests` cover localhost port detection, runtime-backed `homie ports` listing from real session output, MCP `get_artifacts` Diri `listeningPorts`, and app inspector counts; TCP forwarding E2E still pending |
| ART-003 | `PullRequestMonitor.swift` | PR monitor chips and comments | `homie-runtime`, `homie-app` | partial | `cargo test -p homie-runtime --test pr_monitor` covers GitHub PR payload parsing, review-thread counts, rollup ladder and PR URL coordinates; background polling/session wiring and app chips/popover E2E still pending |
| GIT-001 | `DirijorGit/*`, `WorktreeDiffLoader.swift` | Git head/worktree/diff loading | `homie-runtime`, `homie-app` | partial | `cargo test -p homie-runtime --test git_diff_loading`; `cargo test -p homie-cli --test session_diff_cli`; `cargo test -p homie-proto session_read_diff_uses_diri_base64_wire` cover Diri-compatible diff DTO, tracked/untracked diff loading, HEAD/default-branch comparison, and runtime-backed CLI session diff; app Changes panel E2E still pending |
| GIT-002 | `RepoLocator.swift`, `Worktrees.swift` | Repo locate and worktree APIs | `homie-runtime`, `homie-cli` | partial | `cargo test -p homie-runtime --test worktree_safety`; `cargo test -p homie-runtime --test worktree_git`; `cargo test -p homie-cli --test worktree_cli`; `cargo test -p homie-cli --test mcp_worktree_tools_cli`; `cargo test -p homie-client --tests`; `cargo test -p homie-app --tests` cover worktree overview projection, cleanup eligibility model, Diri-style porcelain parsing, real git worktree create/list/remove, client dispatch, CLI E2E, and MCP bridge worktree create/list/remove; full app sheet interaction E2E still pending |
| AUTO-001 | `InjectionBuilder.swift`, `MCPBridge.swift`, `Forward.swift` | Prompt injection/bus/forwarding automation | `homie-orchestrator`, `homie-cli` | partial | `cargo test -p homie-orchestrator --test automation_injection` covers Diri-style base env, Claude/Codex hook/MCP/notify argv injection, session id flag and return-to-login-shell wrapping; MCP stdio, forwarding and full automation E2E still pending |

## 8. Remote, Node, Usage, And Update Lock

| ID | Diri source | Required Homie behavior | Homie owner | Status | Required evidence |
|----|-------------|-------------------------|-------------|--------|-------------------|
| REM-001 | `diri-node/src/*`, `NODE.md` | First-party node server, remote spawn, checkpoint, provider adapters | `homie-remote`, `homie-runtime` | partial | `cargo test -p homie-proto node_hello_and_usage_match_diri_wire`; `cargo test -p homie-proto node_checkpoint_move_match_diri_wire`; `cargo test -p homie-proto node_account_login_match_diri_wire` cover first-party node hello/status/usage, checkpoint/blob/move, account/login and provider-call DTO wire contracts; real node server, remote spawn, checkpoint file transfer, move lease runtime, account runtime, provider adapters and network E2E still pending |
| REM-002 | `remote_access.rs`, `RemoteConfig.swift` | Remote settings, companion access, pairing/token config | `homie-app`, `homie-remote` | partial | `cargo test -p homie-remote --test companion_config`; `cargo test -p homie-proto host_catalog_and_remote_config_match_diri_wire` cover Diri-compatible companion config load/save/remove, owner-only token file, endpoint label, explicit pairing URL, redacted Debug, and Diri-compatible remote config DTO fields `bindHost`/`forwardAnyPort`; app settings wiring and listener E2E still pending |
| REM-003 | `host.sync_prefs`, `host.locate_repo` | Host preference sync and repo location | `homie-remote` | partial | `cargo test -p homie-remote --test prefs_sync` covers Diri-style secretless prefs sync include list, mkdir/rsync argv and missing-rsync errors; `cargo test -p homie-proto host_sync_prefs_round_trips_diri_wire`; `cargo test -p homie-proto host_catalog_and_remote_config_match_diri_wire`; `cargo test -p homie-remote --test host_locate_repo`, `cargo test -p homie-client client_dispatches_host_locate_repo_from_project_facts`, and `cargo test -p homie-cli --test host_locate_repo_cli` cover Diri-style `host.sync_prefs` DTO wire contract, Diri-compatible host catalog DTOs, `host.locate_repo` origin discovery, linked worktree config, not-cloned/no-origin result projection, storage-backed project fact matching, and CLI fixture E2E; real remote SSH/node E2E still pending |
| USAGE-001 | `diri-app/src/usage/*`, `diri-usage` | Local/fleet usage parsing, pricing, cache, transcript watcher | `homie-llm`, `homie-storage`, `homie-app` | partial | `cargo test -p homie-storage --test diri_storage_indexing`; `cargo test -p homie-cli --test usage_summary_cli`; `cargo test -p homie-llm --test usage_pricing`; `cargo test -p homie-llm --test usage_transcript_parser`; `cargo test -p homie-storage --test usage_transcript_import`; `cargo test -p homie-storage --test usage_scan_cache`; `cargo test -p homie-proto node_hello_and_usage_match_diri_wire` cover usage ledger schema, dedupe, aggregate query, `homie usage summary` CLI over seeded usage records, Diri-compatible API-equivalent pricing/cache estimate helpers, Claude/Codex transcript-to-neutral-event parsing, storage import/dedupe from parsed transcript events, durable usage scan file offset-cache repository, and fleet usage DTO wire contracts; filesystem watcher, incremental parser using saved offset, tail hash calculation, pricing snapshot persistence, fleet merge runtime, and usage UI E2E still pending |
| UPDATE-001 | `diri-updater/src/*`, `UPDATING.md` | Feed, trust, codesign, install, rollback | `homie-updater`, packaging | partial | updater trust + install E2E |
| PKG-001 | `PACKAGING.md`, `scripts/package.sh` | App packaging, CLI inclusion, codesign/notarization, DMG | `scripts/package`, release | partial | packaged app launch + notarization gate |
| PERF-001 | `PERF.md`, `perf-gate.sh` | Packaged startup/perf budgets | package/release | partial | perf gate evidence |

## 9. Current Homie Completion Statement

Current status is **not Diri parity complete**.

Current completed slice:

- local shell PTY spawn through `RuntimeSupervisor`;
- hook/status library tests;
- scrollback model tests;
- token parity subset;
- package binary collision fixed;
- app starts a real local shell session and displays live PTY output.

Still incomplete before Homie can claim Diri parity:

- full app/client/runtime protocol wiring;
- complete Diri UI surfaces and interactions;
- full holder manager/resource-governor crash matrix beyond current holder-owned PTY reopen and process-tree kill tests;
- remote node;
- MCP orchestration server;
- artifact/browser/port/PR monitors;
- usage UI/fleet accounting;
- updater install/rollback;
- packaging notarization and visual/perf gates.

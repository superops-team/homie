# Diri 全功能模块抽象清单

```yaml
change_id: diri-module-inventory
beads: homie-729
status: baseline_complete_needs_per_module_prd
purpose: 为 Homie 完整复刻 Diri 提供模块级功能列表、Homie 改造面、后续 PRD/Spec/OpenSpec 拆解入口
source_reference:
  swift_sources: diri/Sources
  rust_sources: diri/diri/crates
  swift_tests: diri/Tests
  homie_lock: docs/research/diri-parity-lock.md
```

## 1. 使用规则

- 本文是后续 PRD/spec/OpenSpec 的源清单，不是实现证据。
- 每个模块后续必须单独写中文 PRD/spec，更新对应 `specs/` 长期组件规格，再拆 `openspec/changes/<change-id>/plan.md` 和 `tasks.md`。
- 任何功能只有通过真实代码路径和验证证据后，才能更新 `docs/research/diri-parity-lock.md`。
- Homie 当前不追求兼容旧实现；应按 Diri 功能和产品体验重构 Homie 模块边界。

## 2. 模块总览

| Module ID | Diri module | 功能域 | Homie 目标模块 | 当前锁表行 |
|-----------|-------------|--------|----------------|------------|
| M01 | `diri-app` root/workbench/surface shell | 桌面主工作台 | `homie-app`, `homie-ui`, `homie-client` | UI-001 |
| M02 | `diri-app/sidebar/*`, session surfaces | Sidebar 会话/项目导航 | `homie-app`, `homie-ui`, `homie-storage` | UI-002 |
| M03 | `diri-term`, `terminal_pane` | Terminal grid/input/find/selection | `homie-term`, `homie-app`, `homie-runtime` | UI-003, TERM-001..005 |
| M04 | `inspector`, `diff`, artifacts | 右侧 Inspector/变更/产物 | `homie-app`, `homie-runtime`, `homie-storage` | UI-004, ART-001..003, GIT-001 |
| M05 | `navigation`, `quick_open`, `switcher`, `history` | 导航、命令、历史恢复 | `homie-app`, `homie-ui`, `homie-storage`, `homie-runtime` | UI-005, AG-004 |
| M06 | `settings`, `remote_access`, `surface_shell` | 设置与偏好 | `homie-app`, `homie-storage`, `homie-remote` | UI-006, REM-002 |
| M07 | `worktrees`, `DirijorGit`, worktree CLI | Worktree/Git 工作区 | `homie-runtime`, `homie-cli`, `homie-app`, `homie-storage` | UI-007, GIT-001, GIT-002 |
| M08 | `notifications`, `sounds`, macOS bridge | 通知、声音、菜单栏 | `homie-app`, `homie-ui`, macOS bridge | UI-008, RT-005 |
| M09 | `diri-ui` | Design system/tokens/icons | `homie-ui`, assets | UI-009 |
| M10 | `diri-proto`, `DirijorProtocol` | 协议/DTO/wire | `homie-proto` | API-001 |
| M11 | `diri-client`, `DirijorClient` | Client attach/reconnect/events | `homie-client` | API-002 |
| M12 | `dirijor-cli` | CLI/session/worktree/events/ports/MCP | `homie-cli`, `homie-client` | API-003 |
| M13 | `dirijor-mcp`, `DirijorMCP` | MCP server/tools | `homie-cli`, `homie-orchestrator`, `homie-runtime` | API-004, API-005 |
| M14 | `diri-engine`, `DirijorDaemonKit` | Runtime/session/PTY/log/status | `homie-runtime`, `homie-agents`, `homie-storage` | RT-001..010 |
| M15 | `DirijorHolderKit`, `dirijord-holder` | Holder/PTY survivability | `homie-runtime` holder | RT-007, RT-009 |
| M16 | `DirijorDetection`, manifests | Agent detection/status reducer | `homie-agents`, assets | AG-001..004, RT-004 |
| M17 | `DirijorCore` | Core domain models/catalog | `homie-proto`, `homie-agents`, `homie-ui` | AG-001, API-001 |
| M18 | `diri-node`, remote protocols | Remote node/handoff | `homie-remote`, `homie-runtime`, `homie-client` | REM-001..003 |
| M19 | `diri-usage`, usage UI | Usage/pricing/accounting | `homie-llm`, `homie-storage`, `homie-app` | USAGE-001 |
| M20 | `diri-updater`, packaging scripts | Update/package/release/perf | `homie-updater`, `scripts/package` | UPDATE-001, PKG-001, PERF-001 |

## 3. 模块功能列表与开发拆解入口

### M01 Desktop Workbench

Diri sources:

- Rust: `diri/diri/crates/diri-app/src/root.rs`, `workbench.rs`, `surface_shell.rs`, `main.rs`, `daemon_launch.rs`, `session_surfaces.rs`, `seam.rs`, `fonts.rs`
- Swift: `Sources/DirijorClient/*`, `Sources/DirijorCore/SessionRecord.swift`, `SessionStatus.swift`

功能列表:

- App 启动与 daemon/runtime 连接。
- 主窗口 layout：sidebar、terminal pane、right inspector、floating surfaces。
- Session selected state、connection/degraded states、storage/runtime health。
- New session / attach existing session / restore last session。
- Window chrome、toolbar chips、surface shell、overlay stacking。
- App-level keyboard routing：palette、quick open、settings、find、escape cascade。
- Runtime-backed UI state subscription，不允许 preview/static 假状态。

Homie 改造面:

- `crates/homie-app/src/main.rs`
- `crates/homie-client`
- `specs/desktop-shell/README.md`
- `specs/runtime-supervisor/README.md`

后续 PRD/spec:

- `prd-spec/features/diri-desktop-workbench/`
- `specs/desktop-shell/README.md`
- `openspec/changes/diri-desktop-workbench/`

### M02 Sidebar Sessions And Projects

Diri sources:

- Rust: `diri-app/src/session_surfaces.rs`, `workbench.rs`, `navigation.rs`, `worktrees.rs`
- Swift/Core: `SessionRecord.swift`, `Attention.swift`, `Worktrees.swift`

功能列表:

- Project sections and session rows。
- Status glyphs、attention badges、needs-input visual state。
- Hover card with status/context/actions。
- Inline rename。
- Pin/archive/unarchive。
- Drag reorder。
- Multi-select and bulk actions。
- Sidebar width/collapse/persistence。
- “Local agents” footer/status rollup。

Homie 改造面:

- `crates/homie-ui` sidebar model。
- `crates/homie-app` sidebar rendering/actions。
- `homie-storage` preferences/session metadata。

后续 PRD/spec:

- `prd-spec/features/diri-sidebar-sessions/`
- `specs/desktop-shell/README.md`
- `openspec/changes/diri-sidebar-sessions/`

### M03 Terminal Pane And Terminal Core

Diri sources:

- Rust: `diri-term/src/buffer.rs`, `element.rs`, `find.rs`, `keys.rs`, `metrics.rs`, `repaint.rs`, `scrollback.rs`, `selection.rs`, `theme.rs`
- Rust app: `diri-app/src/terminal_pane.rs`, `clipboard_transfer.rs`, `query_editor.rs`
- Swift daemon: `PTY.swift`, `HeadlessScreen.swift`, `OutputLog.swift`

功能列表:

- Grid buffer/cells/damage/cursor。
- GPUI terminal renderer and repaint pacing。
- Keyboard encoding and paste encoding。
- Selection, word selection, copy。
- Find bar, match highlights, next/previous。
- Scrollback viewport, wheel routing, fetch/cache/compose。
- Resize geometry sync。
- Terminal header chips/status/toolbar。
- Theme/metrics/font fallback。
- Live PTY integration and replay from output log。

Homie 改造面:

- `crates/homie-term`
- `crates/homie-app`
- `crates/homie-runtime`
- `crates/homie-client`

后续 PRD/spec:

- `prd-spec/features/diri-terminal-parity/`
- `specs/desktop-shell/README.md`
- `specs/runtime-supervisor/README.md`
- `openspec/changes/diri-terminal-parity/`

### M04 Inspector, Diff, Artifacts

Diri sources:

- Rust: `diri-app/src/inspector.rs`, `diff.rs`
- Swift daemon: `ArtifactScanner.swift`, `BrowserPool.swift`, `PortForwarder.swift`, `PullRequestMonitor.swift`, `WorktreeDiffLoader.swift`
- Tests: `ArtifactScannerTests.swift`, `BrowserPoolTests.swift`, `PortForwardTests.swift`, `PullRequestMonitorTests.swift`, `WorktreeDiffTests.swift`

功能列表:

- Right inspector tabs: Info, Changes, Artifacts。
- Session info card: agent, project, directory, memory, updated time。
- Git status card and unavailable state。
- Diff summary, file list, virtualized diff loading。
- Artifact cards: PR, preview, generic links, browser preview。
- Port list and forwarding state。
- Pull request monitor chips/comments/checks。
- Details table and copy/open actions。

Homie 改造面:

- `crates/homie-app`
- `crates/homie-runtime`
- `crates/homie-client`
- `crates/homie-storage`
- `crates/homie-cli` for ports/PR commands

后续 PRD/spec:

- `prd-spec/features/diri-inspector-artifacts/`
- `specs/mcp-automation/README.md`
- `openspec/changes/diri-inspector-artifacts/`

### M05 Navigation, Quick Open, Switcher, History

Diri sources:

- Rust: `navigation.rs`, `quick_open.rs`, `switcher.rs`, `history.rs`, `fuzzy.rs`, `palette.rs`
- Swift daemon: `HistoryScanner.swift`, `CodexTranscript.swift`, `CursorChatStore.swift`, `TitleWatcher.swift`
- Tests: `HistoryScannerTests.swift`, `TitleWatcherTests.swift`

功能列表:

- Command palette with action registry and keyboard navigation。
- Quick open over files/folders/sessions/worktrees。
- Session switcher。
- Overview board/list。
- History scan by agent type and transcript source。
- Resume from history with cwd/transcript validation。
- Fuzzy scoring and ranking。
- Title watcher / first prompt title extraction。

Homie 改造面:

- `crates/homie-ui` ranking/history model。
- `crates/homie-app` surfaces。
- `crates/homie-storage` history tables。
- `crates/homie-runtime` transcript scanners。

后续 PRD/spec:

- `prd-spec/features/diri-navigation-history/`
- `specs/session-context-store/README.md`
- `openspec/changes/diri-navigation-history/`

### M06 Settings And Preferences

Diri sources:

- Rust: `settings.rs`, `remote_access.rs`, `updates.rs`, `surface_shell.rs`
- Swift daemon: `PrefsSync.swift`, `ResourceGovernor.swift`, `RemoteAccessTests.swift`, `PrefsSyncTests.swift`

功能列表:

- General/Terminal/Resources/Remote tabs。
- Persisted preferences。
- Terminal font/theme。
- Resource governor settings, memory/idle hibernate。
- Remote companion access / pairing / token config。
- Update settings。
- Preference sync to remote host。
- Error handling and optimistic-save rollback。

Homie 改造面:

- `crates/homie-app`
- `crates/homie-storage`
- `crates/homie-remote`
- `crates/homie-updater`

后续 PRD/spec:

- `prd-spec/features/diri-settings-preferences/`
- `specs/desktop-shell/README.md`
- `specs/remote-node-handoff/README.md`
- `openspec/changes/diri-settings-preferences/`

### M07 Worktrees And Git

Diri sources:

- Rust app: `worktrees.rs`
- Swift: `DirijorGit/*`, `RepoLocator.swift`, `WorktreeDiffLoader.swift`
- CLI: `WorktreeCommands.swift`
- Tests: `WorktreeDetectionTests.swift`, `WorktreeDiffTests.swift`

功能列表:

- Locate repo from path。
- List worktrees。
- Create worktree。
- Remove worktree。
- Cleanup suggestions: stale, clean, merged, non-main。
- Branch/head detection。
- Dirty state。
- Worktree linked to session。
- Worktree diff loading。

Homie 改造面:

- `crates/homie-runtime`
- `crates/homie-cli`
- `crates/homie-app`
- `crates/homie-storage`

后续 PRD/spec:

- `prd-spec/features/diri-worktrees-git/`
- `specs/runtime-supervisor/README.md`
- `openspec/changes/diri-worktrees-git/`

### M08 Notifications, Sounds, Menu Bar

Diri sources:

- Rust: `notifications.rs`, `sounds.rs`
- Swift/macOS: vendor macOS notification bridge, app menu/window APIs
- Detection/hooks: `HookParsing.swift`, `Reducer.swift`

功能列表:

- Notification center model。
- Native macOS notification delivery。
- Menu bar/status extra。
- Status rollup。
- Sounds for status changes。
- Quick approve/deny action from known agent capability。
- Needs-input modal and action routing。
- Safe redaction for notification text。

Homie 改造面:

- `crates/homie-ui`
- `crates/homie-app`
- `crates/homie-agents`
- macOS bridge or packaging entitlement decisions

后续 PRD/spec:

- `prd-spec/features/diri-notifications-menubar/`
- `specs/desktop-shell/README.md`
- `openspec/changes/diri-notifications-menubar/`

### M09 Design System

Diri sources:

- Rust: `diri-ui/src/tokens.rs`, `components.rs`, `status.rs`, `brand.rs`, `icon.rs`, `svg.rs`
- Assets: `diri-ui/assets/*`
- Example: `diri-ui/examples/gallery.rs`

功能列表:

- Tokens: radius, metrics, spacing, typography, colors, fills, motion。
- Status glyphs and status color mapping。
- Brand mark and icon set。
- Floating surfaces, cards, rows, chips, badges。
- Gallery / screenshot fixtures。
- Light/dark variants。
- Accessibility and density rules。

Homie 改造面:

- `crates/homie-ui`
- `assets/`
- `crates/homie-app`
- screenshot gate scripts

后续 PRD/spec:

- `prd-spec/features/diri-design-system/`
- `specs/desktop-shell/README.md`
- `openspec/changes/diri-design-system/`

### M10 Protocol And Wire Contract

Diri sources:

- Rust: `diri-proto/src/control.rs`, `frames.rs`, `grid.rs`, `hosts.rs`, `methods.rs`, `model.rs`, `node.rs`, `paths.rs`, `remote.rs`
- Swift: `DirijorProtocol/*`
- Tests: `WireTests.swift`, `GridTests.swift`, `ScrollbackWireTests.swift`, `HostsConfigTests.swift`, `control_roundtrip.rs`

功能列表:

- ControlMessage request/response/event。
- Method catalog。
- Error envelope。
- Session/runtime DTOs。
- Grid/cell codecs and scrollback cells。
- Host/remote config。
- Paths and path safety。
- Node protocol。
- Forward/backward wire fixtures。

Homie 改造面:

- `crates/homie-proto`
- `crates/homie-client`
- `crates/homie-cli`
- `crates/homie-remote`

后续 PRD/spec:

- `prd-spec/features/diri-protocol-contract/`
- `specs/runtime-supervisor/README.md`
- `openspec/changes/diri-protocol-contract/`

### M11 Client Attachment And Event Subscription

Diri sources:

- Rust: `diri-client/src/client.rs`, `connection.rs`, `attachment.rs`, `state.rs`, `node_client.rs`
- Swift: `DirijorClient/*`
- Tests: `EventSubscriptionTests.swift`

功能列表:

- Daemon endpoint discovery。
- Connect/reconnect/backoff。
- Subscribe events。
- Resume event cursor。
- Attach session。
- Read screen/scrollback/diff/artifacts。
- Send text/input/resize。
- Remote node client。

Homie 改造面:

- `crates/homie-client`
- `crates/homie-proto`
- `crates/homie-remote`

后续 PRD/spec:

- `prd-spec/features/diri-client-attachment/`
- `specs/runtime-supervisor/README.md`
- `openspec/changes/diri-client-attachment/`

### M12 CLI

Diri sources:

- Swift: `dirijor-cli/Dirijor.swift`, `SessionCommands.swift`, `WorktreeCommands.swift`, `EventCommands.swift`, `Ports.swift`, `Forward.swift`, `MCPBridge.swift`, `MCPLineage.swift`, `MCPLineageTools.swift`, `DaemonConn.swift`, `CLIOutput.swift`
- Tests: `CommandGrammarTests.swift`

功能列表:

- doctor/status。
- session create/list/attach/snapshot/send/kill/archive/wake/history。
- events list/wait/subscribe。
- worktree create/list/remove/overview。
- ports list/forward。
- hook/notify forwarders。
- MCP bridge command surface。
- CLI output formats。
- lineage/whoami/children。

Homie 改造面:

- `crates/homie-cli`
- `crates/homie-client`
- `crates/homie-runtime`
- `crates/homie-orchestrator`

后续 PRD/spec:

- `prd-spec/features/diri-cli-automation/`
- `specs/mcp-automation/README.md`
- `openspec/changes/diri-cli-automation/`

### M13 MCP Automation

Diri sources:

- Rust: `dirijor-mcp/src/main.rs`
- Swift: `DirijorMCP/McpServer.swift`, `Tools.swift`, CLI MCP bridge/lineage files

功能列表:

- MCP stdio server。
- Tools: spawn/list/status/send/wait/read/release。
- Worktree tools。
- Artifact/browser/test tools。
- whoami/list_children/wait_children。
- Lineage and permission scope。
- Safe error envelopes。

Homie 改造面:

- `crates/homie-cli`
- `crates/homie-orchestrator`
- `crates/homie-runtime`
- `crates/homie-storage`

后续 PRD/spec:

- `prd-spec/features/diri-mcp-automation/`
- `specs/mcp-automation/README.md`
- `openspec/changes/diri-mcp-automation/`

### M14 Runtime Supervisor

Diri sources:

- Rust: `diri-engine/src/*`
- Swift: `DirijorDaemonKit/Daemon.swift`, `AgentSession.swift`, `SessionRegistry.swift`, `SessionLogStorage.swift`, `PersistenceStore.swift`, `StatusEngine.swift`, `ResourceGovernor.swift`
- Tests: runtime and daemon kit tests

功能列表:

- Spawn session and PTY。
- Session registry and persistence。
- Output log and tail offsets。
- Headless screen。
- Status reducer integration。
- Hook/notify ingestion。
- Event bus/connection hub。
- Resource governor。
- Hibernate/wake/archive/reopen。
- Crash recovery。
- Session migration/checkpoints。

Homie 改造面:

- `crates/homie-runtime`
- `crates/homie-storage`
- `crates/homie-agents`
- `crates/homie-client`

后续 PRD/spec:

- `prd-spec/features/diri-runtime-supervisor/`
- `specs/runtime-supervisor/README.md`
- `openspec/changes/diri-runtime-supervisor/`

### M15 Holder And Process Tree

Diri sources:

- Swift: `DirijorHolderKit/*`, `dirijord-holder/main.swift`
- C: `CDirijorPTY/cdirijor_pty.c`
- Tests: `HolderTests.swift`, `ProcessTreeTests.swift`

功能列表:

- Holder server protocol。
- PTY ownership outside runtime process。
- Holder output log。
- Holder client/manager lifecycle。
- Unix socket protocol。
- Process tree enumeration and kill。
- Start-time safety check。
- Resize/stat/terminate/kill-tree。

Homie 改造面:

- `crates/homie-runtime/src/holder.rs`
- `crates/homie-runtime/src/bin/homie-runtime-holder.rs`
- `crates/homie-runtime/src/process_tree.rs`

后续 PRD/spec:

- `prd-spec/features/diri-holder-process-tree/`
- `specs/runtime-supervisor/README.md`
- `openspec/changes/diri-holder-process-tree/`

### M16 Agent Catalog And Detection

Diri sources:

- Swift: `DirijorCore/AgentCatalog.swift`, `AgentDescriptor.swift`, `AgentKind.swift`, `Resources.swift`
- Detection: `DirijorDetection/*`
- Rust: `diri-engine/src/hooks.rs`, `status/*`
- Tests: `ManifestAndRegionTests.swift`, `GoldenScreenTests.swift`, `TrustDialogTests.swift`, `ReducerTests.swift`

功能列表:

- 19-agent descriptor catalog。
- Agent kind metadata。
- Approve/deny/resume keystrokes。
- Manifest schema。
- Region extraction。
- Predicate engine。
- Status reducer。
- Risk classification。
- Golden screen fixtures。
- Hook/notify parsing and redaction。
- Agent readiness/binary/env/profile config。

Homie 改造面:

- `assets/agent-descriptors`
- `crates/homie-agents`
- `crates/homie-runtime`
- `crates/homie-storage`

后续 PRD/spec:

- `prd-spec/features/diri-agent-detection/`
- `specs/agent-adapter-contract/README.md`
- `openspec/changes/diri-agent-detection/`

### M17 Core Domain Models

Diri sources:

- Swift: `DirijorCore/*`
- Rust proto/model crates

功能列表:

- Identifiers。
- SessionRecord/SessionStatus。
- NeedsInputDetail。
- Attention levels。
- AgentDescriptor/AgentKind。
- Titles。
- Resource bundle。
- Worktree core structs。

Homie 改造面:

- `crates/homie-proto/src/model.rs`
- `crates/homie-agents`
- `crates/homie-ui`

后续 PRD/spec:

- `prd-spec/features/diri-core-models/`
- `specs/storage-indexing/README.md`
- `openspec/changes/diri-core-models/`

### M18 Remote Node And Handoff

Diri sources:

- Rust: `diri-node/src/*`
- Swift: `RemoteConfig.swift`, `PrefsSync.swift`, remote tests
- Tests: `RemoteAccessTests.swift`, `RemoteSpawnTests.swift`, `PrefsSyncTests.swift`

功能列表:

- First-party node server。
- Node config/accounts。
- Remote spawn。
- Checkpoint/handoff。
- Remote provider adapters。
- Companion access。
- Host preference sync。
- Host repo locate。
- Fleet usage。

Homie 改造面:

- `crates/homie-remote`
- `crates/homie-runtime`
- `crates/homie-client`
- `crates/homie-storage`
- `crates/homie-app` settings

后续 PRD/spec:

- `prd-spec/features/diri-remote-node-handoff/`
- `specs/remote-node-handoff/README.md`
- `openspec/changes/diri-remote-node-handoff/`

### M19 Usage Accounting

Diri sources:

- Rust: `diri-usage/src/lib.rs`
- Rust app: `diri-app/src/usage/*`
- Node usage: `diri-node/src/usage.rs`

功能列表:

- Local usage parsing。
- Fleet usage projection。
- Pricing snapshots。
- Token/cache/reasoning accounting。
- Transcript watcher。
- Usage cache。
- Cost and latency display。
- Usage UI panel。

Homie 改造面:

- `crates/homie-llm`
- `crates/homie-storage`
- `crates/homie-app`
- `crates/homie-remote`

后续 PRD/spec:

- `prd-spec/features/diri-usage-accounting/`
- `specs/llm-proxy/README.md`
- `openspec/changes/diri-usage-accounting/`

### M20 Updater, Packaging, Perf

Diri sources:

- Rust: `diri-updater/src/*`
- Scripts: `scripts/package.sh`, `install-local.sh`, `perf-gate.sh`, `release.sh`
- Docs: `PACKAGING.md`, `UPDATING.md` if present in source tree

功能列表:

- Release feed。
- Version compare。
- Trust/codesign verification。
- Download/install/rollback。
- App bundle packaging。
- CLI inclusion。
- Holder inclusion。
- Codesign/notarization。
- DMG generation。
- Local install。
- Startup/perf gates。

Homie 改造面:

- `crates/homie-updater`
- `scripts/package`
- `Makefile`
- release docs and verification docs

后续 PRD/spec:

- `prd-spec/features/diri-release-updater-packaging/`
- `specs/packaging-updater/README.md`
- `openspec/changes/diri-release-updater-packaging/`

## 4. 后续规格生产顺序

建议顺序：

1. `diri-core-models`
2. `diri-protocol-contract`
3. `diri-runtime-supervisor`
4. `diri-holder-process-tree`
5. `diri-agent-detection`
6. `diri-client-attachment`
7. `diri-desktop-workbench`
8. `diri-terminal-parity`
9. `diri-sidebar-sessions`
10. `diri-inspector-artifacts`
11. `diri-navigation-history`
12. `diri-settings-preferences`
13. `diri-worktrees-git`
14. `diri-notifications-menubar`
15. `diri-mcp-automation`
16. `diri-cli-automation`
17. `diri-remote-node-handoff`
18. `diri-usage-accounting`
19. `diri-release-updater-packaging`
20. `diri-design-system`

原因：

- 先稳定协议、模型、runtime、agent detection，再做 UI 和 automation。
- UI 的每个 surface 必须依赖真实 client/runtime data，不允许再做静态页。
- Release/updater/perf 最后收口，避免把半成品打包成“完成”。

## 5. 当前 Homie 模块改造总表

| Homie module | 需要承接的 Diri modules | 主要改造方向 |
|--------------|--------------------------|--------------|
| `homie-proto` | M10, M17, M18 | 完整 method/event/DTO/grid/remote/host fixtures |
| `homie-client` | M11, M18 | reconnect, event resume, attach/read/write/resize, remote node client |
| `homie-runtime` | M14, M15, M03, M04, M07 | PTY/session/log/screen/status/artifacts/worktrees/resource/migration |
| `homie-agents` | M16 | manifest/golden screens/status reducer/hook readiness/transcripts |
| `homie-storage` | M05, M06, M07, M17, M19 | sessions/history/preferences/worktrees/artifacts/usage indexes |
| `homie-term` | M03 | terminal renderer, find, selection, scrollback, theme, repaint pacing |
| `homie-ui` | M02, M05, M08, M09 | tokens, sidebar model, notification model, quick open ranking, gallery |
| `homie-app` | M01..M09, M19, M20 UI | all user-facing workbench surfaces |
| `homie-cli` | M12, M13, M07, M04 | session/events/worktree/ports/MCP/hook/notify |
| `homie-orchestrator` | M13, M16 | MCP lineage, parent/child routing, automation bus |
| `homie-remote` | M18 | node server/client, handoff, host prefs/repo locate |
| `homie-llm` | M19 plus Homie-specific LLM custody | usage, pricing, proxy telemetry |
| `homie-updater` | M20 | update feed/trust/install/rollback |
| `scripts/package` | M20 | app/CLI/holder bundling, dmg, codesign, notarization, perf |

## 6. 覆盖校验

Covered Diri top-level source groups:

- `Sources/CDirijorPTY`: M15
- `Sources/DirijorClient`: M11
- `Sources/DirijorCore`: M17, M16
- `Sources/DirijorDaemonKit`: M14, M04, M05, M07, M18
- `Sources/DirijorDetection`: M16
- `Sources/DirijorGit`: M07
- `Sources/DirijorHolderKit`: M15
- `Sources/DirijorMCP`: M13
- `Sources/DirijorProtocol`: M10, M18
- `Sources/dirijor-cli`: M12, M13
- `Sources/dirijord`: M14
- `Sources/dirijord-holder`: M15
- `diri/diri/crates/diri-app`: M01..M09
- `diri/diri/crates/diri-client`: M11
- `diri/diri/crates/diri-engine`: M14, M16
- `diri/diri/crates/diri-node`: M18, M19
- `diri/diri/crates/diri-proto`: M10
- `diri/diri/crates/diri-term`: M03
- `diri/diri/crates/diri-ui`: M09
- `diri/diri/crates/diri-updater`: M20
- `diri/diri/crates/diri-usage`: M19
- `diri/diri/crates/dirijor-mcp`: M13

If a future scan finds a Diri source path not mapped above, this document must be updated before writing PRD/spec for implementation.

## 7. Feature Atom Matrix

每个 feature atom 是后续 PRD/OpenSpec/TDD 的最小追踪单位。实现前必须把对应 atom 展开到中文 PRD/spec 和 `openspec/changes/<change-id>/tasks.md`。

| Feature ID | Module | 功能原子项 | Diri source / tests | 用户可见行为 | Homie owner | Component spec | Planned PRD | Planned OpenSpec | Verification gate |
|------------|--------|------------|---------------------|--------------|-------------|----------------|-------------|------------------|-------------------|
| M01-F001 | M01 | App 启动连接 runtime 并渲染 live workbench | `diri-app/src/root.rs`, `workbench.rs`, `DirijorClient/*`, `SessionIntegrationTests.swift` | 打开 app 后看到真实 session/workbench，不是静态页 | `homie-app`, `homie-client`, `homie-runtime` | `specs/desktop-shell/README.md`, `specs/runtime-supervisor/README.md` | `prd-spec/features/diri-desktop-workbench/` | `openspec/changes/diri-desktop-workbench/` | packaged app launch + screenshot + live session snapshot |
| M01-F002 | M01 | App overlay/floating surface stack 与 Esc cascade | `surface_shell.rs`, `seam.rs`, `navigation.rs` | palette/settings/find/worktrees 等浮层互斥并可 Esc 关闭 | `homie-app`, `homie-ui` | `specs/desktop-shell/README.md` | `prd-spec/features/diri-desktop-workbench/` | `openspec/changes/diri-desktop-workbench/` | GPUI interaction E2E |
| M02-F001 | M02 | Sidebar session rows/status glyph/selection | `session_surfaces.rs`, `Attention.swift`, `SessionRecord.swift` | 左侧 session 列表可选择，状态可见 | `homie-app`, `homie-ui` | `specs/desktop-shell/README.md` | `prd-spec/features/diri-sidebar-sessions/` | `openspec/changes/diri-sidebar-sessions/` | sidebar state tests + screenshot |
| M02-F002 | M02 | Sidebar rename/pin/archive/drag/multi-select | `workbench.rs`, `worktrees.rs` | session 可重命名、置顶、归档、拖拽、多选 | `homie-ui`, `homie-app`, `homie-storage` | `specs/desktop-shell/README.md`, `specs/storage-indexing/README.md` | `prd-spec/features/diri-sidebar-sessions/` | `openspec/changes/diri-sidebar-sessions/` | app interaction E2E + storage persistence tests |
| M03-F001 | M03 | Terminal grid rendering and repaint | `diri-term/src/element.rs`, `repaint.rs`, `metrics.rs` | terminal pane 渲染真实 grid，resize 不错位 | `homie-term`, `homie-app` | `specs/desktop-shell/README.md` | `prd-spec/features/diri-terminal-parity/` | `openspec/changes/diri-terminal-parity/` | terminal screenshot + repaint tests |
| M03-F002 | M03 | Terminal input/paste/find/selection/scrollback | `keys.rs`, `find.rs`, `selection.rs`, `scrollback.rs`, `grid_input_find.rs` | 可输入、粘贴、搜索、选择、滚动历史 | `homie-term`, `homie-app`, `homie-client` | `specs/desktop-shell/README.md`, `specs/runtime-supervisor/README.md` | `prd-spec/features/diri-terminal-parity/` | `openspec/changes/diri-terminal-parity/` | real PTY interaction E2E |
| M04-F001 | M04 | Inspector Info/Changes/Artifacts tabs | `inspector.rs`, `diff.rs`, `WorktreeDiffTests.swift` | 右侧 inspector 显示 Info/Changes/Artifacts | `homie-app`, `homie-client`, `homie-runtime` | `specs/desktop-shell/README.md` | `prd-spec/features/diri-inspector-artifacts/` | `openspec/changes/diri-inspector-artifacts/` | inspector screenshot + diff fixture tests |
| M04-F002 | M04 | Artifact/port/PR/browser preview cards | `ArtifactScanner.swift`, `BrowserPool.swift`, `PortForwarder.swift`, `PullRequestMonitor.swift` | 产物、预览、端口、PR 卡片可见并可打开 | `homie-runtime`, `homie-app`, `homie-cli` | `specs/mcp-automation/README.md` | `prd-spec/features/diri-inspector-artifacts/` | `openspec/changes/diri-inspector-artifacts/` | artifact/browser/port/PR E2E |
| M05-F001 | M05 | Command palette and Quick Open | `palette.rs`, `quick_open.rs`, `fuzzy.rs` | Cmd+P/Cmd+K 能打开 actions/session/file 搜索 | `homie-app`, `homie-ui` | `specs/desktop-shell/README.md` | `prd-spec/features/diri-navigation-history/` | `openspec/changes/diri-navigation-history/` | palette/quick-open interaction E2E |
| M05-F002 | M05 | History scan and resume | `history.rs`, `HistoryScanner.swift`, `CodexTranscript.swift`, `CursorChatStore.swift` | 可从历史恢复 agent session | `homie-runtime`, `homie-storage`, `homie-app` | `specs/session-context-store/README.md` | `prd-spec/features/diri-navigation-history/` | `openspec/changes/diri-navigation-history/` | transcript fixture + resume E2E |
| M06-F001 | M06 | Settings tabs and persisted preferences | `settings.rs`, `PrefsSync.swift`, `ResourceGovernorSettingsTests.swift` | 设置 General/Terminal/Resources/Remote 可保存 | `homie-app`, `homie-storage` | `specs/desktop-shell/README.md`, `specs/storage-indexing/README.md` | `prd-spec/features/diri-settings-preferences/` | `openspec/changes/diri-settings-preferences/` | settings persistence + screenshot E2E |
| M06-F002 | M06 | Remote settings and companion access | `remote_access.rs`, `RemoteConfig.swift`, `RemoteAccessTests.swift` | 可配置 remote companion/pairing/token path | `homie-app`, `homie-remote`, `homie-storage` | `specs/remote-node-handoff/README.md` | `prd-spec/features/diri-settings-preferences/` | `openspec/changes/diri-settings-preferences/` | remote settings E2E without token leak |
| M07-F001 | M07 | Repo locate and worktree overview | `RepoLocator.swift`, `GitWorktrees.swift`, `WorktreeDetectionTests.swift` | 当前 repo/worktrees 可发现并显示 | `homie-runtime`, `homie-client`, `homie-app` | `specs/runtime-supervisor/README.md` | `prd-spec/features/diri-worktrees-git/` | `openspec/changes/diri-worktrees-git/` | git fixture tests |
| M07-F002 | M07 | Worktree create/remove/cleanup | `WorktreeCommands.swift`, `worktrees.rs` | 可创建、删除、清理 worktree | `homie-cli`, `homie-runtime`, `homie-storage` | `specs/mcp-automation/README.md` | `prd-spec/features/diri-worktrees-git/` | `openspec/changes/diri-worktrees-git/` | real git worktree E2E |
| M08-F001 | M08 | Notification rollup and needs-input modal | `notifications.rs`, `HookParsing.swift`, `Reducer.swift` | 需要用户操作时有通知/弹层 | `homie-app`, `homie-ui`, `homie-agents` | `specs/desktop-shell/README.md`, `specs/agent-adapter-contract/README.md` | `prd-spec/features/diri-notifications-menubar/` | `openspec/changes/diri-notifications-menubar/` | hook fixture + app modal E2E |
| M08-F002 | M08 | Native notification/menu bar/sounds/quick approve-deny | `sounds.rs`, macOS notification bridge | 系统通知、菜单栏、声音、快速批准/拒绝 | `homie-app`, macOS bridge, `homie-runtime` | `specs/desktop-shell/README.md` | `prd-spec/features/diri-notifications-menubar/` | `openspec/changes/diri-notifications-menubar/` | macOS GUI/native notification E2E |
| M09-F001 | M09 | Design tokens/brand/icons/status glyphs | `diri-ui/src/*`, `diri-ui/assets/*`, `gallery.rs` | UI 使用统一 Diri token/glyph/icon/brand | `homie-ui`, `homie-app`, `assets` | `specs/desktop-shell/README.md` | `prd-spec/features/diri-design-system/` | `openspec/changes/diri-design-system/` | token tests + icon/gallery screenshot gate |
| M10-F001 | M10 | Protocol method/event/DTO catalog | `diri-proto/src/methods.rs`, `control.rs`, `DirijorProtocol/*`, `WireTests.swift` | client/CLI/MCP 使用稳定 wire contract | `homie-proto`, `homie-client`, `homie-cli` | `specs/runtime-supervisor/README.md` | `prd-spec/features/diri-protocol-contract/` | `openspec/changes/diri-protocol-contract/` | method-by-method contract fixtures |
| M10-F002 | M10 | Grid/scrollback/remote/host wire fixtures | `grid.rs`, `frames.rs`, `remote.rs`, `hosts.rs` | terminal/remote 数据跨进程正确传输 | `homie-proto`, `homie-term`, `homie-remote` | `specs/remote-node-handoff/README.md` | `prd-spec/features/diri-protocol-contract/` | `openspec/changes/diri-protocol-contract/` | grid/scrollback/hosts roundtrip tests |
| M11-F001 | M11 | Client connect/reconnect/event resume | `diri-client/src/connection.rs`, `state.rs`, `EventSubscriptionTests.swift` | app/CLI 断线后能恢复事件 | `homie-client` | `specs/runtime-supervisor/README.md` | `prd-spec/features/diri-client-attachment/` | `openspec/changes/diri-client-attachment/` | reconnect/resume integration tests |
| M11-F002 | M11 | Session attachment read/write/resize/scrollback/diff/artifacts | `attachment.rs`, `client.rs` | app/CLI 通过 client 操作 session | `homie-client`, `homie-runtime` | `specs/runtime-supervisor/README.md` | `prd-spec/features/diri-client-attachment/` | `openspec/changes/diri-client-attachment/` | client runtime E2E |
| M12-F001 | M12 | CLI session/events/worktree/ports commands | `SessionCommands.swift`, `EventCommands.swift`, `WorktreeCommands.swift`, `Ports.swift` | `homie` CLI 覆盖 Diri CLI 操作 | `homie-cli`, `homie-client` | `specs/mcp-automation/README.md` | `prd-spec/features/diri-cli-automation/` | `openspec/changes/diri-cli-automation/` | command grammar + real runtime CLI E2E |
| M12-F002 | M12 | CLI hook/notify/forward/MCP bridge/lineage | `Forward.swift`, `MCPBridge.swift`, `MCPLineage*.swift` | agent 可通过 CLI/MCP 操作 Homie | `homie-cli`, `homie-orchestrator`, `homie-storage` | `specs/mcp-automation/README.md` | `prd-spec/features/diri-cli-automation/` | `openspec/changes/diri-cli-automation/` | MCP stdio + lineage E2E |
| M13-F001 | M13 | MCP server tool surface | `dirijor-mcp/src/main.rs`, `DirijorMCP/Tools.swift` | MCP tools 可 spawn/list/wait/send/read/release | `homie-cli`, `homie-orchestrator`, `homie-runtime` | `specs/mcp-automation/README.md` | `prd-spec/features/diri-mcp-automation/` | `openspec/changes/diri-mcp-automation/` | MCP protocol E2E |
| M13-F002 | M13 | MCP lineage/parent-child permission scope | `MCPLineage.swift`, `MCPLineageTools.swift` | 子 agent 身份、父子 session、权限边界正确 | `homie-orchestrator`, `homie-storage`, `homie-cli` | `specs/mcp-automation/README.md` | `prd-spec/features/diri-mcp-automation/` | `openspec/changes/diri-mcp-automation/` | lineage fixture tests |
| M14-F001 | M14 | Session lifecycle PTY/log/screen/registry | `session.rs`, `pty.rs`, `log.rs`, `screen.rs`, `registry.rs`, `SessionIntegrationTests.swift` | runtime 能 spawn/input/output/restore | `homie-runtime`, `homie-storage` | `specs/runtime-supervisor/README.md` | `prd-spec/features/diri-runtime-supervisor/` | `openspec/changes/diri-runtime-supervisor/` | real PTY lifecycle tests |
| M14-F002 | M14 | Status/resource/checkpoint/migration/event bus | `StatusEngine.swift`, `ResourceGovernor.swift`, `ScreenCheckpoint.swift`, `SessionMigrator.swift`, `EventBus.swift` | 状态、防抖、资源、恢复、事件一致 | `homie-runtime`, `homie-agents` | `specs/runtime-supervisor/README.md` | `prd-spec/features/diri-runtime-supervisor/` | `openspec/changes/diri-runtime-supervisor/` | crash/recovery/resource/status tests |
| M15-F001 | M15 | Holder PTY server/client/manager | `DirijorHolderKit/*`, `dirijord-holder/main.swift`, `HolderTests.swift` | app/runtime 崩溃后 PTY 存活并可 adopt | `homie-runtime` | `specs/runtime-supervisor/README.md` | `prd-spec/features/diri-holder-process-tree/` | `openspec/changes/diri-holder-process-tree/` | holder crash/adoption tests |
| M15-F002 | M15 | Process tree kill/sampling | `HolderProcessTree.swift`, `ProcessTree.swift`, `ProcessTreeTests.swift` | terminate 能安全清理进程树并采样资源 | `homie-runtime` | `specs/runtime-supervisor/README.md` | `prd-spec/features/diri-holder-process-tree/` | `openspec/changes/diri-holder-process-tree/` | process-tree fixture tests |
| M16-F001 | M16 | Agent catalog/manifest/readiness | `AgentCatalog.swift`, `AgentDescriptor.swift`, `AgentReadiness.swift` | agent descriptor/approve/deny/resume/env 可用 | `homie-agents`, `homie-storage`, assets | `specs/agent-adapter-contract/README.md` | `prd-spec/features/diri-agent-detection/` | `openspec/changes/diri-agent-detection/` | manifest catalog + readiness tests |
| M16-F002 | M16 | Detection regions/predicates/reducer/golden screens | `DirijorDetection/*`, `GoldenScreenTests.swift`, `ReducerTests.swift` | status/needs-input/risk 检测与 Diri golden 一致 | `homie-agents`, `homie-runtime` | `specs/agent-adapter-contract/README.md` | `prd-spec/features/diri-agent-detection/` | `openspec/changes/diri-agent-detection/` | golden screen tests |
| M17-F001 | M17 | Core identifiers/session/needs-input/attention models | `DirijorCore/*`, `CoreTests.swift` | 所有模块共享同一核心语义 | `homie-proto`, `homie-ui`, `homie-agents` | `specs/storage-indexing/README.md` | `prd-spec/features/diri-core-models/` | `openspec/changes/diri-core-models/` | model serde/contract tests |
| M18-F001 | M18 | Remote node server/accounts/spawn/checkpoint | `diri-node/src/*`, `RemoteSpawnTests.swift` | remote node 可启动/恢复 session | `homie-remote`, `homie-runtime`, `homie-client` | `specs/remote-node-handoff/README.md` | `prd-spec/features/diri-remote-node-handoff/` | `openspec/changes/diri-remote-node-handoff/` | local node E2E |
| M18-F002 | M18 | Host prefs sync/repo locate/companion access | `PrefsSync.swift`, `RemoteConfig.swift`, `RemoteAccessTests.swift` | host 偏好同步、repo 定位、companion 配置可用 | `homie-remote`, `homie-app`, `homie-storage` | `specs/remote-node-handoff/README.md` | `prd-spec/features/diri-remote-node-handoff/` | `openspec/changes/diri-remote-node-handoff/` | host protocol tests |
| M19-F001 | M19 | Usage parsing/pricing/token accounting | `diri-usage/src/lib.rs`, `diri-node/src/usage.rs` | usage/cost/token/cache/latency 统计准确 | `homie-llm`, `homie-storage` | `specs/llm-proxy/README.md` | `prd-spec/features/diri-usage-accounting/` | `openspec/changes/diri-usage-accounting/` | usage fixture tests |
| M19-F002 | M19 | Usage UI/fleet projection/transcript watcher | `diri-app/src/usage/*` | app 显示本机/远端 usage | `homie-app`, `homie-remote`, `homie-storage` | `specs/observability/README.md` | `prd-spec/features/diri-usage-accounting/` | `openspec/changes/diri-usage-accounting/` | usage UI E2E |
| M20-F001 | M20 | Updater feed/trust/install/rollback | `diri-updater/src/*` | app 可检查/安装/回滚更新 | `homie-updater`, `homie-app` | `specs/packaging-updater/README.md` | `prd-spec/features/diri-release-updater-packaging/` | `openspec/changes/diri-release-updater-packaging/` | updater trust/install tests |
| M20-F002 | M20 | Packaging/dmg/notarization/perf gates | `scripts/package.sh`, `install-local.sh`, `perf-gate.sh` | release artifact 可安装、签名、性能达标 | `scripts/package`, `Makefile` | `specs/packaging-updater/README.md` | `prd-spec/features/diri-release-updater-packaging/` | `openspec/changes/diri-release-updater-packaging/` | packaged launch + dmg + perf gate |

## 8. Diri Test Coverage Matrix

| Diri test suite | Covered modules | Required Homie verification |
|-----------------|-----------------|-----------------------------|
| `DirijorCoreTests/*` | M16, M17 | `homie-proto`/`homie-agents` model/catalog tests |
| `DirijorProtocolTests/*` | M10 | protocol DTO/grid/control/hosts fixtures |
| `DirijorDetectionTests/*` | M16, M14 | manifest schema, region, reducer, golden screen tests |
| `DirijorDaemonKitTests/SessionIntegrationTests.swift` | M14 | real PTY lifecycle E2E |
| `DirijorDaemonKitTests/OutputLogTests.swift` | M14 | output offset replay tests |
| `DirijorDaemonKitTests/HeadlessScreenTests.swift` | M03, M14 | headless terminal parser tests |
| `DirijorDaemonKitTests/EventSubscriptionTests.swift` | M10, M11, M14 | event subscribe/resume integration tests |
| `DirijorDaemonKitTests/HolderTests.swift` | M15 | holder adoption/crash tests |
| `DirijorDaemonKitTests/ProcessTreeTests.swift` | M15 | process-tree sampling/kill tests |
| `DirijorDaemonKitTests/ResourceGovernorSettingsTests.swift` | M06, M14 | resource preference + hibernate tests |
| `DirijorDaemonKitTests/ScreenCheckpointTests.swift` | M14 | checkpoint write/read/migration tests |
| `DirijorDaemonKitTests/HistoryScannerTests.swift` | M05 | transcript/history scanner fixtures |
| `DirijorDaemonKitTests/TitleWatcherTests.swift` | M05 | title extraction tests |
| `DirijorDaemonKitTests/ArtifactScannerTests.swift` | M04 | artifact scanner tests |
| `DirijorDaemonKitTests/BrowserPoolTests.swift` | M04 | browser preview/pool tests |
| `DirijorDaemonKitTests/PortForwardTests.swift` | M04, M12 | port forwarding/listing E2E |
| `DirijorDaemonKitTests/PullRequestMonitorTests.swift` | M04 | PR monitor fixture tests |
| `DirijorDaemonKitTests/WorktreeDetectionTests.swift` | M07 | repo/worktree detection tests |
| `DirijorDaemonKitTests/WorktreeDiffTests.swift` | M04, M07 | diff loader fixture tests |
| `DirijorDaemonKitTests/RemoteAccessTests.swift` | M06, M18 | remote access config tests |
| `DirijorDaemonKitTests/RemoteSpawnTests.swift` | M18 | remote node spawn E2E |
| `DirijorDaemonKitTests/PrefsSyncTests.swift` | M06, M18 | host preference sync tests |
| `DirijorCLITests/CommandGrammarTests.swift` | M12, M13 | CLI grammar and MCP bridge tests |

## 9. Dependency Matrix

| Module | Blocked by | Blocks | Required contracts |
|--------|------------|--------|--------------------|
| M17 Core Models | none | M10, M14, M16, UI modules | model serde, IDs, status, needs-input |
| M10 Protocol | M17 | M11, M12, M13, M18 | method/event/DTO/grid/host/remote |
| M14 Runtime | M10, M16, M17 | M01, M03, M04, M07, M11, M12 | session lifecycle, event bus, storage |
| M15 Holder | M14 | M14, M03 | holder protocol, PTY, process tree |
| M16 Agent Detection | M17 | M14, M08, M12 | manifest/reducer/hook/risk |
| M11 Client | M10, M14 | M01, M03, M04, M07, M12, M13 | attach/read/send/resize/events |
| M03 Terminal | M10, M11, M14 | M01 | grid/input/scrollback/find |
| M01 Workbench | M11, M03 | M02..M09 | shell state, keyboard, overlays |
| M02 Sidebar | M01, M11, M17 | M05, M08 | session projection, status glyph |
| M04 Inspector | M11, M14, M07 | M01 | artifacts, diff, PR, ports |
| M05 Navigation | M01, M11, M14 | M12 | quick open, history, title |
| M06 Settings | M17 | M18, M20, M14 | preferences schema |
| M07 Worktrees/Git | M14, M12 | M04, M01 | repo/worktree APIs |
| M08 Notifications | M16, M14, M01 | M12 | needs-input, native action safety |
| M09 Design System | none | all UI modules | tokens/icons/glyphs |
| M12 CLI | M10, M11, M14 | M13, automation | command grammar, output schema |
| M13 MCP | M10, M11, M12, M17 | automation flows | stdio tools, lineage |
| M18 Remote | M10, M11, M14, M06 | M19, M12 | node protocol, account config |
| M19 Usage | M14, M18 | M01, M06 | usage records, pricing |
| M20 Release | all runtime/app basics | user distribution | package/update/perf gates |

## 10. Verification Environment Matrix

| Module | Unit | Integration | Real PTY | macOS GUI | Packaged app | Network/remote | Screenshot/E2E |
|--------|------|-------------|----------|-----------|--------------|----------------|----------------|
| M01 | yes | yes | yes | yes | yes | no | yes |
| M02 | yes | yes | no | yes | yes | no | yes |
| M03 | yes | yes | yes | yes | yes | no | yes |
| M04 | yes | yes | optional | yes | yes | optional | yes |
| M05 | yes | yes | optional | yes | yes | no | yes |
| M06 | yes | yes | no | yes | yes | remote for Remote tab | yes |
| M07 | yes | yes | no | optional | optional | no | optional |
| M08 | yes | yes | optional | yes | yes | no | yes |
| M09 | yes | no | no | yes | yes | no | yes |
| M10 | yes | yes | no | no | no | remote fixtures | no |
| M11 | yes | yes | yes | no | optional | remote fixtures | no |
| M12 | yes | yes | yes | no | packaged CLI | optional | no |
| M13 | yes | yes | optional | no | optional | optional | no |
| M14 | yes | yes | yes | no | optional | optional | no |
| M15 | yes | yes | yes | no | yes | no | no |
| M16 | yes | yes | optional | no | optional | no | golden screen |
| M17 | yes | yes | no | no | no | no | no |
| M18 | yes | yes | optional | no | optional | yes | no |
| M19 | yes | yes | no | yes | optional | yes | yes |
| M20 | yes | yes | no | yes | yes | yes | yes |

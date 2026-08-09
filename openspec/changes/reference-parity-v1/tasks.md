# Homie Reference Parity V1 OpenSpec Tasks

> Change ID: `reference-parity-v1`  
> Source PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`  
> Beads: `homie-h7n`

## Task Status

| Status | Meaning |
|--------|---------|
| todo | Not started |
| red | Failing test or contract written |
| green | Implementation passes focused verification |
| refactor | Cleanup while tests stay green |
| done | Task evidence recorded and accepted |

## Task To Functional Case Mapping

| OpenSpec task | Functional cases | Gate meaning |
|---------------|------------------|--------------|
| T-001 | FC-001, FC-002, FC-003, FC-004 | Reference coverage, naming, component spec entry gates |
| T-002 | FC-003, FC-006 | Protocol and client contract parity |
| T-003 | FC-005 | Agent catalog and status detection parity |
| T-004 | FC-007, FC-018 | Runtime lifecycle, recovery, resource supervisor |
| T-005 | FC-004, FC-007, FC-018 | Storage schema, migrations, preferences, output index |
| T-006 | FC-008 | Terminal grid, input, scrollback, selection, find |
| T-007 | FC-009 | GPUI design system, tokens, glyphs, core surfaces |
| T-008 | FC-009, FC-012 | Sidebar, workbench, terminal pane, inspector, artifact surfaces |
| T-009 | FC-009, FC-011 | Navigation surfaces and history resume |
| T-010 | FC-010 | Worktree and project safety |
| T-011 | FC-012 | Artifact, port, PR, browser, test_run |
| T-012 | FC-013, FC-018 | LLM proxy, virtual key, usage, metrics, no-leak gates |
| T-013 | FC-014 | CLI, hook/notify, MCP automation |
| T-014 | FC-015 | Remote host, node, account, companion access, handoff |
| T-015 | FC-016 | Context, memory, task, intent orchestration |
| T-016 | FC-017, FC-018 | Packaging, updater, release, packaged perf |
| T-017 | FC-001, FC-013, FC-015, FC-017, FC-018 | Security gauntlet and release readiness |

## Tasks

### T-001: Reference coverage matrix and component spec impact

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-1, FR-2 |
| Component spec | `specs/README.md`, all affected specs |
| Beads | `homie-h7n` |
| Files | `docs/research/reference-feature-coverage.md`, `specs/*/README.md`, `docs/verification/reference-parity-v1/*` |

Objective:

- Convert every Reference product capability into a tracked Homie component/spec/test owner.

RED:

- Add a coverage check that fails when any row is `missing` or `partial` without owner and follow-up.

GREEN:

- Update the coverage matrix and affected component specs.

Acceptance:

- Coverage report has no unowned Reference gap.
- Component spec impact report is recorded.

Evidence:

- `docs/verification/reference-parity-v1/spec-review-report.md`

### T-002: Protocol, event, and client contract parity

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-4, FR-5 |
| Component spec | `specs/runtime-supervisor/README.md`, `specs/session-context-store/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-proto`, `crates/homie-client`, `tests/fixtures/protocol` |

Objective:

- Implement Reference method/event parity plus Homie LLM/profile/task/memory protocol extensions.

RED:

- Add protocol fixtures for request/response/event roundtrip, unknown enum decode, event resume, frame/grid decode.

GREEN:

- Implement typed protocol DTOs, event seq subscription, error envelope, and client reconnect behavior.

Acceptance:

- Protocol tests cover all FR-5 methods/events and safe error redaction.

### T-003: Agent catalog, manifest schema, and status detection parity

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-3 |
| Component spec | `specs/agent-adapter-contract/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-agents`, `assets/agent-descriptors`, `tests/fixtures/agents` |

Objective:

- Load and validate Reference's current 19-agent catalog with status authority, resume, approval, and risk rules.

RED:

- Add manifest schema tests and golden screen fixtures for first-class agents.

GREEN:

- Implement manifest loader, status reducer, approval/resume descriptor, and readiness API.

Acceptance:

- No first-class Reference agent falls back to process-only status unless explicitly marked unavailable with evidence.

### T-004: Runtime supervisor, PTY, output log, holder-equivalent

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-4, FR-15 |
| Component spec | `specs/runtime-supervisor/README.md`, `specs/observability/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-runtime`, `crates/homie-storage`, `tests/e2e/runtime` |

Objective:

- Own PTY/process/session lifecycle, offset-addressed output log, headless screen, resource governor, and recovery.

RED:

- Add tests for spawn/input/output/resize/kill/archive/hibernate/restart/output replay.

GREEN:

- Implement runtime supervisor and holder-equivalent process/PTY ownership.

Acceptance:

- App exit does not kill sessions; runtime restart preserves readable session state/output.

### T-005: Storage schema, migration, preferences, and indexing

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-18 |
| Component spec | `specs/storage-indexing/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-storage`, `migrations/sqlite`, `tests/fixtures/storage` |

Objective:

- Store Reference parity product state and Homie LLM/context/task/memory state in SQLite with forward-only migrations.

RED:

- Add migration and repository tests for sessions/projects/worktrees/artifacts/usage/preferences/virtual keys/output index.

GREEN:

- Implement schema, repository APIs, and owner-only config boundaries.

Acceptance:

- Empty and existing database migration tests pass; high-volume output bytes stay outside SQLite blob.

### T-006: Terminal grid, input, scrollback, selection, and find

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-6 |
| Component spec | `specs/desktop-shell/README.md`, `specs/runtime-supervisor/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-term`, `tests/fixtures/terminal` |

Objective:

- Implement terminal rendering and interaction parity with Reference.

RED:

- Add grid fixture, input encoding, scrollback, selection, find, and resize tests.

GREEN:

- Implement grid buffer, TerminalElement, input encoder, scrollback cache, selection, find model, repaint pacing.

Acceptance:

- Real PTY smoke covers fish/vim/agent TUI, 1000-line scrollback, and resize without line loss.

### T-007: GPUI design system and core UI surfaces

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-7 |
| Component spec | `specs/desktop-shell/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-ui`, `crates/homie-app`, `assets/icons`, `tests/ui` |

Objective:

- Recreate Reference tokens, brand marks, status glyphs, window chrome, floating surface recipe, and keyboard map.

RED:

- Add deterministic gallery/screenshot fixtures for all token/glyph/surface states.

GREEN:

- Implement tokens, icons, brand marks, status glyphs, floating surfaces, key bindings, and preview harness.

Acceptance:

- Empty/typical/stress screenshots pass fidelity review; min/narrow windows show no overlap.

### T-008: Sidebar, workbench, terminal pane, inspector

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-7, FR-10 |
| Component spec | `specs/desktop-shell/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-app/src/sidebar`, `terminal_pane`, `workbench`, `inspector` |

Objective:

- Implement Reference's main daily workbench: leading sidebar, terminal pane, right inspector, workbench split.

RED:

- Add store projection tests for ordering, selection, pin/archive, drag reorder, resident panes, inspector tabs.

GREEN:

- Implement sidebar interactions, terminal chips/overlays, workbench split, inspector Info/Changes/Artifacts.

Acceptance:

- Full app smoke covers create session, select, rename, pin, archive, inspect diff/artifacts.

### T-009: Navigation surfaces, history, quick open, overview, switcher

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-7, FR-9 |
| Component spec | `specs/desktop-shell/README.md`, `specs/session-context-store/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-app/src/navigation`, `history`, `quick_open`, `switcher` |

Objective:

- Implement command palette, quick open, Ctrl-Tab switcher, overview board/list, and history resume.

RED:

- Add fuzzy ranking fixture tests and transcript history fixture tests.

GREEN:

- Implement surfaces, cache-warmed directory index, history scanner, resume-from-history.

Acceptance:

- Real Claude/Codex history resume path is verified.

### T-010: Worktree and project management

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-8 |
| Component spec | `specs/session-context-store/README.md`, `specs/runtime-supervisor/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-runtime`, `crates/homie-storage`, `crates/homie-app/src/worktrees` |

Objective:

- Implement project/worktree operations, safe cleanup, and session/worktree binding.

RED:

- Add git fixture tests for create/list/remove/overview and unsafe cleanup rejection.

GREEN:

- Implement worktree repository/runtime/UI/MCP/CLI paths.

Acceptance:

- Dirty/unmerged/main worktrees cannot be removed by suggestion flow.

### T-011: Artifact, port, PR monitor, browser pool, and test_run

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-10 |
| Component spec | `specs/mcp-automation/README.md`, `specs/observability/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-runtime`, `crates/homie-app`, `tests/e2e/browser` |

Objective:

- Capture and surface PR/issue/link/preview/port artifacts and provide browser/test automation.

RED:

- Add artifact parser fixtures, PR status fixtures, browser/test_run contract tests.

GREEN:

- Implement scanners, PR monitor, toolbar/inspector chips, MCP get_artifacts, browser/test_run.

Acceptance:

- test_run returns per-engine structured results and screenshot file paths only on failure.

### T-012: LLM proxy, virtual key, usage, and metrics

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-11, FR-12 |
| Component spec | `specs/llm-proxy/README.md`, `specs/virtual-key-credentials/README.md`, `specs/observability/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-llm`, `crates/homie-storage`, `crates/homie-app/src/usage` |

Objective:

- Preserve Homie's LLM custody while matching Reference usage UX.

RED:

- Add fake provider streaming/failure tests, virtual key scope/revoke tests, no-leak regression tests, usage fixture tests.

GREEN:

- Implement proxy, provider routing, metrics, pricing snapshot, transcript/proxy/node usage aggregation.

Acceptance:

- Managed agent receives no real provider key; usage UI matches fixture totals.

### T-013: CLI, hook/notify, MCP automation

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-14 |
| Component spec | `specs/mcp-automation/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-cli`, `crates/homie-runtime`, `tests/e2e/mcp` |

Objective:

- Provide Reference-equivalent shell and MCP automation surface.

RED:

- Add CLI grammar tests, hook/notify fail-open tests, MCP tool contract tests.

GREEN:

- Implement session/worktree/artifacts/events/ports/hook/notify/mcp commands and tools.

Acceptance:

- Real local MCP orchestration E2E spawns, waits, reads output, and cleans up agent sessions.

### T-014: Remote hosts, node, accounts, companion access, handoff

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-13 |
| Component spec | `specs/remote-node-handoff/README.md`, `specs/virtual-key-credentials/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-runtime`, `crates/homie-client`, `crates/homie-cli`, `crates/homie-app/src/settings` |

Objective:

- Implement remote host/node parity with Homie credential rules.

RED:

- Add host config validation tests, node auth/capability fixtures, handoff dry-run failure tests.

GREEN:

- Implement host catalog, remote spawn, prefs sync, locate repo, node account, move/fork, companion access.

Acceptance:

- Local loopback node harness proves spawn, account status, usage merge, and move/fork handoff.

### T-015: Context, memory, task, and intent orchestration

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-20 |
| Component spec | `specs/session-context-store/README.md`, `specs/memory-controller/README.md`, `specs/task-controller/README.md`, `specs/intent-orchestrator/README.md` |
| Beads | `homie-h7n` |
| Files | `crates/homie-context`, `crates/homie-memory`, `crates/homie-task`, `crates/homie-runtime` |

Objective:

- Attach Homie's own context/memory/task/orchestration to Reference-equivalent surfaces.

RED:

- Add contract tests for session summaries, task claim/update/return, memory write candidate redaction, intent route decisions.

GREEN:

- Implement minimal controllers and UI/MCP/runtime integration.

Acceptance:

- Agent sessions can be linked to task state and safe context summaries without leaking raw secrets.

### T-016: Packaging, updater, release, and performance gates

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-16, FR-17, FR-19 |
| Component spec | `specs/packaging-updater/README.md`, `specs/observability/README.md` |
| Beads | `homie-h7n` |
| Files | `scripts/package`, `scripts/release`, `crates/homie-updater`, `docs/verification/reference-parity-v1` |

Objective:

- Ship Homie parity as a signed/notarized app with safe updater and packaged perf gate.

RED:

- Add updater trust decision tests, package manifest tests, perf-gate script dry-run tests.

GREEN:

- Implement app bundle packaging, DMG/update zip, updater helper, release script, deterministic packaged perf gate.

Acceptance:

- Old build updates to new build; normal/large packaged perf gate passes; release readiness report is complete.

### T-017: Security gauntlet and release readiness

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-19 |
| Component spec | all security-sensitive specs |
| Beads | `homie-h7n` |
| Files | `.githooks`, `Makefile`, `docs/verification/reference-parity-v1` |

Objective:

- Prove V1 parity does not regress Homie security baseline.

RED:

- Add no-leak tests and capability-diff checks for credential, log, event, metric, artifact, browser, remote, updater paths.

GREEN:

- Run and document full local gauntlet.

Acceptance:

- Release readiness report has no `not_run` security gate without explicit blocking reason.


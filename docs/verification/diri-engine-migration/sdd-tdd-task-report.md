# diri-engine-migration SDD/TDD Task Report

```yaml
change_id: diri-engine-migration
report_type: sdd-tdd-task-report
status: pass
beads: homie-cj5
source_prd: prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md
openspec_tasks: openspec/changes/diri-engine-migration/tasks.md
```

## T-001: OpenSpec and component spec state correction

Status: done

Changes:

- Replaced `openspec/changes/diri-engine-migration/plan.md` with gap-closure in-progress plan.
- Added `openspec/changes/diri-engine-migration/tasks.md`.
- Added `openspec/changes/diri-engine-migration/alignment-report.md`.
- Added `docs/verification/diri-engine-migration/functional-cases.md`.
- Updated impacted component specs: runtime supervisor, agent adapter contract, desktop shell, session context store, observability.

Evidence:

- `rg -n "状态: ✅ 完成|Status: complete|⏭️" openspec/changes/diri-engine-migration docs/verification/diri-engine-migration` only matches the functional-case command text itself.

## T-002: Runtime live PTY supervisor

Status: done

RED:

- `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` initially failed because `spawn_shell` returned `created`, invalid cwd persisted a session, and reopened `send_text` wrote to output log.

GREEN:

- `RuntimeSupervisor` now keeps a live session registry backed by holder IPC.
- `spawn_shell` validates cwd and `/bin/sh`, launches `homie-runtime-holder`, and marks the session `running`.
- `homie-runtime-holder` owns the PTY, writes output to `runtime/output/<session>.log`, and serves write/stat/terminate requests over a short Unix socket path under `/tmp/homie-runtime-holders`.
- `send_text` writes through holder IPC and returns `SessionNotLive` for missing holders.
- `read_output_range` provides offset-addressed replay over the holder-produced output log.
- `session_status_report` feeds the holder-produced output log into `HeadlessScreen`, classifies screen observations, and runs `homie-agents::StatusReducer` to project `running`, `needs_input`, `idle`, and `exited`.
- Reopen adopts live holders, restores normal holder exits as `exited`, and marks missing holder evidence as `detached`.
- `terminate_session` sends a holder terminate request, waits for socket/pid cleanup, and persists `exited`.
- Holder termination now uses process-tree enumeration and group signaling so detached child processes do not survive `terminate_session`.
- Holder `Stat` now exposes geometry, tree size, epoch offset, and current log offset; `RuntimeSupervisor::resize_session` updates holder geometry through `Resize` IPC.
- `session_snapshot` now combines SQLite session metadata, holder stat, runtime status report, and offset-addressed replay after supervisor reopen.
- `homie session snapshot` now exposes runtime snapshot JSON through the CLI for attach/resume smoke coverage.
- `write_screen_checkpoint` and `read_screen_checkpoint` persist and restore headless screen checkpoint data across supervisor reopen.
- `archive` and `hibernate` now stop holder processes, while `wake` restarts holder-owned PTY sessions before marking them running.

Evidence:

- `cargo test -p homie-runtime --test session_lifecycle -- --nocapture`: pass, 12 tests.
- Covered cases: live PTY output, offset replay, screen lines, spawn failure cleanup, holder adoption after supervisor drop, explicit terminate cleanup, exited restore, detached recovery, runtime headless screen -> reducer status projection, detached child process-tree termination, holder stat/resize/log offset metadata, reopen attach snapshot composition, screen checkpoint persistence, and hibernate/wake resource lifecycle.

## T-003: Agent status reducer and hook parser

Status: done

RED:

- `cargo test -p homie-agents --tests -- --nocapture` initially failed due to missing `StatusReducer`, `StatusSignal`, `HookEvent`, `NotifyEvent`, `parse_claude_hook`, and `parse_codex_notify`.

GREEN:

- Added `homie_agents::status` with hooks-primary, screen-primary, process-only reducer behavior.
- Added `homie_agents::hooks` with stable hook/notify events and structured payload redaction.
- Wired `homie-runtime` screen observations into `homie-agents::StatusReducer` via `RuntimeSupervisor::session_status_report`.
- Wired `homie-cli hook` and `homie-cli notify` to `homie-agents` parsers so automation entrypoints emit structured redacted JSON.

Evidence:

- `cargo test -p homie-agents status_reducer -- --nocapture`: pass.
- `cargo test -p homie-agents hook_parser -- --nocapture`: pass.
- `cargo test -p homie-runtime --test session_lifecycle -- --nocapture`: pass, includes `runtime_status_report_uses_headless_screen_and_reducer`.
- `cargo test -p homie-cli --tests -- --nocapture`: pass, includes hook/notify parser entry tests.
- `cargo test -p homie-cli --test session_snapshot_cli -- --nocapture`: pass.
- `cargo run -p homie-cli -- hook PermissionRequest ...`: pass, redacted structured JSON output.
- `cargo run -p homie-cli -- notify ...`: pass, Codex turn-complete structured JSON output.

## T-004: Terminal scrollback real model

Status: done

RED:

- Added `crates/homie-term/tests/scrollback.rs` covering fetch request, fetch result validation, alt-screen reset, and wheel routing.

GREEN:

- Replaced empty `ReadScrollbackCellsResult` stub with a real result struct.
- Implemented geometry, cache, in-flight/queued fetch range, content-sequence cache invalidation, row-count validation, and mode-aware wheel routing.

Evidence:

- `cargo test -p homie-term --test scrollback -- --nocapture`: pass.

## T-005: Diri design token parity

Status: done

RED:

- Expanded token tests to cover typography, toolbar metrics, spring motion, semantic colors, fill, space, and memory formatting.

GREEN:

- Added dependency-free token structs/enums to `homie-ui`: `Typo`, `TypeStyle`, `FontWeightToken`, `SemanticColors`, `RgbaToken`, `Spring`, `Fill`, `Space`, and `MemoryFormat`.

Evidence:

- `cargo test -p homie-ui --test tokens -- --nocapture`: pass.

## T-006: Diri-style app preview shell without placeholders

Status: done

RED:

- Added `crates/homie-app/tests/app_shell_copy_regression.rs`, which initially failed on `Next implementation slices`.

GREEN:

- Replaced roadmap cards with a Diri-style preview shell: sidebar rows, terminal preview, inspector tabs, footer status.
- Added palette model metadata regression test.
- Kept live session operations preview-only until client/protocol wiring exists.

Evidence:

- `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture`: pass.
- `cargo check -p homie-app`: pass with no errors.

## Workspace Gates

- `cargo fmt --all -- --check`: pass after `cargo fmt --all`.
- `cargo test --workspace`: pass before the holder cleanup/status follow-up.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass before the holder cleanup/status follow-up.
- Follow-up focused gates after holder cleanup/status changes:
  - `cargo fmt --all -- --check`: pass.
  - `cargo test -p homie-runtime --test session_lifecycle -- --nocapture`: pass, 12 tests.
  - `cargo clippy -p homie-runtime -p homie-agents -p homie-proto -p homie-storage --all-targets -- -D warnings`: pass.
  - `cargo test -p homie-storage --tests -- --nocapture`: pass.

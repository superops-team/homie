# diri-engine-migration Code Review Report

```yaml
change_id: diri-engine-migration
report_type: two-round-code-review
status: pass
beads: homie-cj5
```

## Round 1: Explicit Correctness Review

Scope reviewed:

- `crates/homie-runtime`: live PTY registry, shell spawn, PTY input, output pump, failure paths.
- `crates/homie-agents`: status reducer, hook parser, redaction.
- `crates/homie-term`: scrollback result model, cache, geometry, wheel routing.
- `crates/homie-ui`: Diri-aligned token additions.
- `crates/homie-app`: Diri-style preview shell and placeholder-copy regression.
- OpenSpec, PRD, component specs, and verification docs.

Findings and actions:

| Severity | Finding | Action | Status |
|----------|---------|--------|--------|
| P1 | Runtime tests proved old fake-session behavior; implementation now launches `/bin/sh` through `homie-runtime-holder` and fails closed for missing holders | Added holder IPC registry, holder-owned PTY/output log, `SessionNotLive` | fixed |
| P1 | Holder socket liveness alone could falsely report `running` after PTY exit | Added holder `Stat.status`, status-file restore, `exited` projection, and focused lifecycle tests | fixed |
| P1 | Runtime status projection only reflected holder/socket state, not Diri screen/reducer semantics | Added `session_status_report` output-log -> headless screen -> screen observation -> `StatusReducer` pipeline and real PTY test | fixed |
| P1 | Holder termination could miss a detached child outside the root shell process group | Added process-tree enumeration, start-time checked signaling, holder kill-tree path, and detached child integration test | fixed |
| P1 | Holder stat did not expose attach/resume metadata such as geometry and log epoch/offset | Added `Resize` IPC, holder geometry state, log offset/epoch reporting, and integration coverage | fixed |
| P1 | Hook parser string redaction missed JSON `authorization` value shape | Added structured JSON redaction before safe summary generation | fixed |
| P1 | Scrollback result was an empty tuple stub | Replaced with real `ReadScrollbackCellsResult`, geometry/cache/fetch validation | fixed |
| P2 | App shell still contained roadmap copy | Replaced with Diri-style preview shell and source regression test | fixed |
| P2 | Clippy flagged dead code/style issues in app/term tests | Removed unused import/variables, added focused allow for intentional preview APIs, fixed style lints | fixed |

Round 1 decision: pass.

## Round 2: Boundary and Risk Review

Checks:

- UI does not own live PTY registry or write storage state directly.
- Runtime only claims the verified holder-owned PTY adoption and detached child kill paths, not full Diri holder-manager/resource-governor crash parity.
- Hook parser redacts secret-bearing JSON keys and strings before diagnostics.
- Spawn failure does not persist half-created sessions.
- Non-live session input fails closed instead of appending fake output.
- Runtime status report is derived from real holder-produced output logs, not direct fake reducer fixtures.
- Functional cases still map back to PRD/OpenSpec.

Residual risks:

| Risk | Status | Follow-up |
|------|--------|-----------|
| `homie-client` crate still absent, so app live operations remain preview-only | accepted within PRD non-goals | add client/protocol wiring before enabling real UI session actions |
| full Diri holder-manager/resource-governor crash matrix not implemented | accepted within PRD non-goals | future runtime resilience change |
| remote node, MCP, updater, full RootView/StoreRuntime parity not implemented | accepted within PRD non-goals | split follow-up Beads/OpenSpec after local gap closure |

Round 2 decision: pass.

## Verification Re-run

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | pass |
| `cargo test --workspace` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` | pass, 9 tests after holder status/cleanup, runtime reducer pipeline, process-tree, and holder stat follow-up |
| `cargo clippy -p homie-runtime -p homie-agents -p homie-proto -p homie-storage --all-targets -- -D warnings` | pass after holder status/cleanup and runtime reducer pipeline follow-up |
| `cargo test -p homie-storage --tests -- --nocapture` | pass, 9 tests |

## Gate Decision

Decision: pass

Reason:

- No P0/P1 code review findings remain open.
- Functional verification and workspace gates pass.
- Remaining gaps are explicitly outside this gap-closure scope and documented as residual risks.

# diri-engine-migration Gap Closure OpenSpec Plan

> Change ID: `diri-engine-migration`  
> Source PRD: `prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md`  
> Beads: `homie-cj5`  
> Status: `in_progress`

## 1. Summary

This plan corrects the previous `diri-engine-migration` state drift. The earlier migration moved several Diri-derived modules into Homie, but current code still has key gaps: `RuntimeSupervisor` does not own a live PTY session, `homie-term::scrollback` still contains stub result types, Diri status reducer and hook parsing are missing, `homie-ui` does not yet contain the full Diri design token set, and `homie-app` still renders implementation-roadmap copy.

The implementation will close the first local-product gap slice while preserving Homie's long-term layering: runtime owns PTY/process state; app consumes runtime state through protocol/client or preview-only data; agent status and hook parsing remain testable in `homie-agents`; UI design tokens live in `homie-ui`.

## 2. Goals

| Goal | Source requirement | Acceptance |
|------|--------------------|------------|
| G-1 | FR-1 | `RuntimeSupervisor` launches and drives a real PTY shell; failure does not persist half-created sessions |
| G-2 | FR-2, FR-3 | `homie-agents` exposes status reducer and hook/notify parser with tests and redaction |
| G-3 | FR-4 | `homie-term::scrollback` has real fetch result, cache, geometry, and wheel routing behavior |
| G-4 | FR-5 | `homie-ui` token constants and tests cover Diri radius, typography, metrics, motion, colors, fill, space, and memory formatting |
| G-5 | FR-6 | `homie-app` removes roadmap placeholder copy and presents a Diri-style preview shell without bypassing client/protocol boundaries |
| G-6 | FR-7 | OpenSpec, functional cases, verification reports, and Beads status reflect actual delivery |

## 3. Non-Goals

- Do not migrate Diri's Swift daemon into Homie as a business facts source.
- Do not complete Diri remote node, updater, MCP, or full RootView/StoreRuntime parity in this change.
- Do not inject provider raw keys into managed agent runtime.
- Do not let `homie-app` directly own PTY, storage writes, or live session registry.
- Do not claim full Diri runtime crash parity beyond the verified minimal holder path; the current holder-owned PTY covers supervisor drop/reopen adoption, while process-tree, resource-governor, and full crash-matrix parity remain follow-up scope.

## 4. Affected Component Specs

| Component spec | Impact | Required update |
|----------------|--------|-----------------|
| `specs/runtime-supervisor/README.md` | yes | live PTY registry, spawn failure, send_text fail-closed, output log semantics |
| `specs/agent-adapter-contract/README.md` | yes | status reducer, hook/notify parser, stable events, redaction |
| `specs/desktop-shell/README.md` | yes | Diri-style preview shell, token usage boundary, no direct runtime ownership |
| `specs/session-context-store/README.md` | yes | session status/output index/read_output semantics |
| `specs/observability/README.md` | yes | safe status/hook/runtime process logs |
| `specs/storage-indexing/README.md` | conditional | only if repository/schema changes are needed |

## 5. Implementation Scope

| Area | Files/modules | Reason |
|------|---------------|--------|
| OpenSpec and evidence | `openspec/changes/diri-engine-migration/*`, `docs/verification/diri-engine-migration/*` | dev-loop gates and state consistency |
| Runtime | `crates/homie-runtime/src/*`, `crates/homie-runtime/tests/*` | holder-owned PTY ownership, lifecycle, output replay, and restore semantics |
| Agents | `crates/homie-agents/src/status.rs`, `crates/homie-agents/src/hooks.rs`, tests | Diri reducer and hook parser parity |
| Terminal | `crates/homie-term/src/scrollback.rs`, tests | replace stub scrollback |
| UI design | `crates/homie-ui/src/lib.rs`, tests | design token parity |
| App shell | `crates/homie-app/src/main.rs`, app tests | remove placeholder copy and preserve architecture boundary |

## 6. Data, State, and Security Impact

| Topic | Impact | Handling |
|-------|--------|----------|
| Credential / virtual key | no raw key changes | keep managed agent config secretless; parser redacts secret-bearing payloads |
| Session context | live sessions now have real process/output state | spawn failure must not persist fake `created` sessions |
| Memory | no direct impact | no memory writes in this change |
| Task state | no direct impact | only verification docs and Beads state affected |
| Observability | status/hook/runtime failures become visible | record safe summaries only; no raw hook payload secret leakage |
| UI state | preview shell copy changes | no direct storage writes from UI |

## 7. Test Strategy

| Layer | Required cases | Command or evidence |
|-------|----------------|---------------------|
| Unit | status reducer, hook parser, scrollback, token parity, app copy regression | `cargo test -p homie-agents`, `cargo test -p homie-term`, `cargo test -p homie-ui`, `cargo test -p homie-app` |
| Integration | live PTY shell spawn/input/output/terminate, holder adoption, exited restore, detached recovery, and failure cleanup | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` |
| E2E/manual | app compile smoke and functional case execution | `cargo check -p homie-app`, `docs/verification/diri-engine-migration/functional-verification-report.md` |
| Security | hook parser redaction and no raw secret in errors | `cargo test -p homie-agents hook_parser -- --nocapture` |

## 8. Functional Case Gate

Functional cases are defined in `docs/verification/diri-engine-migration/functional-cases.md`.

| Case | Covered goal | Required before closeout |
|------|--------------|--------------------------|
| FC-DIRI-001 | G-1 | yes |
| FC-DIRI-002 | G-1 | yes |
| FC-DIRI-003 | G-1 | yes |
| FC-DIRI-004 | G-2 | yes |
| FC-DIRI-005 | G-2 | yes |
| FC-DIRI-006 | G-3 | yes |
| FC-DIRI-007 | G-4 | yes |
| FC-DIRI-008 | G-5 | yes |
| FC-DIRI-009 | G-6 | yes |
| FC-DIRI-010 | G-1 | yes |
| FC-DIRI-011 | G-1, G-2 | yes |
| FC-DIRI-012 | G-1 | yes |
| FC-DIRI-013 | G-1 | yes |
| FC-DIRI-014 | G-2 | yes |
| FC-DIRI-015 | G-1 | yes |
| FC-DIRI-016 | G-1 | yes |
| FC-DIRI-017 | G-1 | yes |
| FC-DIRI-018 | G-1 | yes |

## 9. Release Gates

- `docs/verification/diri-engine-migration/spec-review-report.md` is pass.
- `docs/verification/diri-engine-migration/functional-cases.md` covers all P0/P1 requirements.
- `openspec/changes/diri-engine-migration/alignment-report.md` is pass.
- Required Rust checks and tests pass or are recorded as blocked with exact reasons.
- `docs/verification/diri-engine-migration/functional-verification-report.md` records every FC-DIRI case result.
- `docs/verification/diri-engine-migration/release-readiness-report.md` records final gate status.
- Beads issue state matches the actual delivery status.

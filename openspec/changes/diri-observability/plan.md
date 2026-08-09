# Diri Observability/Event/Evidence Parity Phase 1 OpenSpec Plan

> Change ID: `diri-observability`
> Beads: `homie-wm7`
> Source PRD: `prd-spec/features/diri-observability/2026-08-07-diri-observability-design.md`
> Functional cases: `docs/verification/diri-observability/functional-cases.md`
> Status: `in_progress`

## 1. Summary

This change establishes Homie's first Diri observability foundation slice. It upgrades `specs/observability/README.md` from a broad intent document into a testable contract for safe fields, Diri EventBus event semantics, metrics write failure reporting, usage evidence projection, and functional verification evidence.

Implementation stays intentionally narrow: a new `crates/homie-observability` pure model crate with focused tests. It does not wire runtime, LLM proxy, storage, client protocol, or CLI behavior. Those integration paths remain owned by later lane-specific PRD/OpenSpec changes.

## 2. Goals

| Goal | Source requirement | Acceptance |
|------|--------------------|------------|
| G-1 | FR-1 | Safe field whitelist keeps only explicitly allowed fields and strips dangerous/raw fields |
| G-2 | FR-2 | Safe event model maps Diri event catalog, session/kind filters, `seq`, and `events.dropped` visibility |
| G-3 | FR-3 | `metrics.write_failed` projection preserves main-flow success and emits only safe fields |
| G-4 | FR-4 | Usage evidence projection preserves Diri-aligned token/cache/cost summary fields and rejects invalid values |
| G-5 | FR-5 | Command/functional evidence model preserves honest gate status and emits safe verification events |
| G-6 | FR-6 | PRD/spec/OpenSpec/evidence traceability is complete and recorded |

## 3. Non-Goals

- Do not modify `homie-runtime`, `homie-llm`, `homie-storage`, `homie-client`, `homie-cli`, or UI crates.
- Do not add SQLite schemas, runtime event loops, socket subscriptions, or live metrics sinks.
- Do not update broader Diri parity lock rows as implemented.
- Do not make `homie-observability` part of the root workspace in this phase; verify it via `--manifest-path` to avoid touching unrelated lane files.
- Do not store raw prompt, raw request/response, Authorization, cookies, provider keys, complete tool args, complete tool results, or private local credential material in fixtures or docs.

## 4. Affected Component Specs

| Component spec | Impact | Required update |
|----------------|--------|-----------------|
| `specs/observability/README.md` | direct | Add safe whitelist, Diri EventBus mapping, metrics failure contract, usage evidence projection, phase 1 gates |
| `specs/runtime-supervisor/README.md` | read-only dependency | No edit; later runtime lane consumes event/metrics contract |
| `specs/llm-proxy/README.md` | future dependency | No edit; later usage/metrics lane consumes usage projection |
| `specs/mcp-automation/README.md` | future dependency | No edit; later hook/MCP evidence uses safe fields |
| `specs/desktop-shell/README.md` | future dependency | No edit; later UI usage/evidence display uses projection |

## 5. Implementation Scope

| Area | Files/modules | Reason |
|------|---------------|--------|
| PRD and evidence | `prd-spec/features/diri-observability/*`, `docs/verification/diri-observability/*` | dev-loop traceability |
| OpenSpec | `openspec/changes/diri-observability/*` | task and verification contract |
| Component spec | `specs/observability/README.md` | durable observability contract |
| Pure model crate | `crates/homie-observability/*` | safe fields, events, metrics, usage, evidence APIs |

## 6. Data, State, And Security Impact

| Topic | Impact | Handling |
|-------|--------|----------|
| Credentials | No credential storage or runtime credential flow changes | Dangerous fields are stripped by whitelist tests |
| Runtime state | No runtime behavior changes | Event model is a contract for later runtime implementation |
| LLM metrics | No proxy behavior changes | `metrics.write_failed` and usage projection are pure models |
| Storage | No schema/repository changes | Evidence path remains file-based docs and tests |
| Verification | Adds report models and docs | Status vocabulary prevents `not_run` becoming `pass` |

## 7. Functional Case Gate

| Case | Covered goal | Required before closeout |
|------|--------------|--------------------------|
| FC-OBS-001 | G-1 | yes |
| FC-OBS-002 | G-2 | yes |
| FC-OBS-003 | G-3 | yes |
| FC-OBS-004 | G-4 | yes |
| FC-OBS-005 | G-5 | yes |
| FC-OBS-006 | G-6 | yes |

## 8. Test Strategy

| Layer | Required cases | Command or evidence |
|-------|----------------|---------------------|
| Unit | safe fields, event filtering, metrics failure, usage projection, evidence status | `cargo test --manifest-path crates/homie-observability/Cargo.toml` |
| Format | Rust formatting for new crate | `cargo fmt --manifest-path crates/homie-observability/Cargo.toml -- --check` |
| Static | Compile and lint new crate | `cargo check --manifest-path crates/homie-observability/Cargo.toml`; `cargo clippy --manifest-path crates/homie-observability/Cargo.toml --all-targets -- -D warnings` |
| Documentation | PRD/OpenSpec/evidence files exist and align | FC-OBS-006 command |
| Security | Dangerous field tests and repository hook if available | crate tests, `.githooks/pre-commit` if runnable |

## 9. Release Gates

- `docs/verification/diri-observability/spec-review-report.md` is pass or pass with recorded constraints.
- `docs/verification/diri-observability/functional-cases.md` covers all P0/P1 requirements.
- `openspec/changes/diri-observability/alignment-report.md` maps every FR to tasks and functional cases.
- Focused Rust tests/checks pass or are recorded as blocked with exact reason.
- `docs/verification/diri-observability/functional-verification-report.md` records FC-OBS-001 through FC-OBS-006 results.
- `docs/verification/diri-observability/code-review-report.md` records two review passes.
- `docs/verification/diri-observability/release-readiness-report.md` records final status and residual risks.

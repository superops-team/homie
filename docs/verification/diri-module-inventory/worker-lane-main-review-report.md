# Worker lane main review report

updated_at: 2026-08-07 23:14:10 CST
scope: diri-storage-indexing, diri-agent-detection, diri-virtual-key-credentials, diri-observability

## 1. Conclusion

四个 worker lane 的 focused 实现和测试已经通过主线复验，但 Homie 仍未完成 Diri 全量 parity。`make parity-lock` 仍列出 UI/runtime/API/agent/terminal/artifact/git/automation/remote/usage/update/package/perf 的 partial 或 missing 行，因此不能关闭总目标。

## 2. Code review findings

| Severity | Category | Location | Symptom / Source / Consequence / Remedy | Status |
|----------|----------|----------|------------------------------------------|--------|
| medium | Architecture | `Cargo.toml`, `crates/homie-observability/Cargo.toml` | Symptom: `homie-observability` was created as an independent sub-workspace with its own lockfile. Source: Brooks-Lint Change Propagation / Dependency Disorder. Consequence: `cargo check/test --workspace` did not cover observability, so a lane could appear green while mainline gates missed it. Remedy: add `crates/homie-observability` to the root workspace, use `serde_json.workspace = true`, and remove the sub-crate lockfile. | fixed |
| medium | Spec Gate | `specs/agent-adapter-contract/README.md` | Symptom: the agent adapter spec had a Diri parity table but missed the mandatory `Owned feature atoms`, `Required Diri test mapping`, and `Pre-implementation gaps` markers. Source: repo OpenSpec/spec mapping contract. Consequence: `make spec-diri-mapping-check` failed and the spec could not be used as durable ownership evidence. Remedy: add the mandatory field table with owned atoms and residual gaps. | fixed |
| low | Hygiene | verification and PRD docs | Symptom: several evidence lines used concrete sensitive-header examples. Source: repo security baseline and LoopX public-boundary scan. Consequence: `loopx check` failed even though the code path was redacting correctly. Remedy: replace concrete examples with abstract sensitive-value wording. | fixed |

## 3. Brooks review

Mode: PR Review
Scope: worker lane changes plus mainline integration points.
Health Score: 90/100

No critical maintainability issue remains in the reviewed scope after fixes. The biggest risk was not inside the pure model code itself, but at the workspace boundary: a new crate outside the root workspace breaks conceptual integrity because normal project gates stop seeing it. That has been corrected.

Residual warning: the lane implementations are mostly first-stage foundations. They do not by themselves complete UI/runtime/remote/automation parity, and downstream integration must not mark parity rows implemented until the real runtime or UI path uses the new models.

## 4. Brooks test review

Mode: Test Quality Review
Scope: tests touched by storage, agent adapter, virtual key, and observability lanes.

Test Suite Map:

```text
Unit tests: Rust crate unit tests across homie-agents, homie-runtime, homie-proto, homie-term
Integration tests: storage, agent manifest/golden screens, virtual key, observability, runtime/client/CLI
E2E tests: not part of these four foundation lanes
Coverage areas: schema inventory, catalog/readiness, hook redaction, virtual key scope, safe events, usage evidence
Not covered here: Diri side-by-side UI automation, packaged app full interaction parity, remote node execution parity
```

Findings: no mock-abuse or obscurity blocker was found in the lane test files. The tests use direct behavior assertions and focused fixtures. Remaining test risk is coverage scope, not test structure: these foundation tests prove models/contracts, but do not prove full product parity.

## 5. Verification commands

| Command | Result | Notes |
|---------|--------|-------|
| `cargo check --workspace` | pass | confirmed `homie-observability` is now included |
| `cargo test --workspace` | pass | all workspace tests passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass | all workspace targets passed |
| `cargo fmt --all -- --check` | pass | formatting gate passed |
| `make module-inventory-check` | pass | module inventory valid |
| `make spec-diri-mapping-check` | pass | restored after agent spec fix |
| `loopx --registry .loopx/registry.json check --scan-root /Users/bytedance/workspace/github/homie` | pass | public boundary scan clean |
| `git diff --check` | pass | whitespace gate passed |
| `make parity-lock` | pass_with_remaining_gaps | lock is valid but rows remain incomplete |

## 6. Remaining parity blockers

`make parity-lock` still reports incomplete rows:

```text
UI-001..UI-009 partial
RT-004, RT-005, RT-009, RT-010 partial
API-001, API-003, API-004, API-005 partial
AG-002, AG-003 partial; AG-004 missing
TERM-002, TERM-004, TERM-005 partial
ART-001, ART-002 partial; ART-003 missing
GIT-001, GIT-002 partial
AUTO-001 missing
REM-001 partial; REM-002, REM-003 missing
USAGE-001, UPDATE-001, PKG-001, PERF-001 partial
```

Next execution should continue from LoopX-selected P0 UI parity work, then move into automation/remote/release rows after the UI surface has real screenshot and interaction evidence.

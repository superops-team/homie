# Workspace Storage Bootstrap Code Review Report

```yaml
change_id: workspace-storage-bootstrap
report_type: code-review
status: pass
beads: homie-mgl
reviewer: TRAE CLI
```

## 1. Scope

Reviewed files:

- `Cargo.toml`
- `rust-toolchain.toml`
- `Makefile`
- `crates/homie-proto/`
- `crates/homie-storage/`
- `crates/homie-cli/`
- `specs/storage-indexing/README.md`

## 2. Round 1: Syntax / Runtime / Interface

| Finding | Severity | Evidence | Action | Status |
|---------|----------|----------|--------|--------|
| Missing `serde_json` dev dependency in `homie-proto` tests | P1 | `cargo test --workspace` failed with unresolved crate `serde_json` | Added `serde_json.workspace = true` under `homie-proto` dev-dependencies | fixed |
| CLI doctor JSON field shape must be stable camelCase | P2 | `DoctorOutput` derives serde camelCase | Confirmed output matches functional case | pass |
| Storage health should report WAL and foreign keys from SQLite, not constants | P1 | `health_check()` uses PRAGMA query values | No action needed | pass |

## 3. Round 2: Boundary / Safety / Maintainability

| Finding | Severity | Evidence | Action | Status |
|---------|----------|----------|--------|--------|
| SQLite schema must enforce key relationship invariants | P0 | integration test `sqlite_constraints` covers FK/default/profile binding/model pricing uniqueness | No action needed | pass |
| Usage metrics must support cache/cost/latency fields without raw request data | P0 | integration test `usage_metrics_schema` inserts safe metric columns only | No action needed | pass |
| Scope must not drift into runtime/LLM/secret envelope implementation | P1 | no runtime socket, no provider proxy, no secret encryption code introduced | No action needed | pass |
| Output log should not be stored as SQLite blob | P1 | schema uses `output_log_path` + `output_tail_offset` | No action needed | pass |

## 4. Commands

| Command | Result |
|---------|--------|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo test --workspace` | pass |
| `.githooks/pre-commit` | pass |

## 5. Gate Decision

Decision: pass

Reason:

- No open P0/P1 findings remain.
- Implementation matches the scoped bootstrap PRD and OpenSpec.

# Code Review Report: Diri Proto Node Hello Usage Fixtures

```yaml
change_id: diri-proto-node-usage-fixtures
beads: homie-7i0
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Missing DTO | `crates/homie-proto/src/lib.rs` | Node hello/status/usage protocol models were absent. | fixed: added node constants, provider kind, hello/status DTOs, and usage DTOs. |
| medium | Wire contract | `ProviderKind` | Provider map keys must serialize as lowercase. | fixed and tested with BTreeMap fixture. |
| medium | Secret semantics | `NodeHelloResult` | Hello result must not include token/secret fields. | fixed by model shape and tested. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| low | Scope | proto crate | Checkpoint/move/account login DTOs are larger surfaces. | accepted: out of scope for this slice. |
| low | Runtime | node protocol | DTOs do not imply a working node server. | accepted: parity lock remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-proto node_hello_and_usage_match_diri_wire -- --nocapture` | pass |
| `cargo test -p homie-proto --tests` | pass |
| `cargo check -p homie-proto` | pass |
| `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |

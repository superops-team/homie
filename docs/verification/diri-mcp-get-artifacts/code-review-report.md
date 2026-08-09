# Code Review Report: Diri MCP get_artifacts Runtime

```yaml
change_id: diri-mcp-get-artifacts
beads: homie-pyt
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| high | Missing behavior | `crates/homie-cli/src/main.rs` | `get_artifacts` was advertised in MCP descriptors but returned unsupported. | fixed: dispatch calls `HomieClient::scan_session_artifacts`. |
| high | Real path | `mcp_get_artifacts_cli.rs` | Test must prove artifacts come from real session output, not static scanner fixtures. | fixed: test writes output through `control-stdio` and reads through `mcp-stdio --data-dir`. |
| medium | Output contract | `get_artifacts` branch | Diri expects `listeningPorts`; Homie runtime scanner stores `ports`. | fixed: MCP response uses `listeningPorts`. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Type boundary | `get_artifacts` branch | Runtime artifact types are not serde-serializable. Deriving serde there would widen a runtime API contract unnecessarily. | fixed: MCP layer projects artifacts/ports into explicit JSON. |
| medium | Scope | PR artifacts | Diri can enrich PR artifacts with live GitHub stats, but Homie PR monitor parity is separate. | accepted: no `pr` stats in this slice; readiness and parity lock keep ART-003 partial. |
| low | Parameter compatibility | `get_artifacts` branch | Diri uses `session_id`; Homie convention often uses `sessionId`. | pass: both spellings accepted. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test mcp_get_artifacts_cli -- --nocapture` | pass |
| `cargo test -p homie-runtime --test artifact_scanner` | pass |
| `cargo test -p homie-cli --test ports_cli -- --nocapture` | pass |
| `cargo check -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass |

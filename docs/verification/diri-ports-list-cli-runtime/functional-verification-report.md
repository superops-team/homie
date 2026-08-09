# Functional Verification Report: Diri Ports List CLI Runtime

```yaml
change_id: diri-ports-list-cli-runtime
beads: homie-979
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DPLC-001..002 | `cargo test -p homie-cli --test ports_cli -- --nocapture` | failed: unrecognized subcommand `ports` |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DPLC-001 | `cargo test -p homie-cli --test ports_cli -- lists_ports_from_runtime_session_output --nocapture` | pass |
| FC-DPLC-002 | `cargo test -p homie-cli --test ports_cli -- ports_cli_reports_empty_state --nocapture` | pass |
| FC-DPLC-003 | `cargo test -p homie-cli --test ports_cli -- --nocapture` | pass: 2 tests |
| FC-DPLC-003 | `cargo check -p homie-client -p homie-cli` | pass |
| FC-DPLC-003 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DPLC-003 | `cargo fmt --all -- --check` | pass |
| FC-DPLC-003 | scoped `git diff --check` | pass |

## Scope Notes

- `homie ports --json` now reads real runtime session output through `HomieClient::list_ports`.
- Empty state JSON and human output match Diri intent.
- TCP forwarding is not implemented in this slice, so `ART-002` remains partial.

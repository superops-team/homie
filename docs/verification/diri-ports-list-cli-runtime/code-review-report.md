# Code Review Report: Diri Ports List CLI Runtime

```yaml
change_id: diri-ports-list-cli-runtime
beads: homie-979
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `crates/homie-cli/tests/ports_cli.rs` | Initial test waited for snapshot output but asserted against ports immediately; runtime reopen/adoption can race the final CLI path. | fixed: test now polls `homie ports --json`, the actual user-facing path. |
| low | Scope | parity lock | Ports list is not port forwarding. | accepted: `ART-002` remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test ports_cli -- --nocapture` | pass |
| `cargo check -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |

## Remaining Risk

- TCP forwarding, remote token handling, and forward permission gates still need a separate lane.

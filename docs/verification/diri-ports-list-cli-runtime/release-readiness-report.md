# Release Readiness Report: Diri Ports List CLI Runtime

```yaml
change_id: diri-ports-list-cli-runtime
beads: homie-979
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `HomieClient::list_ports`.
- `homie ports --data-dir <dir> [--json]`.
- Real runtime session output E2E for `http://localhost:5173`.
- Empty state handling.
- Parity lock evidence updated.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| CLI ports E2E | `cargo test -p homie-cli --test ports_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Beads

- `homie-979` is complete for this bounded slice.
- Parent artifact/CLI parity remains open for forwarding.

## Remaining Work

- TCP port forwarding channel and remote token policy.
- App action wiring for opening/forwarding ports.

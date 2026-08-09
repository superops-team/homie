# Functional Verification Report: Diri Session History Protocol

```yaml
change_id: diri-session-history-protocol
beads: homie-ehm
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Summary

This slice exposes the transcript history scanner through real client/CLI paths:

- `HomieClient::session_history` scans fixture roots and writes results to storage.
- `Method::SESSION_HISTORY` dispatches via client protocol.
- `homie session history` scans Claude/Codex roots and prints JSON.

Full app history UI and real resume E2E remain pending.

## Results

| Case | Command | Result |
|------|---------|--------|
| FC-DSHP-001 | `cargo test -p homie-runtime --test history_scanner -- --nocapture` | pass |
| FC-DSHP-002 | `cargo test -p homie-cli --test session_history_cli -- --nocapture` | pass |
| FC-DSHP-003 | `cargo check -p homie-client -p homie-cli` | pass |
| FC-DSHP-003 | `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DSHP-004 | scoped `git diff --check`; `make parity-lock` | pass / pass_with_remaining_gaps |


# Functional Verification Report: Diri Proto Node Checkpoint Move Fixtures

```yaml
change_id: diri-proto-node-checkpoint-move
beads: homie-uxm
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DPNC-001 | `cargo test -p homie-proto node_checkpoint_move_match_diri_wire -- --nocapture` | failed: checkpoint/blob/move DTOs were missing. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DPNC-001 | `cargo test -p homie-proto node_checkpoint_move_match_diri_wire -- --nocapture` | pass |
| FC-DPNC-002 | `cargo test -p homie-proto --tests` | pass |
| FC-DPNC-002 | `cargo check -p homie-proto` | pass |
| FC-DPNC-002 | `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| FC-DPNC-002 | `cargo fmt --all -- --check` | pass after running `cargo fmt --all` |

## Scope Notes

- Implements checkpoint/blob/move DTO wire fixtures only.
- Does not implement file transfer, move lease runtime, or remote handoff E2E.

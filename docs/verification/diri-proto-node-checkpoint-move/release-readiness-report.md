# Release Readiness Report: Diri Proto Node Checkpoint Move Fixtures

```yaml
change_id: diri-proto-node-checkpoint-move
beads: homie-uxm
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- Checkpoint prepare/manifest/id DTOs.
- Blob has/read/put/chunk DTOs.
- Checkpoint stage result DTO.
- Transfer mode and move phase enums.
- Move commit/abort/record DTOs.
- Diri-compatible serde fixture for checkpoint and move wire fields.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Checkpoint/move fixture | `cargo test -p homie-proto node_checkpoint_move_match_diri_wire -- --nocapture` | pass |
| Proto tests | `cargo test -p homie-proto --tests` | pass |
| Build | `cargo check -p homie-proto` | pass |
| Lint | `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |

## Remaining Work

- Actual checkpoint file transfer.
- Move lease lifecycle.
- Remote handoff E2E.

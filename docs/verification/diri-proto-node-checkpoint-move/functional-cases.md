# Functional Cases: Diri Proto Node Checkpoint Move Fixtures

```yaml
change_id: diri-proto-node-checkpoint-move
beads: homie-uxm
```

## FC-DPNC-001: Checkpoint and move wire fixtures

- Command: `cargo test -p homie-proto node_checkpoint_move_match_diri_wire -- --nocapture`
- Expected:
  - Checkpoint prepare/manifest/blob/stage structs serialize with Diri camelCase fields.
  - Move commit/abort/record structs serialize with Diri camelCase fields and optional omissions.

## FC-DPNC-002: Quality gates

- Commands:
  - `cargo test -p homie-proto --tests`
  - `cargo check -p homie-proto`
  - `cargo clippy -p homie-proto --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`

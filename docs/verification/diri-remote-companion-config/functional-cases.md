# Functional Cases: Diri Remote Companion Config

```yaml
change_id: diri-remote-companion-config
beads: homie-0gh
```

## FC-DRCC-001: Companion config model

- Command: `cargo test -p homie-remote --test companion_config -- --nocapture`
- Expected:
  - Config load/save round-trips Diri-compatible camelCase JSON.
  - Saved file mode is owner-only on Unix.
  - Remove is idempotent.
  - Pairing URL includes token only through explicit helper.
  - Debug output redacts token.

## FC-DRCC-002: Build

- Command: `cargo check -p homie-remote`
- Expected: exit code 0.

## FC-DRCC-003: Lint

- Command: `cargo clippy -p homie-remote --all-targets -- -D warnings`
- Expected: exit code 0.

## FC-DRCC-004: Hygiene and parity lock

- Commands:
  - `git diff --check -- crates/homie-remote prd-spec/features/diri-remote-companion-config openspec/changes/diri-remote-companion-config docs/verification/diri-remote-companion-config`
  - `make parity-lock`
- Expected:
  - diff check passes.
  - `REM-002` may move to partial after evidence, not implemented.

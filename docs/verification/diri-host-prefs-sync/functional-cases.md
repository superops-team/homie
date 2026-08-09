# Functional Cases: Diri Host Prefs Sync

```yaml
change_id: diri-host-prefs-sync
beads: homie-cue
```

## FC-DHPS-001: Prefs sync model

- Command: `cargo test -p homie-remote --test prefs_sync -- --nocapture`
- Expected:
  - Only fixed include-list items are synced.
  - Credentials/auth/projects/todos are never present in argv.
  - `rsync -a` command is additive and lacks `--delete`.
  - Empty local tool config is success with no commands.
  - Missing remote rsync maps to a clear error message.

## FC-DHPS-002: Build

- Command: `cargo check -p homie-remote`
- Expected: exit code 0.

## FC-DHPS-003: Lint

- Command: `cargo clippy -p homie-remote --all-targets -- -D warnings`
- Expected: exit code 0.

## FC-DHPS-004: Hygiene and parity lock

- Commands:
  - `git diff --check -- crates/homie-remote prd-spec/features/diri-host-prefs-sync openspec/changes/diri-host-prefs-sync docs/verification/diri-host-prefs-sync`
  - `make parity-lock`
- Expected:
  - diff check passes.
  - `REM-003` may move to partial after evidence, not implemented.

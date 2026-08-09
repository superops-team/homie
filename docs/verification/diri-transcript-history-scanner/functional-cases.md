# Functional Cases: Diri Transcript History Scanner

```yaml
change_id: diri-transcript-history-scanner
beads: homie-941
```

## FC-DTHS-001: Runtime history scanner fixtures

- Command: `cargo test -p homie-runtime --test history_scanner -- --nocapture`
- Expected:
  - Claude fixture scans cwd, latest AI title, transcript path, and cwd_exists.
  - Codex fixture scans session_meta id/cwd and first user prompt title.
  - Tracked ids are excluded.
  - Scanned entries upsert into `homie-storage` history table.
  - Resume command is generated for Claude/Codex with valid cwd.

## FC-DTHS-002: Runtime check

- Command: `cargo check -p homie-runtime`
- Expected: exit code 0.

## FC-DTHS-003: Runtime lint

- Command: `cargo clippy -p homie-runtime --all-targets -- -D warnings`
- Expected: exit code 0.

## FC-DTHS-004: Hygiene and parity lock

- Commands:
  - `git diff --check -- crates/homie-runtime prd-spec/features/diri-transcript-history-scanner openspec/changes/diri-transcript-history-scanner docs/verification/diri-transcript-history-scanner`
  - `make parity-lock`
- Expected:
  - diff check passes.
  - parity lock remains honest; `AG-004` should not be marked implemented until app history/resume E2E exists.

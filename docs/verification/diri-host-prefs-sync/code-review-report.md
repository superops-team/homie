# Code Review Report: Diri Host Prefs Sync

```yaml
change_id: diri-host-prefs-sync
beads: homie-cue
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Security | `homie-remote` prefs sync | Without a fixed include list, remote prefs sync could accidentally copy credentials or transcripts. | fixed: sync specs only include Diri-approved preference items. |
| medium | Correctness | `homie-remote` command argv | Remote sync must be additive and non-destructive. | fixed: rsync argv uses `-a` and never includes `--delete`. |
| low | Scope | parity lock | Prefs sync model alone does not complete host protocol parity. | accepted: REM-003 remains partial until locate_repo and remote E2E exist. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-remote --test prefs_sync -- --nocapture` | pass |
| `cargo check -p homie-remote` | pass |
| `cargo clippy -p homie-remote --all-targets -- -D warnings` | pass |


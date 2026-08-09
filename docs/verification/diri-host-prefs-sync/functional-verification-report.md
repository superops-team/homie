# Functional Verification Report: Diri Host Prefs Sync

```yaml
change_id: diri-host-prefs-sync
beads: homie-cue
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Summary

This slice advances `REM-003` from missing to partial for `host.sync_prefs`:

- `homie-remote` now has fixed Claude/Codex prefs sync specs.
- Present item discovery only includes Diri-approved preference files/directories.
- Credentials, auth files, projects/transcripts, todos and non-listed items are excluded.
- mkdir/rsync argv builders use non-interactive ssh options and additive `rsync -a`, with no `--delete`.
- Missing remote rsync maps to a clear user-facing error.

`host.locate_repo` and real remote execution remain pending, so `REM-003` is partial.

## Results

| Case | Command | Result |
|------|---------|--------|
| FC-DHPS-001 | `cargo test -p homie-remote --test prefs_sync -- --nocapture` | pass |
| FC-DHPS-002 | `cargo check -p homie-remote` | pass |
| FC-DHPS-003 | `cargo clippy -p homie-remote --all-targets -- -D warnings` | pass |


# Functional Verification Report: Diri Transcript History Scanner

```yaml
change_id: diri-transcript-history-scanner
beads: homie-941
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Summary

This slice advances `AG-004` from no scanner implementation toward first-stage history scanner parity:

- `homie-runtime::history` scans Claude and Codex transcript roots.
- Claude fixtures extract cwd, first user prompt fallback, latest AI title, transcript path, last active time, and cwd existence.
- Codex fixtures extract `session_meta` id/cwd and first user message title.
- Tracked ids are excluded.
- Scanner results write through `homie-storage` `history_entries`.
- Resume commands are projected for Claude and Codex without copying transcript contents.

`AG-004` remains `missing` in parity lock until app history surface and resume E2E are implemented.

## Results

| Case | Command | Result |
|------|---------|--------|
| FC-DTHS-001 | `cargo test -p homie-runtime --test history_scanner -- --nocapture` | pass |
| FC-DTHS-002 | `cargo check -p homie-runtime` | pass |
| FC-DTHS-003 | `cargo clippy -p homie-runtime --all-targets -- -D warnings` | pass |
| FC-DTHS-004 | scoped `git diff --check` | pass |
| FC-DTHS-004 | `make parity-lock` | pass_with_remaining_gaps |

## Remaining Gaps

- App history surface and query UI.
- Runtime/client protocol entry for `session.history` and `session.resume_from_history`.
- Real resume E2E with Claude/Codex binaries.

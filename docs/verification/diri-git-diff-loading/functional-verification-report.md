# Functional Verification Report: Diri Git Diff Loading

```yaml
change_id: diri-git-diff-loading
beads: homie-xsr
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DGDL-002 | `cargo test -p homie-runtime --test git_diff_loading -- --nocapture` | failed: missing `SessionDiffBase`, `DiffRowKind`, `load_git_diff` |
| FC-DGDL-003 | `cargo test -p homie-cli --test session_diff_cli -- --nocapture` | failed: unrecognized `session diff` subcommand |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DGDL-001 | `cargo test -p homie-proto session_read_diff_uses_diri_base64_wire -- --nocapture` | pass |
| FC-DGDL-002 | `cargo test -p homie-runtime --test git_diff_loading -- --nocapture` | pass |
| FC-DGDL-003 | `cargo test -p homie-cli --test session_diff_cli -- --nocapture` | pass |
| FC-DGDL-004 | `cargo check -p homie-proto -p homie-runtime -p homie-client -p homie-cli` | pass |
| FC-DGDL-004 | `cargo clippy -p homie-proto -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DGDL-004 | `cargo fmt --all -- --check` | pass |

## Scope Notes

- Runtime diff loader covers tracked and untracked files.
- `session.read_diff` DTO uses base64 patch wire semantics.
- CLI exposes `homie session diff` for verification and future inspector wiring.
- App Changes panel E2E is still pending, so `GIT-001` and `UI-004` remain partial.

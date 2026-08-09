# Functional Cases: Diri Git Diff Loading

```yaml
change_id: diri-git-diff-loading
beads: homie-xsr
```

## FC-DGDL-001: Proto base64 wire

- Command: `cargo test -p homie-proto session_read_diff_uses_diri_base64_wire -- --nocapture`
- Expected: `patch` serializes as base64 and `baseRef` is optional.

## FC-DGDL-002: Runtime git diff loader

- Command: `cargo test -p homie-runtime --test git_diff_loading -- --nocapture`
- Expected: tracked and untracked changes are included; HEAD comparison excludes committed branch changes.

## FC-DGDL-003: CLI runtime-backed session diff

- Command: `cargo test -p homie-cli --test session_diff_cli -- --nocapture`
- Expected: real runtime session cwd in git repo returns diff JSON with patch text and counts.

## FC-DGDL-004: Quality gates

- Commands: check, clippy, diff, parity lock.


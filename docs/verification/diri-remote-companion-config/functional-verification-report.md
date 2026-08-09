# Functional Verification Report: Diri Remote Companion Config

```yaml
change_id: diri-remote-companion-config
beads: homie-0gh
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Summary

This slice advances `REM-002` from missing to partial:

- `homie-remote` now has `RemoteCompanionConfig`.
- Config load/save/remove uses Diri-compatible camelCase JSON.
- Saved config file is owner-only on Unix.
- Endpoint label and explicit pairing URL helpers exist.
- Debug output redacts the pairing token.

Listener startup, Tailscale discovery, and app settings E2E remain pending.

## Results

| Case | Command | Result |
|------|---------|--------|
| FC-DRCC-001 | `cargo test -p homie-remote --test companion_config -- --nocapture` | pass |
| FC-DRCC-002 | `cargo check -p homie-remote` | pass |
| FC-DRCC-003 | `cargo clippy -p homie-remote --all-targets -- -D warnings` | pass |


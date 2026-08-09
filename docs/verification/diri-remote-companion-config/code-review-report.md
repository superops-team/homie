# Code Review Report: Diri Remote Companion Config

```yaml
change_id: diri-remote-companion-config
beads: homie-0gh
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Security | `RemoteCompanionConfig` | Companion token config must not be world-readable and must not leak through debug output. | fixed: save uses owner-only mode on Unix and custom Debug redacts token. |
| medium | Correctness | `homie-remote` | `REM-002` had no model for Diri-compatible remote companion config. | fixed: added load/save/remove, endpoint label and pairing URL helpers. |
| low | Scope | parity lock | Config model alone does not prove remote listener or app settings E2E. | accepted: `REM-002` remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-remote --test companion_config -- --nocapture` | pass |
| `cargo check -p homie-remote` | pass |
| `cargo clippy -p homie-remote --all-targets -- -D warnings` | pass |


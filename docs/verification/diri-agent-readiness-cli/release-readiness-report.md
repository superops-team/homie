# Release Readiness Report: Diri Agent Readiness CLI

```yaml
change_id: diri-agent-readiness-cli
beads: homie-8ua
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `homie agent readiness`.
- Descriptor directory loading.
- Isolated fake binary resolver via `--bin-dir`.
- CLI E2E for available/unavailable launchable agents and non-launchable omission.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| CLI readiness | `cargo test -p homie-cli --test agent_readiness_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-agents -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-agents -p homie-cli --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Remaining Work

- App new-agent/readiness UI E2E.
- Real installed agent smoke and login-state checks.

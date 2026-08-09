# Functional Verification Report: Diri Agent Readiness CLI

```yaml
change_id: diri-agent-readiness-cli
beads: homie-8ua
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DARC-001 | `cargo test -p homie-cli --test agent_readiness_cli -- --nocapture` | failed: unrecognized subcommand `agent` |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DARC-001 | `cargo test -p homie-cli --test agent_readiness_cli -- --nocapture` | pass |
| FC-DARC-002 | `cargo check -p homie-agents -p homie-cli` | pass |
| FC-DARC-002 | `cargo clippy -p homie-agents -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DARC-002 | `cargo fmt --all -- --check` | pass |
| FC-DARC-002 | scoped `git diff --check` | pass |

## Scope Notes

- `--bin-dir` is isolated and does not fall back to host PATH, making fixture E2E deterministic.
- This is PATH/stat readiness projection only; app UI and real login checks remain pending.

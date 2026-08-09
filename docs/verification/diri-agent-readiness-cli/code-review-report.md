# Code Review Report: Diri Agent Readiness CLI

```yaml
change_id: diri-agent-readiness-cli
beads: homie-8ua
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `crates/homie-cli/src/main.rs` | First implementation allowed `--bin-dir` resolver to fall back to host PATH, so tests could pass/fail based on locally installed `claude`. | fixed: `--bin-dir` now isolates resolution to that directory. |
| low | Scope | parity lock | CLI readiness does not prove app readiness UI or real agent login state. | accepted: AG-003 remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test agent_readiness_cli -- --nocapture` | pass |
| `cargo check -p homie-agents -p homie-cli` | pass |
| `cargo clippy -p homie-agents -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |


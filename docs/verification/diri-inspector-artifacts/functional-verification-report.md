# Diri Inspector Artifacts Functional Verification Report

```yaml
change_id: diri-inspector-artifacts
beads: homie-3q0
status: pass
validated_at: 2026-08-07
```

## Summary

This slice wires runtime artifact scanning into the user-facing inspector:

- `HomieClient::scan_session_artifacts` reads selected session output and reuses runtime `scan_artifacts`.
- `homie-app` refreshes an `ArtifactSummary` for the selected session.
- Inspector Artifacts rows now show real port, PR, preview, and link counts rather than hard-coded `none`.

`UI-004`, `ART-001`, and `ART-002` remain `partial` until browser preview, port forwarding, diff, and PR monitor E2E are complete.

## Functional Cases

| Case | Command | Result | Evidence |
|------|---------|--------|----------|
| FC-DIA-001 | `cargo test -p homie-client --tests -- --nocapture` | pass | 8 tests passed, including session artifact scan from live runtime output |
| FC-DIA-002 | `cargo test -p homie-app --tests -- --nocapture` | pass | App regression confirms inspector artifact rows come from runtime scan |
| FC-DIA-003 | `cargo clippy -p homie-client -p homie-app --all-targets -- -D warnings` | pass | touched client/app crates pass clippy |

## Gate Decision

Decision: pass


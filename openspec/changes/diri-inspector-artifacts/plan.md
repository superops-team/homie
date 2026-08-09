# Diri Inspector Artifacts OpenSpec Plan

> Change ID: `diri-inspector-artifacts`  
> Beads: `homie-3q0`  
> Source PRD: `prd-spec/features/diri-inspector-artifacts/2026-08-07-diri-inspector-artifacts-design.md`  
> Status: `in_progress`

## 1. Summary

Wire runtime artifact scanning into Homie client and app inspector so the right inspector no longer displays static `none` artifact data.

## 2. Scope

In scope:

- `HomieClient::scan_session_artifacts`.
- App artifact summary state.
- Inspector Artifacts rows from real scan result.
- Tests and parity evidence.

Out of scope:

- Browser pool preview rendering.
- Port forwarding lifecycle.
- Pull-request monitor comments/checks.

## 3. Verification Cases

| Case | Purpose | Command |
|------|---------|---------|
| FC-DIA-001 | Client artifact scan from session output | `cargo test -p homie-client --tests -- --nocapture` |
| FC-DIA-002 | App inspector no static artifact none | `cargo test -p homie-app --tests -- --nocapture` |
| FC-DIA-003 | Touched crates pass clippy | `cargo clippy -p homie-client -p homie-app --all-targets -- -D warnings` |
| FC-DIA-004 | Parity lock remains truthful | `make parity-lock` |


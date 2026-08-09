# Functional Verification Report: Diri host.locate_repo

```yaml
change_id: diri-host-locate-repo
beads: homie-37j
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DHLR-001 | `cargo test -p homie-proto host_locate_repo_round_trips_diri_spelling -- --nocapture` | failed: unresolved `HostLocateRepoParams` / `HostLocateRepoResult` |
| FC-DHLR-002..004 | `cargo test -p homie-remote --test host_locate_repo -- --nocapture` | failed: unresolved `discover_repo_origin` / `locate_repo` |
| FC-DHLR-005 | `cargo test -p homie-cli --test host_locate_repo_cli -- --nocapture` | failed: unrecognized subcommand `host` |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DHLR-001 | `cargo test -p homie-proto host_locate_repo_round_trips_diri_spelling -- --nocapture` | pass |
| FC-DHLR-002..004 | `cargo test -p homie-remote --test host_locate_repo -- --nocapture` | pass: 4 tests |
| FC-DHLR-005 | `cargo test -p homie-client client_dispatches_host_locate_repo_from_project_facts -- --nocapture` | pass |
| FC-DHLR-005 | `cargo test -p homie-cli --test host_locate_repo_cli -- --nocapture` | pass |
| FC-DHLR-006 | `cargo check -p homie-proto -p homie-remote -p homie-client -p homie-cli` | pass |
| FC-DHLR-006 | `cargo clippy -p homie-remote -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| FC-DHLR-006 | scoped `git diff --check` | pass |
| FC-DHLR-006 | `make parity-lock` | pass with remaining partial rows |

## Scope Notes

- `homie-remote` now covers Diri-style `.git/config` origin discovery, linked worktree `gitdir` config discovery, match, not-cloned, and no-origin projections.
- `homie-client` now covers storage-backed project fact matching through `projects.remote_origin`; the target clone fixture intentionally has no `.git/config`.
- `homie-cli` now exposes `homie host locate-repo` with fixture candidates for local E2E.

## Remaining Gaps

- Real remote SSH/node host scan is still not implemented.
- App remote spawn UI still needs to consume `host.locate_repo`.
- `REM-003` remains `partial`.

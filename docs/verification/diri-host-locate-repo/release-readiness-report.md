# Release Readiness Report: Diri host.locate_repo

```yaml
change_id: diri-host-locate-repo
beads: homie-37j
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- `HostLocateRepoParams` and `HostLocateRepoResult` with Diri-compatible `originURL/sessionID` JSON spelling.
- `homie-remote` repo origin discovery and matching helper.
- `HomieClient::locate_repo` and `Method::HOST_LOCATE_REPO` dispatch.
- `homie host locate-repo` CLI command.
- Functional tests for found, not-cloned, no-origin, linked worktree, storage-backed project facts, and CLI fixture E2E.
- Updated `specs/remote-node-handoff/README.md` and `docs/research/diri-parity-lock.md`.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Proto DTO | `cargo test -p homie-proto host_locate_repo_round_trips_diri_spelling -- --nocapture` | pass |
| Remote locate | `cargo test -p homie-remote --test host_locate_repo -- --nocapture` | pass |
| Client dispatch | `cargo test -p homie-client client_dispatches_host_locate_repo_from_project_facts -- --nocapture` | pass |
| CLI E2E | `cargo test -p homie-cli --test host_locate_repo_cli -- --nocapture` | pass |
| Build | `cargo check -p homie-proto -p homie-remote -p homie-client -p homie-cli` | pass |
| Lint | `cargo clippy -p homie-remote -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass_with_remaining_gaps |

## Beads

- `homie-37j` is complete for this bounded slice.
- `homie-h7n.5` remains open for broader remote/usage/update/package/perf parity.

## Remaining Work

- Real host/node lookup over SSH or first-party remote node.
- App remote-spawn UI consumption of `host.locate_repo`.
- Full REM-003 remote E2E before marking implemented.

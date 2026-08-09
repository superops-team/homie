# Diri Workbench Runtime Wiring Tasks

> Change ID: `diri-workbench-runtime-wiring`  
> Beads: `homie-3tz`

## Task Mapping

| Task | Scope | Functional cases | Beads |
|------|-------|------------------|-------|
| T-001 | Update desktop shell spec for live-connected default | FC-DWRW-001 | `homie-3tz` |
| T-002 | Implement app runtime session projection and SpawnShell action | FC-DWRW-001, FC-DWRW-002 | `homie-3tz` |
| T-003 | Implement sidebar selection and runtime resize helper | FC-DWRW-002, FC-DWRW-003 | `homie-3tz` |
| T-004 | Update tests, evidence, parity lock, and LoopX writeback | FC-DWRW-004, FC-DWRW-005 | `homie-3tz` |

## Tasks

### T-001: Desktop shell spec update

Objective:

- Update `specs/desktop-shell/README.md` to reflect that `homie-app` now uses `homie-client` live-connected mode as the default path.

Acceptance:

- The spec no longer treats preview-only as the current target once `homie-client` exists.

### T-002: Runtime session projection and SpawnShell

Objective:

- Add session rows to app state.
- Refresh session rows from `HomieClient::list_sessions`.
- Implement `spawn_runtime_shell` through `HomieClient::spawn_shell`.
- Select the newly spawned session and refresh terminal output.

Acceptance:

- FC-DWRW-001 passes.

### T-003: Sidebar selection and resize helper

Objective:

- Render real session rows from state.
- Add click handler to select a session id.
- Implement `sync_terminal_geometry` through `HomieClient::resize_session` with duplicate-size suppression.

Acceptance:

- FC-DWRW-002 and FC-DWRW-003 pass.

### T-004: Evidence and lock update

Objective:

- Record verification under `docs/verification/diri-workbench-runtime-wiring/`.
- Update parity lock evidence for `UI-001` and `UI-003` while keeping statuses `partial`.
- Run local gates and write validated progress to LoopX.

Acceptance:

- FC-DWRW-004 and FC-DWRW-005 pass.


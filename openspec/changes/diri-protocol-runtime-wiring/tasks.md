# Diri Protocol Runtime Wiring Tasks

> Change ID: `diri-protocol-runtime-wiring`  
> Beads: `homie-qci`

## Task Mapping

| Task | Scope | Functional cases | Beads |
|------|-------|------------------|-------|
| T-001 | Transport contract tests | FC-DPRW-001, FC-DPRW-002 | `homie-qci` |
| T-002 | `homie-client` NDJSON control transport | FC-DPRW-001, FC-DPRW-002 | `homie-qci` |
| T-003 | CLI runtime-backed session/control commands | FC-DPRW-003, FC-DPRW-004 | `homie-qci` |
| T-004 | Evidence, parity lock, and LoopX writeback | FC-DPRW-005 | `homie-qci` |

## Tasks

### T-001: Transport contract tests

Objective:

- Add tests proving external `ControlMessage` request handling is not an in-process-only facade.
- Cover `events.subscribe` event-frame emission and cursor response.
- Cover `events.wait` timeout semantics through the shared request path.

Acceptance:

- FC-DPRW-001 and FC-DPRW-002 fail before implementation and pass after implementation.

### T-002: `homie-client` NDJSON control transport

Objective:

- Implement `HomieClient::handle_control_message`.
- Implement `HomieClient::serve_control_stream`.
- Use safe `ErrorEnvelope` responses for unsupported or failing requests.
- Make `events.subscribe` emit event frames and a final response.
- Make `events.wait` wait until event or timeout.

Acceptance:

- FC-DPRW-001 and FC-DPRW-002 pass.
- `cargo clippy -p homie-client --all-targets -- -D warnings` passes.

### T-003: CLI runtime-backed commands

Objective:

- Add `homie control-stdio --data-dir <dir>`.
- Route `homie session create/list/snapshot` through `HomieClient`.
- Remove direct storage-only session creation from CLI session lifecycle commands.

Acceptance:

- FC-DPRW-003 and FC-DPRW-004 pass.
- `cargo clippy -p homie-cli --all-targets -- -D warnings` passes.

### T-004: Evidence and lock update

Objective:

- Record command outputs and gate decisions under `docs/verification/diri-protocol-runtime-wiring/`.
- Update `docs/research/diri-parity-lock.md`:
  - `API-002` may become `implemented` only after transport evidence passes.
  - `API-003` remains `partial` unless worktree/ports/MCP bridge E2E also lands.
- Run `make parity-lock`.
- Write LoopX validated progress and spend one slot after validation.

Acceptance:

- FC-DPRW-005 passes.
- LoopX todo `todo_b1212391c2f6` receives evidence update.


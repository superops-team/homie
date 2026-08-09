# T-002 Protocol And Event Contracts Report

```yaml
change_id: reference-parity-v1
openspec_task: T-002
beads: homie-7kj
status: pass
functional_cases:
  - FC-003
  - FC-006
```

## 1. Summary

T-002 implemented the first protocol contract slice in `homie-proto`:

- Reference parity method catalog constants.
- Reference parity event catalog constants.
- `RequestId`.
- `ControlMessage` request/response/event envelope.
- `SessionStatus` lenient unknown-value decode.
- `ErrorEnvelope` safe details.

This is a protocol contract foundation only. It does not implement runtime socket transport or handlers.

## 2. RED

Added failing contract tests:

- `crates/homie-proto/tests/protocol_contract.rs`

Initial failure:

- `ControlMessage`, `EventName`, `Method`, `RequestId`, and `SessionStatus` were missing.
- `ErrorEnvelope::new` was missing.

## 3. GREEN

Implemented the minimal protocol types in:

- `crates/homie-proto/src/lib.rs`
- `crates/homie-proto/Cargo.toml`

## 4. Verification

Command:

```bash
cargo test -p homie-proto
```

Result:

- Exit code: 0
- Unit tests: 1 passed
- Protocol contract tests: 5 passed
- Doc tests: 0 tests

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie app/CLI/proto/storage tests passed.

Safety checks:

```bash
rg -n -i "<old-reference-name-pattern>" .
git diff --check
```

Result:

- Old reference name scan: no matches.
- Markdown/patch whitespace check: pass.

## 5. Remaining Scope

Still deferred to later tasks:

- T-004 runtime handler implementation.
- T-013 CLI/MCP command plumbing.
- Transport framing and event ring implementation.
- Full FC-006 execution against a real runtime process.


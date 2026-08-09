# T-004 Runtime Supervisor Lifecycle Report

```yaml
change_id: reference-parity-v1
openspec_task: T-004
beads: homie-vv6
status: pass
functional_cases:
  - FC-007
  - FC-018
```

## 1. Summary

T-004 implemented a minimal `homie-runtime` lifecycle slice backed by `homie-storage`.

Implemented:

- `homie-runtime` crate.
- `RuntimeSupervisor::open`.
- `spawn_shell`.
- `list_sessions`.
- `send_text`.
- `read_output`.
- `archive`.
- `hibernate`.
- `wake`.
- Persistent output log path under `runtime/output/<session>.log`.
- Storage `update_session_status` helper.

This slice does not implement real PTY ownership yet. It establishes runtime lifecycle and persistence boundaries for later PTY/process work.

## 2. RED

Added lifecycle test:

- `crates/homie-runtime/tests/session_lifecycle.rs`

The test requires:

- session spawn persists.
- sent text is appended to output log.
- archive/hibernate/wake update stored status.
- reopening runtime with the same data dir preserves session and output.

## 3. GREEN

Implemented:

- `crates/homie-runtime/Cargo.toml`
- `crates/homie-runtime/src/lib.rs`
- workspace registration in `Cargo.toml`
- `Storage::update_session_status`

## 4. Verification

Focused command:

```bash
cargo test -p homie-runtime
```

Result:

- Exit code: 0
- Runtime lifecycle tests: 1 passed
- Doc tests: 0 tests

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/LLM/proto/runtime/storage tests passed.

Safety checks:

```bash
rg -n -i "<old-reference-name-pattern>" .
git diff --check
```

Result:

- Old reference name scan: no matches.
- Markdown/patch whitespace check: pass.

## 5. Remaining Scope

Still deferred:

- Real PTY/process ownership.
- Data-channel attach and grid frames.
- Event ring and state snapshot transport.
- Resource governor.
- Runtime integration with agent descriptors and virtual key issuance.


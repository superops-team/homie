# T-014 Remote Node Handoff Report

```yaml
change_id: reference-parity-v1
openspec_task: T-014
beads: homie-038
status: pass
functional_cases:
  - FC-015
```

## 1. Summary

T-014 implemented the first remote/node/handoff safety contract slice.

Implemented:

- `homie-remote` crate.
- `HostEntry`.
- `HostNodeConfig`.
- host/node validation.
- `HandoffPlan`.
- credential/build-output exclusion logic.
- quarantine-by-default handoff planning.

This is not a live remote node service yet. It establishes the credential-safe host and handoff contract.

## 2. RED

Added remote safety tests:

- `crates/homie-remote/tests/remote_safety.rs`

The tests require:

- node endpoint and token file must be configured together.
- node endpoint must include host and port.
- handoff excludes credential-shaped files and build outputs.
- handoff plans restore through quarantine.

## 3. GREEN

Implemented:

- `crates/homie-remote/Cargo.toml`
- `crates/homie-remote/src/lib.rs`
- workspace registration in `Cargo.toml`

## 4. Verification

Focused command:

```bash
cargo test -p homie-remote
```

Result:

- Exit code: 0
- Remote safety tests: 2 passed
- Doc tests: 0 tests

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/context/LLM/memory/orchestrator/proto/remote/runtime/storage/task/term/UI tests passed.

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

- Live node service.
- SSH fallback execution.
- remote spawn.
- node account login/status/default.
- real move/fork handoff.


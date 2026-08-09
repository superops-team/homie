# T-003 Agent Catalog Manifest Report

```yaml
change_id: reference-parity-v1
openspec_task: T-003
beads: homie-64r
status: pass
functional_cases:
  - FC-005
```

## 1. Summary

T-003 introduced the `homie-agents` crate and bundled Reference parity agent descriptors.

Implemented:

- `AgentManifest` schema.
- `StatusAuthority` enum.
- `ResumeSpec` and manifest-driven approval/deny keystrokes.
- `load_manifest` JSON loader.
- 19 bundled agent descriptors under `assets/agent-descriptors/`.

This is a catalog contract slice. It does not implement screen parsing or golden screen reducers; those remain part of later runtime/status detection work.

## 2. RED

Added failing catalog tests:

- `crates/homie-agents/tests/manifest_catalog.rs`

The tests require:

- 19 expected agent IDs.
- First-class agents not falling back to process-only status.
- Approval and resume capabilities loaded from data.

## 3. GREEN

Implemented:

- `crates/homie-agents/Cargo.toml`
- `crates/homie-agents/src/lib.rs`
- `assets/agent-descriptors/*.json`
- workspace registration in `Cargo.toml`

## 4. Verification

Command:

```bash
cargo test -p homie-agents
```

Result:

- Exit code: 0
- Manifest catalog tests: 3 passed
- Doc tests: 0 tests

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/proto/storage tests passed.

Safety checks:

```bash
find assets/agent-descriptors -type f -name '*.json' | sort | wc -l
rg -n -i "<old-reference-name-pattern>" .
git diff --check
```

Result:

- Descriptor count: 19
- Old reference name scan: no matches
- Markdown/patch whitespace check: pass

## 5. Remaining Scope

Still deferred:

- Golden screen fixtures and status reducer implementation.
- Agent readiness CLI/runtime endpoint.
- Runtime spawn integration.
- FC-005 execution against real installed binaries.


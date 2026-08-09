# T-011 Artifacts PR Browser Test Report

```yaml
change_id: reference-parity-v1
openspec_task: T-011
beads: homie-xsu
status: pass
functional_cases:
  - FC-012
```

## 1. Summary

T-011 implemented the first artifact scanning contract slice in `homie-runtime`.

Implemented:

- `ArtifactKind`.
- `SessionArtifact`.
- `ListeningPort`.
- `ArtifactScan`.
- `scan_artifacts` for PR links, preview URLs, generic links and local ports.

This is not the browser/test_run executor yet. It establishes the session artifact projection that later UI/MCP/browser work consumes.

## 2. RED

Added artifact tests:

- `crates/homie-runtime/tests/artifact_scanner.rs`

The tests require:

- PR link detection.
- localhost preview/port detection.
- generic link detection.
- URL/port deduplication.

## 3. GREEN

Implemented:

- artifact scan types and parser in `crates/homie-runtime/src/lib.rs`.

## 4. Verification

Focused command:

```bash
cargo test -p homie-runtime
```

Result:

- Exit code: 0
- Artifact scanner tests: 2 passed
- Runtime lifecycle tests: 1 passed
- Worktree safety tests: 1 passed

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/context/LLM/memory/orchestrator/proto/runtime/storage/task/term/UI tests passed.

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

- Persistent artifact repository APIs.
- PR status monitor.
- MCP `get_artifacts`.
- Browser/test_run execution.


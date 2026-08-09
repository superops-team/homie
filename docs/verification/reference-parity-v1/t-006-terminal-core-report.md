# T-006 Terminal Grid Interaction Report

```yaml
change_id: reference-parity-v1
openspec_task: T-006
beads: homie-6h6
status: pass
functional_cases:
  - FC-008
```

## 1. Summary

T-006 implemented a minimal `homie-term` terminal core slice.

Implemented:

- `GridBuffer` with width-based wrapping.
- Text search returning row/column/length matches.
- Basic named-key terminal encoding for Enter, Escape and arrow keys.

This is not the GPUI renderer yet. It establishes the terminal model and input encoding foundation for later UI work.

## 2. RED

Added terminal tests:

- `crates/homie-term/tests/grid_input_find.rs`

The tests require:

- text wraps to configured width.
- find returns row/column matches.
- named keys encode to terminal escape sequences.

## 3. GREEN

Implemented:

- `crates/homie-term/Cargo.toml`
- `crates/homie-term/src/lib.rs`
- workspace registration in `Cargo.toml`

## 4. Verification

Focused command:

```bash
cargo test -p homie-term
```

Result:

- Exit code: 0
- Terminal tests: 3 passed
- Doc tests: 0 tests

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/context/LLM/memory/orchestrator/proto/runtime/storage/task/term tests passed.

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

- Real terminal emulator integration.
- Scrollback fetching from runtime.
- Selection model.
- GPUI terminal element rendering.
- Real PTY manual smoke.


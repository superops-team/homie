# T-009 Navigation History Surfaces Report

```yaml
change_id: reference-parity-v1
openspec_task: T-009
beads: homie-d95
status: pass
functional_cases:
  - FC-009
  - FC-011
```

## 1. Summary

T-009 implemented the first navigation/history logic slice in `homie-ui`.

Implemented:

- `fuzzy_score` with prefix, continuous substring, word-boundary and acronym scoring.
- `rank_items`.
- `HistoryEntry::can_resume`.

This is not the rendered command palette, quick open, switcher or history UI yet. It establishes deterministic navigation and history eligibility logic.

## 2. RED

Added navigation tests:

- `crates/homie-ui/tests/navigation.rs`

The tests required:

- prefix and word-boundary scoring.
- continuous substring ranking.
- history resume disabled for missing cwd or transcript.

The RED loop exposed two scoring issues:

- Continuous substring needed priority over scattered character matches.
- Acronym matching needed priority for multi-word labels.

## 3. GREEN

Implemented:

- fuzzy/ranking/history primitives in `crates/homie-ui/src/lib.rs`.

## 4. Verification

Focused command:

```bash
cargo test -p homie-ui
```

Result:

- Exit code: 0
- Navigation tests: 3 passed
- Token tests: 3 passed
- Workbench state tests: 3 passed

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

- Rendered command palette.
- Quick open directory indexing.
- History scanner against real transcripts.
- Ctrl-Tab switcher and overview board.


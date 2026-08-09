# OpenSpec Plan: Diri Transcript History Scanner

> Change ID: `diri-transcript-history-scanner`  
> Beads: `homie-941`

## Scope

Implement the first Homie transcript history scanner for Diri parity row `AG-004` and atom `M05-F002`. The scanner reads local fixture roots for Claude and Codex transcripts, projects safe history metadata, and writes entries into `homie-storage`.

## Modules

| Module | Change |
|--------|--------|
| `crates/homie-runtime/src/history.rs` | New scanner and resume command model |
| `crates/homie-runtime/src/lib.rs` | Export history module |
| `crates/homie-runtime/tests/history_scanner.rs` | Fixture tests for Claude/Codex scan, tracked dedupe, storage write, resume command |
| docs | PRD/OpenSpec/evidence |

## Functional Cases

| Case | Command |
|------|---------|
| FC-DTHS-001 | `cargo test -p homie-runtime --test history_scanner -- --nocapture` |
| FC-DTHS-002 | `cargo check -p homie-runtime` |
| FC-DTHS-003 | `cargo clippy -p homie-runtime --all-targets -- -D warnings` |
| FC-DTHS-004 | scoped `git diff --check`; `make parity-lock` |

## Acceptance

- Claude fixture produces a history entry with latest AI title.
- Codex fixture produces a history entry with first user prompt title.
- Tracked ids are excluded.
- Scan results can be stored through `homie-storage`.
- Resume command is generated only for supported agents with existing cwd.

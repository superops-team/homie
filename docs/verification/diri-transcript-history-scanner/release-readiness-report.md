# Release Readiness Report: Diri Transcript History Scanner

```yaml
change_id: diri-transcript-history-scanner
beads: homie-941
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## 1. Source

- PRD: `prd-spec/features/diri-transcript-history-scanner/2026-08-08-diri-transcript-history-scanner-design.md`
- OpenSpec: `openspec/changes/diri-transcript-history-scanner/`
- Functional cases: `docs/verification/diri-transcript-history-scanner/functional-cases.md`
- Beads: `homie-941`

## 2. Delivered

- New `homie-runtime::history` module.
- Claude transcript scanner for cwd, title, transcript path, active time and resume id.
- Codex transcript scanner for session metadata and first user prompt title.
- Tracked conversation id dedupe.
- Storage writer through `homie-storage` history repository API.
- Resume command projection for supported agent kinds.

## 3. Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Runtime scanner tests | `cargo test -p homie-runtime --test history_scanner -- --nocapture` | pass |
| Build | `cargo check -p homie-runtime` | pass |
| Lint | `cargo clippy -p homie-runtime --all-targets -- -D warnings` | pass |
| Diff hygiene | scoped `git diff --check` | pass |
| Parity lock | `make parity-lock` | pass_with_remaining_gaps |

## 4. Parity Status

`AG-004` remains `missing` in the parity lock because full parity also requires app history surface, protocol/client methods, and real resume E2E.

## 5. Risk

Risk is low for this scoped implementation. Scanner tests use local fixture roots and do not read real user HOME data.

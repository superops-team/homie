# Code Review Report: Diri Transcript History Scanner

```yaml
change_id: diri-transcript-history-scanner
beads: homie-941
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `crates/homie-runtime/src/history.rs` | Initial implementation was missing; `AG-004` had no real scanner code. | fixed: added Claude/Codex fixture scanner and storage writer. |
| medium | Security | `crates/homie-runtime/src/history.rs` | Transcript scanners can accidentally persist raw prompts. | fixed: scanner only persists metadata/title summary, cwd, ids and paths through `HistoryEntryUpsert`; no transcript content is stored in metadata. |
| low | Scope | parity lock | Scanner alone does not prove full resume parity. | accepted: parity lock keeps `AG-004` missing until app/protocol resume E2E exists. |

## Brooks Review

Mode: PR Review  
Scope: runtime history scanner and tests  
Health Score: 92/100

No critical issue remains. The implementation follows Diri's read-only scanning pattern and keeps storage writes behind the existing repository API. The remaining risk is scope: this is scanner foundation, not full history UI/resume parity.

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-runtime --test history_scanner -- --nocapture` | pass |
| `cargo check -p homie-runtime` | pass |
| `cargo clippy -p homie-runtime --all-targets -- -D warnings` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass_with_remaining_gaps |

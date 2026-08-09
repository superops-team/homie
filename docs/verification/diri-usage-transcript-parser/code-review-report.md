# Code Review Report: Diri Usage Transcript Parser

```yaml
change_id: diri-usage-transcript-parser
beads: homie-hd1
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| high | Missing behavior | `crates/homie-llm/src/lib.rs` | Homie had pricing helpers but no transcript parser API. | fixed: added `parse_transcript_usage_events` and neutral usage event model. |
| high | Data semantics | Claude parser | Cache creation total and 5m/1h duration buckets must not be collapsed. | fixed: event model preserves `cache_write_tokens`, `cache_write_5m_tokens`, and `cache_write_1h_tokens`. |
| medium | Codex model context | Codex parser | Token-count rows need the latest `session_meta`/`turn_context` model to estimate cost. | fixed: parser retains Codex model context while reading JSONL. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Scope | parser implementation | Parser could drift into watcher/storage import responsibilities. | pass: parser only returns events; no storage writes or offset cache. |
| medium | Robustness | parser implementation | Bad transcript lines should not abort parsing. | pass: bad JSON and non-usage rows are skipped. |
| low | Lint | `parse_transcript_usage_events` | Clippy flagged a nested Codex model-context condition. | fixed: collapsed into a single condition. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-llm --test usage_transcript_parser -- --nocapture` | pass |
| `cargo test -p homie-llm` | pass |
| `cargo check -p homie-llm` | pass |
| `cargo clippy -p homie-llm --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |

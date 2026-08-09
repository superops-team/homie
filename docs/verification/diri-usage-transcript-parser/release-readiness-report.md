# Release Readiness Report: Diri Usage Transcript Parser

```yaml
change_id: diri-usage-transcript-parser
beads: homie-hd1
status: pass_with_scope_limit
validated_at: 2026-08-08
```

## Delivered

- Neutral `TranscriptUsageEvent` model.
- Claude transcript usage parser.
- Codex transcript usage parser with model context retention.
- Stable transcript source event ids.
- Bad-line tolerance, unknown-model cost behavior, and negative token clamping.

## Gate Results

| Gate | Command | Result |
|------|---------|--------|
| Transcript parser tests | `cargo test -p homie-llm --test usage_transcript_parser -- --nocapture` | pass |
| homie-llm tests | `cargo test -p homie-llm` | pass |
| Build | `cargo check -p homie-llm` | pass |
| Lint | `cargo clippy -p homie-llm --all-targets -- -D warnings` | pass |
| Format | `cargo fmt --all -- --check` | pass |
| Diff hygiene | scoped `git diff --check` | pass |

## Remaining Work

- Map `TranscriptUsageEvent` into `homie-storage::RecordUsage`.
- Directory watcher and offset cache.
- Pricing snapshot persistence.
- Usage UI and fleet merge.

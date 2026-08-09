# Functional Verification Report: Diri Usage Transcript Parser

```yaml
change_id: diri-usage-transcript-parser
beads: homie-hd1
status: pass_with_scope_limit
verified_at: 2026-08-08
```

## RED Evidence

| Case | Command | RED result |
|------|---------|------------|
| FC-DUTP-001..003 | `cargo test -p homie-llm --test usage_transcript_parser -- --nocapture` | failed: `UsageProviderKind` and `parse_transcript_usage_events` were missing; `tempfile` was missing from test dependencies. |

## GREEN Evidence

| Case | Command | Result |
|------|---------|--------|
| FC-DUTP-001 | `cargo test -p homie-llm --test usage_transcript_parser -- parses_claude_usage_events --nocapture` | pass |
| FC-DUTP-002 | `cargo test -p homie-llm --test usage_transcript_parser -- parses_codex_usage_events_with_model_context --nocapture` | pass |
| FC-DUTP-003 | `cargo test -p homie-llm --test usage_transcript_parser -- bad_unknown_and_negative_inputs_are_safe --nocapture` | pass |
| FC-DUTP-001..003 | `cargo test -p homie-llm --test usage_transcript_parser -- --nocapture` | pass: 3 passed |
| FC-DUTP-004 | `cargo test -p homie-llm` | pass |
| FC-DUTP-004 | `cargo check -p homie-llm` | pass |
| FC-DUTP-004 | `cargo clippy -p homie-llm --all-targets -- -D warnings` | pass after collapsing the nested Codex model-context condition |
| FC-DUTP-004 | `cargo fmt --all -- --check` | pass after running `cargo fmt --all` |
| FC-DUTP-004 | scoped `git diff --check` | pass |

## Scope Notes

- Implements pure file-to-event transcript parsing.
- Does not implement directory watcher, offset cache, storage import, usage UI, or fleet merge.

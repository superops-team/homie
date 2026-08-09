# Functional Cases: Diri Usage Transcript Parser

```yaml
change_id: diri-usage-transcript-parser
beads: homie-hd1
```

## FC-DUTP-001: Claude transcript usage parsing

- Command: `cargo test -p homie-llm --test usage_transcript_parser -- parses_claude_usage_events --nocapture`
- Expected:
  - One Claude assistant usage line becomes one event.
  - Event includes session id, model, input/output/cache read/cache write/cache write duration fields.
  - Estimated cost matches Diri-compatible pricing helper.

## FC-DUTP-002: Codex transcript usage parsing

- Command: `cargo test -p homie-llm --test usage_transcript_parser -- parses_codex_usage_events_with_model_context --nocapture`
- Expected:
  - Codex model is retained from `session_meta`/`turn_context`.
  - `token_count` event becomes provider=`codex` usage event.
  - Cached input becomes cache_read_tokens.
  - Estimated cost matches OpenAI helper.

## FC-DUTP-003: Bad input and stable id behavior

- Command: `cargo test -p homie-llm --test usage_transcript_parser -- bad_unknown_and_negative_inputs_are_safe --nocapture`
- Expected:
  - Bad JSON and non-usage lines are skipped.
  - Unknown model still yields usage event with `estimated_cost=None`.
  - Negative token counts clamp to zero.
  - Re-parsing the same file produces the same source_event_id.

## FC-DUTP-004: Quality gates

- Commands:
  - `cargo test -p homie-llm`
  - `cargo check -p homie-llm`
  - `cargo clippy -p homie-llm --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - scoped `git diff --check`
  - `make parity-lock`
- Expected: all commands pass.

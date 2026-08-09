# OpenSpec Tasks: Diri Usage Transcript Parser

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Update LLM proxy spec | Spec records transcript parser as usage import predecessor and keeps watcher/storage out of scope | FC-DUTP-004 |
| T-002 | Add RED parser tests | Tests fail before parser model/functions exist | FC-DUTP-001, FC-DUTP-002, FC-DUTP-003 |
| T-003 | Implement Claude parser | Claude assistant usage line becomes neutral usage event with cache write duration fields and estimate | FC-DUTP-001 |
| T-004 | Implement Codex parser | Codex session_meta/turn_context model is retained and token_count becomes neutral usage event | FC-DUTP-002 |
| T-005 | Implement safety/id behavior | Bad JSON skipped, unknown model cost None, negatives clamp, source_event_id stable | FC-DUTP-003 |
| T-006 | Run quality gates and update parity lock | homie-llm tests/check/clippy/fmt/diff/parity pass; USAGE-001 evidence updated | FC-DUTP-004 |

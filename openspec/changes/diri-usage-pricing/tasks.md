# OpenSpec Tasks: Diri Usage Pricing Estimate

| Task | Description | Acceptance | Cases |
|------|-------------|------------|-------|
| T-001 | Update LLM proxy spec | Spec records Diri-compatible API-equivalent pricing helper and estimate-not-billed semantics | FC-DUP-004 |
| T-002 | Add RED pricing tests | Tests fail before helper exists | FC-DUP-001, FC-DUP-002, FC-DUP-003 |
| T-003 | Implement pricing helper | `homie-llm` exposes model matchers and estimate functions matching Diri | FC-DUP-001, FC-DUP-002 |
| T-004 | Implement safety semantics | Unknown models return None and negative tokens clamp to zero | FC-DUP-003 |
| T-005 | Run quality gates | homie-llm tests/check/clippy/fmt/diff/parity pass | FC-DUP-004 |
| T-006 | Update parity lock and close Bead | USAGE-001 evidence includes `homie-llm` pricing tests; watcher/UI/fleet remain pending | FC-DUP-004 |

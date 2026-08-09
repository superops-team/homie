# Release Readiness Report

```yaml
change_id: diri-virtual-key-credentials
beads: homie-e1s
dev_loop_step: 10
status: pass_with_note
risk_tier: Tier 3 high-stakes
```

## 1. Source

| Field | Value |
|-------|-------|
| PRD | `prd-spec/features/diri-virtual-key-credentials/2026-08-07-diri-virtual-key-credentials-design.md` |
| Component spec | `specs/virtual-key-credentials/README.md` |
| OpenSpec | `openspec/changes/diri-virtual-key-credentials/` |
| Functional cases | `docs/verification/diri-virtual-key-credentials/functional-cases.md` |
| Beads | `homie-e1s` |

## 2. Change Summary

Implemented the first Diri/Homie credential parity foundation slice:

- Long-lived component spec now defines Diri/Homie credential adaptation, cross-spec mandatory gates, raw provider key forbidden matrix, and first-stage tests.
- `homie-llm` now exposes secretless managed proxy config and raw-key propagation guard.
- Tests now cover virtual key issue/validate/revoke/expired/unknown/scope/model denial, managed config serialization, and raw-key rejection for remote/MCP/agent/log destinations.

## 3. Gate Results

| Gate | Command | Exit code | Status | Notes |
|------|---------|-----------|--------|-------|
| RED | `cargo test -p homie-llm --test virtual_key` | 101 | pass | Failed on missing new API as expected before implementation |
| Focused tests | `cargo test -p homie-llm --test virtual_key` | 0 | pass | 5 tests passed |
| Crate tests | `cargo test -p homie-llm` | 0 | pass | 5 integration tests and 0 doctests passed |
| Scoped fmt | `cargo fmt --package homie-llm -- --check` | 0 | pass | `homie-llm` is formatted |
| Scoped check | `cargo check -p homie-llm` | 0 | pass | Crate compiles |
| Scoped clippy | `cargo clippy -p homie-llm --all-targets -- -D warnings` | 0 | pass | No clippy warnings in lane crate |
| Workspace check | `cargo check --workspace` | 0 | pass | Workspace compiles |
| Scoped diff check | `git diff --check -- <lane paths>` | 0 | pass | No whitespace errors in edited lane paths |
| Workspace fmt | `cargo fmt --all -- --check` | 1 | partial | Failed in out-of-scope `crates/homie-storage/tests/diri_storage_indexing.rs`; not modified because user limited write scope |

## 4. Security Gate

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Raw key not in managed agent config | pass | `managed_proxy_config_serializes_without_raw_provider_key` |
| Raw key rejected for remote node payload | pass | `raw_provider_key_is_rejected_for_cross_module_destinations` |
| Raw key rejected for MCP payload | pass | `raw_provider_key_is_rejected_for_cross_module_destinations` |
| Raw key rejected for log/event payload | pass | `raw_provider_key_is_rejected_for_cross_module_destinations` |
| Errors do not echo secrets | pass | lifecycle and raw-key tests assert rendered errors omit test secrets |
| Debug output does not echo issued virtual key secret | pass | `managed_proxy_config_serializes_without_raw_provider_key` |

## 5. New Dependencies

No new external dependency was introduced.

`serde_json.workspace = true` was added as a `dev-dependency` for `homie-llm` tests only. It already exists in the workspace dependency set and is used for serialization contract assertions.

## 6. Not Run

| Gate | Status | Reason |
|------|--------|--------|
| `cargo audit` | not_run | Not required for the scoped first-stage code change and may need local tool availability |
| Full HTTP proxy smoke | not_run | Full proxy is out of scope |
| Remote node E2E | not_run | Remote lane/files are out of scope |
| MCP stdio E2E | not_run | MCP lane/files are out of scope |
| Real provider credential smoke | not_run | Tests must not require or expose real provider credentials |

## 7. Residual Risk

- `remote-node-handoff` and `mcp-automation` specs should later add explicit references to the new cross-spec credential gate. This lane did not edit those files per user instruction.
- Full provider proxy validation, usage accounting, and secret envelope persistence remain for later LLM/storage lanes.
- Repository-level `cargo fmt --all -- --check` is currently blocked by an out-of-scope storage test formatting diff.

## 8. Readiness Verdict

The first-stage foundation-security slice is ready within its approved scope. Focused tests, clippy, crate check, workspace check, and scoped diff checks pass. The only partial gate is an out-of-scope workspace formatting issue that was not introduced or modified by this lane.


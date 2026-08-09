# Release Readiness Report

```yaml
change_id: diri-agent-detection
beads: homie-v4b
risk_tier: tier_2_normal_with_security_sensitive_redaction
status: pass
source_prd: prd-spec/features/diri-agent-detection/2026-08-07-diri-agent-detection-design.md
openspec: openspec/changes/diri-agent-detection
component_spec: specs/agent-adapter-contract/README.md
```

## 1. Delivered Scope

- Added Chinese PRD, spec review, functional cases, OpenSpec plan/tasks/alignment for `diri-agent-detection`.
- Updated `specs/agent-adapter-contract/README.md` with Diri combined manifest, catalog/readiness, golden screen, hook redaction, failure mode, and test mapping contracts.
- Replaced `assets/agent-descriptors/*.json` with Diri combined manifests carrying both `agent` descriptor blocks and detection `rules`.
- Added `homie-agents` catalog/readiness/resume projection APIs.
- Added Diri golden screen tests for Claude Code, Codex, Cursor, and Gemini.
- Strengthened hook/notify redaction for nested payloads, Authorization/Cookie headers, and URL query secret values.

## 2. Functional Case Results

### FC-DA-001

- Command: `cargo test -p homie-agents --test manifest_catalog bundled_catalog_projects_diri_manifest_fields -- --nocapture`
- Result: pass as part of `cargo test -p homie-agents --test manifest_catalog`
- Evidence: 8 manifest catalog tests passed.

### FC-DA-002

- Command: `cargo test -p homie-agents --test manifest_catalog catalog_resolves_aliases_and_falls_back_for_unknown_ids -- --nocapture`
- Result: pass as part of `cargo test -p homie-agents --test manifest_catalog`
- Evidence: alias resolution and unknown fallback test passed.

### FC-DA-003

- Command: `cargo test -p homie-agents --test manifest_catalog readiness_projects_launchable_agents_only -- --nocapture`
- Result: pass as part of `cargo test -p homie-agents --test manifest_catalog`
- Evidence: readiness test passed; `shell` and `generic` excluded from binary probes.

### FC-DA-004

- Command: `cargo test -p homie-agents --test golden_screens -- --nocapture`
- Result: pass
- Evidence: 5 golden screen tests passed.

### FC-DA-005

- Command: `cargo test -p homie-agents --test hook_parser -- --nocapture`
- Result: pass
- Evidence: 5 hook parser tests passed.

### FC-DA-006

- Command: `cargo test -p homie-agents --test manifest_catalog every_bundled_manifest_decodes_strictly -- --nocapture`
- Result: pass as part of `cargo test -p homie-agents --test manifest_catalog`
- Evidence: strict bundled manifest decode test passed.

### FC-DA-007

| Command | Exit | Result | Notes |
|---------|------|--------|-------|
| `cargo fmt --all -- --check` | 1 then 0 | pass after `cargo fmt --all` | Initial failure was formatting-only; formatted and rechecked. |
| `cargo check -p homie-agents` | 0 | pass | Focused crate check. |
| `cargo clippy -p homie-agents --all-targets -- -D warnings` | 0 | pass | No warnings. |
| `cargo test -p homie-agents` | 0 | pass | 31 tests passed plus doc-tests. |
| `git diff --check -- prd-spec/features/diri-agent-detection openspec/changes/diri-agent-detection docs/verification/diri-agent-detection specs/agent-adapter-contract/README.md assets/agent-descriptors crates/homie-agents` | 0 | pass | Scoped whitespace check. |

### FC-DA-008

- Command: `.githooks/pre-commit`
- Exit: 0
- Result: pass

## 3. TDD Evidence

RED failures observed before implementation:

- `cargo test -p homie-agents --test manifest_catalog`: failed because `AgentCatalog` and `ResumeStyle` did not exist.
- `cargo test -p homie-agents --test golden_screens`: failed because all descriptor files were simplified descriptors, not detection manifests.
- `cargo test -p homie-agents --test hook_parser`: failed because inline secret-bearing header text leaked inside `safe_summary`.

GREEN fixes:

- Added catalog/readiness/resume APIs and Diri descriptor fields.
- Synced Diri combined manifests into `assets/agent-descriptors`.
- Enhanced redaction regexes for headers and URL query values.

## 4. Final Test Summary

`cargo test -p homie-agents`:

- Unit tests: 13 passed.
- `golden_screens`: 5 passed.
- `hook_parser`: 5 passed.
- `manifest_catalog`: 8 passed.
- `status_reducer`: 5 passed.
- Doc-tests: 0 run, pass.

## 5. New Dependencies

- None.

## 6. Not Run

- Full workspace tests were not run because the user requested this lane to avoid other lane files and focus on `homie-agents`.
- Runtime/PTY/UI E2E was not run because runtime changes are explicitly out of scope for this first-stage implementation.

## 7. Residual Risk

- Diri manifests were copied into Homie assets and validated by Homie tests, but future drift requires an explicit sync or review process.
- `AgentReadiness` is a resolver-injected pure contract; real login-shell PATH resolution remains a runtime lane responsibility.
- `homie-proto::model::AgentDescriptor` still has the older simplified shape. This change intentionally avoids protocol/runtime wiring; protocol projection should be handled in a later `homie-proto` scoped change.

# Diri Virtual Key Credentials Alignment Report

```yaml
change_id: diri-virtual-key-credentials
beads: homie-e1s
dev_loop_step: 4
status: aligned
source_prd: prd-spec/features/diri-virtual-key-credentials/2026-08-07-diri-virtual-key-credentials-design.md
functional_cases: docs/verification/diri-virtual-key-credentials/functional-cases.md
openspec_plan: openspec/changes/diri-virtual-key-credentials/plan.md
openspec_tasks: openspec/changes/diri-virtual-key-credentials/tasks.md
```

## 1. Alignment Verdict

- PRD requirements: fully mapped.
- OpenSpec tasks: fully mapped to functional cases.
- Component specs: `specs/virtual-key-credentials/README.md` requires update before implementation.
- Write scope: aligned with user-approved paths.
- Deferred scope: complete HTTP proxy, secret envelope, remote node, MCP and runtime wiring are explicitly out of scope.

## 2. PRD To OpenSpec Mapping

| PRD requirement | OpenSpec task | Functional cases | Evidence path | Status |
|-----------------|---------------|------------------|---------------|--------|
| FR-1 Component spec gates | T-001, T-004 | FC-DVKC-005, FC-DVKC-006 | `functional-verification-report.md#fc-dvkc-005`, `#fc-dvkc-006` | aligned |
| FR-2 Virtual key lifecycle | T-002 | FC-DVKC-001, FC-DVKC-002, FC-DVKC-003 | `functional-verification-report.md#fc-dvkc-001`, `#fc-dvkc-002`, `#fc-dvkc-003` | aligned |
| FR-3 Managed agent config secretless | T-003 | FC-DVKC-004 | `functional-verification-report.md#fc-dvkc-004` | aligned |
| FR-4 Raw key propagation denial | T-004 | FC-DVKC-005 | `functional-verification-report.md#fc-dvkc-005` | aligned |
| FR-5 Secretless errors/evidence | T-002, T-003, T-004, T-005 | FC-DVKC-002, FC-DVKC-004, FC-DVKC-005, FC-DVKC-007 | `functional-verification-report.md`, `release-readiness-report.md` | aligned |

## 3. Component Spec Alignment

| Component spec | Required update | OpenSpec owner | Verification | Status |
|----------------|-----------------|----------------|--------------|--------|
| `specs/virtual-key-credentials/README.md` | Add Diri/Homie adaptation, cross-spec gates, raw-key forbidden matrix, first-stage tests | T-001 | FC-DVKC-006 | update-required |
| `specs/llm-proxy/README.md` | No edit in this lane; consumed via managed proxy config contract | T-003 | FC-DVKC-004 | no-edit |
| `specs/remote-node-handoff/README.md` | No edit in this lane; final should flag follow-up if stronger no-raw-key gate is needed | T-004 | FC-DVKC-005 | no-edit |
| `specs/mcp-automation/README.md` | No edit in this lane; final should flag follow-up if MCP spec needs explicit reference | T-004 | FC-DVKC-005 | no-edit |

## 4. Functional Case Coverage

| Functional case | Covered tasks | Covers risk | Execution command |
|-----------------|---------------|-------------|-------------------|
| FC-DVKC-001 | T-002 | Matching scope validation | `cargo test -p homie-llm issued_virtual_key_validates_only_for_matching_scope -- --nocapture` |
| FC-DVKC-002 | T-002 | Revoked/expired/unknown fail closed | `cargo test -p homie-llm revoked_expired_and_unknown_virtual_keys_are_rejected -- --nocapture` |
| FC-DVKC-003 | T-002 | Profile/provider/model scope denial | `cargo test -p homie-llm scope_denied_covers_profile_provider_and_model -- --nocapture` |
| FC-DVKC-004 | T-003 | Managed config secretless serialization | `cargo test -p homie-llm managed_proxy_config_serializes_without_raw_provider_key -- --nocapture` |
| FC-DVKC-005 | T-004 | Raw key rejected for remote/MCP/agent/log destinations | `cargo test -p homie-llm raw_provider_key_is_rejected_for_cross_module_destinations -- --nocapture` |
| FC-DVKC-006 | T-001 | Spec and OpenSpec alignment | `rg` document gates |
| FC-DVKC-007 | T-005 | Local quality/security gate | `cargo test -p homie-llm`, `cargo fmt --all -- --check`, `cargo check --workspace`, `git diff --check` |

## 5. Scope Guard

| Potential drift | Decision | Rationale |
|-----------------|----------|-----------|
| Implement full HTTP proxy | out of scope | Belongs to `llm-proxy` / usage lane after L0 gate |
| Implement secret envelope | out of scope | Needs crypto format decision and storage work |
| Modify remote node code/spec | out of scope | User restricted writes; final can flag follow-up |
| Modify MCP code/spec | out of scope | User restricted writes; final can flag follow-up |
| Add global redaction framework | out of scope | Observability lane owns safe field whitelist |

## 6. Gate Result

All PRD FR items have an OpenSpec task, at least one functional case, and an evidence path. Implementation may proceed with SDD/TDD inside the approved write scope.


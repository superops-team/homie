# Diri Virtual Key Credentials OpenSpec Tasks

> Change ID: `diri-virtual-key-credentials`  
> Source PRD: `prd-spec/features/diri-virtual-key-credentials/2026-08-07-diri-virtual-key-credentials-design.md`  
> Functional cases: `docs/verification/diri-virtual-key-credentials/functional-cases.md`  
> Beads: `homie-e1s`

## Task Status

| Status | Meaning |
|--------|---------|
| todo | Not started |
| red | Failing test or contract written |
| green | Implementation passes focused verification |
| refactor | Cleanup while tests stay green |
| done | Task evidence recorded and accepted |

## Task To Functional Case Mapping

| OpenSpec task | PRD requirement | Functional cases |
|---------------|-----------------|------------------|
| T-001 | FR-1 | FC-DVKC-006 |
| T-002 | FR-2, FR-5 | FC-DVKC-001, FC-DVKC-002, FC-DVKC-003 |
| T-003 | FR-3, FR-5 | FC-DVKC-004 |
| T-004 | FR-1, FR-4, FR-5 | FC-DVKC-005 |
| T-005 | FR-5 | FC-DVKC-007 |

## Tasks

### T-001: Strengthen virtual-key-credentials component spec

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-1 |
| Component spec | `specs/virtual-key-credentials/README.md` |
| Files | `specs/virtual-key-credentials/README.md` |
| Functional cases | FC-DVKC-006 |

Objective:

- Make `virtual-key-credentials` the explicit L0 security contract for provider custody, virtual key usage and cross-module raw-key denial.

RED:

- Document gate fails until component spec includes `Diri Behavior Parity`, `Cross-Spec Mandatory Gates`, and `Raw Provider Key Forbidden Matrix`.

GREEN:

- Add Diri/Homie adaptation table.
- Add cross-spec mandatory gates for LLM proxy, remote node handoff, MCP automation, agent adapter, observability, context and memory.
- Add forbidden destination matrix and first-stage test mapping.

Acceptance:

- FC-DVKC-006 passes.

Evidence:

- `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-006`

### T-002: Preserve virtual key lifecycle fail-closed behavior

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-2, FR-5 |
| Component spec | `specs/virtual-key-credentials/README.md` |
| Files | `crates/homie-llm/src/lib.rs`, `crates/homie-llm/tests/virtual_key.rs` |
| Functional cases | FC-DVKC-001, FC-DVKC-002, FC-DVKC-003 |

Objective:

- Keep the first virtual-key store behavior explicit and complete: issue, validate, revoke, expiry, unknown key and scope denial.

RED:

- Add or update tests for:
  - matching scope succeeds;
  - wrong session/profile/provider fails;
  - disallowed model fails;
  - expired, revoked and unknown keys fail closed;
  - errors do not echo secrets.

GREEN:

- Implement only the minimal lifecycle behavior required by tests.
- Keep public API small and independent of HTTP proxy implementation.

Refactor:

- Keep repeated scope construction in test helpers.

Acceptance:

- FC-DVKC-001, FC-DVKC-002 and FC-DVKC-003 pass.

Evidence:

- `docs/verification/diri-virtual-key-credentials/sdd-tdd-task-report.md#t-002`
- `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-001`
- `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-002`
- `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-003`

### T-003: Add secretless managed agent proxy config

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-3, FR-5 |
| Component spec | `specs/virtual-key-credentials/README.md`, `specs/llm-proxy/README.md` |
| Files | `crates/homie-llm/src/lib.rs`, `crates/homie-llm/tests/virtual_key.rs` |
| Functional cases | FC-DVKC-004 |

Objective:

- Define the first agent-visible LLM config shape: local proxy URL plus virtual key, never raw provider credentials.

RED:

- Add a serialization test that creates `ManagedLlmProxyConfig` from an issued virtual key and asserts the JSON does not contain fake raw provider key, `Authorization`, `secretRef`, or provider request headers.

GREEN:

- Add `ManagedLlmProxyConfig` with `base_url`, `virtual_key`, `expires_at`, and `scope`.
- Add construction helper from `IssuedVirtualKey`.
- Derive serialization only for safe fields.

Refactor:

- Do not introduce provider proxy/router types in this task.

Acceptance:

- FC-DVKC-004 passes.

Evidence:

- `docs/verification/diri-virtual-key-credentials/sdd-tdd-task-report.md#t-003`
- `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-004`

### T-004: Reject raw provider key propagation for cross-module destinations

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-1, FR-4, FR-5 |
| Component spec | `specs/virtual-key-credentials/README.md` |
| Files | `crates/homie-llm/src/lib.rs`, `crates/homie-llm/tests/virtual_key.rs` |
| Functional cases | FC-DVKC-005 |

Objective:

- Provide a small public policy that later remote/MCP/agent/log callers can use before constructing payloads.

RED:

- Add tests that pass a fake raw provider key to remote node, MCP, managed agent config, and log/event destinations and expect fail-closed rejection.
- Add tests that virtual-key/proxy payloads are allowed.
- Assert error strings do not contain the fake raw provider key.

GREEN:

- Add `CredentialDestination`.
- Add `CredentialPropagationPolicy` or equivalent helper.
- Add `VirtualKeyError::RawProviderKeyForbidden` or equivalent error.
- Keep detection explicit and conservative for this first stage.

Refactor:

- Do not pull in regex or observability dependencies; this is a contract guard, not a global redaction engine.

Acceptance:

- FC-DVKC-005 passes.

Evidence:

- `docs/verification/diri-virtual-key-credentials/sdd-tdd-task-report.md#t-004`
- `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-005`

### T-005: Verification and evidence closeout

| Field | Value |
|-------|-------|
| Status | todo |
| Source requirement | FR-5 |
| Component spec | `docs/development/quality-gates.md` |
| Files | `docs/verification/diri-virtual-key-credentials/*` |
| Functional cases | FC-DVKC-007 |

Objective:

- Execute focused tests and local quality gates, then record honest evidence.

RED:

- Verification report is incomplete until it lists commands, exit codes, status, and residual risk.

GREEN:

- Run focused tests and document gates.
- Run format/check/diff gates where possible.
- Record blocked or partial results without weakening claims.

Acceptance:

- FC-DVKC-007 has evidence.
- Release readiness report identifies any non-lane workspace failures.

Evidence:

- `docs/verification/diri-virtual-key-credentials/functional-verification-report.md`
- `docs/verification/diri-virtual-key-credentials/release-readiness-report.md`
- `docs/verification/diri-virtual-key-credentials/code-review-report.md`
- `docs/verification/diri-virtual-key-credentials/e2e-report.md`


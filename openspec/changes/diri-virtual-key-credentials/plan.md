# Diri Virtual Key Credentials OpenSpec Plan

> Change ID: `diri-virtual-key-credentials`  
> Beads: `homie-e1s`  
> Lane: `lane-foundation-security`  
> Source PRD: `prd-spec/features/diri-virtual-key-credentials/2026-08-07-diri-virtual-key-credentials-design.md`  
> Functional cases: `docs/verification/diri-virtual-key-credentials/functional-cases.md`

## 1. Status

`in_progress`

This change is the first foundation-security slice for Diri/Homie credential parity. It updates the long-lived credential contract and adds the minimum `homie-llm` implementation needed to prove:

- virtual keys are scoped, expirable and revocable;
- managed agents receive virtual key proxy config, not provider raw keys;
- raw provider key propagation to remote/MCP/agent/log/event destinations is rejected.

## 2. Goals

| Goal | PRD requirements | Functional cases |
|------|------------------|------------------|
| G-1 Strengthen credential component contract | FR-1 | FC-DVKC-006 |
| G-2 Preserve virtual key fail-closed lifecycle | FR-2, FR-5 | FC-DVKC-001, FC-DVKC-002, FC-DVKC-003 |
| G-3 Define secretless managed agent proxy config | FR-3, FR-5 | FC-DVKC-004 |
| G-4 Reject raw provider key propagation to cross-module destinations | FR-1, FR-4, FR-5 | FC-DVKC-005 |
| G-5 Produce local verification and release evidence | FR-5 | FC-DVKC-007 |

## 3. Non-Goals

- No OpenAI-compatible HTTP endpoint implementation.
- No provider HTTP forwarding, streaming, usage accounting, or pricing.
- No encrypted secret envelope format implementation.
- No remote node, MCP automation, runtime, CLI, storage, or observability code changes.
- No edits outside the user-approved lane scope.

## 4. Component Impact

| File or module | Change | Reason |
|----------------|--------|--------|
| `specs/virtual-key-credentials/README.md` | Update | Source of truth for cross-spec raw-key gates |
| `crates/homie-llm/src/lib.rs` | Update | Add minimum public credential contract API |
| `crates/homie-llm/tests/virtual_key.rs` | Update | Add RED/GREEN contract tests |
| `docs/verification/diri-virtual-key-credentials/*` | Add | dev-loop evidence |
| `openspec/changes/diri-virtual-key-credentials/*` | Add | OpenSpec plan/tasks/alignment |

## 5. Architecture

```text
provider raw key
  -> local Homie credential custody only
  -> provider resolver short-lived memory only
  -> never serialized to remote/MCP/agent/log/event/context/memory/metrics/report

session spawn
  -> homie-llm issues virtual key
  -> ManagedLlmProxyConfig { base_url, virtual_key, expires_at, scope }
  -> managed agent calls local proxy using virtual key
  -> proxy validates scope/model/expiry/revoke before provider access
```

First-stage API boundaries:

- `InMemoryVirtualKeyStore`: existing lifecycle store.
- `ManagedLlmProxyConfig`: agent-visible secretless config.
- `CredentialDestination`: cross-module destination classification.
- `CredentialPropagationPolicy`: fail-closed raw provider key guard for payload construction.

## 6. Security Gates

| Gate | Rule | Verification |
|------|------|--------------|
| No raw key to remote | Remote payload cannot contain provider raw key | FC-DVKC-005 |
| No raw key to MCP | MCP tool args/result cannot contain provider raw key | FC-DVKC-005 |
| No raw key to agent config | Agent-visible config only contains virtual key/proxy URL | FC-DVKC-004, FC-DVKC-005 |
| No raw key to log/event | Log/event payload rejects provider raw key | FC-DVKC-005 |
| Secretless errors | Errors never echo presented or raw provider key | FC-DVKC-002, FC-DVKC-005 |

## 7. Verification Plan

Focused:

```bash
cargo test -p homie-llm
```

Document alignment:

```bash
rg -n "Cross-Spec Mandatory Gates|Raw Provider Key Forbidden Matrix|Diri Behavior Parity" specs/virtual-key-credentials/README.md
rg -n "FC-DVKC-00[1-6]" openspec/changes/diri-virtual-key-credentials/tasks.md openspec/changes/diri-virtual-key-credentials/alignment-report.md
```

Quality:

```bash
cargo fmt --all -- --check
cargo check --workspace
git diff --check
```

Results must be recorded under `docs/verification/diri-virtual-key-credentials/`.


# Diri Parity Dev Loop Dispatch Report

```yaml
change_id: diri-spec-dependency-analysis
status: dispatched
dispatched_at: 2026-08-07
source:
  - docs/verification/diri-module-inventory/spec-dependency-analysis.md
  - docs/research/diri-module-inventory.md
```

## Dispatch Summary

Four TRAE CLI workers were dispatched for the first dependency layer and adjacent spec-hardening lane. Each worker is constrained to dev-loop: PRD/spec review, functional cases, OpenSpec plan/tasks/alignment, SDD/TDD, verification evidence, and final report.

## Workers

| Worker | Agent ID | Lane | Beads | Scope |
|--------|----------|------|-------|-------|
| worker-storage | `019fdb8b-ed3a-7500-93dd-1f092bedfb60` | `lane-foundation-storage` | `homie-q7n` | `specs/storage-indexing`, `crates/homie-storage`, `prd-spec/features/diri-storage-indexing`, `openspec/changes/diri-storage-indexing`, `docs/verification/diri-storage-indexing` |
| worker-security | `019fdb8c-5a93-72a1-9f47-663157c34b89` | `lane-foundation-security` | `homie-e1s` | `specs/virtual-key-credentials`, `crates/homie-llm`, `prd-spec/features/diri-virtual-key-credentials`, `openspec/changes/diri-virtual-key-credentials`, `docs/verification/diri-virtual-key-credentials` |
| worker-observability | `019fdb8c-d951-7202-8c4e-217ee5287543` | `lane-foundation-observability` | `homie-wm7` | `specs/observability`, `prd-spec/features/diri-observability`, `openspec/changes/diri-observability`, `docs/verification/diri-observability` |
| worker-agent | `019fdb8d-6370-7e32-954d-ca5aaafa7315` | `lane-agent` | `homie-v4b` | `specs/agent-adapter-contract`, `assets/agent-descriptors`, `crates/homie-agents`, `prd-spec/features/diri-agent-detection`, `openspec/changes/diri-agent-detection`, `docs/verification/diri-agent-detection` |

## Blocking Rules

- No worker may mark parity rows implemented without evidence.
- No worker may implement before PRD/spec, functional cases, OpenSpec, and alignment exist.
- No worker may edit outside its write scope.
- L1+ implementation that depends on L0 contracts must stop at spec/OpenSpec if the L0 contract is still incomplete.
- Production credentials, notarization, remote node access, or external services require explicit evidence gates and must be reported as blockers if unavailable.

## Immediate Supervision Checklist

| Check | Command / action |
|-------|------------------|
| Worker completion | `wait_agent` for one of the dispatched agent ids |
| Public boundary | `loopx --registry .loopx/registry.json check --scan-root /Users/bytedance/workspace/github/homie` |
| Inventory gates | `make module-inventory-check` and `make spec-diri-mapping-check` |
| Parity lock | `make parity-lock` |


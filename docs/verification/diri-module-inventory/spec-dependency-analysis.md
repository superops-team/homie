# Diri Component Spec Dependency Analysis

```yaml
change_id: diri-spec-dependency-analysis
status: ready_for_parallel_dev_loop_dispatch
source:
  - docs/research/diri-module-inventory.md
  - specs/README.md
  - docs/verification/diri-module-inventory/bingo-component-spec-review-report.md
```

## 1. Dependency Layers

| Layer | Specs | Why this layer |
|-------|-------|----------------|
| L0 Foundation contracts | `storage-indexing`, `virtual-key-credentials`, `observability` | Data, credential, and evidence rules are shared by most later modules. |
| L1 Core protocol/runtime | `runtime-supervisor`, `agent-adapter-contract`, `llm-proxy` | Runtime, agent status, and model proxy contracts define real behavior boundaries. |
| L2 Client/context/orchestration | `session-context-store`, `intent-orchestrator` | These consume L0/L1 state and define session/history/routing semantics. |
| L3 Automation/remote | `mcp-automation`, `remote-node-handoff`, `task-controller`, `memory-controller` | These rely on protocol/runtime/credential/context contracts. |
| L4 Product UI/release | `desktop-shell`, `packaging-updater` | UI and release gates consume every lower layer and require real screenshot/package evidence. |

## 2. Spec DAG

| Spec | Blocked by | Blocks | Parallel lane | Disjoint write scope |
|------|------------|--------|---------------|----------------------|
| `storage-indexing` | none | `runtime-supervisor`, `session-context-store`, `desktop-shell`, `remote-node-handoff`, `llm-proxy` | lane-foundation-storage | `specs/storage-indexing`, `prd-spec/features/diri-storage-indexing`, `openspec/changes/diri-storage-indexing`, storage tests only |
| `virtual-key-credentials` | none | `llm-proxy`, `remote-node-handoff`, `mcp-automation`, `observability` | lane-foundation-security | `specs/virtual-key-credentials`, `prd-spec/features/diri-virtual-key-credentials`, `openspec/changes/diri-virtual-key-credentials`, `crates/homie-llm` tests only |
| `observability` | none | all specs with logs/events/evidence | lane-foundation-observability | `specs/observability`, `prd-spec/features/diri-observability`, `openspec/changes/diri-observability`, observability docs/tests only |
| `runtime-supervisor` | `storage-indexing`, `observability`, `agent-adapter-contract` | `homie-client`, `desktop-shell`, `mcp-automation`, `remote-node-handoff` | lane-runtime | `specs/runtime-supervisor`, `prd-spec/features/diri-runtime-supervisor`, `openspec/changes/diri-runtime-supervisor`, `crates/homie-runtime` |
| `agent-adapter-contract` | `storage-indexing`, `observability` | `runtime-supervisor`, `desktop-shell`, `mcp-automation` | lane-agent | `specs/agent-adapter-contract`, `prd-spec/features/diri-agent-detection`, `openspec/changes/diri-agent-detection`, `crates/homie-agents`, `assets/agent-descriptors` |
| `llm-proxy` | `virtual-key-credentials`, `storage-indexing`, `observability` | `remote-node-handoff`, `usage-accounting`, `desktop-shell` | lane-llm | `specs/llm-proxy`, `prd-spec/features/diri-usage-accounting`, `openspec/changes/diri-usage-accounting`, `crates/homie-llm` |
| `session-context-store` | `storage-indexing`, `runtime-supervisor`, `observability` | `intent-orchestrator`, `desktop-shell`, `mcp-automation` | lane-context | `specs/session-context-store`, `prd-spec/features/diri-navigation-history`, `openspec/changes/diri-navigation-history`, `crates/homie-context` |
| `intent-orchestrator` | `session-context-store`, `runtime-supervisor`, `agent-adapter-contract` | `mcp-automation`, `task-controller`, `desktop-shell` | lane-orchestrator | `specs/intent-orchestrator`, `prd-spec/features/diri-intent-orchestrator`, `openspec/changes/diri-intent-orchestrator`, `crates/homie-orchestrator` |
| `mcp-automation` | `runtime-supervisor`, `session-context-store`, `intent-orchestrator`, `virtual-key-credentials` | `desktop-shell`, CLI automation | lane-automation | `specs/mcp-automation`, `prd-spec/features/diri-mcp-automation`, `openspec/changes/diri-mcp-automation`, `crates/homie-cli`, `crates/homie-orchestrator` MCP files |
| `remote-node-handoff` | `runtime-supervisor`, `llm-proxy`, `virtual-key-credentials`, `storage-indexing` | `desktop-shell`, `packaging-updater`, usage fleet | lane-remote | `specs/remote-node-handoff`, `prd-spec/features/diri-remote-node-handoff`, `openspec/changes/diri-remote-node-handoff`, `crates/homie-remote` |
| `task-controller` | `intent-orchestrator`, `session-context-store` | `mcp-automation`, `desktop-shell` | lane-task | `specs/task-controller`, `prd-spec/features/diri-task-controller`, `openspec/changes/diri-task-controller`, `crates/homie-task` |
| `memory-controller` | `session-context-store`, `observability`, `virtual-key-credentials` | `intent-orchestrator`, `desktop-shell` | lane-memory | `specs/memory-controller`, `prd-spec/features/diri-memory-controller`, `openspec/changes/diri-memory-controller`, `crates/homie-memory` |
| `desktop-shell` | all L0-L3 contracts relevant to active surface | user-facing parity closeout | lane-ui | `specs/desktop-shell`, UI PRDs/OpenSpecs, `crates/homie-app`, `crates/homie-ui`, screenshot evidence |
| `packaging-updater` | runtime/app/CLI/remote/update contracts | release closeout | lane-release | `specs/packaging-updater`, release PRDs/OpenSpecs, `crates/homie-updater`, `scripts/package`, package evidence |

## 3. Dispatch Policy

- Dispatch L0 foundation lanes first because their contracts constrain later work.
- L1 may begin only after the corresponding L0 mapping tables exist.
- L2/L3 workers may prepare PRD/spec/OpenSpec skeletons, but implementation must not begin until blocked-by gates are satisfied.
- L4 UI/release workers may only work on verification gaps or skeleton planning until lower layers complete.
- Each worker must follow dev-loop: spec review, functional cases, OpenSpec plan/tasks/alignment, SDD/TDD, verification, two review passes, E2E evidence.

## 4. Initial Parallel Batch

| Worker | Assignment | Reason |
|--------|------------|--------|
| worker-storage | `storage-indexing` | Highest shared dependency; disjoint write scope. |
| worker-security | `virtual-key-credentials` | High-risk credential gate; blocks remote/MCP/LLM. |
| worker-observability | `observability` | Required by evidence/logging for every module. |
| worker-agent | `agent-adapter-contract` | Can proceed in parallel with L0 as spec hardening only; implementation waits on storage/observability. |

## 5. Stop Conditions

- Stop a worker if it needs to edit outside its write scope.
- Stop a worker if it attempts implementation before PRD/spec/OpenSpec/functional cases are complete.
- Stop a worker if it tries to mark parity rows implemented without evidence.
- Stop a worker if it needs production credentials, network services, notarization, or remote node access not available locally.


# Diri 7ba3407 全功能对齐重基线 OpenSpec Alignment Report

```yaml
change_id: diri-7ba3407-parity-rebaseline
report_type: openspec-alignment
status: pass
beads: homie-t3u
source_prd: prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md
capability_matrix: docs/research/diri-7ba3407-capability-matrix.md
openspec_plan: openspec/changes/diri-7ba3407-parity-rebaseline/plan.md
openspec_tasks: openspec/changes/diri-7ba3407-parity-rebaseline/tasks.md
```

## 1. Requirement To Task Mapping

| PRD requirement | Priority | OpenSpec task | Capability spec | Component spec | Verification | Status |
|-----------------|----------|---------------|-----------------|----------------|--------------|--------|
| FR-01 冻结基线与完成判定 | P0 | T-000, T-502 | `diri-parity-governance` | observability + all mappings | matrix/status validator + final gate | mapped |
| FR-02 协议与独立 Runtime Client | P0 | T-101 | `runtime-client-boundary` | runtime-client-transport, runtime-supervisor | UDS/reconnect/attachment/cross-entry E2E | mapped |
| FR-03 Runtime/PTY/Holder 生命周期 | P0 | T-102 | `runtime-session-lifecycle` | runtime-supervisor | holder/process/resource/migrate/shutdown E2E | mapped |
| FR-04 Agent 启动/检测/Resume/权限 | P0 | T-102, T-402 | `runtime-session-lifecycle` | agent-adapter, credentials | fake/real agent spawn and virtual-key tests | mapped |
| FR-05 核心模型/Storage/事实源 | P0 | T-103 | `runtime-client-boundary` | storage-indexing | migrations/repositories/restart recovery | mapped |
| FR-06 Desktop Workbench/Sidebar | P0 | T-201 | `desktop-product-surface` | desktop-shell | real app interaction/screenshot | mapped |
| FR-07 Terminal 完整交互 | P0 | T-202 | `desktop-product-surface` | desktop-shell, runtime-supervisor | live PTY/grid/selection/scrollback/visual/perf | mapped |
| FR-08 导航/设置/macOS 原生能力 | P1 | T-203 | `desktop-product-surface` | desktop-shell, storage-indexing | file index/navigation/native action E2E | mapped |
| FR-09 Inspector/Git/Artifact/PR/Port | P1 | T-204 | `desktop-product-surface` | desktop-shell, runtime-supervisor | diff/worktree/artifact/PR/port E2E | mapped |
| FR-10 完整 CLI | P0 | T-301 | `automation-surface` | runtime-client-transport, mcp-automation | grammar fixtures + runtime CLI E2E | mapped |
| FR-11 MCP/Lineage/Browser/Test | P0 | T-302 | `automation-surface` | mcp-automation, context, orchestrator | schema/permission/transcript/sidecar E2E | mapped |
| FR-12 Remote Node/Account/Handoff | P1 | T-401 | `remote-usage-release` | remote-node-handoff, credentials | two-node failure/lease/service E2E | mapped |
| FR-13 Usage/LLM Proxy/Virtual Key | P0 | T-402 | `remote-usage-release` | llm-proxy, credentials, storage, observability | fake provider SSE/usage/no-leak E2E | mapped |
| FR-14 Context/Memory/Task/Orchestrator | P1 | T-403 | `homie-control-plane` | context, memory, task, orchestrator | durable cross-entry workflow | mapped |
| FR-15 Updater/Packaging/性能 | P0 | T-501 | `remote-usage-release` | packaging-updater, observability | universal/sign/notary/install/rollback/perf | mapped |
| FR-16 SDD/TDD/Evidence/最终准出 | P0 | T-000, all child tasks, T-502 | `diri-parity-governance`, `diri-parity-delivery-waves` | observability + all | RED/GREEN/evidence/final matrix validator | mapped |

## 2. Component Spec Impact

| Component spec | Impact | Evidence | Status |
|----------------|--------|----------|--------|
| `specs/runtime-client-transport/README.md` | new | endpoint client/transport contract added | updated |
| `specs/runtime-supervisor/README.md` | yes | Diri 7ba3407 daemon/lifecycle amendment | updated |
| `specs/agent-adapter-contract/README.md` | yes | manifest-driven runtime amendment | updated |
| `specs/desktop-shell/README.md` | yes | service-backed UI and evidence amendment | updated |
| `specs/storage-indexing/README.md` | yes | service ownership and durable facts amendment | updated |
| `specs/mcp-automation/README.md` | yes | exact catalog/lineage/sidecar amendment | updated |
| `specs/remote-node-handoff/README.md` | yes | node/handoff consistency amendment | updated |
| `specs/llm-proxy/README.md` | yes | HTTP/SSE/usage amendment | updated |
| `specs/virtual-key-credentials/README.md` | yes | envelope and propagation amendment | updated |
| `specs/packaging-updater/README.md` | yes | release/update/performance amendment | updated |
| `specs/session-context-store/README.md` | yes | product integration amendment | updated |
| `specs/memory-controller/README.md` | yes | durable source/permission amendment | updated |
| `specs/task-controller/README.md` | yes | shared task integration amendment | updated |
| `specs/intent-orchestrator/README.md` | yes | typed route execution amendment | updated |
| `specs/observability/README.md` | yes | evidence vocabulary and scope amendment | updated |
| `specs/README.md` | yes | canonical baseline and client component index | updated |

## 3. Beads Alignment

| Bead | Title | Status | Spec ID | Expected state |
|------|-------|--------|---------|----------------|
| `homie-t3u` | Diri 7ba3407 parity rebaseline and executable delivery plan | closed | `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md` | closed after Wave 0 spec/evidence gates passed |
| `homie-h7n` | Reference parity V1 product spec | open historical parent | `reference-parity-v1` | retained as historical planning; not the new implementation entry |
| `homie-h7n.1`..`homie-h7n.5` | Historical parity group gaps | open | `diri-parity-child-tasks` | child wave Beads explicitly assume remaining ownership before status changes |

## 4. Coverage Checks

| Check | Result | Evidence |
|-------|--------|----------|
| Every PRD FR has at least one task | pass | Section 1 maps FR-01..FR-16 |
| Every task has a test or verification path | pass | `tasks.md` Task Contracts |
| Every affected component spec is updated or explicitly marked no impact | pass | Section 2 |
| Every proposal capability has a delta spec | pass | eight directories under `specs/` |
| No unowned security/credential impact remains | pass | credential, remote, MCP, proxy and package specs |
| Beads state matches delivery state | pass | `homie-t3u` is in progress |
| OpenSpec strict validation passes | pass | `openspec validate diri-7ba3407-parity-rebaseline --strict` |
| 16-dimension spec review passes | pass | `docs/verification/diri-7ba3407-parity-rebaseline/spec-review-report.md` |

## 5. Risks And Follow-Ups

| Risk | Source | Mitigation | Follow-up Bead |
|------|--------|------------|----------------|
| Runtime/client rewrite affects all consumers | FR-02 | T-101 first; cross-entry fixture before deleting shortcut | Wave 1A Bead |
| Holder regression already exists | FR-03 | T-102 RED starts from current detached failures | Wave 1B Bead |
| App monolith creates concurrent edit conflicts | FR-06..FR-09 | child PRDs assign surface modules and avoid overlapping edits | Wave 2 Beads |
| Secret envelope details undecided | FR-13 | focused security/package research before T-402 code | Wave 4B Bead |
| Release credentials may be unavailable | FR-15 | blocked gate, no ad-hoc substitution | Wave 5A Bead |
| Historical illegal statuses may be misread | FR-16 | new final validator ignores/supersedes invalid historical evidence | Wave 0/T-502 |

## 6. Gate Decision

Decision: pass

Reason:

- PRD, component specs and tasks are fully mapped.
- Strict OpenSpec validation passed.
- The 16-dimension spec review passed with no blocking finding.
- This decision approves the Wave 0 planning artifacts only; implementation parity remains partial.

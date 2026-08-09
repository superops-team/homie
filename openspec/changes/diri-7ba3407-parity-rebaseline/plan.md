# Diri 7ba3407 全功能对齐重基线 OpenSpec Plan

> Change ID: `diri-7ba3407-parity-rebaseline`  
> Source PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`  
> Beads: `homie-t3u`  
> Status: reviewed

## 1. Summary

本计划固定内嵌 Diri commit `7ba3407`，把当前碎片化 parity 工作重组为 15 个依赖有序的纵向任务。Wave 0 只修复需求、合同、追踪和证据基线；Wave 1 至 Wave 5 由独立 child change 实施，每个 child change 必须有自己的 Bead、中文 PRD、OpenSpec、RED/GREEN 测试和 evidence。

核心架构是独立 runtime daemon + endpoint client。runtime 拥有 live session/PTY/holder/events，SQLite 拥有 durable facts，UI/CLI/MCP/remote 通过同一 client/service 边界访问。错误 shortcut 在替代路径通过后直接删除，不保留兼容 fallback。

## 2. Goals

| Goal | Source requirement | Acceptance |
|------|--------------------|------------|
| G-1 固定且可审计的 parity 基线 | FR-01, FR-16 | 20 模块、遗漏能力、状态和证据规则全部可追踪 |
| G-2 独立 runtime/client 架构 | FR-02, FR-03 | app/CLI/MCP 连接同一 daemon，holder/reconnect/recovery 通过 |
| G-3 manifest 驱动 agent 生命周期 | FR-04 | fake/real agent spawn、status、resume 和 permission E2E |
| G-4 单一 durable facts 边界 | FR-05 | UI 无 storage 依赖，migration/repository/recovery 通过 |
| G-5 完整本地桌面产品 | FR-06..FR-09 | Workbench/Terminal/Navigation/Inspector 真实交互与截图通过 |
| G-6 完整自动化入口 | FR-10, FR-11 | CLI/MCP schemas、lineage、browser/test 和 streaming E2E |
| G-7 远端和 usage/proxy | FR-12, FR-13 | node/handoff/fleet/proxy/virtual-key E2E |
| G-8 Homie 扩展接入 | FR-14 | context/memory/task/orchestrator 有真实纵向闭环 |
| G-9 可交付 release | FR-15, FR-16 | universal signed/notarized package、update/rollback/perf 通过 |

## 3. Non-Goals

- 本 change 不直接修改产品代码、数据库 schema 或依赖。
- 不从 Diri main 持续吸收新功能。
- 不保持当前 embedded runtime、UI direct-storage 或 advertised unsupported tools 的兼容性。
- 不以真实 provider key、真实用户 HOME 或生产远端作为强制单元/集成测试前提。
- 不在 master plan 中预先决定 child wave 的低层实现细节；每个 child wave 通过独立 PRD 和 package/security research 决策。

## 4. Affected Component Specs

| Component spec | Impact | Required update |
|----------------|--------|-----------------|
| `specs/runtime-client-transport/README.md` | new | control/data transport、reconnect、attachment、backpressure |
| `specs/runtime-supervisor/README.md` | yes | daemon、agent spawn、holder、resource、migration、shutdown |
| `specs/agent-adapter-contract/README.md` | yes | manifest 驱动 runtime 和 effective config |
| `specs/desktop-shell/README.md` | yes | service-backed UI、状态矩阵、interaction/screenshot gate |
| `specs/storage-indexing/README.md` | yes | service-owned repository、恢复事实、migration gate |
| `specs/mcp-automation/README.md` | yes | exact schemas、完整 tools、lineage、sidecar |
| `specs/remote-node-handoff/README.md` | yes | node server/account/handoff/network E2E |
| `specs/llm-proxy/README.md` | yes | HTTP/SSE/provider/usage |
| `specs/virtual-key-credentials/README.md` | yes | envelope、scope、跨进程传播禁止 |
| `specs/packaging-updater/README.md` | yes | dependency closure、release/update/perf |
| `specs/session-context-store/README.md` | yes | runtime/MCP/UI context integration |
| `specs/memory-controller/README.md` | yes | durable source-attributed memory |
| `specs/task-controller/README.md` | yes | durable shared task lifecycle |
| `specs/intent-orchestrator/README.md` | yes | typed route execution and audit |
| `specs/observability/README.md` | yes | legal status vocabulary and scope-aware evidence |

## 5. Implementation Scope And Dependency Order

| Task | Child change | Primary files/modules | Dependency | Relative effort |
|------|--------------|-----------------------|------------|-----------------|
| T-000 | `diri-7ba3407-parity-rebaseline` | PRD, matrix, component specs, OpenSpec, verification | none | S |
| T-101 | `diri-runtime-daemon-client-transport` | `homie-proto`, `homie-client`, `homie-runtime`, daemon binary, transport tests | T-000 | XL |
| T-102 | `diri-agent-session-runtime` | `homie-runtime`, `homie-agents`, holder, process/resource tests | T-101 | XL |
| T-103 | `diri-storage-core-facts` | `homie-storage`, migrations, repositories, recovery tests | T-101 | L |
| T-201 | `diri-desktop-workbench-sidebar` | `homie-app`, `homie-ui`, client projections, UI E2E | T-101, T-103 | L |
| T-202 | `diri-terminal-interaction` | `homie-term`, app terminal element, attachment/scrollback tests | T-101, T-102 | L |
| T-203 | `diri-navigation-settings-native` | app navigation/settings, macOS bridge, file index | T-101, T-103 | L |
| T-204 | `diri-inspector-git-artifacts` | app inspector, runtime git/artifact/PR/port services | T-101, T-103 | L |
| T-301 | `diri-cli-complete-surface` | `homie-cli`, client facade, grammar fixtures | T-101, T-102, T-103 | M |
| T-302 | `diri-mcp-browser-automation` | MCP stdio, orchestrator/context, sidecar, package closure | T-102, T-204 | XL |
| T-401 | `diri-remote-node-handoff` | `homie-remote`, node binary/service, runtime, checkpoint/lease | T-101, T-102, T-103 | XL |
| T-402 | `diri-usage-llm-proxy` | `homie-llm`, credential, storage, usage UI/CLI | T-102, T-103 | XL |
| T-403 | `homie-control-plane-integration` | context/memory/task/orchestrator + app/CLI/MCP | T-102, T-103, T-302 | L |
| T-501 | `diri-updater-packaging-performance` | updater, scripts/package, scripts/release, app update UI | T-201..T-402 | XL |
| T-502 | `diri-7ba3407-final-parity-gate` | all verification suites and final report | T-501, T-403 | M |

Relative effort:

- S: 1-2 engineering days.
- M: 3-5 engineering days.
- L: 6-10 engineering days.
- XL: 10-15 engineering days.

These are planning ranges, not delivery commitments. T-101/T-102/T-103 are the critical path. After they pass, T-201/T-202/T-203/T-204, T-301, T-401 and T-402 can be assigned in parallel without sharing file ownership blindly.

## 6. File Ownership For Child Changes

| Child change | Exclusive primary owner | Shared contracts requiring coordination |
|--------------|-------------------------|-----------------------------------------|
| T-101 | `crates/homie-client`, runtime transport/daemon entry | `homie-proto`, runtime supervisor |
| T-102 | holder, process, agent launch/reducer integration | `homie-proto`, storage session records |
| T-103 | migrations/repositories | `homie-proto` durable models |
| T-201..T-204 | distinct `homie-app` modules by surface | client projections, `homie-ui` tokens |
| T-301 | CLI command grammar/output | client typed methods |
| T-302 | MCP handlers/sidecar | orchestrator/context, package closure |
| T-401 | remote/node/handoff | runtime client, credential policy |
| T-402 | LLM proxy/usage/credential | storage, app usage projection |
| T-403 | control-plane domain services | storage, MCP/app entry points |
| T-501 | updater/package/release scripts | all binary/resource artifact lists |

Each child PRD must refine this ownership to exact files before implementation. Concurrent waves must not edit the same monolithic `homie-app/src/main.rs` sections without first extracting already-approved responsibility modules in their own RED/GREEN change.

## 7. Data, State, And Security Impact

| Topic | Impact | Handling |
|-------|--------|----------|
| Credential / virtual key | High | encrypted envelope; scoped short-lived virtual key; raw-key propagation denial |
| Session live state | High | runtime/holder sole owner; storage only recovery facts |
| SQLite | High | forward-only migrations, transactional repositories, schema-too-new fail closed |
| Terminal output | High volume | offset log + bounded attachment; no SQLite blobs |
| Remote handoff | High | quarantine, content hashes, operation ID, lease commit |
| Context/memory/task | Sensitive | safe references, source attribution, permission filters |
| MCP/browser | High privilege | trusted identity, lineage matrix, bounded sidecar |
| Update/package | Supply chain | HTTPS allowlist, SHA256, Team ID, codesign/spctl/notary/staple |
| Observability | Sensitive | allowlisted fields and legal gate vocabulary only |

## 8. TDD Strategy

Every child change follows:

1. Write a contract/fixture or behavior test that fails for the current shortcut.
2. Run the focused command and record the expected failure.
3. Implement the smallest vertical production path.
4. Run focused unit and integration tests.
5. Run real process/UI/network/package E2E required by the child PRD.
6. Remove the superseded shortcut.
7. Run workspace/security gates.
8. Record evidence and update capability status only after all required gates pass.

Tests must assert observable behavior, protocol frames, state transitions, durable facts or rendered output. Source substring assertions cannot be the primary gate.

## 9. Test Strategy

| Layer | Required cases | Command or evidence |
|-------|----------------|---------------------|
| Spec | artifact completeness and 16-dimension review | `openspec status`, `openspec validate`, verification reports |
| Unit | reducers, parsers, permissions, migrations, pricing, redaction | focused `cargo test -p <crate>` |
| Integration | UDS/client/holder/storage/fake provider/fake node | child change integration suites |
| Process E2E | app/CLI/MCP shared daemon and restart/recovery | `tests/e2e` and evidence commands |
| UI | interaction, first-frame, keyboard/focus, screenshots | GPUI test harness + real screenshot report |
| Remote | two-node loopback handoff/failure matrix | remote E2E evidence |
| Security | credential propagation, MCP lineage, update trust | security report and hook/audit commands |
| Release | universal bundle/install/update/rollback | package release-readiness report |
| Performance | packaged budgets with samples | performance report |

## 10. Release Gates

Wave 0:

- `openspec status --change diri-7ba3407-parity-rebaseline` reports 4/4.
- `openspec validate diri-7ba3407-parity-rebaseline --strict` passes.
- PRD FR-01..FR-16 map to tasks and verification.
- component specs cite the new baseline.
- spec review has no blocking finding.
- `make parity-lock`, inventory/mapping checks and `git diff --check` pass.

Final parity:

- every required capability matrix row is `implemented`;
- no public protocol/MCP/CLI catalog exposes an unimplemented operation;
- `cargo fmt`, check, clippy and workspace tests pass;
- app/CLI/MCP/runtime/remote/updater real E2E pass;
- security, package, notarization, screenshot and performance gates pass;
- no `not_run`, `blocked`, `partial`, `fail` or illegal evidence status remains in required final gates;
- Beads state and release-readiness evidence match.

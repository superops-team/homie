# Homie Reference Parity V1 OpenSpec Plan

> Change ID: `reference-parity-v1`  
> Source PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`  
> Beads: `homie-h7n`  
> Status: draft

## 1. Summary

本变更把 `/Users/bytedance/workspace/github/reference` 的当前首版产品功能和产品设计转化为 Homie 的完整 V1 parity 执行计划。范围覆盖本地 agent orchestration、Reference 对齐的 GPUI 工作台、终端渲染、session 生命周期、worktree、history、artifact/port/PR、usage、CLI、hook/notify、MCP、remote/node/handoff、packaging、updater、性能和安全门禁。

实现必须遵守 Homie 架构边界：Rust 是业务事实源，UI 通过 client/protocol 访问 runtime，SQLite 是本地事实源，真实 provider key 只在 Homie credential custody 中出现，managed agent 只能拿 virtual key 和 local proxy URL。本计划允许分阶段实现，但不允许用阶段性完成替代 V1 准出。

## 2. Goals

| Goal | Source requirement | Acceptance |
|------|--------------------|------------|
| G-1 Reference coverage gate | FR-1 | `docs/research/reference-feature-coverage.md` 无未解释 `missing`/`partial` |
| G-2 Homie 架构承载 parity | FR-2 | crate 分层、protocol、storage、runtime、UI 依赖方向符合规范 |
| G-3 agent catalog parity | FR-3 | Reference 19 个 manifests 全部可加载、校验、检测状态 |
| G-4 session/runtime parity | FR-4 | spawn/list/attach/input/resize/archive/hibernate/history 等 lifecycle E2E 通过 |
| G-5 protocol/event parity | FR-5 | Reference methods/events 加 Homie LLM/profile/task/memory 方法全部有 contract tests |
| G-6 terminal parity | FR-6 | grid/input/scrollback/selection/find/resize fixture 和真实 PTY 验证通过 |
| G-7 UI design parity | FR-7 | sidebar、terminal、surfaces、settings、inspector、menu bar screenshot/fidelity gate 通过 |
| G-8 worktree/project parity | FR-8 | create/list/remove/overview/cleanup safety tests 和 E2E 通过 |
| G-9 history/resume parity | FR-9 | Claude/Codex history scan/resume 真实验证通过 |
| G-10 artifact/browser parity | FR-10 | artifact/port/PR/browser/test_run contract 和 E2E 通过 |
| G-11 usage parity | FR-11 | proxy/transcript/node usage 汇总与 fixture 误差在预算内 |
| G-12 Homie LLM custody | FR-12 | real key 不进入 agent env/log/event，virtual key scope/revoke 测试通过 |
| G-13 remote/node/handoff parity | FR-13 | remote spawn、node account、move/fork/handoff harness 通过 |
| G-14 automation parity | FR-14 | CLI、hook/notify、MCP tools E2E 通过 |
| G-15 resource/perf strategy | FR-15 | active state、governor、resident renderer、idle/no-timer 证据通过 |
| G-16 packaging/updater parity | FR-16 | signed/notarized bundle、DMG、update zip、old-to-new 验证通过 |
| G-17 packaged perf gate | FR-17 | normal/large packaged perf gate 通过并记录证据 |
| G-18 storage/preferences | FR-18 | SQLite migration/repository/preferences 全覆盖 |
| G-19 security | FR-19 | secret scan、pre-commit、redaction、capability diff 通过 |
| G-20 Homie context/memory/task/orchestration | FR-20 | session context、task、memory candidate、intent routing 接入工作台和 MCP |

## 3. Non-Goals

- 不迁移或接管 Reference 现有运行中 session。
- 不保持 Reference socket、state.json、Swift package 或 bundle 兼容。
- 不复制 Reference 的实现代码作为 Homie 长期架构捷径。
- 不绕过 Homie virtual key、credential custody、SQLite 和 security baseline。
- 不把 P0/P1/P2 阶段性落地当成 V1 release 完成。

## 4. Affected Component Specs

| Component spec | Impact | Required update |
|----------------|--------|-----------------|
| `specs/desktop-shell/README.md` | yes | 定义 Reference parity UI surfaces、窗口、sidebar、terminal、settings、inspector、menu bar |
| `specs/runtime-supervisor/README.md` | yes | 定义 PTY、holder-equivalent、output log、status、resource governor |
| `specs/agent-adapter-contract/README.md` | yes | 定义 Reference manifest schema、status authority、approval/resume/hook |
| `specs/llm-proxy/README.md` | yes | 定义 OpenAI-compatible proxy、metrics、usage、safe errors |
| `specs/virtual-key-credentials/README.md` | yes | 定义 provider key custody、virtual key scope、remote/node key policy |
| `specs/session-context-store/README.md` | yes | 定义 session context、history、lineage、artifact summary |
| `specs/storage-indexing/README.md` | yes | 增补 parity schema、preferences、output index、migration |
| `specs/observability/README.md` | yes | 定义 logs/events/metrics/redaction/evidence |
| `specs/task-controller/README.md` | yes | 定义 task state 和 agent claim/update/return |
| `specs/memory-controller/README.md` | yes | 定义 memory write candidate 和 redaction |
| `specs/intent-orchestrator/README.md` | yes | 定义 palette/new-agent/MCP spawn routing |
| `specs/packaging-updater/README.md` | yes | 新增 bundle、signing、notarization、updater trust model |
| `specs/remote-node-handoff/README.md` | yes | 新增 remote hosts、node、accounts、handoff |
| `specs/mcp-automation/README.md` | yes | 新增 CLI、MCP tools、hook/notify、browser/test_run |

组件 spec 更新是实现前置条件。当前 change 的交付物是 PRD/OpenSpec/evidence，组件 spec 本体可作为后续第一批实现任务落地，但不得在代码实现前跳过。

## 5. Implementation Scope

| Area | Files/modules | Reason |
|------|---------------|--------|
| Product specs | `prd-spec/features/reference-parity-v1/*` | Reference parity 需求事实源 |
| Execution spec | `openspec/changes/reference-parity-v1/*` | 需求到任务、组件和验证映射 |
| Verification | `docs/verification/reference-parity-v1/*` | spec review、alignment、后续 release readiness |
| Research coverage | `docs/research/reference-feature-coverage.md` | 记录 Reference 功能覆盖状态 |
| Component specs | `specs/*/README.md` | 后续实现前必须更新长期合同 |
| Rust crates | `crates/homie-*` | 后续按 OpenSpec tasks 实现 |
| Assets/scripts/tests | `assets/`, `scripts/`, `tests/` | UI assets、packaging、quality gates 和 fixtures |

## 6. Data, State, and Security Impact

| Topic | Impact | Handling |
|-------|--------|----------|
| Credential / virtual key | high | 真实 key 只在 Homie secret envelope，agent env 只注入 virtual key/proxy URL |
| Session context | high | session、lineage、events、summary、artifact、usage 进入 SQLite/context store |
| Memory | medium | 首版只写候选和来源，不写 raw prompt/secret/tool args |
| Task state | medium | session 可 claim/update/return task，任务状态不归属单一 agent |
| Observability | high | logs/events/metrics/trace 全部 safe fields，失败产生 evidence |
| Remote/node | high | token owner-only，handoff checkpoint 不含 credential，失败 fail closed |
| Updater | high | codesign、Team ID、bundle id、spctl、version、HTTPS host pin 全部验证 |
| Browser/test artifacts | medium | screenshot 写文件路径，不内联 bytes；console/log 脱敏 |

## 7. Test Strategy

| Layer | Required cases | Command or evidence |
|-------|----------------|---------------------|
| Spec | PRD self-review、coverage matrix、component impact、OpenSpec alignment | `docs/verification/reference-parity-v1/spec-review-report.md` |
| Unit | protocol/grid/input/manifest/storage/virtual-key/fuzzy/updater | `cargo test --workspace --lib` |
| Integration | fake runtime、real PTY、LLM fake provider、MCP、hook/notify、output replay | `cargo test --workspace --tests` |
| E2E/manual | app first frame、session lifecycle、history resume、worktree、notification、remote/node | `docs/verification/reference-parity-v1/e2e-report.md` |
| UI | deterministic preview screenshots、min/narrow windows、surface fidelity | `docs/verification/reference-parity-v1/ui-fidelity-report.md` |
| Performance | packaged normal/large memory and idle CPU, resize churn | `docs/verification/reference-parity-v1/perf-report.md` |
| Security | pre-commit、secret scan、capability diff、redaction tests、updater validation | `docs/verification/reference-parity-v1/security-report.md` |
| Release | signed/notarized app、DMG、update zip、old-to-new updater | `docs/verification/reference-parity-v1/release-readiness-report.md` |

## 8. Release Gates

- `docs/verification/reference-parity-v1/spec-review-report.md` is pass.
- `openspec/changes/reference-parity-v1/alignment-report.md` is pass.
- 所有受影响组件 spec 已创建或更新，并在本 change 的后续任务中保持最新。
- `docs/research/reference-feature-coverage.md` 无未解释 `missing`/`partial`。
- Rust/Swift/GPUI 相关 build、fmt、lint、test、security、smoke、packaging、perf gate 均通过。
- Reference parity 功能矩阵 P0/P1/P2 均有真实验证证据。
- Security-sensitive paths 覆盖 regression tests。
- Beads `homie-h7n` 状态与 release readiness 实际结论一致。


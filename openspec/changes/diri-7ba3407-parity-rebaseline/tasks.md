# Diri 7ba3407 全功能对齐 OpenSpec Tasks

> Change ID: `diri-7ba3407-parity-rebaseline`  
> Source PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`  
> Beads: `homie-t3u`

Section 1 is the executable checklist for this Wave 0 change and is complete. T-101 已由 child Bead `homie-nep` 完成；Sections 2-6 中其余 program milestones 由独立 child Beads 承接，未完成项保持 unchecked 且不重新打开 `homie-t3u`。A milestone checkbox closes only when its child change has completed its own fine-grained `tasks.md` and evidence.

## 1. Wave 0: 重基线与规格门禁

- [x] 1.1 创建并认领 Bead `homie-t3u`，固定 `change_id=diri-7ba3407-parity-rebaseline` 和 `baseline_commit=7ba3407`
- [x] 1.2 编写中文主 PRD 和 `docs/research/diri-7ba3407-capability-matrix.md`
- [x] 1.3 修订受影响的长期组件 specs，并新增 `specs/runtime-client-transport/README.md`
- [x] 1.4 创建 OpenSpec proposal、design、八个 capability specs、plan 和本任务清单
- [x] 1.5 完成 FR-01..FR-16 alignment report，确认每项都有 task、component owner 和 verification
- [x] 1.6 执行 16 维 spec review，修复阻断项并记录 `docs/verification/diri-7ba3407-parity-rebaseline/spec-review-report.md`
- [x] 1.7 运行 OpenSpec、parity lock、mapping、format、diff 和 secret gates，记录 release-readiness evidence
- [x] 1.8 在 Wave 0 evidence 全部通过后关闭 `homie-t3u`，不得把后续实现任务标为完成

## 2. Wave 1: Runtime、Agent 与 Durable Facts

- [x] 2.1 T-101 创建并执行 `diri-runtime-daemon-client-transport`：实施 Bead `homie-nep` 已关闭；独立 daemon、UDS control/data、endpoint client、reconnect、events resume、attachment/backpressure 的证据位于 `docs/verification/diri-runtime-daemon-client-transport/`
- [ ] 2.2 T-102 创建并执行 `diri-agent-session-runtime`：实施 Bead `homie-t3u.1`，依赖已关闭的 `homie-nep`；manifest-driven spawn、holder adoption、process tree、resource governor、resume/migrate/shutdown；checkpoint `48f522b` 的 RED 仅为 adoption 与 live PTY 两项 `detached != running`，`runtime_holder_stat_tracks_resize_and_log_offsets` 已通过
- [ ] 2.3 T-103 创建并执行 `diri-storage-core-facts`：实施 Bead `homie-t3u.2`，依赖已关闭的 `homie-nep`；service-owned repositories、runtime recovery facts、effective config、lineage/remote/update metadata；RED 必须证明 UI direct-storage 和缺失 migration 合同

## 3. Wave 2: 本地桌面产品

- [ ] 3.1 T-201 创建并执行 `diri-desktop-workbench-sidebar`：runtime-backed workbench/sidebar、持久化 action、disconnect/reconnect、first frame 和 side-by-side screenshot
- [ ] 3.2 T-202 创建并执行 `diri-terminal-interaction`：live grid、selection/copy/paste/find/resize、offset scrollback、theme/repaint；删除脆弱 source-text 功能门禁
- [ ] 3.3 T-203 创建并执行 `diri-navigation-settings-native`：file Quick Open/index/cache、overview/switcher/history、settings、menu/notification/sound/approve-deny
- [ ] 3.4 T-204 创建并执行 `diri-inspector-git-artifacts`：Info/Changes/Artifacts、large diff、worktree、artifact/PR/browser preview、port list/forward

## 4. Wave 3: CLI 与 MCP 自动化

- [ ] 4.1 T-301 创建并执行 `diri-cli-complete-surface`：session get/read/send/wait/spawn/release/archive undo、status/artifacts/forward/ports/events subscribe 和 grammar fixtures
- [ ] 4.2 T-302 创建并执行 `diri-mcp-browser-automation`：精确 JSON Schemas、完整 lineage、`summarize_children`、`report_to_parent`、browser/test sidecar 和 package closure

## 5. Wave 4: Remote、Usage 与 Homie 扩展

- [ ] 5.1 T-401 创建并执行 `diri-remote-node-handoff`：authenticated node、accounts、remote spawn、checkpoint/blob、move/fork/quarantine/lease 和 service package
- [ ] 5.2 T-402 创建并执行 `diri-usage-llm-proxy`：incremental transcript watcher、pricing snapshot、fleet merge、OpenAI-compatible HTTP/SSE proxy、virtual key 和 usage UI
- [ ] 5.3 T-403 创建并执行 `homie-control-plane-integration`：context、memory、task、orchestrator 的 UI/CLI/MCP/runtime/storage 纵向闭环，并单独报告 extension 状态

## 6. Wave 5: Release 与最终准出

- [ ] 6.1 T-501 创建并执行 `diri-updater-packaging-performance`：universal dependency-closed bundle、Developer ID、notary/staple、DMG/feed、install/rollback 和 packaged budgets
- [ ] 6.2 T-502 创建并执行 `diri-7ba3407-final-parity-gate`：全矩阵、workspace、app/CLI/MCP/runtime/remote/updater、安全、视觉和性能交叉验证
- [ ] 6.3 只有 T-502 全部通过后，才将 overall result 更新为 `parity_complete` 并关闭最终 Bead

## 7. Task Contracts

| Task | Source requirement | Component specs | Required RED | Required GREEN | Evidence |
|------|--------------------|-----------------|--------------|----------------|----------|
| T-000 | FR-01, FR-16 | all amended specs | current status/OpenSpec/evidence inconsistencies | strict OpenSpec + 16-dimension review + alignment pass | `docs/verification/diri-7ba3407-parity-rebaseline/` |
| T-101 | FR-02 | runtime-client-transport, runtime-supervisor, observability | production client embeds supervisor; no reconnect/attachment | app/CLI/MCP shared daemon restart and attachment recovery E2E | `homie-nep`（closed）；`docs/verification/diri-runtime-daemon-client-transport/` |
| T-102 | FR-03, FR-04 | runtime-supervisor, agent-adapter, credentials | checkpoint `48f522b` 的 adoption/live PTY 两项测试返回 detached；spawn ignores manifest | fake/real agent spawn, holder crash/adopt, resume/migrate/resource/shutdown | `homie-t3u.1`（依赖 closed `homie-nep`）；`docs/verification/diri-agent-session-runtime/` |
| T-103 | FR-05 | storage-indexing, runtime-supervisor | direct UI storage and missing durable recovery facts | transactional repositories/migrations and restart recovery | `homie-t3u.2`（依赖 closed `homie-nep`）；`docs/verification/diri-storage-core-facts/` |
| T-201 | FR-06 | desktop-shell, runtime-client-transport | pin/archive/local fake state and incomplete workbench | persisted typed actions, reconnect and real screenshot/interaction | `docs/verification/diri-desktop-workbench-sidebar/` |
| T-202 | FR-07 | desktop-shell, runtime-supervisor | source-text test and incomplete live terminal behavior | live PTY terminal interaction, row fetch, visual/perf gates | `docs/verification/diri-terminal-interaction/` |
| T-203 | FR-08 | desktop-shell, storage-indexing | session-only Quick Open and preference-only native controls | file index/navigation/settings/native action E2E | `docs/verification/diri-navigation-settings-native/` |
| T-204 | FR-09 | desktop-shell, runtime-supervisor | static inspector tabs and parser-only services | live inspector/git/worktree/artifact/PR/port E2E | `docs/verification/diri-inspector-git-artifacts/` |
| T-301 | FR-10 | runtime-client-transport, mcp-automation | missing CLI commands/stream grammar | complete grammar fixtures and runtime E2E | `docs/verification/diri-cli-complete-surface/` |
| T-302 | FR-11 | mcp-automation, context, orchestrator, packaging | advertised unsupported tools and generic schemas | exact schemas, lineage matrix, sidecar/test packaged E2E | `docs/verification/diri-mcp-browser-automation/` |
| T-401 | FR-12 | remote-node-handoff, credentials, packaging | DTO/config-only remote state | two-node spawn/account/handoff/failure/service E2E | `docs/verification/diri-remote-node-handoff/` |
| T-402 | FR-13 | llm-proxy, credentials, storage, observability | parser/key models without HTTP/provider path | fake provider SSE, usage/fleet/UI, envelope/no-leak gates | `docs/verification/diri-usage-llm-proxy/` |
| T-403 | FR-14 | context, memory, task, orchestrator | isolated models with no product dependencies | durable cross-entry control-plane workflow | `docs/verification/homie-control-plane-integration/` |
| T-501 | FR-15 | packaging-updater, observability | ad-hoc/single-arch/not-run package gates | signed/notarized install/update/rollback and packaged perf | `docs/verification/diri-updater-packaging-performance/` |
| T-502 | FR-01..FR-16 | all | any incomplete matrix row or required gate | no incomplete rows, illegal statuses or unexecuted required gates | `docs/verification/diri-7ba3407-final-parity-gate/` |

## 8. Execution Rules

- 每个未勾选的实现任务必须先创建独立 child PRD/OpenSpec，master task 不授权直接编码。
- `homie-h7n.*` 仅作为历史需求来源，不得继续作为任何新实施任务的 owner。
- 每个 child task 继续拆成单次会话可完成的 RED、GREEN、REFACTOR、EVIDENCE steps。
- 外部依赖先用 local stub/fake；真实 provider/remote/release credential 只用于最终 smoke。
- 任何任务发现新的 Diri capability 时，先更新 matrix、PRD/spec 和 alignment，再实现。
- 不降低断言、不伪造 pass、不保留已被替换的错误 production fallback。

# Reference Parity V1 Functional Verification Cases

```yaml
change_id: reference-parity-v1
report_type: functional-case-design
status: designed
beads: homie-h7n
source_prd: prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md
dev_loop_step: 2
```

## 1. Purpose

本文件是 dev-loop Step 2 的功能验证 Case 设计。它定义 Reference Parity V1 后续实现完成后必须执行的可复现验证路径。

状态说明：

- `designed`: Case 已设计，等待实现后执行。
- `pass`: 已按真实代码路径执行并通过。
- `blocked`: 环境或依赖缺失，不能执行，必须写明原因。
- `fail`: 已执行但失败，必须回到实现或 Case 设计修正。
- `not_run`: 尚未执行；不得作为准出通过。

## 2. Case List

### FC-001: Reference 名称脱敏与文档路径一致性

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-1, FR-19 |
| Risk | 文档泄漏旧参考项目名称或路径 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-001` |

Preconditions:

- 当前工作区包含 `reference-parity-v1` PRD、OpenSpec 和 verification 文档。

Steps:

```bash
OLD_REFERENCE_PATTERN='<old-reference-name-pattern>'
rg -n -i "$OLD_REFERENCE_PATTERN" .
find . -iname '*<old-reference-name-pattern>*' -print
git diff --check
```

Expected:

- 前两个命令无命中。
- `git diff --check` 退出码为 0。

Failure handling:

- 替换旧名称和旧路径，重新执行本 Case。

### FC-002: Coverage Matrix 无未归属缺口

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-1 |
| Risk | Reference 功能项没有 owner 或验证路径 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-002` |

Steps:

```bash
rg -n "missing|partial" docs/research/reference-feature-coverage.md
rg -n "covered-by-reference-parity-v1" docs/research/reference-feature-coverage.md
```

Expected:

- `missing|partial` 只允许出现在状态定义说明或“无未解释缺口”文字中，不允许出现在功能矩阵行的当前状态列。
- 每个 Reference 功能矩阵行状态为 `covered-by-reference-parity-v1` 或有明确 follow-up owner。

### FC-003: OpenSpec FR 到 Task 映射完整

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-1 through FR-20 |
| Risk | PRD 需求没有执行任务 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-003` |

Steps:

```bash
rg -n "FR-[0-9]+" openspec/changes/reference-parity-v1/alignment-report.md
rg -n "T-[0-9]{3}" openspec/changes/reference-parity-v1/tasks.md
```

Expected:

- FR-1 到 FR-20 全部在 alignment report 中出现。
- 每个 OpenSpec task 至少有一个 Source requirement、Case coverage 和 Evidence 字段。

### FC-004: Component Spec 前置门禁

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-2 |
| Risk | 从 PRD 直接写代码，绕过长期组件合同 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-004` |

Steps:

```bash
rg -n "must update before implementation|new component spec required" openspec/changes/reference-parity-v1/alignment-report.md docs/verification/reference-parity-v1/component-spec-impact-report.md
test -f specs/desktop-shell/README.md
test -f specs/runtime-supervisor/README.md
test -f specs/agent-adapter-contract/README.md
test -f specs/llm-proxy/README.md
test -f specs/virtual-key-credentials/README.md
test -f specs/session-context-store/README.md
test -f specs/observability/README.md
test -f specs/task-controller/README.md
test -f specs/memory-controller/README.md
test -f specs/intent-orchestrator/README.md
test -f specs/packaging-updater/README.md
test -f specs/remote-node-handoff/README.md
test -f specs/mcp-automation/README.md
```

Expected:

- 当前设计阶段允许后续 `test -f` 失败并记录为 `blocked_for_implementation`。
- 进入代码实现前，所有 listed component specs 必须存在。

### FC-005: Agent Catalog Manifest Parity

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-3 |
| Risk | agent catalog 缺项或状态检测退化 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-005` |

Future execution commands:

```bash
cargo test -p homie-agents manifest_catalog_loads_all_reference_agents
cargo test -p homie-agents manifest_status_rules_have_golden_fixtures
cargo run -p homie-cli -- agent readiness --json
```

Expected:

- 19 个 Reference agent id 全部存在。
- first-class agent 不退化为 process-only。
- approval/deny/resume/status authority 均可由 manifest 读取。

### FC-006: Protocol And Event Contract Parity

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-4, FR-5 |
| Risk | UI/runtime/MCP/client 对协议理解不一致 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-006` |

Future execution commands:

```bash
cargo test -p homie-proto protocol_methods_cover_reference_parity
cargo test -p homie-proto event_resume_replays_without_gap
cargo test -p homie-client reconnect_and_resubscribe_preserves_seq
```

Expected:

- `hello`、session、worktree、events、hook、browser/test、LLM、profile、task、memory 方法全部有 DTO 或明确 unsupported error。
- unknown enum/value lenient decode。
- safe error envelope 不泄漏敏感字段。

### FC-007: Runtime Session Lifecycle And Recovery

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-4, FR-15 |
| Risk | app 关闭或 runtime 重启导致 session/output 丢失 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-007` |

Future execution commands:

```bash
cargo test -p homie-runtime session_spawn_input_resize_archive_recover
cargo test -p homie-runtime output_log_replay_is_offset_addressed
cargo run -p homie-cli -- session spawn --kind shell --cwd "$PWD" --title fc-007 --json
cargo run -p homie-cli -- session send-text --session <id> --text "echo fc-007" --submit
cargo run -p homie-cli -- session read-output --session <id> --mode screen --json
```

Expected:

- spawn/input/output/resize/archive/unarchive/hibernate/wake/history 可用。
- app 退出不杀 session。
- runtime restart 后 session list 和 output 仍可读。

### FC-008: Terminal Grid Interaction

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-6 |
| Risk | terminal rendering/input 与真实 PTY 不一致 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-008` |

Future execution commands:

```bash
cargo test -p homie-term grid_fixture_roundtrip
cargo test -p homie-term terminal_key_encoding_matches_reference_cases
cargo test -p homie-term scrollback_selection_find_crosses_live_history
```

Manual steps:

- 启动真实 shell session。
- 输入 `seq 1 1000`。
- 滚动到历史区域，跨 live/history seam 选择文本并复制。
- `Cmd-F` 搜索 `999`，执行 next/previous。
- 调整侧边栏和窗口大小。

Expected:

- 无最后一行丢失、无明显 resize 跳帧、copy/find 行为一致。

### FC-009: Desktop Shell And UI Fidelity

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-7 |
| Risk | UI 只实现功能但不对齐产品设计 |
| Evidence | `docs/verification/reference-parity-v1/ui-fidelity-report.md` |

Future execution commands:

```bash
cargo run -p homie-app -- --preview --scenario empty
cargo run -p homie-app -- --preview --scenario typical
cargo run -p homie-app -- --preview --scenario stress
```

Manual steps:

- 截图 empty/typical/stress。
- 检查 window chrome、sidebar、status glyph、terminal pane、floating surfaces、settings、history、worktrees、overview、inspector。
- 检查 900x560 和窄窗口无重叠。

Expected:

- 视觉 token、布局、动效策略和键盘映射与 Reference design 对齐；偏差必须列入 intentional deviations。

### FC-010: Worktree And Project Safety

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-8 |
| Risk | cleanup 删除 dirty/unmerged/main worktree |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-010` |

Future execution commands:

```bash
cargo test -p homie-runtime worktree_overview_marks_stale_suggestions
cargo test -p homie-runtime worktree_cleanup_rejects_dirty_unmerged_main
cargo run -p homie-cli -- worktree create --repo <tmp-repo> --branch fc-010
cargo run -p homie-cli -- worktree overview --repo <tmp-repo> --json
```

Expected:

- safe cleanup only for stale clean merged worktrees。
- `force=false` default；force 必须显式确认并记录。

### FC-011: History Scan And Resume

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-9 |
| Risk | 历史会话无法恢复或重复显示 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-011` |

Future execution commands:

```bash
cargo test -p homie-runtime history_scanner_dedupes_tracked_sessions
cargo run -p homie-cli -- session history --json
cargo run -p homie-cli -- session resume-from-history --entry <entry-json> --json
```

Expected:

- Claude/Codex history entries 包含 id、kind、cwd、title、transcript path、last active、cwd exists。
- tracked sessions 不重复。
- dead cwd 不直接 resume。

### FC-012: Artifact, Port, PR, Browser, And Test Run

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-10 |
| Risk | artifact surface 与 MCP/browser 自动化断裂 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-012` |

Future execution commands:

```bash
cargo test -p homie-runtime artifact_scanner_detects_pr_preview_link_port
cargo test -p homie-runtime pr_monitor_projects_checks_comments_threads
cargo run -p homie-cli -- artifacts get --session <id> --json
cargo run -p homie-cli -- mcp-call --tool test_run < test-run-input.json
cargo run -p homie-cli -- mcp-call --tool browser < browser-input.json
```

Expected:

- PR/check/comment/preview/port chips 有结构化数据。
- browser/test_run 返回结构化结果；失败截图只返回文件路径，不内联图片 bytes。

### FC-013: Usage, Cost, And LLM Proxy Custody

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-11, FR-12, FR-19 |
| Risk | real provider key 泄漏或 usage 口径漂移 |
| Evidence | `docs/verification/reference-parity-v1/security-report.md#fc-013` |

Future execution commands:

```bash
cargo test -p homie-llm virtual_key_scope_expiry_revoke
cargo test -p homie-llm fake_provider_streaming_records_usage_without_raw_payload
cargo test -p homie-llm metrics_write_failure_does_not_block_response
rg -n "Authorization|provider_key|raw prompt|cookie" docs/verification crates tests
```

Expected:

- managed agent env/config 无真实 provider key。
- streaming success/failure 都有 safe metrics。
- pricing snapshot 固定历史 cost。
- metrics 写失败产生 `metrics.write_failed`。

### FC-014: CLI, Hook, Notify, And MCP Automation

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-14 |
| Risk | 自动化入口不可用或 hook 阻塞 agent |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-014` |

Future execution commands:

```bash
cargo test -p homie-cli command_grammar_covers_reference_parity
cargo test -p homie-cli hook_notify_fail_open_on_daemon_error
cargo run -p homie-cli -- mcp-tools
cargo run -p homie-cli -- mcp-call --tool list_agents < empty.json
```

Expected:

- CLI command grammar 覆盖 session/worktree/artifacts/events/ports/hook/notify/mcp。
- hook/notify 失败时退出 0，不阻塞 agent。
- MCP tools 带 lineage 和 permission 约束。

### FC-015: Remote Host, Node, Accounts, And Handoff

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-13 |
| Risk | 远端执行复制 credential 或 handoff 破坏 workspace |
| Evidence | `docs/verification/reference-parity-v1/remote-node-report.md` |

Future execution commands:

```bash
cargo test -p homie-runtime host_config_rejects_invalid_node_token_paths
cargo test -p homie-runtime handoff_stages_in_quarantine_and_aborts_before_commit
cargo run -p homie-cli -- node hello --endpoint <loopback> --token-file <tmp-token>
cargo run -p homie-cli -- session spawn --host <loopback-host> --kind shell --cwd <remote-cwd> --json
```

Expected:

- host/node token owner-only。
- provider raw key 不进入 checkpoint 或 transfer manifest。
- move/fork failure before commit aborts both sides。

### FC-016: Context, Memory, Task, And Intent Orchestration

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-20 |
| Risk | Homie 自有能力未接入 Reference parity 工作流 |
| Evidence | `docs/verification/reference-parity-v1/dev-loop-case-execution-report.md#fc-016` |

Future execution commands:

```bash
cargo test -p homie-context session_context_summary_omits_raw_secret_fields
cargo test -p homie-task agent_can_claim_update_block_and_return_task
cargo test -p homie-memory write_candidate_requires_source_and_redaction
cargo test -p homie-runtime intent_routes_new_agent_palette_and_mcp_spawn
```

Expected:

- session context、task、memory candidate、intent routing 都能关联 session lineage。
- 不写 raw secret、raw prompt、完整 tool args/result。

### FC-017: Packaging, Updater, And Release Trust

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-16, FR-17, FR-19 |
| Risk | 未验证 bundle 或自动重启破坏 live session |
| Evidence | `docs/verification/reference-parity-v1/release-readiness-report.md#fc-017` |

Future execution commands:

```bash
cargo test -p homie-updater rejects_wrong_team_bundle_or_version
scripts/package/package.sh
scripts/package/perf-gate.sh --app dist/Homie.app --scenario all
```

Manual steps:

- 使用旧签名 app 执行手动 Check for Updates。
- 下载新版本，验证签名、公证、版本。
- 点击 restart-to-update。
- 确认失败恢复路径保留旧 bundle。

Expected:

- 无未验证 bundle 安装。
- 不自动重启 live app。
- packaged perf gate 记录 normal/large footprint、avg CPU、peak CPU。

### FC-018: Full Local Quality Gate

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-1 through FR-20 |
| Risk | 单点测试通过但准出链路不完整 |
| Evidence | `docs/verification/reference-parity-v1/release-readiness-report.md#fc-018` |

Future execution commands:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
.githooks/pre-commit
git diff --check
```

Expected:

- 所有命令退出码为 0。
- 未运行项必须标记 `not_run` 或 `blocked`，不能写成 pass。

## 3. Coverage Matrix

| PRD FR | P0/P1 | Functional Cases |
|--------|-------|------------------|
| FR-1 | P0 | FC-001, FC-002, FC-003, FC-018 |
| FR-2 | P0 | FC-004, FC-018 |
| FR-3 | P0 | FC-005 |
| FR-4 | P0 | FC-006, FC-007 |
| FR-5 | P0 | FC-006 |
| FR-6 | P0 | FC-008 |
| FR-7 | P0 | FC-009 |
| FR-8 | P1 | FC-010 |
| FR-9 | P1 | FC-011 |
| FR-10 | P1 | FC-012 |
| FR-11 | P1 | FC-013 |
| FR-12 | P0 | FC-013, FC-015 |
| FR-13 | P2 | FC-015 |
| FR-14 | P1 | FC-014 |
| FR-15 | P1 | FC-007, FC-017 |
| FR-16 | P0 | FC-017 |
| FR-17 | P0 | FC-017 |
| FR-18 | P0 | FC-004, FC-007, FC-018 |
| FR-19 | P0 | FC-001, FC-013, FC-015, FC-017, FC-018 |
| FR-20 | P1 | FC-016 |

## 4. Execution Order

1. FC-001 to FC-004: spec and implementation-entry gates.
2. FC-005 to FC-008: foundation and runtime gates.
3. FC-009 to FC-014: local product and automation gates.
4. FC-015 to FC-017: remote/node and release gates.
5. FC-018: final full local gate.

## 5. Evidence Rules

- 每次执行必须记录命令、退出码、环境、实际输出摘要和证据路径。
- 失败 Case 必须回到 dev-loop Step 5 或 Step 2/4 修正，不能跳过。
- 当前文件只表示 Case 设计完成，不表示任何 Case 已通过。


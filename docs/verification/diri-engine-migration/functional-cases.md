# diri-engine-migration 功能验证 Case

```yaml
change_id: diri-engine-migration
report_type: functional-case-design
status: designed
beads: homie-cj5
source_prd: prd-spec/features/diri-engine-migration/2026-08-06-diri-engine-migration-gap-closure-design.md
spec_review: docs/verification/diri-engine-migration/spec-review-report.md
```

## 1. 验证目标

本功能验证 Case 清单用于约束 `diri-engine-migration` gap-closure 实现。所有 P0/P1 需求必须在开发前具备可执行、可判定、可留痕的验证 Case；实现完成后必须逐条执行并把结果写入 `docs/verification/diri-engine-migration/functional-verification-report.md`。

## 2. Case 清单

### FC-DIRI-001: Runtime 启动真实 PTY shell

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 类型 | 集成验证 |
| 前置条件 | macOS/Unix；`/bin/sh` 存在；临时数据目录可写 |
| 执行命令 | `cargo test -p homie-runtime runtime_spawn_shell_uses_live_pty -- --nocapture` |
| 输入数据 | 测试通过 `RuntimeSupervisor::spawn_shell` 创建 shell session，并发送 `printf homie-live-pty` |
| 预期结果 | 测试能从 `read_output` 读取 shell 实际输出 `homie-live-pty`；session 状态不是仅文件写入模拟 |
| 通过标准 | 命令退出码为 0；测试断言输出来自 PTY pump；证据记录命令与输出摘要 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-001` |
| 失败处理 | 回到 Runtime live session registry 和 PTY pump 实现，不允许降低为文件追加测试 |

### FC-DIRI-002: PTY spawn 失败不写半成品 session

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 类型 | 异常路径集成验证 |
| 前置条件 | 临时数据目录可写；不存在的 binary path |
| 执行命令 | `cargo test -p homie-runtime runtime_spawn_failure_does_not_persist_created_session -- --nocapture` |
| 输入数据 | 使用不存在的 shell/binary 或无效 cwd 调用 spawn |
| 预期结果 | spawn 返回明确错误；`list_sessions` 中没有状态为 `created` 的半成品 session |
| 通过标准 | 命令退出码为 0；断言 storage 无半写 session；错误信息不包含敏感 env |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-002` |
| 失败处理 | 调整 spawn 顺序为先校验/启动 PTY，再落库或更新状态 |

### FC-DIRI-003: Runtime live session 输入缺失时 fail closed

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 类型 | 边界验证 |
| 前置条件 | 临时数据目录可写；存在历史 session 但不在 live registry |
| 执行命令 | `cargo test -p homie-runtime runtime_send_text_requires_live_session -- --nocapture` |
| 输入数据 | 对不存在或非 live session id 调用 `send_text` |
| 预期结果 | 返回 `SessionNotLive` 或等价稳定错误；不会把输入追加到 output log 冒充成功 |
| 通过标准 | 命令退出码为 0；output log 未出现输入文本 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-003` |
| 失败处理 | 修正 `send_text` 路径，禁止静默 fallback 到文件追加 |

### FC-DIRI-004: Status reducer 覆盖 needs input、idle 和 subagent 隔离

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-2 |
| 类型 | 单元验证 |
| 前置条件 | `homie-agents` status reducer 已迁移 |
| 执行命令 | `cargo test -p homie-agents status_reducer -- --nocapture` |
| 输入数据 | hooks-primary、screen-primary、process-only 三组 fixture |
| 预期结果 | reducer 正确输出 working、needs_input、idle、turn_completed；subagent hook 不污染 parent status |
| 通过标准 | 命令退出码为 0；测试覆盖 startup grace、idle confirmation、blocker clear、process exit |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-004` |
| 失败处理 | 回到 reducer 类型适配或信号折叠逻辑 |

### FC-DIRI-005: Hook/notify parser 稳定事件与脱敏

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-3 |
| 类型 | 单元验证 / 安全验证 |
| 前置条件 | `homie-agents::hooks` 已实现 |
| 执行命令 | `cargo test -p homie-agents hook_parser -- --nocapture` |
| 输入数据 | Claude permission request、notification、subagent、session end；Codex notify；含 token/authorization/cookie/password 的异常 payload |
| 预期结果 | 输出稳定 `HookEvent`/`NotifyEvent`；未知 payload fail-open；所有敏感字段被脱敏 |
| 通过标准 | 命令退出码为 0；测试断言输出不包含原始 secret 值 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-005` |
| 失败处理 | 调整 parser 和 redaction，不允许把原始 JSON 解析散落到 runtime |

### FC-DIRI-006: Scrollback 真实视口模型

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-4 |
| 类型 | 单元验证 |
| 前置条件 | `homie-term::scrollback` 已移除 stub 结果 |
| 执行命令 | `cargo test -p homie-term scrollback -- --nocapture` |
| 输入数据 | live grid、历史 rows、row count mismatch、alt screen、wheel route fixture |
| 预期结果 | `begin_fetch` 生成有效请求；`complete_fetch` 缓存 rows；`apply_geometry` 更新 max offset；alt screen 清理历史视口 |
| 通过标准 | 命令退出码为 0；不再存在空元组 `ReadScrollbackCellsResult` 作为真实结果 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-006` |
| 失败处理 | 回到 scrollback data model 和 cache/geometry 实现 |

### FC-DIRI-007: Design token 与 Diri 源文件对齐

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-5 |
| 类型 | 单元验证 / 设计回归 |
| 前置条件 | `homie-ui` token 已补齐 |
| 执行命令 | `cargo test -p homie-ui token_parity -- --nocapture` |
| 输入数据 | Diri `tokens.rs` 中 radius、typography、metrics、motion、MemoryFormat 的关键常量 |
| 预期结果 | Homie token 值与 Diri 对齐；测试不只验证 Homie 当前值 |
| 通过标准 | 命令退出码为 0；报告列出关键 token 对齐范围 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-007` |
| 失败处理 | 补齐或调整 `homie-ui` token；禁止在 app 中继续硬编码替代 token |

### FC-DIRI-008: Homie app 首屏去占位

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-6 |
| 类型 | 回归验证 / 编译 smoke |
| 前置条件 | `homie-app` preview shell 已调整 |
| 执行命令 | `cargo test -p homie-app app_shell_copy_regression -- --nocapture` 和 `cargo check -p homie-app` |
| 输入数据 | `crates/homie-app/src/**/*.rs` 源文本和 app 编译路径 |
| 预期结果 | 源文本不包含 `Next implementation slices`、`PTY-backed execution is the next runtime slice`；app 可编译 |
| 通过标准 | 两个命令退出码为 0；若后续有 UI screenshot harness，应追加截图证据 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-008` |
| 失败处理 | 去除占位文案或补齐 preview shell；编译失败回到 app shell 实现 |

### FC-DIRI-009: OpenSpec 与 PRD 状态一致

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-7 |
| 类型 | 文档门禁 |
| 前置条件 | OpenSpec plan/tasks/alignment 已更新 |
| 执行命令 | `rg -n \"状态: ✅ 完成|Status: complete|⏭️\" openspec/changes/diri-engine-migration docs/verification/diri-engine-migration` |
| 输入数据 | OpenSpec 和 verification 文档 |
| 预期结果 | 未完成项不被标为完成；延期项必须有 task、风险或 follow-up owner |
| 通过标准 | 搜索结果经人工确认无状态漂移；alignment report 决策为 pass 或明确 blocked |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-009` |
| 失败处理 | 修正文档状态，禁止进入实现完成声明 |

### FC-DIRI-010: Holder-owned PTY survival and restore semantics

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 类型 | 集成验证 / 恢复验证 |
| 前置条件 | macOS/Unix；`homie-runtime-holder` 可从测试路径启动；临时数据目录和 `/tmp/homie-runtime-holders` 可写 |
| 执行命令 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` |
| 输入数据 | 测试创建 shell session，drop `RuntimeSupervisor` 后重开并继续写入；显式 terminate 后检查 socket/pid 清理；holder 正常退出后重开应恢复为 `exited`；缺失 holder/status 证据时应恢复为 `detached` |
| 预期结果 | holder 子进程持有 PTY/output log；supervisor 重开可 adopt running holder；terminate 会标记 session `exited` 并清理 holder socket/pid；异常缺失 holder 证据时 session 标记 `detached` |
| 通过标准 | 命令退出码为 0；`runtime_reopen_can_adopt_holder_and_continue_session`、`runtime_terminate_marks_exited_and_removes_holder_files`、`runtime_reopen_marks_exited_holder_status_exited`、`runtime_reopen_marks_missing_holder_detached` 均通过 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-010` |
| 失败处理 | 回到 holder IPC、status file、registry restore 或 terminate cleanup 实现，不允许用 supervisor 内存 registry 冒充 crash survival |

### FC-DIRI-011: Runtime headless status pipeline

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1, FR-2 |
| 类型 | 集成验证 / 状态管线验证 |
| 前置条件 | `homie-runtime-holder` 可从测试路径启动；`homie-runtime` 依赖 `homie-agents` status reducer；临时数据目录可写 |
| 执行命令 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` |
| 输入数据 | 测试通过真实 shell session 输出 `homie-status:working`、Codex/Diri 风格权限确认文本、`homie-status:idle`，再显式 terminate |
| 预期结果 | Runtime 从 holder-produced output log 构建 headless screen，分类为 screen observation，送入 `StatusReducer` 后输出 `running`、`needs_input`、`idle`、`exited`；`needs_input` 来源为 screen scrape，摘要来自可见确认提示 |
| 通过标准 | 命令退出码为 0；`runtime_status_report_uses_headless_screen_and_reducer` 通过；不允许直接构造 reducer 输入绕过真实 PTY/log/screen 路径 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-011` |
| 失败处理 | 回到 output log -> `HeadlessScreen` -> screen observation -> `StatusReducer` 接线，不允许只保留 socket/pid 状态投影 |

### FC-DIRI-012: Holder process-tree termination

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 类型 | 集成验证 / 进程树恢复验证 |
| 前置条件 | Unix/macOS；`python3` 可用；`homie-runtime-holder` 可从测试路径启动 |
| 执行命令 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` |
| 输入数据 | 真实 shell session 启动一个 `python3` 子进程，子进程调用 `os.setsid()` 脱离 root shell process group，并忽略 `SIGTERM` 后 sleep |
| 预期结果 | Holder `Stat` 能看到 root shell + detached child tree；`terminate_session` 调用 holder process-tree kill 后 detached child 也被清理，不留下后台进程 |
| 通过标准 | 命令退出码为 0；`runtime_terminate_kills_detached_child_tree` 通过；不能只依赖 `kill(-pgid)` 覆盖同组进程 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-012` |
| 失败处理 | 回到 holder process-tree 枚举、pid start-time 防复用、SIGTERM/SIGKILL/SIGCONT 顺序，不允许留下脱组子进程 |

### FC-DIRI-013: Holder stat metadata, resize, and log offsets

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 类型 | 集成验证 / attach-resume 元数据验证 |
| 前置条件 | `homie-runtime-holder` 可从测试路径启动；临时数据目录可写 |
| 执行命令 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` |
| 输入数据 | 真实 shell session 启动后查询 holder `Stat`；发送 PTY 输出后再次查询 log offset；调用 `RuntimeSupervisor::resize_session(100, 30)` 后查询 holder geometry |
| 预期结果 | `Stat` 返回 `cols=120`、`rows=40`、`epoch_offset=0`、初始 `log_offset=0`；输出后 `log_offset > epoch_offset`；resize 后 `cols=100`、`rows=30` |
| 通过标准 | 命令退出码为 0；`runtime_holder_stat_tracks_resize_and_log_offsets` 通过；不能只在内存中改 geometry 而不通过 holder IPC |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-013` |
| 失败处理 | 回到 holder `Stat`/`Resize` IPC 和 `RuntimeSupervisor::resize_session` 接线，保证 attach/resume 能拿到真实 holder 元数据 |

### FC-DIRI-014: CLI hook/notify parser entry

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-3 |
| 类型 | CLI 集成验证 / 安全验证 |
| 前置条件 | `homie-cli` 依赖 `homie-agents` hook parser；本地可运行 `cargo run -p homie-cli` |
| 执行命令 | `cargo test -p homie-cli --tests -- --nocapture`；`cargo run -p homie-cli -- hook PermissionRequest '<json>'`；`cargo run -p homie-cli -- notify '<json>'`；`cargo run -p homie-cli -- hook FutureHook '<json>'` |
| 输入数据 | Claude permission request payload、Codex `agent-turn-complete` notify payload、未知 hook + `authorization` secret payload |
| 预期结果 | CLI 输出稳定 JSON 事件、needsInput、sessionId、firstPromptTitle、safeSummary；未知 hook fail-open 且退出码为 0；输出不包含原始 secret |
| 通过标准 | 命令退出码为 0；测试断言 secret 不泄漏；真实 CLI smoke 输出包含解析后的事件而不是空 `{}` |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-014` |
| 失败处理 | 回到 `homie-cli` hook/notify 命令接线，不允许保留空输出或只在 `homie-agents` 单测中解析 |

### FC-DIRI-015: Runtime session snapshot for attach/resume

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 类型 | 集成验证 / attach-resume 快照验证 |
| 前置条件 | `homie-runtime-holder` 可从测试路径启动；临时数据目录可写 |
| 执行命令 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` |
| 输入数据 | 创建真实 shell session，写入 `snapshot-ready`，drop supervisor 后重开 runtime，通过 `session_snapshot(session_id, offset, max_bytes)` 读取快照 |
| 预期结果 | 快照组合 SQLite session、holder stat、status report、offset replay；reopen 后仍为 `running`，输出 replay 精确返回 `snapshot-ready`，holder log offset 覆盖 replay 范围 |
| 通过标准 | 命令退出码为 0；`runtime_reopen_snapshot_combines_registry_holder_status_and_replay` 通过 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-015` |
| 失败处理 | 回到 runtime registry restore、holder stat、status report、offset log replay 接线，不允许 client/protocol 未来重复拼装不一致快照 |

### FC-DIRI-016: CLI session snapshot entry

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 类型 | CLI 集成验证 |
| 前置条件 | `homie-cli` 依赖 `homie-runtime`；临时数据目录可写 |
| 执行命令 | `cargo test -p homie-cli --test session_snapshot_cli -- --nocapture` |
| 输入数据 | 测试创建真实 SQLite session 后运行 `homie session snapshot --data-dir <dir> --id <session>` |
| 预期结果 | CLI 输出 JSON，包含 session id、detached status、outputText；命令通过 `RuntimeSupervisor::session_snapshot` 读取，不直接拼 storage |
| 通过标准 | 命令退出码为 0；`session_snapshot_command_reads_runtime_snapshot_json` 通过 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-016` |
| 失败处理 | 回到 `homie-cli session snapshot` 与 runtime snapshot 接线，不允许 CLI 只读 storage 冒充 attach snapshot |

### FC-DIRI-017: Runtime screen checkpoint persistence

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 类型 | 集成验证 / checkpoint 验证 |
| 前置条件 | `homie-runtime-holder` 可从测试路径启动；临时数据目录可写 |
| 执行命令 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` |
| 输入数据 | 创建真实 shell session，输出 `checkpoint-line`，调用 `write_screen_checkpoint`，drop supervisor 后重开 runtime 并调用 `read_screen_checkpoint` |
| 预期结果 | checkpoint 持久化 session id、output offset、content seq 和 headless screen lines；重开后读回内容完全一致 |
| 通过标准 | 命令退出码为 0；`runtime_screen_checkpoint_survives_supervisor_reopen` 通过 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-017` |
| 失败处理 | 回到 headless screen checkpoint 序列化和 runtime checkpoint 路径，不允许只依赖当前进程内存 |

### FC-DIRI-018: Runtime hibernate/wake resource lifecycle

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1 |
| 类型 | 集成验证 / 资源生命周期验证 |
| 前置条件 | `homie-runtime-holder` 可从测试路径启动；临时数据目录可写 |
| 执行命令 | `cargo test -p homie-runtime --test session_lifecycle -- --nocapture` |
| 输入数据 | 创建真实 shell session 后调用 `hibernate`，确认 holder socket/pid 清理；随后调用 `wake`，确认 holder 重新 running，并发送 PTY 文本 |
| 预期结果 | `hibernate` 不只是改 SQLite 状态，而是停止 holder；`wake` 重启 holder 并恢复可交互 session；status projection 在 hibernated/wake/running 之间一致 |
| 通过标准 | 命令退出码为 0；`runtime_hibernate_stops_holder_and_wake_restarts_it` 通过 |
| 证据路径 | `docs/verification/diri-engine-migration/functional-verification-report.md#fc-diri-018` |
| 失败处理 | 回到 archive/hibernate/wake 与 holder lifecycle 接线，不允许只更新 storage status |

## 3. 覆盖矩阵

| PRD 需求 | 覆盖 Case | 覆盖结论 |
|----------|-----------|----------|
| FR-1 真实 PTY runtime 接线 | FC-DIRI-001, FC-DIRI-002, FC-DIRI-003, FC-DIRI-010, FC-DIRI-011, FC-DIRI-012, FC-DIRI-013, FC-DIRI-015, FC-DIRI-016, FC-DIRI-017, FC-DIRI-018 | 覆盖正常、异常、边界、holder 恢复、runtime 状态管线、进程树终止、holder stat/resize/log offset、runtime/CLI attach snapshot、screen checkpoint、hibernate/wake 资源生命周期路径 |
| FR-2 Diri status reducer 迁移 | FC-DIRI-004, FC-DIRI-011 | 覆盖 reducer 核心状态、subagent 隔离、runtime screen pipeline 接入 |
| FR-3 Hook/notify parsing | FC-DIRI-005, FC-DIRI-014 | 覆盖稳定事件、未知 payload、脱敏、CLI 入口 |
| FR-4 Scrollback 真实模型 | FC-DIRI-006 | 覆盖视口、缓存、geometry、alt screen、wheel route |
| FR-5 Design token 完整对齐 | FC-DIRI-007 | 覆盖关键 token parity |
| FR-6 Homie app 去占位 | FC-DIRI-008 | 覆盖文案回归和编译 smoke |
| FR-7 状态和文档一致 | FC-DIRI-009 | 覆盖 OpenSpec/verification 状态漂移 |

## 4. 执行计划

1. 先执行文档门禁 FC-DIRI-009，确认 OpenSpec 更新后没有状态漂移。
2. 按实现顺序执行 FC-DIRI-001 到 FC-DIRI-018 的 RED/GREEN 验证。
3. 每个 Case 第一次执行失败必须记录 RED 结果；实现后重新执行并记录 GREEN 结果。
4. 全部 P0/P1 Case 通过后，才能进入代码审查和 E2E 门禁。


# Diri Storage Core Durable Facts 设计文档

```yaml
change_id: diri-storage-core-facts
status: ready_for_review
beads: homie-t3u.2
master_task: T-103
parent_change_id: diri-7ba3407-parity-rebaseline
baseline_commit: 7ba3407
checkpoint_commit: 48f522b
date: 2026-08-09
```

## 1. 概述

### 1.1 背景

T-103 是 Diri `7ba3407` 重基线的 Wave 1C。它承接主 PRD FR-05，目标不是重新建设
SQLite，而是在已完成的 storage/indexing 第一阶段之上补齐可由 runtime/service 独占消费的
durable facts 合同。

截至 2026-08-09，实测与代码审计确认：

- `cargo test -p homie-storage --tests` 全绿，测试组计数为 `0 + 6 + 4 + 2 + 5 + 2 + 2`。
- 当前 schema version 是 `3`。
- `Storage::migrate` 已在一个 SQLite transaction 内按 v1 -> v2 -> v3 顺序执行 migration，
  支持幂等和 schema-too-new fail closed。
- v1 已有 `effective_agent_configs` 表，但没有 freeze/readback repository；当前
  `create_session_with_parent` 把 `sessions.effective_config_id` 写成 `NULL`。
- v3 已有 session core metadata、history、project/worktree、usage ledger 和 usage scan cache
  repository。
- v2 已有 `hosts`、`node_accounts`、`handoff_records`，但缺少 service-owned repository、
  operation/lease 幂等字段和完整状态转换约束。
- 当前没有 updater receipt 表。
- runtime recovery 目前由 `sessions.status`、holder socket/status file、output log 和 event log
  临时拼接；缺少 typed recovery fact repository 和原子 checkpoint metadata。
- `homie-runtime` 持有 `Storage`，但 `RuntimeActor` 多处通过 `RuntimeSupervisor::storage()`
  直接调用 storage API，`prepare_shutdown` 还直接取得 SQLite connection。
- `homie-app/Cargo.toml` 直接依赖 `homie-storage`；`homie-app/src/main.rs` 直接
  `open_or_create`、migrate、seed 并加载/保存 `SettingsPreferences`。这不是待预防的问题，
  而是必须在 T-103 删除的现存架构违规。
- `homie-cli` 的 doctor 和 usage summary 仍直接打开 storage。Wave 1A 明确把这些路径留给
  T-103 收口。

因此，本 change 的 RED 不能把 schema v3、ordered transaction migrations、
`effective_agent_configs` 表或已有 repository 写成缺失。RED 必须针对当前确实不存在的
v3 -> v4 migration 合同、service-owned API、effective config freeze/readback、runtime
recovery facts，以及 app/CLI direct-storage 删除。

### 1.2 目标

1. 在现有 v1 -> v3 migration 基线上定义并实现 ordered v4 migration，不重写历史 migration。
2. 通过 typed repository/service API 固化 runtime recovery、effective config 和安全持久化事实。
3. 删除 `homie-app -> homie-storage` normal dependency 和 app 内 SQLite 打开路径，settings
   必须经 owning service/client 读写。
4. 将 CLI doctor/usage durable reads 改为 owning service/client 路径，不再由 CLI 直接打开
   SQLite。
5. 为后续 lineage、remote handoff 和 updater 提供受约束、可幂等的 durable metadata
   foundation，但不实现这些工作流。
6. 接收 T-102 G3 冻结的 resolved launch/effective-config shape，先完成 T-103 durable
   repository；T-103 的六个 service method/DTO 由 T-103 proto owner 在 T-102 完成并释放
   shared 文件后实现。
7. 保持 parity 结论真实：本 change 是 foundation，不单独关闭 UI、remote、usage 或 updater
   parity rows。

### 1.3 非目标

- 不实现 Settings 完整交互、截图或原生 macOS action。
- 不实现 filesystem transcript watcher、fleet usage、usage UI 或 LLM HTTP proxy。
- 不实现 node server、网络认证、checkpoint blob transfer、provider resume 或 handoff commit
  工作流。
- 不实现 updater feed fetch、download、签名验证、安装、回滚或 release packaging。
- 不修复 T-102 的 holder/PTY/manifest spawn 生命周期；T-102 负责产生和验证 live facts，
  T-103 负责持久化 facts。
- 不把 storage row 作为 live session 的充分证据。
- 不在规格阶段修改 `homie-proto` 或任何产品代码。
- 不更新 parity lock、master tasks、其他 component specs 或历史 evidence。
- 不为当前 direct-storage 路径保留 compatibility fallback。

## 2. 现状基线与缺口

| 能力 | 当前事实 | T-103 判定 |
|------|----------|------------|
| Ordered migrations | v1 -> v3 单 transaction、幂等、too-new fail closed | 已存在，新增 v4 和 previous-version/rollback 证据 |
| Effective config schema | `effective_agent_configs` 已存在 | 表已存在，freeze/readback、immutable snapshot 和 session binding 缺失 |
| Session metadata | v3 core metadata API 已存在 | 保留并复用 |
| History/worktree/usage | repository 与测试已存在 | 保留并复用，不重复实现 |
| Runtime recovery | ad hoc 读取 sessions + holder/status/log | typed joined facts、checkpoint metadata 和原子更新缺失 |
| Lineage | `parent_session_id` 与 direct child API 已存在 | provenance/audit/idempotency foundation 缺失 |
| Remote metadata | hosts/accounts/handoff 表已存在 | repository、operation/lease/hash/phase 约束缺失 |
| Update metadata | 无 durable receipt | foundation 缺失 |
| App storage boundary | app 直接依赖并打开 storage | 必须删除 |
| CLI storage boundary | doctor/usage 直接依赖 storage | 必须删除 |

## 3. 用户与系统场景

### 场景 1：设置经 owning service 持久化

**Given** app 已连接 runtime daemon
**When** 用户打开设置或保存 terminal/remote preferences
**Then** app 通过 typed client method 读取或更新带 revision 的 settings snapshot，app binary
不链接 `homie-storage`，写失败时不保留假成功状态。

### 场景 2：Agent 配置冻结后可重读

**Given** T-102 已解析 profile、manifest、permission 和 managed LLM route
**When** runtime 创建 session 并冻结 effective config
**Then** session 与 immutable config 在同一 repository transaction 中绑定；后续 profile
修改不改变 readback；snapshot 不包含 provider raw key 或 virtual key material。

### 场景 3：Daemon 重启从持久事实恢复候选

**Given** session、holder/output/checkpoint/event metadata 已提交
**When** daemon 重启
**Then** runtime 读取 bounded recovery candidates，重新验证 holder/process/output 后才决定
running/detached/exited；仅凭 storage 中的 `last_observed_status` 不得报告 running。

### 场景 4：Migration 失败不留下半升级

**Given** 一个真实 schema v3 fixture
**When** v4 migration 中任一步骤失败
**Then** schema version 和所有 v4 DDL/DML 一起 rollback；重试可重新执行完整 v4 migration。

### 场景 5：后续 remote/update 可复用 durable metadata

**Given** remote 或 updater 后续 wave 需要 operation id、phase、hash、lease 或 receipt
**When** 对应 service 写入 metadata
**Then** repository 提供幂等创建和 compare-and-set 状态转换；本 change 不执行网络传输或安装。

## 4. 功能需求

### FR-01：保留并证明既有基线

- 规格和测试必须把 schema v3、ordered transaction migrations、`effective_agent_configs` 表、
  session core metadata、history/worktree/usage repositories 标为已存在。
- 新 RED 不得通过删除、弱化或改写上述现有测试制造。
- baseline gate 必须先运行 `cargo test -p homie-storage --tests` 并记录每个 test binary 的
  实际计数。

### FR-02：Ordered v4 migration

- v4 必须追加到现有 v1/v2/v3 顺序，不重写已发布 migration 常量的语义。
- 空库必须按 `[1, 2, 3, 4]` 迁移；真实 v3 fixture 只应用 `[4]`；重复 migrate 返回空
  `applied`。
- v4 DDL、data backfill、index 和 `schema_migrations(version=4)` 必须在同一 transaction。
- 必须覆盖 empty DB、v3 -> v4、故障注入 rollback、重复执行和 schema-too-new。
- migration 不提供 downgrade、双写或 compatibility fallback。

### FR-03：Service-owned repository 边界

- production SQLite connection 只由 runtime daemon 的 storage owner 持有。
- `homie-app` 和 `homie-cli` 的 normal dependencies 最终不得包含 `homie-storage`。
- `homie-app` 必须删除 `open_ready_storage`、`open_or_create`、`StorageConfig` 和
  storage-owned `SettingsPreferences` 使用。
- runtime/service 调用 typed repository，不向 client/app/CLI 暴露 `Storage`、`Connection`
  或 SQL。
- runtime 的 WAL checkpoint/optimize 必须通过 storage-owned `flush` API，不直接调用
  `Storage::connection()`。
- tests 可以使用明确的 storage test harness；不得以 production public connection 作为领域
  API。

### FR-04：Settings、health 与 usage service

- settings 读取返回 `{ preferences, revision }`；更新携带 `expected_revision`，并以 compare-
  and-set 防止两个 client 静默覆盖。
- `storage.health` 返回 schema version、foreign keys、journal mode 和 safe database
  identity；它不返回 live runtime 状态。
- usage summary 复用现有 `UsageQuery`/`UsageTotals` 语义，通过 client method 返回 safe
  aggregate。
- app settings、CLI doctor 和 CLI usage summary 都必须经过 owning service/client。
- method 只有在 handler 存在并通过 integration test 后才能进入 capability discovery。

### FR-05：Immutable effective config freeze/readback

v4 必须增强现有 `effective_agent_configs`，至少承载：

- `snapshot_version`；
- agent/runtime/LLM/provider/permission profile ids；
- runtime descriptor safe snapshot；
- managed LLM route safe snapshot，仅含 local proxy URL、provider/model scope 和
  `virtual_key_id` 引用，不含 key material；
- permission safe snapshot；
- skill ids、MCP server ids、workspace scope；
- deterministic `config_hash`；
- `frozen_at`。

Repository 必须：

- 提供 session create/bind + config freeze 的原子操作，或提供语义等价的单 transaction API；
- 每个 session 最多绑定一个 frozen config；
- frozen row 没有 update API；
- 提供按 session id 的 safe readback；
- profile 后续变化不影响 readback；
- 任一外键、序列化或 session bind 失败时不留下 session/config 半写。

### FR-06：Runtime recovery facts

v4 必须新增每 session 一行的 recovery metadata，和现有 `sessions.output_log_path`、
`sessions.output_tail_offset` 联合形成 `RuntimeRecoveryFacts`。新增 metadata 至少包括：

- `holder_instance_id` 和 `holder_pid` hint；
- `holder_started_at`；
- `output_epoch`；
- checkpoint path、checkpoint output offset、checkpoint content sequence；
- checkpointed event sequence；
- last runtime instance id；
- last observed durable status；
- updated timestamp。

规则：

- pid、instance id 和 last status 都是 recovery hint，不是 live proof。
- output bytes、terminal grid 和 checkpoint blob 不进入 SQLite。
- offset、checkpoint metadata 和 durable status 的多字段更新必须原子提交。
- recovery candidate query 必须有稳定排序和上限。
- T-102/runtime 重新验证 holder/process 后，才可通过 repository 提交新的 durable
  assessment。

### FR-07：Lineage durable metadata foundation

- `sessions.parent_session_id` 继续作为 direct parent 的单一关系事实，不新增第二套 parent
  graph。
- session + parent + frozen config 的创建必须可在一个 transaction 中提交。
- 新增 safe lineage audit metadata，至少记录 operation id、actor、subject、relation/action、
  decision、safe reason code 和 timestamp。
- operation id 必须唯一；audit 不保存 prompt、完整 tool args/result 或任意 credential。
- 本 change 只提供 direct relationship 和 audit repository；recursive permission enforcement、
  summarize/report workflows 仍由 T-302/T-403 验收。

### FR-08：Remote durable metadata foundation

- 复用并增强现有 `hosts`、`node_accounts`、`handoff_records`，不得声称这些表不存在。
- handoff metadata 必须补齐 operation id、checkpoint id、phase、lease id、manifest hash 和
  safe error code。
- operation id 必须唯一；lease/checkpoint 的重复提交必须幂等或返回 stable conflict。
- repository 只保存 safe manifest metadata/hash/ref，不保存 blob、provider home、token 或
  raw key。
- 本 change 不实现 listener、SSH/node、checkpoint transfer、provider resume 或 move/fork
  commit E2E，因此 `REM-*` 仍为 `partial`。

### FR-09：Updater durable receipt foundation

- v4 新增 update receipt metadata，至少记录 operation id、from/target version、phase、
  feed host、archive SHA256、bundle id、team id、staged/previous path ref、safe error code 和
  timestamps。
- operation id 唯一；phase transition 使用 compare-and-set，非法倒退 fail closed。
- receipt 不保存 feed Authorization/basic-auth、cookie 或下载 body。
- 本 change 不实现 feed/download/sign/install/rollback，因此 `UPDATE-001`、`PKG-001` 和
  `PERF-001` 仍为 `partial`。

### FR-10：安全与数据最小化

- raw provider key、virtual key material、Authorization、cookie、raw prompt/response、
  terminal bytes、完整 tool args/result 不进入新增列、JSON、log 或 evidence。
- JSON snapshot 必须有 version、size bound 和 safe-field validation。
- paths 在 wire/error 中只按既定 safe path policy 暴露。
- corrupt JSON、unknown snapshot version、invalid phase 和 negative offset fail closed。

### FR-11：Shared contract freeze 与并行 ownership

Cross-change 合同分为两类，不得混用：

1. **Launch/effective-config shape**：由 T-102 G3 先冻结并 handoff，作为
   S103-GREEN-02 durable repository 的输入合同。T-103 不修改该 launch shape 的 owner
   文件。
2. **T-103 service methods**：以下六个 method/DTO 属于 T-103，由
   `S103-proto-integration` 在 T-102 完成且释放 shared proto/runtime 文件后实现。规格阶段
   只冻结名称和语义，不修改 `homie-proto`。

| Method | Request/response | T-103 用途 |
|--------|------------------|------------|
| `storage.health` | empty -> `StorageHealthResult` | CLI doctor durable health |
| `settings.get` | empty -> `SettingsSnapshot` | app settings load |
| `settings.update` | `SettingsUpdateRequest` -> `SettingsSnapshot` | revisioned settings save |
| `usage.summary` | `UsageSummaryRequest` -> `UsageSummaryResult` | CLI safe aggregate |
| `session.effective_config` | session id -> `EffectiveAgentConfigSnapshot` | safe freeze readback |
| `runtime.recovery.summary` | bounded filter -> `RuntimeRecoverySummary` | admin/restart diagnostics |

现有 `session.set_parent`、`session.list_children` 和 `session.parent` 继续复用，不新增同义 method。
`host.*` handoff 和 `update.*` workflow methods 由 T-401/T-501 定义；T-103 的 metadata
foundation 不得提前广告这些能力。

Cross-change 无环 DAG 固定为：

```text
T-102 G3: freeze resolved launch/effective-config shape
  -> T-103 S103-GREEN-02: implement v4 effective-config repository
  -> T-103 repository GREEN handoff gate
  -> T-102 G5: manifest spawn integration
  -> T-102 complete and release shared proto/runtime files
  -> T-103 S103-GREEN-05..07: implement six service methods and integration
```

T-103 storage-only RED、GREEN-01 及其他不触碰 T-102 文件的工作可与 T-102 并行；
S103-GREEN-02 只等待 T-102 G3 contract handoff，不等待整个 T-102。T-103 shared
proto/runtime integration 仍等待 T-102 完成并释放具体文件。

### FR-12：真实状态与准出

- `homie-storage` focused tests、migration/repository tests、runtime restart integration、
  app/CLI dependency scans 和 cross-process service tests必须通过。
- `homie-app` direct storage dependency 和调用路径必须被删除，而不是只承诺“不新增”。
- `homie-cli` durable reads 必须完成同类收口。
- 本 change 通过只代表 storage core facts foundation 完成。
- `UI-005`、`UI-006`、`API-005`、`REM-001..003`、`USAGE-001`、`UPDATE-001`、
  `PKG-001`、`PERF-001` 不得因本 change 单独改为 `implemented`。

## 5. 方案设计

### 5.1 v4 增量模型

| 对象 | v3 基线 | v4 增量 |
|------|---------|---------|
| `preferences` | key/value/updated time 已存在 | monotonic `revision`，支持 settings compare-and-set |
| `effective_agent_configs` | 表和基本引用已存在 | versioned safe snapshots、hash、session uniqueness |
| `sessions` | core metadata、parent、output fields 已存在 | frozen config/recovery transaction 约束和必要索引 |
| `session_runtime_recovery` | 不存在 | holder/output epoch/checkpoint/event/runtime hints |
| `lineage_audit_events` | 不存在 | 幂等 safe provenance/decision audit |
| `hosts/node_accounts/handoff_records` | 表已存在 | typed repository 与 operation/checkpoint/phase/lease/hash |
| `update_receipts` | 不存在 | update phase/identity/hash/path/safe error receipt |

不增加第二套 session、project、usage、history 或 worktree 表。

### 5.2 生产数据流

```text
app / CLI
  -> homie-client typed method
  -> runtime dispatcher
  -> RuntimeActor / owning domain service
  -> homie-storage typed repository
  -> SQLite v4 durable facts
  -> authoritative response/event
  -> app / CLI projection
```

Runtime recovery：

```text
load bounded durable candidates
  -> read holder/output/checkpoint evidence
  -> T-102 validates live process
  -> classify running/detached/exited
  -> atomically commit durable assessment
  -> publish runtime event/snapshot
```

### 5.3 实现 ownership

- `crates/homie-storage/src/lib.rs` 只有 `S103-storage-impl` 一个实现 owner，所有任务串行。
- T-102 不得编辑 `crates/homie-storage/src/lib.rs`；T-102 G5 只能消费 T-103 repository
  GREEN handoff。
- storage RED tests 可由独立 test owner 编写，但不得同时编辑 `lib.rs`。
- `homie-proto`、`homie-runtime/src/runtime_actor.rs`、dispatcher 和 T-102 lifecycle 文件在
  T-102 完成/释放前只读；释放后由 T-103 各自单一 integration owner 接线六个 service
  methods。
- app owner 只在 proto/client/runtime service 可用后删除 storage dependency，不增加临时
  adapter 或双路径。

## 6. 边界情况

| 场景 | 处理 |
|------|------|
| v3 fixture 含已有 effective config row | v4 backfill version/hash 所需默认值；无法安全 backfill 时整次 rollback |
| profile 在 freeze 后被修改/禁用 | 已冻结 session readback 不变；新 spawn 使用新 profile 状态 |
| duplicate settings revision | 返回 stable conflict，app 重新加载，不覆盖 |
| holder pid 被复用 | holder instance/start evidence 验证失败，不能根据 pid 标 running |
| checkpoint offset 超过 output tail | recovery fact 拒绝写入或恢复时 fail closed |
| duplicate lineage/remote/update operation | 返回原 receipt 或 stable conflict，不重复推进 |
| storage service unavailable | app/CLI 返回 safe unavailable，不回退 direct SQLite |
| T-102 G3 尚未 handoff launch/effective-config shape | S103-GREEN-02 blocked；其他不依赖该 shape 的 storage-only 工作可推进 |
| T-102 尚未完成或仍持有 shared proto/runtime file | 六个 service method integration blocked；不得反向阻塞 T-103 repository GREEN handoff |

## 7. 受影响长期规格

| Spec | 影响 |
|------|------|
| `specs/storage-indexing/README.md` | 更新 schema v3 实况、v4 增量、service ownership、recovery/effective/metadata 合同 |
| `specs/session-context-store/README.md` | 固化 parent 单一事实源、safe lineage audit、durable event/provenance 边界 |

其他长期规格本 change 只读取，不修改；shared runtime/agent/client 合同通过本 PRD 和 OpenSpec
contract-freeze 协调。

## 8. 测试计划

| 层级 | 必测内容 |
|------|----------|
| Baseline | 现有 storage 全测试和计数保持全绿 |
| Migration | empty、v3->v4、ordered applied、idempotent、rollback、too-new |
| Repository | freeze/readback、recovery atomicity、revision conflict、operation idempotency |
| Restart | daemon restart 从 facts + holder/output 重新验证，不凭 row 伪造 running |
| Boundary | app/CLI Cargo normal dependency 无 storage；source 无 direct open |
| Service | app settings、CLI doctor/usage 走 client/daemon handler |
| Security | schema/fixture/result 扫描敏感字段为零 |
| Regression | history/worktree/session/usage 现有 suites 不回归 |

## 9. 验收标准

- Bead `homie-t3u.2`、PRD、OpenSpec 和 change id 一致。
- OpenSpec proposal/design/specs/plan/tasks/alignment/delegation 完整且 strict-valid。
- v4 ordered migration 和所有 RED/GREEN contract 有可执行 task。
- effective config freeze/readback 与 runtime recovery facts 可跨 reopen/restart。
- lineage/remote/update metadata repository 只证明 foundation，不宣称工作流完成。
- `cargo tree -p homie-app -e normal` 和 `cargo tree -p homie-cli -e normal` 不包含
  `homie-storage`。
- app settings、CLI doctor/usage 经 owning service/client；没有 production direct-storage
  fallback。
- `crates/homie-storage/src/lib.rs` 只有一个实现 owner。
- 所有要求都有 RED -> GREEN -> REFACTOR -> EVIDENCE 任务和验证映射。
- parity rows 保持真实，不由本 change 单独关闭 UI/remote/usage/update rows。

## 10. Beads 追踪

- Bead: `homie-t3u.2`
- Parent: `homie-t3u`
- Depends on: closed `homie-nep` / T-101
- Cross-change DAG: T-102 G3 -> S103-GREEN-02 -> repository GREEN handoff -> T-102 G5 ->
  T-102 complete/file release -> S103 shared integration
- 关闭条件：实现与 evidence 完成后，release-readiness 明确证明本 PRD 验收；规格生成和
  OpenSpec strict pass 本身不足以关闭 Bead。

# Diri Virtual Key Credentials Functional Verification Cases

```yaml
change_id: diri-virtual-key-credentials
beads: homie-e1s
dev_loop_step: 2
status: designed
source_prd: prd-spec/features/diri-virtual-key-credentials/2026-08-07-diri-virtual-key-credentials-design.md
source_spec_review: docs/verification/diri-virtual-key-credentials/spec-review-report.md
```

## 1. Purpose

本文件定义 foundation-security 第一阶段的功能验证 Case。所有 Case 必须通过真实 `homie-llm` 公共接口或文档 gate 执行，不允许用孤立 mock 替代被测主体。

状态说明：

- `designed`: Case 已设计，等待实现后执行。
- `pass`: 已执行并通过。
- `blocked`: 环境或依赖缺失，必须写明原因。
- `fail`: 已执行但失败，必须回到实现或 Case 设计修正。
- `not_run`: 尚未执行，不得作为准出通过。

## 2. Case List

### FC-DVKC-001: Virtual key issue and matching scope validation

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-2 |
| Risk | virtual key 未绑定 session/profile/provider/model，agent 可越权调用 |
| Evidence | `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-001` |

Preconditions:

- `crates/homie-llm` 可编译。

Steps:

```bash
cargo test -p homie-llm issued_virtual_key_validates_only_for_matching_scope -- --nocapture
```

Expected:

- 命令退出码为 0。
- 通过 matching scope 和 allowed model 校验。
- wrong session 被拒绝为 scope mismatch。

Failure handling:

- 回到 `InMemoryVirtualKeyStore::issue` / `validate` 修复 scope 比对，不允许放宽断言。

### FC-DVKC-002: Revoke, expired, and unknown key fail closed

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-2, FR-5 |
| Risk | 旧 key、撤销 key 或未知 key 继续可用 |
| Evidence | `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-002` |

Steps:

```bash
cargo test -p homie-llm revoked_expired_and_unknown_virtual_keys_are_rejected -- --nocapture
```

Expected:

- expired key 返回 `Expired`。
- revoked key 返回 `Revoked`。
- unknown key 返回 `NotFound`。
- 错误字符串不包含 presented secret。

Failure handling:

- 回到 key lifecycle 状态机修复，不允许把过期时间或 revoked 状态作为非阻塞 warning。

### FC-DVKC-003: Scope denied covers session, profile, provider, and model

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-2 |
| Risk | 仅校验 session 或 model，漏掉 profile/provider 越权 |
| Evidence | `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-003` |

Steps:

```bash
cargo test -p homie-llm scope_denied_covers_profile_provider_and_model -- --nocapture
```

Expected:

- wrong agent profile 返回 `ScopeMismatch`。
- wrong provider 返回 `ScopeMismatch`。
- disallowed model 返回 `ModelNotAllowed`。

Failure handling:

- 修复 `VirtualKeyScope` equality 或 model allow-list 检查。

### FC-DVKC-004: Managed agent config is secretless

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-3, FR-5 |
| Risk | agent config/event payload 携带真实 provider key、Authorization 或 secret ref |
| Evidence | `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-004` |

Steps:

```bash
cargo test -p homie-llm managed_proxy_config_serializes_without_raw_provider_key -- --nocapture
```

Expected:

- 序列化后的 managed config 包含 local proxy URL、virtual key、expiry 和 scope。
- 序列化内容不包含 fake raw provider key、`Authorization`、`secretRef` 或 provider request header。

Failure handling:

- 删除 agent-visible config 中的 raw secret 字段，改为 secretless metadata。

### FC-DVKC-005: Raw provider key propagation is rejected for remote, MCP, agent, and logs

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-1, FR-4, FR-5 |
| Risk | raw provider key 进入 remote node、MCP tool result、agent config、log/event |
| Evidence | `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-005` |

Steps:

```bash
cargo test -p homie-llm raw_provider_key_is_rejected_for_cross_module_destinations -- --nocapture
```

Expected:

- remote node payload 拒绝 raw provider key。
- MCP payload 拒绝 raw provider key。
- managed agent config payload 拒绝 raw provider key。
- log/event payload 拒绝 raw provider key。
- virtual key/proxy config payload 允许通过。
- 错误字符串不包含 fake raw provider key。

Failure handling:

- 修复 propagation policy，不允许以 redaction 后继续传递 raw key 的方式绕过；本阶段必须 fail closed。

### FC-DVKC-006: Component spec and OpenSpec alignment gate

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-1 |
| Risk | 文档没有把 P0/P1 安全合同映射到执行任务和测试 |
| Evidence | `docs/verification/diri-virtual-key-credentials/functional-verification-report.md#fc-dvkc-006` |

Steps:

```bash
rg -n "Cross-Spec Mandatory Gates|Raw Provider Key Forbidden Matrix|Diri Behavior Parity" specs/virtual-key-credentials/README.md
rg -n "FC-DVKC-00[1-6]" openspec/changes/diri-virtual-key-credentials/tasks.md openspec/changes/diri-virtual-key-credentials/alignment-report.md
rg -n "FR-[1-5]" openspec/changes/diri-virtual-key-credentials/alignment-report.md
```

Expected:

- 组件 spec 包含 cross-spec gates、禁止传播矩阵、Diri/Homie 适配。
- OpenSpec tasks 和 alignment report 覆盖 FC-DVKC-001 到 FC-DVKC-006。
- FR-1 到 FR-5 均有任务和验证映射。

Failure handling:

- 回到 spec/OpenSpec 补齐映射，不允许实施未映射需求。

### FC-DVKC-007: Local quality and security gate

| Field | Value |
|-------|-------|
| Status | designed |
| Covers | FR-5 |
| Risk | 代码能局部通过但格式、编译或补丁安全检查失败 |
| Evidence | `docs/verification/diri-virtual-key-credentials/release-readiness-report.md` |

Steps:

```bash
cargo test -p homie-llm
cargo fmt --all -- --check
cargo check --workspace
git diff --check
```

Expected:

- 已执行命令退出码为 0。
- 若 workspace 命令因既有非本 lane 问题失败，报告必须标记 `blocked` 或 `partial`，并说明是否与本次变更相关。

Failure handling:

- 本 lane 引入的失败必须修复后复验。
- 非本 lane 既有失败不得静默改动其他 lane 文件，应在 release readiness 中记录。

## 3. Coverage Matrix

| Requirement | Cases |
|-------------|-------|
| FR-1 Component spec gates | FC-DVKC-005, FC-DVKC-006 |
| FR-2 Virtual key lifecycle | FC-DVKC-001, FC-DVKC-002, FC-DVKC-003 |
| FR-3 Managed agent config secretless | FC-DVKC-004, FC-DVKC-005 |
| FR-4 Raw key propagation denial | FC-DVKC-005 |
| FR-5 Secretless errors/evidence | FC-DVKC-002, FC-DVKC-004, FC-DVKC-005, FC-DVKC-007 |

## 4. Execution Plan

1. Run focused `cargo test -p homie-llm` cases after RED/GREEN implementation.
2. Run document alignment gates with `rg`.
3. Run local quality gates.
4. Record actual commands, exit codes and outputs in `functional-verification-report.md` and `release-readiness-report.md`.


# Code Review Report

```yaml
change_id: diri-virtual-key-credentials
beads: homie-e1s
dev_loop_steps:
  - 8
  - 9
status: pass_after_fixes
```

## 1. 审查范围

- 文件/模块：
  - `specs/virtual-key-credentials/README.md`
  - `crates/homie-llm/src/lib.rs`
  - `crates/homie-llm/tests/virtual_key.rs`
  - `crates/homie-llm/Cargo.toml`
  - `prd-spec/features/diri-virtual-key-credentials/`
  - `openspec/changes/diri-virtual-key-credentials/`
  - `docs/verification/diri-virtual-key-credentials/`
- 变更类型：新增 foundation-security PRD/OpenSpec/evidence；更新 credential component spec；新增 `homie-llm` credential contract API 和 tests。
- 调用链/数据流：
  - virtual key issue -> managed proxy config -> agent-visible serialization;
  - payload destination -> raw-key propagation guard -> fail-closed error;
  - virtual key validate -> scope/model/expiry/revoke checks.
- 参考规则：
  - `AGENTS.md`
  - `docs/development/standards.md`
  - `docs/development/quality-gates.md`
  - `specs/virtual-key-credentials/README.md`
  - `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`

## 2. 旧问题复核

| ID/标题 | 位置 | 状态 | 依据 |
|---|---|---|---|
| Component spec 缺少 cross-spec credential gate | `specs/virtual-key-credentials/README.md` | fixed | Added sections 12-15 for Diri/Homie adaptation, cross-spec gates, forbidden matrix and first-stage contract |
| revoke/expiry/scope fixtures 不足 | `crates/homie-llm/tests/virtual_key.rs` | fixed | Tests now cover unknown key, profile/provider/model denial, secretless config and raw-key propagation denial |

## 3. Findings

| 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|
| high | Security | `crates/homie-llm/src/lib.rs:80` | First implementation stored the raw provider key inside `CredentialPropagationPolicy`, making key material cloneable and longer-lived than needed. That conflicted with the spec rule that raw provider keys only appear in short-lived resolver/upstream memory. | fixed: made `CredentialPropagationPolicy` zero-state and changed `ensure_payload_is_secretless` to receive raw key material only for a single validation call |
| medium | Security | `crates/homie-llm/src/lib.rs:24` and `crates/homie-llm/src/lib.rs:39` | Derived `Debug` for `IssuedVirtualKey` and `ManagedLlmProxyConfig` would print virtual key secrets. The component spec forbids presented virtual key secret in logs/events/evidence. | fixed: implemented custom `Debug` for both types and added regression assertions in `managed_proxy_config_serializes_without_raw_provider_key` |

## 4. 对抗式复盘

- Boundary check: raw provider key no longer lives in a policy object; tests still prove the guard rejects raw-key payloads for remote node, MCP, managed agent config and log/event destinations.
- Debug/log check: issued virtual key and managed config do not reveal virtual key secret through `Debug`; serialized managed config still intentionally contains the virtual key because it is the agent-visible credential.
- Scope check: implementation did not add HTTP proxy, remote node, MCP, runtime or storage behavior.
- Error check: `RawProviderKeyForbidden` includes only destination, not payload or secret.
- Residual design risk: payload scanning is exact-match first-stage guard, not a general redaction engine. This is acceptable for the first slice because observability owns global redaction and later callers must pass the raw key they are about to propagate.

## 5. 修复摘要

- Made `CredentialPropagationPolicy` stateless.
- Added custom `Debug` for `IssuedVirtualKey` and `ManagedLlmProxyConfig`.
- Added test assertions that `Debug` output does not include virtual key secret.
- Re-ran focused tests and scoped quality gates after fixes.

## 6. 验证结果

| 命令 | 结果 | 说明 |
|---|---|---|
| `cargo test -p homie-llm` | pass | 5 integration tests passed |
| `cargo check -p homie-llm` | pass | crate check passed |
| `cargo clippy -p homie-llm --all-targets -- -D warnings` | pass | no warnings |
| `cargo fmt --package homie-llm -- --check` | pass | scoped formatting passed |
| `git diff --check -- <lane paths>` | pass | no whitespace errors in lane diff |
| `cargo check --workspace` | pass | latest serial run passed |
| `cargo fmt --all -- --check` | partial | out-of-scope `crates/homie-storage/tests/diri_storage_indexing.rs` needs formatting |

## 7. 剩余风险

- `remote-node-handoff` and `mcp-automation` specs should later explicitly reference this credential gate. This lane did not edit them because the user restricted writes.
- Full provider proxy, secret envelope persistence, usage accounting and remote/MCP E2E remain deferred to their owning lanes.


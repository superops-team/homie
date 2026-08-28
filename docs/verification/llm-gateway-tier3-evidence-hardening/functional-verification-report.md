# Functional Verification Report — llm-gateway-tier3-evidence-hardening

> 执行日期：2026-08-28
> 执行环境：macOS 15, Rust 1.95.0（`homie/rust-toolchain.toml` 锁定）
> 环境说明：`tests/gateway.rs` 使用 wiremock 在 loopback 绑定端口；沙箱禁止套接字绑定（`Operation not permitted`），因此 FC-01/FC-03/FC-04f 的集成测试在授权（escalated）环境下执行；纯单元测试（FC-04 各 mutant）在沙箱内 `--offline` 执行。

## 执行结果总览

| Case | 内容 | 结果 | 证据 |
|------|------|------|------|
| FC-01 | gateway 全量测试基线 | ✅ 44/44 pass（33 单元 + 11 集成），exit 0 | `fc-01-baseline.log` |
| FC-02 | 负向对照（密钥泄露 grep gate） | ✅ 坏样本命中、源码零命中 | `fc-02-negative-control.log` |
| FC-03 | 对抗通过（真实 axum 路由） | ✅ 11/11 集成 pass，exit 0 | `fc-03-adversarial.log` |
| FC-04 | 手动变异 M1–M5 | ✅ 5/5 杀死，恢复后 44/44 | `fc-04a..f-*.log` |
| FC-05 | 静态门禁 + 契约一致性 | ✅ diff 无空白错误、§10.1 存在 | `fc-05-static-gates.log` |
| FC-06 | 手动逐行覆盖率核对 | ✅ 见下（含 2 处 partial） | 本报告 §FC-06 |

## FC-04 变异明细

| # | 变异 | 被杀死的测试 | 日志 |
|---|------|-------------|------|
| M1 | `auth::accept` 用明文 `key.to_owned()` 替代 `hash_key(key)`（查找侧不哈希，非对称） | `create_then_accept_and_delete`、`accept_updates_last_used` | `fc-04a-mutant-m1.log` |
| M2 | `auth::accept` 删除 `UPDATE last_used_at` | `accept_updates_last_used` | `fc-04b-mutant-m2.log` |
| M3 | `policy::allow` off-by-one（`>=` → `>`） | `rate_limiter_allows_within_window`、`rate_limiter_resets_on_new_window` | `fc-04c-mutant-m3.log` |
| M4 | `routes::apply_model_route` 删除 `model.is_string()` 守卫 | `apply_model_route_passes_through_non_string_model` | `fc-04d-mutant-m4.log` |
| M5 | `config::from_file` 删除非 loopback 拒绝分支 | `rejects_non_loopback_listen` | `fc-04e-mutant-m5.log` |

每个 mutant 均先观察失败（exit 101），再恢复（python 精确反替换；沙箱禁止写 `.git/index.lock`，`git checkout` 不可用），恢复后全量 44/44 通过（`fc-04f-restored.log`）。

## FC-06 手动逐行覆盖率核对

覆盖率工具（`cargo-llvm-cov` / `cargo-tarpaulin`）与变异工具（`cargo-mutants`）均缺失，按 `quality-gates.md` §4.1/§4.2 手动 fallback。下表按「安全关键函数 → 捕获测试」逐项核对（本 change 未新增生产代码，故核对对象为 `homie-gateway` 现有安全关键路径）。

| 安全关键函数 | 捕获测试 | 状态 |
|-------------|---------|------|
| `auth::hash_key` | `create_then_accept_and_delete`, `accept_updates_last_used` | covered |
| `auth::create` | `create_then_accept_and_delete`, `list_never_returns_raw_key` | covered |
| `auth::accept` | `create_then_accept_and_delete`, `accept_updates_last_used`, `bad_key_is_rejected_and_never_forwarded`, `revoked_key_returns_unauthorized` | covered |
| `auth::delete` | `create_then_accept_and_delete` | covered |
| `auth::list`（不泄露 raw key） | `list_never_returns_raw_key` | covered |
| `auth::resolve_caller` / `authenticate` / `extract_key` | `bad_key_is_rejected_and_never_forwarded`, `revoked_key_returns_unauthorized`, `master_key_is_accepted_but_not_usage_recorded` | covered |
| `auth::constant_time_eq`（master 比较） | `master_key_is_accepted_but_not_usage_recorded`, `master_key_bypasses_policy` | covered（间接） |
| `auth::random_hex` / `hex_encode` | `random_hex_is_unique_and_sized`, `create` | covered |
| `config::from_file`（缺凭据 / 非 loopback / 模型 / policy / trim / credential_source） | 9 个 config 单测 | covered |
| `config::normalize_models`（空白映射忽略） | `blank_model_overrides_are_ignored_at_config_load` | covered |
| `upstream::resolve_credential`（static / node / node 失败） | `static_mode_resolves_static_key`, `node_mode_falls_back_to_static_key`, `node_mode_without_any_credential_errors` | covered |
| `upstream::extract_usage`（存在 / 缺失 / 非 JSON） | `extracts_usage_when_present`, `usage_absent_is_zero` | covered |
| `routes::apply_model_route`（重写 / 缺失 / 空白 / 非 JSON / 非字符串） | 6 个 routes 单测 | covered |
| `routes::route_key`（只映射 /responses） | `route_key_maps_only_responses_to_codex` | covered |
| `routes::extract_model` | `extract_model_from_body` | covered |
| `routes::check_policy`（rate-limit / quota / master 绕过） | `rate_limit_rejects_excess_requests`, `quota_rejects_when_daily_limit_exceeded`, `master_key_bypasses_policy` | covered |
| `policy::RateLimiter::allow` | `rate_limiter_allows_within_window`, `rate_limiter_resets_on_new_window`, `rate_limiter_zero_means_unconfigured` | covered |
| `policy::QuotaChecker::allow` | `quota_zero_means_unconfigured`, `quota_checks_cumulative_tokens` | covered |
| `policy::deny_response`（429 + 脱敏） | `deny_response_is_429_and_sanitized`（单元，仅状态码）；脱敏 body 由集成断言 | partial |
| `policy::record_audit` | 无直接断言 `gateway_audit` 行（仅在 deny 路径被调用） | **partial（执行但未断言）** |
| `usage::record` / `sum_tokens_since` | `records_usage_per_key`, `quota_checks_cumulative_tokens`, 集成 usage 断言 | covered |
| `db::Db::open`（WAL + schema） | 所有测试隐式覆盖 | covered（隐式） |

**核对结论**：21 个安全关键函数中 19 个 covered、2 个 partial（`deny_response` 的脱敏 body 在单元层未直接断言、`record_audit` 的执行路径无行级断言）。无 uncovered。两处 partial 已记录为 known limit（与 `failure-model.md` KL 一致）。

## 失败处理记录

无。所有 Case 首次执行即通过（基线 44/44、变异 5/5 杀死）。

## 环境限制声明

- 集成测试的 wiremock loopback 绑定在沙箱内被拒（`Operation not permitted`），已在授权环境执行并如实记录。
- `git checkout` 在沙箱内因 `.git/index.lock` 只读不可用，变异恢复改用精确文本反替换，恢复后 `git diff --stat -- homie/crates/homie-gateway` 为空、全量测试 44/44，证明无残留。

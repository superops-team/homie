# Functional Verification Cases — llm-gateway-tier3-evidence-hardening

> change_id: `llm-gateway-tier3-evidence-hardening`
> 覆盖范围：`homie-gateway` 库 crate 的安全关键路径（虚拟密钥、上游转发、模型路由、策略/配额、凭据来源）。
> 原则：每个 Case 可执行、可复现、可判定；证据以 `fc-<n>-<name>.log` 留存；失败即回到实现/修正 Case 后重跑。

## 覆盖矩阵

| 需求项 | 功能验证 Case |
|--------|--------------|
| R1 合并 failure model（每条伤害方式有捕获层） | FC-02, FC-03, FC-04, FC-06 |
| R2 adversarial + 变异 + 覆盖率 | FC-03, FC-04, FC-06 |
| R3 specs/llm-gateway.md §10 Failure Model | FC-01（文档一致性） |
| R4 证据落盘 | 全 Case 的证据文件 + FC-01 |
| 反游戏化（fail-closed / 负向对照） | FC-02 |

---

## FC-01 基线：gateway 全量测试通过

- **前置环境**：`homie/` 工作区，Rust 1.95.0（`homie/rust-toolchain.toml` 锁定）。
- **命令**：`cd homie && cargo test -p homie-gateway`
- **输入数据**：无。
- **预期输出**：44 个测试通过（33 单元 + 11 集成），0 failed。
- **通过标准**：退出码 0；输出含 `test result: ok. 44 passed; 0 failed`（分两个二进制各一条）。
- **证据路径**：`fc-01-baseline.log`
- **失败处理**：任何测试失败即 blocked，定位并修复/记录后重跑。

## FC-02 负向对照：密钥泄露 grep gate 必须能失败

- **目的**：证明「凭据不落入日志/错误体」的检查器 fail-closed，而非空转。
- **命令**：
  ```bash
  grep -RnaE 'sk-[A-Za-z0-9_-]{32,}|upstream-secret|Authorization[[:space:]]*:[[:space:]]*Bearer' homie/crates/homie-gateway --include='*.rs' | grep -v 'test\|format!\|sk-\|placeholder'
  ```
- **前置环境**：在 `/tmp` 构造一个已知坏样本文件 `evil.rs` 含 `sk-` 明文，正样本为 gateway 源码（应无命中）。
- **输入数据**：`evil.rs` 含 `let key = "sk-sample-0123456789abcdef0123456789abcdef";`（明确为 sanitized sample，非真实密钥）。
- **预期输出**：坏样本被命中（非零）；gateway 源码无命中（零）。
- **通过标准**：坏样本命中 + 源码零命中同时成立；`git status --short` 无脏文件。
- **证据路径**：`fc-02-negative-control.log`
- **失败处理**：若源码命中，定位是否真实泄露（高严重度，另立 bugfix）或测试/注释误报。

## FC-03 对抗通过：恶意/畸形输入走真实 axum 路由

- **命令**：`cd homie && cargo test -p homie-gateway --test gateway`
- **前置环境**：wiremock 上游（测试内自动启动）。
- **对抗用例 → 现有集成测试映射**：

| 对抗输入 | 预期行为 | 覆盖测试 |
|---|---|---|
| 无效虚拟密钥 | 401，不转发、不记 usage | `bad_key_is_rejected_and_never_forwarded` |
| 已撤销虚拟密钥 | 401，不转发 | `revoked_key_returns_unauthorized` |
| `/v1/messages`（已移除路由） | 404 | `messages_route_is_gone` |
| 主密钥 | 放行但不记 usage | `master_key_is_accepted_but_not_usage_recorded` |
| 配置了模型映射的请求 | 重写 `model` 且按重写后记录 | `codex_model_is_rewritten_before_forward_and_recorded` |
| 未配置/空白映射 | pass-through 原样 | `unconfigured_model_passes_through_unchanged`、`blank_configured_model_passes_agent_model_through` |
| 非 JSON / 非字符串 model | pass-through（单元） | `apply_model_route_passes_through_non_json`、`..._non_string_model` |
| 超额请求 | 429 `rate_limit_error`，不转发第二次 | `rate_limit_rejects_excess_requests` |
| 超配额 | 429 `quota_error` | `quota_rejects_when_daily_limit_exceeded` |
| 主密钥绕策略 | 不受限 | `master_key_bypasses_policy` |

- **通过标准**：11 集成测试全部通过；退出码 0。
- **证据路径**：`fc-03-adversarial.log`
- **失败处理**：任何失败回退修复实现或修正 Case 后重跑。

## FC-04 手动变异：5 个真实 bug 必须被逐个杀死

- **目的**：验证测试套件不是空转；每个变异必须导致至少一个测试失败。
- **命令**：逐个引入变异 → `cd homie && cargo test -p homie-gateway` → 观察失败 → `git checkout -- <file>` 恢复。
- **变异清单**（真实 bug，非等价变异）：

| # | 位置 | 变异 | 应被哪个测试杀死 |
|---|------|------|------------------|
| M1 | `auth.rs` `accept` | 用明文 `key.to_owned()` 替代 `hash_key(key)`（查找侧不哈希，非对称） | `create_then_accept_and_delete` |
| M2 | `auth.rs` `accept` | 删除 `UPDATE last_used_at` 语句 | `accept_updates_last_used` |
| M3 | `policy.rs` `allow` | `*count >= limit` 改为 `*count > limit`（off-by-one） | `rate_limiter_allows_within_window` |
| M4 | `routes.rs` `apply_model_route` | 删除 `model.is_string()` 守卫 | `apply_model_route_passes_through_non_string_model` |
| M5 | `config.rs` `from_file` | 删除非 loopback 拒绝分支 | `rejects_non_loopback_listen` |

- **通过标准**：5/5 变异均被杀死（每个变异导致测试非零退出）；恢复后 44/44 全绿；`git status --short` 干净。
- **证据路径**：`fc-04a-mutant-m1.log` … `fc-04f-restored.log`
- **失败处理**：某变异未被杀死 = 该行无测试覆盖，必须补测试（如需改 `Tests/` 则另立 bugfix）或记录为 known limit。

## FC-05 反游戏化 / 契约一致性

- **目的**：确认证据不虚报——未运行的层必须显式说明；`specs/llm-gateway.md` §10 已含 Failure Model。
- **命令**：`git diff --check`、`git status --short`、`grep -n 'Failure Model' specs/llm-gateway.md`
- **预期输出**：diff 无空白错误；无脏文件；§10 含 Failure Model 标题。
- **通过标准**：三条同时成立。
- **证据路径**：`fc-05-static-gates.log`
- **失败处理**：任一失败即 blocked。

## FC-06 手动逐行覆盖率：安全关键路径每行有测试

- **目的**：无覆盖率工具，以手动逐行核对替代；证明 gateway 安全关键函数每个分支/行被某测试执行。
- **命令**：无（静态核对）；核对表写入 `functional-verification-report.md`。
- **通过标准**：安全关键函数清单的「已核对行 / 关键行」= 100%（或未覆盖行显式标注 known limit）。
- **证据路径**：`functional-verification-report.md` 内逐行核对表 + FC-04 交叉验证。
- **失败处理**：未覆盖行 → 补测试或记录 known limit。

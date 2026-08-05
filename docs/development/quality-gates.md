# Homie 准出门禁规范

## 1. 目标

Homie 的代码准出信任来自可复现证据，而不是口头说明。每个实质变更必须根据风险等级运行对应门禁，并把结果写入 `docs/verification/<change-id>/`。

门禁设计参考 evidence-first gauntlet 思路：SPEC -> RED -> GREEN -> REFACTOR -> GAUNTLET -> EVIDENCE。

## 2. 风险分层

| Tier | 适用范围 | 最低门禁 |
|------|----------|----------|
| Tier 1 trivial | 文档、注释、无行为配置 | format/lint + secret scan + `git diff --check` |
| Tier 2 normal | 普通功能、Bug 修复、UI 小改 | Tier 1 + unit/integration tests + changed-line coverage + real execution |
| Tier 3 high-stakes | secret、LLM proxy、SQLite migration、permission、runtime/process、并发、public protocol、cost/metrics | Tier 2 + property/fuzz/mutation 或手工 mutation + security/supply-chain + stress/failure tests + evidence report |

## 3. 通用门禁类别

| 类别 | 目的 | Rust/Swift 对应 |
|------|------|----------------|
| Spec Gate | 需求是否清晰、可测、获批 | `prd-spec/`、`specs/`、`openspec/`、spec review |
| Build Gate | 是否能编译 | `cargo check/build`、`swift build` |
| Format Gate | 格式一致 | `cargo fmt --check`、`swift-format` 或 SwiftPM 约定 |
| Lint Gate | 静态问题 | `cargo clippy -- -D warnings`、SwiftLint 后续接入 |
| Unit Gate | 纯逻辑正确性 | `cargo test --workspace --lib`、`swift test` |
| Integration Gate | crate/进程/API/SQLite 集成 | `cargo test --workspace --tests` |
| Coverage Gate | 变更行是否被测试覆盖 | `cargo llvm-cov`，changed-line review |
| Mutation Gate | 测试是否能抓 bug | `cargo-mutants` 或手工 mutation |
| Property/Fuzz Gate | 不变量和 hostile input | `proptest`、`cargo fuzz` 后续 |
| Real Execution Gate | 不只在测试里通过 | app/CLI/runtime/proxy smoke |
| Security Gate | secret、权限、注入、数据泄漏 | `.githooks/pre-commit`、cargo-audit、capability diff |
| Supply Chain Gate | 依赖风险 | cargo-audit、cargo-deny/cargo-license 后续 |
| Performance Gate | 性能预算 | `criterion`、`hyperfine`、manual app startup/latency |
| UI Gate | UI 可用与视觉回归 | GPUI screenshot/manual smoke，后续 Playwright-like harness |
| Suite Health Gate | 测试稳定性 | 随机顺序/重复运行/flake rerun |
| Evidence Gate | 证据可复现 | `docs/verification/<change-id>/*.md` |

## 4. 当前命令基线

项目尚未初始化 Rust/Swift workspace 时，门禁报告必须写 `not_run` 并说明原因。一旦对应工程存在，以下命令成为默认：

### Rust

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --tests
```

可选但 Tier 3 推荐：

```bash
cargo llvm-cov --workspace --branch
cargo mutants --file <changed-file>
cargo audit
cargo deny check
```

### Swift

```bash
swift build
swift test
```

Swift formatting/linting在 Swift package 初始化后确定；未接入前在 evidence 中写 `not_run`，不能写 pass。

### Security

```bash
.githooks/pre-commit
git diff --check
```

### Real Execution

V1 工程存在后至少包含：

```bash
cargo run -p homie-cli -- doctor
cargo run -p homie-cli -- runtime status
cargo run -p homie-app
```

涉及 LLM proxy：

```bash
cargo run -p homie-cli -- llm proxy-status
```

## 5. 专项门禁

### 5.1 SQLite / Storage

触发条件：

- migration、schema、repository、transaction、backup/quarantine。

必须覆盖：

- migration 从空库开始成功。
- foreign key 开启并生效。
- 唯一约束生效。
- corrupt database/state 处理不覆盖原数据。
- transaction rollback 不留下半写状态。
- output log 文件与 SQLite offset/index 一致。

### 5.2 Secret / Credential

触发条件：

- encrypted local secret envelope、provider key、virtual key、Authorization。

必须覆盖：

- raw key 不进入 agent env/config/log/event/context/metrics/report。
- encrypted envelope 解密失败 fail closed。
- revoked/expired virtual key 被拒绝。
- wrong session/profile/provider/model scope 被拒绝。
- safe error code 不泄漏 header/body。

### 5.3 LLM Proxy / Metrics

触发条件：

- OpenAI-compatible endpoint、provider forwarding、usage/cost/cache/tool metrics。

必须覆盖：

- fake provider success/failure/streaming。
- token usage 写入 SQLite。
- cache hit rate 单请求和聚合口径正确。
- pricing snapshot/currency 保存，历史 cost 不随新价格漂移。
- first-token latency、total latency 写入。
- metrics 写失败不阻塞 LLM 响应，产生 `metrics.write_failed`。
- raw request/response 不进入 metrics。

### 5.4 Runtime / PTY / Process

触发条件：

- session spawn/input/resize/terminate/output/status。

必须覆盖：

- fake/shell/Codex runtime smoke。
- process group kill。
- resize 更新 terminal dimensions。
- output log append 与 read_output 一致。
- runtime restart 后历史 session 可读。
- PTY 读取不阻塞 async runtime。

### 5.5 Agent Profile / Permission

触发条件：

- runtime descriptor、agent profile、skills/MCP config、permission profile。

必须覆盖：

- default profile 唯一。
- disabled default profile 不能启动 session。
- running session 使用冻结 `EffectiveAgentConfig`。
- permission profile 明确绑定，禁止隐式 full access。
- profile-skill/MCP binding 唯一。

### 5.6 Desktop UI

触发条件：

- GPUI app shell、sidebar、terminal pane、settings、usage summary。

必须覆盖：

- app 启动到首帧。
- 创建 session flow 可用。
- provider/profile settings 不展示 raw key。
- runtime disconnected/unhealthy 状态可见。
- token/cache/cost/tool latency summary 可见。
- 窄窗口和最小窗口不重叠。

## 6. Evidence Report 要求

每个 `docs/verification/<change-id>/release-readiness-report.md` 必须列出：

- source PRD/spec。
- OpenSpec change。
- Beads issue。
- 风险 Tier。
- 每个门禁命令、退出码、结果。
- 未运行门禁和原因。
- 新依赖和对应 spec/research justification。
- 失败项、修复动作、复验结果。
- 残余风险和后续 Beads。

状态只能是：

- `pass`
- `blocked`
- `not_run`
- `partial`

禁止把 `not_run` 写成 `pass`。

## 7. 初始 Makefile 目标建议

Rust/Swift workspace 初始化后应提供统一入口：

```text
make fmt
make lint
make test-fast
make test
make coverage
make security
make smoke
make pre-commit
make full-check
make gauntlet
```

建议含义：

| Target | 内容 |
|--------|------|
| `fmt` | Rust/Swift format |
| `lint` | clippy + Swift lint |
| `test-fast` | unit + focused integration |
| `test` | full local tests |
| `coverage` | changed-line or workspace coverage |
| `security` | pre-commit + cargo-audit + dependency/license checks |
| `smoke` | doctor/runtime/proxy/app smoke |
| `pre-commit` | commit 前必跑 |
| `full-check` | 合并/交付前必跑 |
| `gauntlet` | Tier 3 全套门禁 |

## 8. Anti-Gaming Rules

1. 不得削弱测试让代码通过。
2. 不得把未运行门禁写成 pass。
3. 不得为追 coverage 写无断言测试。
4. 不得 mock 被测主体，只 mock 外部边界。
5. 不得静默新增依赖、网络、文件系统、shell、secret 能力。
6. 不得绕过 `.githooks/pre-commit`。

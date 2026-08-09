# Code Review Report Round 1

```yaml
change_id: diri-agent-detection
beads: homie-v4b
review_skill: code-review
round: 1
status: pass
```

## 1. 审查范围

- 文件/模块：
  - `specs/agent-adapter-contract/README.md`
  - `assets/agent-descriptors/*.json`
  - `crates/homie-agents/src/lib.rs`
  - `crates/homie-agents/src/detect/redact.rs`
  - `crates/homie-agents/tests/*`
  - `prd-spec/features/diri-agent-detection/*`
  - `openspec/changes/diri-agent-detection/*`
  - `docs/verification/diri-agent-detection/*`
- 变更类型：新增文档、更新组件规格、替换 agent manifests、增加 Rust API 与 tests。
- 调用链/数据流：combined manifest -> `ManifestEngine`/`AgentCatalog` -> readiness/golden tests；hook payload -> parser -> redaction -> `NeedsInputDetail`。
- 参考规则：`AGENTS.md`、`docs/development/standards.md`、`docs/development/quality-gates.md`、`prd-spec/features/diri-agent-detection/2026-08-07-diri-agent-detection-design.md`。

## 2. 旧问题复核

| ID/标题 | 位置 | 状态 | 依据 |
|---|---|---|---|
| Diri catalog/readiness/golden/redaction mapping missing | `specs/agent-adapter-contract/README.md` | fixed | 新增 Diri parity mapping、readiness、golden、redaction、failure mode 和 FC-DA gates |

## 3. Findings

| 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|
| medium | Correctness | `crates/homie-agents/src/detect/redact.rs` | RED test 显示 secret-bearing header value 泄漏到 safe summary | fixed: 使用 header/URL sensitive-value redaction regex，`hook_parser_redacts_nested_headers_and_url_query_secrets` 已通过 |
| medium | Correctness | `crates/homie-agents/src/lib.rs` | RED test 显示 `sessionIDFlag`、`claudeMCP`、`codexMCP` 未被 serde `camelCase` 正确解析 | fixed: 增加精确 `serde(rename = "...")`，manifest catalog tests 已通过 |
| low | Naming/Format | `crates/homie-agents/src/**/*.rs`, tests | `cargo fmt --all -- --check` 初次失败，存在格式差异 | fixed: 运行 `cargo fmt --all`，复验通过 |

## 4. 对抗式复盘

- 反例：unknown agent id 如果沿用 Diri fallback 但保留 approve/deny，会导致下游 quick action 误触发。当前 `AgentManifest::fallback` 显式 `approve=None`、`deny=None`、`binary=None`、`StatusAuthority::Process`。
- 反例：readiness 如果探测 `shell`/`generic`，会把伪 agent 当成缺失 CLI。当前 `launchable()` 只返回 `binary.is_some()`。
- 反例：full manifest 如果没有 rules，golden tests 可能静默退化为 no-op。当前 `every_bundled_manifest_decodes_strictly` 断言 full manifest 必须有 rules。

## 5. 修复摘要

- 修复 redaction inline/header/URL secret 覆盖。
- 修复 Diri manifest acronym field serde mapping。
- 机械格式化 Rust 代码和 tests。

## 6. 验证结果

| 命令 | 结果 | 说明 |
|---|---|---|
| `cargo fmt --all -- --check` | pass | 格式复验通过 |
| `cargo check -p homie-agents` | pass | focused check |
| `cargo clippy -p homie-agents --all-targets -- -D warnings` | pass | no warnings |
| `cargo test -p homie-agents` | pass | 31 tests passed plus doc-tests |
| `git diff --check -- ...scoped paths...` | pass | scoped whitespace check |
| `.githooks/pre-commit` | pass | security hook |

## 7. 剩余风险

- Protocol projection remains older shape in `homie-proto`; intentionally out of scope for this lane.

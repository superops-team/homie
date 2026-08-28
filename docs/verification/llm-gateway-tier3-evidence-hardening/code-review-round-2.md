# Code Review Round 2 — llm-gateway-tier3-evidence-hardening

对抗式复盘：构造反例、复查边界与未覆盖分支、复核第一轮修复是否引入新问题。

## 旧问题复核

| ID | 状态 | 依据 |
|----|------|------|
| R1-01 | fixed | FM-04 现引用 M3（rate-limit off-by-one） |
| R1-02 | fixed | FM-03 现引用 `fc-01-baseline.log` |
| R1-03 | fixed | FC-04 M1 改为非对称变异 |

## 对抗式复盘

- **证据真实性**：44/44、11/11、5/5 数字均来自实际命令输出并留存日志，无形容词替代数字；`deny_response` 单测只断言 429 状态码（body 脱敏由集成断言）已在 `functional-verification-report.md` 标为 partial，未虚报。
- **变异恢复**：沙箱禁止 `git checkout`（`.git/index.lock` 只读），恢复用精确反替换；已用 `git diff --stat -- homie/crates/homie-gateway`（空）与全量 44/44 交叉验证无残留。
- **范围蔓延**：`git status --short` 含两处非本 change 的既有修改（`homie-engine/src/bin/homied-rs.rs`、`homie-engine/src/mcp/http.rs`，Tokio reactor 修复），经 `git diff` 确认为本会话开始前已存在，不属于本 change；提交时仅纳入本 change 文件。
- **known limits 完整性**：`failure-model.md` 已显式记录 KL-04a/b、KL-05a/b、KL-06a、KL-07a/b、KL-08a，无「静默未覆盖」项。
- **契约一致性**：`specs/llm-gateway.md` §10.1 的 Failure Model 表与 `failure-model.md` 口径一致；未改动任何生产代码。

## 撤回/降级

无。第一轮 findings 修复未引入新问题。

## 新增风险（非阻断）

- `policy::record_audit` 的 `gateway_audit` 行无行级断言（仅执行路径被覆盖），已作为 partial 与 known limit 记录，建议后续独立 bugfix 补断言。

## 结论

无 P0/P1 问题，代码（文档/证据）质量达标。

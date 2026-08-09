# Release Readiness Report

```yaml
change_id: diri-runtime-daemon-client-transport
beads: homie-nep
date: 2026-08-08
wave_1a_status: pass
full_workspace_status: partial
diri_parity_status: partial
bead_closure: closed
```

## 1. 准出结论

`diri-runtime-daemon-client-transport` 已满足 PRD 9.1、9.2 和 9.3：

- per-data-dir daemon、explicit launcher、single UDS fixed-frame multiplex transport 已实现；
- app、CLI、MCP 使用同一 daemon instance；
- pure async client 的 request/event/terminal、heartbeat、reconnect、snapshot/offset recovery 已实现；
- embedded/sync production client 已删除；
- bounded queue、capability truth、graceful shutdown、package dependency closure 已验证；
- API-002 的 shared-daemon/reconnect/event/terminal E2E 通过。

因此 Wave 1A scoped release readiness 为 `pass`，允许关闭 `homie-nep`。

## 2. Gate Matrix

| Gate | 状态 | 证据 |
|---|---|---|
| Chinese PRD / component specs / OpenSpec alignment | pass | `spec-review-report.md`, `alignment-report.md` |
| OpenSpec strict validation | pass | `openspec validate diri-runtime-daemon-client-transport --strict` |
| fmt / workspace check / strict clippy | pass | `test-report.md` |
| protocol/runtime/client focused suites | pass | `wave-b-verification-report.md`, `test-report.md` |
| app/CLI/MCP shared-daemon E2E | pass | `e2e-report.md` |
| two-round code review | pass | `code-review-report.md`; 4 findings fixed, second round clean |
| security review | pass | `security-review-report.md`; 0 reportable findings |
| package closure and smoke | pass | `PACKAGED_RUNTIME_SMOKE=pass`, `HELLO_STATE_SNAPSHOT=pass` |
| parity lock | pass | API-002 promoted; only proven row updated |
| diff check | pass | `git diff --check` |
| test process cleanup | pass | test daemon=0, test holder=0 |

## 3. Full Workspace 分类

完整 workspace 不记为全绿：

- T-102 holder/PTY suite：10/13 pass，3 个已知 failures；
- Trae sandbox 拒绝既有 `~/.codex/state_5.sqlite*` 与 ByteSec hook state。

PRD 明确允许 Wave 1A 在 transport/client/daemon scope 内 pass，但要求 RT-001、RT-006、RT-007 保持 partial。当前 parity lock 遵守该约束。

## 4. Review 修复

最终 code review 额外关闭：

1. app event stream close/subscribe failure 后的自动 snapshot/resubscribe；
2. terminal reopen confirmed offset 透传；
3. app command backpressure 的显式反馈与 pending attach 清理；
4. holder launch/status persistence 失败后的 process/session rollback。

对应 RED/GREEN 和 focused regression tests 已记录在 `code-review-report.md`。

## 5. Package 与发布边界

最新 `make smoke` 结果：

```text
PACKAGED_RUNTIME_SMOKE=pass
HELLO_STATE_SNAPSHOT=pass
GUI_LAUNCH=not_run
NOTARIZATION=not_required
```

App bundle 与 tarball CLI closure 均包含 executable `homie-runtime-daemon` 和 `homie-runtime-holder`，并完成 nested local signing。GUI launch、notarization、DMG、updater 和 perf gate 仍由后续 package/release change 负责，PKG-001/PERF-001 保持 partial。

## 6. Parity 与后续任务

本 change 只将 `API-002` 更新为 implemented。以下状态不变：

- RT-001、RT-006、RT-007：T-102 blocker；
- API-001、API-003、API-004、API-005：仍有完整 wire/forward/browser/permission 缺口；
- UI、remote/node、usage、updater、visual/perf：继续 partial。

不得据此声明 Homie 已完整对齐 Diri。

## 7. 证据索引

- `docs/verification/diri-runtime-daemon-client-transport/spec-review-report.md`
- `docs/verification/diri-runtime-daemon-client-transport/wave-b-verification-report.md`
- `docs/verification/diri-runtime-daemon-client-transport/test-report.md`
- `docs/verification/diri-runtime-daemon-client-transport/e2e-report.md`
- `docs/verification/diri-runtime-daemon-client-transport/code-review-report.md`
- `docs/verification/diri-runtime-daemon-client-transport/security-review-report.md`
- `docs/verification/diri-runtime-daemon-client-transport/release-readiness-report.md`

## 8. 最终决策

Wave 1A：`release-ready`。  
Full Diri parity：`not ready`。  
Bead `homie-nep`：已依据本报告关闭。

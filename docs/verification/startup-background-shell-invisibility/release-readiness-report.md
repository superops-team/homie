# Release Readiness Report

## 1. 基本信息

- Beads：`homie-f21`
- change_id：`startup-background-shell-invisibility`
- PRD：`prd-spec/bugfixes/startup-background-shell-invisibility/2026-08-12-startup-background-shell-invisibility-design.md`
- OpenSpec：`openspec/changes/startup-background-shell-invisibility/`
- 风险等级：P0 startup/runtime architecture

## 2. 实施摘要

- Rust Engine 启动不再执行交互 login shell 捕获 PATH。
- 新增 Rust `environment` 模块，提供 startup fallback/cache/override PATH 和用户触发的 bounded lazy refresh。
- 新增 `environment.refresh_path` control method 和 Rust client wrapper。
- 用户打开 new-agent picker 时触发 agent catalog refresh；普通 app/daemon 启动不触发 PATH shell refresh。
- Swift daemon/holder/client/detection/git targets、source、tests 已删除。
- README、CONTRIBUTING、ROADMAP、UPDATING、scripts 和 Rust 注释已清理 Swift daemon legacy 叙述。
- 删除迁移期文档 `homie/PLAN.md`、`homie/PORT.md`、`homie/PERF-AUDIT.md`、`homie/REMOTE_PORT.md`。

## 3. 门禁结果

| Gate | Command | Result | Evidence |
|------|---------|--------|----------|
| Spec review | review-spec | pass | `spec-review-report.md` |
| Functional cases | FC-01..FC-08 | pass | `functional-verification-report.md` |
| Rust env tests | `cargo test --manifest-path homie/Cargo.toml -p homie-engine environment -- --nocapture` | pass | `fc-01-02-environment-tests.log` |
| Heavy rc smoke | `bash run-heavy-rc-smoke.sh` | pass | `fc-03-heavy-rc-smoke.log` |
| Startup exec probe | `bash run-startup-exec-probe.sh` | pass | `fc-04-startup-exec-probe.log` |
| Swift package cleanup | `swift package dump-package` + scan | pass | `fc-05-swift-package.json`, `fc-05-swift-cleanup.log` |
| Swift build | `swift build` | pass | `fc-06-swift-build.log` |
| Rust gates | `cargo fmt --check` + `cargo check --workspace` | pass | `fc-07-rust-gates.log` |
| Legacy scan | product scope `rg` | pass | `fc-08-legacy-scan.log` |
| Code review round 1 | manual review | pass | `code-review-round-1.md` |
| Code review round 2 | manual review | pass | `code-review-round-2.md` |
| E2E | app/engine build + startup probes | pass | `e2e-report.md` |
| Whitespace | `git diff --check` | pass | command output |

## 4. 残余风险

- `environment.refresh_path` 使用用户触发的非交互 `shell -l -c 'printenv PATH'`，符合当前 OpenSpec 默认策略；若后续产品要求完全禁止任何 shell PATH refresh，需要将 T3 收敛为纯 fallback/cache/manual PATH。
- 本次删除 Swift daemon/holder 后，Swift tests 仅覆盖保留 targets；历史 Swift daemon tests 不再适用。
- UI 诊断面板展示后台任务状态属于 P1，未纳入本次 P0。

## 5. 准出结论

P0 需求已完成并通过验证。可提交并关闭 `homie-f21`。

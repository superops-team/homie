# Full Dev Bundle Smoke Functional Verification Report

## 1. 结论

`full-dev-bundle-smoke` 首阶段功能验证通过。

已完成：

- `homie/scripts/dev.sh --full`
- `homie/scripts/dev.sh --full --no-launch`
- `homie/scripts/dev.sh --full --no-launch --smoke`
- full dev bundle 复用 `package.sh --phase verify --app <path>`
- smoke 使用临时 Engine app support/socket，并通过随包 CLI `status` 做 socket round-trip

## 2. Case 执行结果

| Case | 状态 | 证据 |
|---|---|---|
| FC-01 PRD/spec 和 review 风险已收敛 | pass | `fc-01-spec-review.log` |
| FC-02 OpenSpec 三件套完整并覆盖 Case | pass | `fc-02-openspec-alignment.log` |
| FC-03 dev.sh 参数解析和默认路径不回退 | pass | `fc-03-help-syntax.log` |
| FC-04 生成 full dev bundle 并复用 package verify | pass | `fc-04-full-dev-bundle.log` |
| FC-05 临时 Engine smoke 连通随包 CLI | pass | `fc-05-engine-smoke.log` |
| FC-06 静态门禁和范围守卫 | pass | `fc-06-static-gates.log` |

## 3. 关键证据

- Full dev bundle 路径：`/Users/bytedance/workspace/github/homie/homie/target/homie-dev-3f38a56e.s5uHZF/homie dev 3f38a56e.app`
- `package.sh --phase verify --app <app>` 成功，日志显示 `Verified ... homie dev ... app`。
- Temporary Engine smoke 成功，随包 CLI 输出 `No active sessions.`。
- Smoke 日志不包含真实 `~/Library/Application Support/Homie` 路径。
- `bash -n scripts/*.sh homie/scripts/*.sh` 通过。
- `git diff --check` 通过。

## 4. 修复记录

- 首次 full smoke 发现空 `cargo_args[@]` 在 Bash `set -u` 下触发 unbound variable，已抽出 `cargo_build` helper 修复。
- 首次 smoke 使用 `homie doctor` 暴露真实 Application Support state file 检查，已改为随包 CLI `status`，只验证临时 socket round-trip。

## 5. 残余风险

- Full dev smoke 当前不打包 remote helper catalog、sidecar、DMG、notary 或 updater zip，符合首阶段非目标。
- CI full-dev smoke 未接入，符合首阶段“本地先稳定”的边界。

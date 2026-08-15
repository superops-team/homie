# Full Dev App Bundle 功能验证 Case

## 1. 验证目标

本验证 Case 面向 `full-dev-bundle-smoke` 首阶段，目标是证明：

- `dev.sh --full --no-launch --smoke` 生成本机架构 full dev app；
- bundle 包含 GUI、Engine、holder、askpass、MCP、Swift CLI、manifest catalog；
- smoke 使用临时 `HOMIE_APP_SUPPORT` 和 `HOMIE_SOCKET`，不触碰真实用户状态；
- full dev bundle 复用 `package.sh --phase verify` 的核心 bundle 检查；
- 默认快速 UI dev 路径仍保留。

## FC-01: PRD/spec 和 review 风险已收敛

```bash
test -s docs/verification/full-dev-bundle-smoke/spec-review-report.md
rg -n "首阶段关闭口径|本机架构|HOMIE_APP_SUPPORT|full-dev-bundle-smoke|package verify" \
  prd-spec/refactors/full-dev-bundle-smoke/2026-08-13-full-dev-bundle-smoke-design.md \
  docs/verification/full-dev-bundle-smoke/spec-review-report.md
```

通过标准：命中首阶段范围、临时 app support 和 package verify 复用口径。

证据路径：`docs/verification/full-dev-bundle-smoke/fc-01-spec-review.log`

## FC-02: OpenSpec 三件套完整并覆盖 Case

```bash
test -s openspec/changes/full-dev-bundle-smoke/plan.md
test -s openspec/changes/full-dev-bundle-smoke/tasks.md
test -s openspec/changes/full-dev-bundle-smoke/alignment-report.md
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06" \
  openspec/changes/full-dev-bundle-smoke/tasks.md \
  openspec/changes/full-dev-bundle-smoke/alignment-report.md
```

通过标准：三件套存在，并覆盖 FC-01 至 FC-06。

证据路径：`docs/verification/full-dev-bundle-smoke/fc-02-openspec-alignment.log`

## FC-03: dev.sh 参数解析和默认路径不回退

```bash
bash -n homie/scripts/dev.sh
homie/scripts/dev.sh --help
rg -n "full|no-launch|smoke|Launching" homie/scripts/dev.sh
```

通过标准：

- `bash -n` 通过；
- help 输出包含 `--full`、`--no-launch`、`--smoke`；
- 默认路径仍包含 launch 逻辑，未被 full mode 覆盖。

证据路径：`docs/verification/full-dev-bundle-smoke/fc-03-help-syntax.log`

## FC-04: 生成 full dev bundle 并复用 package verify

```bash
homie/scripts/dev.sh --full --no-launch --smoke
```

通过标准：

- 输出 `.app` 路径；
- 调用或等价执行 `package.sh --phase verify --app <app>`；
- bundle 内核心二进制均存在且可执行；
- manifest 数量与 source catalog 一致；
- 不构建 release remote helper catalog、不 notarize、不生成 DMG/update zip。

证据路径：`docs/verification/full-dev-bundle-smoke/fc-04-full-dev-bundle.log`

## FC-05: 临时 Engine smoke 连通随包 CLI

```bash
homie/scripts/dev.sh --full --no-launch --smoke
```

通过标准：

- 使用临时 `HOMIE_APP_SUPPORT` 启动随包 `homied-rs`；
- 使用临时 `HOMIE_SOCKET` 调用随包 CLI；
- `homie doctor` 成功完成；
- smoke 结束后随包 Engine 被关闭或终止；
- smoke log 不包含真实 `~/Library/Application Support/Homie` 写入路径。

证据路径：`docs/verification/full-dev-bundle-smoke/fc-05-engine-smoke.log`

## FC-06: 静态门禁和范围守卫

```bash
bash -n scripts/*.sh homie/scripts/*.sh
git diff --check
git diff --name-only -- homie/scripts/dev.sh homie/scripts/package.sh
```

通过标准：脚本语法和 diff whitespace 通过，范围守卫只显示预期脚本路径。

证据路径：`docs/verification/full-dev-bundle-smoke/fc-06-static-gates.log`

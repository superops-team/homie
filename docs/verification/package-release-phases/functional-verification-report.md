# Package/Release Phases Functional Verification Report

## 1. 结论

`package-release-phases` 首阶段功能验证通过。

已完成：

- `homie/scripts/package.sh --phase preflight`
- `homie/scripts/package.sh --phase verify --app <path>`
- CI bundle job 复用 package verify phase
- `homie/PACKAGING.md` phase 用法说明

未执行完整 package：当前机器缺少 release 所需 Rust targets，preflight 已在长构建前报出：

- `x86_64-apple-darwin`
- `aarch64-unknown-linux-musl`

## 2. Case 执行结果

| Case | 状态 | 证据 |
|---|---|---|
| FC-01 依赖分析和 PRD 风险修复 | pass | `fc-01-spec-dependency.log` |
| FC-02 OpenSpec 三件套完整并覆盖 Case | pass | `fc-02-openspec-alignment.log` |
| FC-03 package 脚本语法和帮助输出 | pass | `fc-03-help-syntax.log` |
| FC-04 preflight 早失败 | pass | `fc-04-preflight.log` |
| FC-05 verify phase 验证已存在 app 且只读 | pass | `fc-05-verify-existing-app.log` |
| FC-06 verify phase 缺失资源失败且只读 | pass | `fc-06-verify-failure-readonly.log` |
| FC-07 默认 package 路径和 CI 复用 verify | pass | `fc-07-default-ci-gates.log` |

## 3. 关键证据

- `bash -n homie/scripts/package.sh` 通过。
- `package.sh --help` 输出 `--phase preflight`、`--phase verify` 和 `--app`。
- `--phase preflight` 未出现 `==> Building homie for Apple silicon`、`==> Building homie for Intel`、`cargo packager`，证明失败发生在长构建前。
- `--phase verify --app homie/dist/homie-dev-7eef934e-arm64-test.app` 通过，并且 app 文件 mtime hash 前后相同。
- 缺失资源的临时 `bad.app` verify 非 0 失败，测试 app 仍存在。
- CI bundle job 调用 `homie/scripts/package.sh --phase verify --app "${app}"`。

## 4. 残余风险

- 当前本机无法执行完整 default package flow，因为缺少 release targets。该风险由 FC-04 preflight 记录，属于环境阻塞，不是实现失败。
- `--local-arm64` 与 `--skip-build` 未在本切片实现，符合 OpenSpec out-of-scope。
- `verify` 当前对 release bundle 强校验 remote helper catalog；对非 release/dev bundle 会跳过 remote helper catalog，以便后续 full dev smoke 复用核心检查。

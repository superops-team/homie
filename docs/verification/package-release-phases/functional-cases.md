# Package/Release 阶段化功能验证 Case

## 1. 验证目标

本验证 Case 面向 `package-release-phases` 首阶段，目标是证明：

- `homie/scripts/package.sh` 默认完整打包路径不被破坏；
- 新增 `--phase preflight` 能在长构建前发现缺失工具或 target；
- 新增 `--phase verify --app <path>` 是只读 bundle 验证，不修改 app；
- CI bundle job 可复用 `verify` phase，避免维护第二套 bundle 结构检查；
- 本轮不引入新的 release runtime，不改变 signing/notary/update artifact 语义。

## FC-01: 依赖分析和 PRD 风险修复已记录

覆盖需求：spec review、依赖排序、首阶段关闭口径。

```bash
test -s docs/verification/package-release-phases/spec-review-report.md
test -s docs/verification/package-release-phases/dependency-priority-analysis.md
rg -n "首阶段关闭口径|verify-only|只读|不新增发布运行时|full-dev-bundle-smoke" \
  prd-spec/refactors/package-release-phases/2026-08-13-package-release-phases-design.md \
  docs/verification/package-release-phases/spec-review-report.md \
  docs/verification/package-release-phases/dependency-priority-analysis.md
```

通过标准：命令退出码为 0，输出命中首阶段边界、只读 verify 和依赖排序。

证据路径：`docs/verification/package-release-phases/fc-01-spec-dependency.log`

## FC-02: OpenSpec 三件套完整并覆盖所有 Case

覆盖需求：OpenSpec 拆解和 PRD/Case 对齐。

```bash
test -s openspec/changes/package-release-phases/plan.md
test -s openspec/changes/package-release-phases/tasks.md
test -s openspec/changes/package-release-phases/alignment-report.md
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06|FC-07" \
  openspec/changes/package-release-phases/tasks.md \
  openspec/changes/package-release-phases/alignment-report.md
```

通过标准：三件套存在，tasks 和 alignment report 覆盖 FC-01 至 FC-07。

证据路径：`docs/verification/package-release-phases/fc-02-openspec-alignment.log`

## FC-03: package 脚本语法和帮助输出

覆盖需求：新增 phase 参数不破坏脚本解析，用户可发现用法。

```bash
bash -n homie/scripts/package.sh
homie/scripts/package.sh --help
```

通过标准：

- `bash -n` 退出码为 0；
- `--help` 输出包含 `--phase preflight`、`--phase verify`、`--app`；
- `--help` 不触发构建、签名、复制或删除。

证据路径：`docs/verification/package-release-phases/fc-03-help-syntax.log`

## FC-04: preflight 在长构建前报告缺失前置条件

覆盖需求：preflight 早失败。

```bash
homie/scripts/package.sh --phase preflight
```

通过标准：

- 在当前机器缺少 required Rust target 或工具时，命令必须在任何 `cargo build` 前失败并输出明确缺失项；
- 在前置条件齐备的机器上，命令必须通过且不生成 app；
- 日志不包含 `==> Building homie for Apple silicon`、`==> Building homie for Intel`、`cargo packager`。

证据路径：`docs/verification/package-release-phases/fc-04-preflight.log`

## FC-05: verify phase 能验证已存在 app bundle

覆盖需求：只读 verify phase 可复用 bundle structure checks。

```bash
app="homie/dist/homie-dev-7eef934e-arm64-test.app"
test -d "${app}"
before="$(find "${app}" -type f -print0 | xargs -0 stat -f '%N %m' | shasum -a 256 | awk '{print $1}')"
homie/scripts/package.sh --phase verify --app "${app}"
after="$(find "${app}" -type f -print0 | xargs -0 stat -f '%N %m' | shasum -a 256 | awk '{print $1}')"
test "${before}" = "${after}"
```

通过标准：

- 对结构完整的 app，verify 退出码为 0；
- verify 不修改 app 内文件 mtime；
- verify 不执行签名、notary、复制、删除、构建。

说明：当前仓库已有 `homie/dist/homie-dev-7eef934e-arm64-test.app` 作为本地结构样本。若执行环境没有该 app，可先用 package/dev bundle 生成样本并记录路径。

证据路径：`docs/verification/package-release-phases/fc-05-verify-existing-app.log`

## FC-06: verify phase 对缺失资源失败且只读

覆盖需求：异常路径和只读性。

```bash
tmp="$(mktemp -d "${TMPDIR:-/tmp}/homie-verify-bad.XXXXXX")"
mkdir -p "${tmp}/bad.app/Contents/Resources/bin"
cat > "${tmp}/bad.app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict></dict></plist>
PLIST
if homie/scripts/package.sh --phase verify --app "${tmp}/bad.app"; then
  echo "verify unexpectedly passed"
  exit 1
fi
test -d "${tmp}/bad.app"
```

通过标准：verify 对缺失核心二进制/manifest 的 app 非 0 失败，错误信息指出缺失资源，且不删除测试 app。

证据路径：`docs/verification/package-release-phases/fc-06-verify-failure-readonly.log`

## FC-07: 默认 package 路径和 CI bundle job 复用 verify

覆盖需求：默认行为兼容、CI 复用唯一 bundle 验证口径。

```bash
rg -n "package\\.sh --phase verify|--app" .github/workflows/ci.yml
rg -n "phase=|verify_app\\(|run_preflight|==> Building homie for Apple silicon" homie/scripts/package.sh
git diff --check
```

通过标准：

- CI bundle job 调用 `homie/scripts/package.sh --phase verify --app ...`；
- 默认无参数路径仍存在：phase 默认值是 `package`，preflight 后继续进入原有完整 package 构建标记；
- `git diff --check` 通过。

完整默认 package 可在具备 required targets/tools 的环境执行：

```bash
HOMIE_DIST_DIR=/private/tmp/homie-package-release-phases homie/scripts/package.sh
```

若本机缺少 target/tool，release readiness 必须记录 preflight 输出，并说明未执行完整 package 的阻塞项。

证据路径：`docs/verification/package-release-phases/fc-07-default-ci-gates.log`

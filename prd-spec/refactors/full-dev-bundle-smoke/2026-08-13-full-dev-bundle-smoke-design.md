# Full Dev App Bundle 与 Smoke 验证设计文档

## 1. 概述

### 1.1 问题/动机

当前 `homie/scripts/dev.sh` 只构建 `homie-app` 主 UI 二进制，并将其包装为临时 `.app`。这个路径适合快速 UI 预览，但不等价于真实可验证的桌面 app：

- 不打包 `homied-rs`。
- 不打包 `homie-holder`。
- 不打包 `homie-ssh-askpass`。
- 不打包 Swift CLI `Resources/bin/homie`。
- 不打包 `homie-mcp`。
- 不打包 Engine manifest catalog。
- 不验证 app 能否启动随包 Engine 并连通 socket。

上一次本机构建验证中，为了得到可真实测试的 app，需要手工补齐这些资源。这说明当前 dev bundle 与 release bundle 差距过大，容易出现“UI 能开，但 Engine/CLI/manifests/holder 打包失败”的漏测。

Waku 的 dev flow 通过 `scripts/dev.ts` 调用统一 `scripts/bundle.sh debug`，debug app 也走接近真实 bundle 的装配路径，并提供 watch/rebuild/relaunch。Homie 不必复制 Bun watch，但应该提供一个可真实验证的 full dev bundle 路径。

### 1.2 目标

1. 新增或扩展开发打包路径，生成包含核心运行依赖的本机 `.app`。
2. 支持快速 smoke：临时 `HOMIE_APP_SUPPORT` 下启动随包 `homied-rs`，用随包 CLI 连接 socket。
3. 将该 smoke 作为本机验证和 CI bundle job 的补充。
4. 保留现有快速 UI dev 能力，不强迫每次 UI 调试都构建所有依赖。

### 1.3 非目标

- 不要求 dev bundle 生成 universal 二进制。
- 不要求 dev bundle notarize。
- 不替代 release `package.sh`。
- 不在本 PRD 中实现 hot reload/watch。

## 2. 现状分析

| 路径 | 当前行为 | 缺口 |
|------|----------|------|
| `homie/scripts/dev.sh` | 构建 `homie-app`，创建临时 `.app`，codesign 后直接执行主程序 | 只含 UI 主二进制 |
| `homie/scripts/package.sh` | 构建 universal app、Swift CLI、Engine、remote helpers、manifests、签名和可选 notarize/DMG | 完整但重，不适合快速本机迭代 |
| CI bundle job | 验证发布包结构和签名 | 不覆盖本机 full dev bundle |

## 3. 方案设计

### 3.1 新增 full dev bundle 模式

在 `homie/scripts/dev.sh` 增加 `--full`，或新增 `homie/scripts/dev-bundle.sh`。推荐 `dev.sh --full`，入口更统一。

首阶段采用本机架构 debug bundle，不做 universal、不做 notary、不构建三平台 remote helper catalog。这样能先覆盖真实本机开发验证，避免把 P0 dev smoke 扩大成 release packaging 重写。

`--full` 首阶段行为：

1. 构建本机架构 debug 或 release：
   - `homie-app --bin homie`
   - `homie-engine --bin homied-rs --bin homie-holder --bin homie-ssh-askpass`
   - `homie-mcp --bin homie-mcp`
   - Swift CLI `swift build --package-path .. -c debug/release --product homie`
2. 创建 `.app`：
   - `Contents/MacOS/homie` 放 Rust UI 主程序；
   - `Contents/Resources/bin/homie` 放 Swift CLI；
   - `Contents/Resources/bin/homied-rs`；
   - `Contents/Resources/bin/homie-holder`；
   - `Contents/Resources/bin/homie-ssh-askpass`；
   - `Contents/Resources/bin/homie-mcp`；
   - `Contents/Resources/bin/manifests`；
   - 首阶段不复制 `sidecar`，除非 OpenSpec 明确当前 dev smoke 需要浏览器能力。
3. nested binaries 和 app 本体 ad-hoc codesign。
4. 输出稳定路径，例如：
   - `homie/dist/homie-dev-<sha>-<arch>.app`
5. 可选启动 app，默认只构建并输出路径，避免自动接管真实 daemon。

### 3.2 Smoke 验证

新增 `homie/scripts/smoke-dev-bundle.sh <app-path>`，或 `dev.sh --full --smoke`。

验证步骤：

1. `codesign --verify --deep --strict <app>`。
2. 检查核心二进制存在且可执行。
3. 检查 manifest 数量与 source catalog 一致。
4. 用临时目录启动随包 Engine：
   - `HOMIE_APP_SUPPORT="$(mktemp -d)" <app>/Contents/Resources/bin/homied-rs`
5. 等待 socket 出现。
6. 使用随包 CLI：
   - `HOMIE_SOCKET=<tmp>/daemon.sock <app>/Contents/Resources/bin/homie status`
   - `HOMIE_SOCKET=<tmp>/daemon.sock <app>/Contents/Resources/bin/homie doctor`
7. 关闭临时 Engine。
8. 确认不触碰真实 `~/Library/Application Support/Homie`。
9. 扫描 smoke log，确认没有连接或写入真实 daemon socket。

### 3.3 与 release package 的关系

`--full` 不复制 release 的 universal/notary/DMG/remote helper 全部逻辑。它只覆盖本机开发验证所需的核心 runtime bundle。release 仍由 `package.sh` 负责。

若后续发现 bundle 装配逻辑重复过多，可将共同逻辑抽出为 `scripts/lib/bundle-layout.sh`。但首阶段不得先引入大型脚本框架或发布编排器；只能抽取 package/dev smoke 共同需要的二进制存在性、manifest 数量、codesign、临时 Engine smoke 检查。

### 3.4 首阶段关闭口径

`homie-ceg` 首阶段只关闭“full dev bundle 可真实启动随包 Engine 并由随包 CLI 连通”：

- 本机架构即可，不要求 universal。
- 只覆盖 app、Engine、holder、askpass、MCP、Swift CLI、manifest catalog。
- remote helper 三平台 catalog、notary、DMG、update zip 仍由 release package 负责。
- smoke 必须使用临时 `HOMIE_APP_SUPPORT` / socket，不读取或写入真实用户状态。
- 实现前必须补齐 OpenSpec plan/tasks/alignment，并把 smoke 证据写入 `docs/verification/full-dev-bundle-smoke/`。

## 4. 实施步骤

1. 为 `dev.sh` 增加参数：
   - `--full`
   - `--smoke`
   - `--no-launch`
   - `--release` 继续沿用现有语义
2. 实现本机架构依赖构建。
3. 复制核心资源并签名。
4. 实现 smoke 验证函数。
5. 在 README/homie README 中记录：
   - 快速 UI dev；
   - full dev bundle；
   - full dev bundle smoke。
6. CI 增加轻量检查：
   - 首阶段优先本地验证；CI 接入应放在 full dev smoke 稳定后；
   - 若接入 CI，先在 macOS job 上跑 `dev.sh --full --no-launch --smoke`，并记录超时/工具前置条件。

## 5. 涉及文件

- `homie/scripts/dev.sh`
- `homie/scripts/package.sh`
- `homie/scripts/smoke-dev-bundle.sh` 或同等新脚本
- `README.md`
- `homie/README.md`
- `.github/workflows/ci.yml`
- `homie/crates/homie-app/src/daemon_launch.rs`
- `homie/crates/homie-engine/src/bin/homied-rs.rs`

## 6. 验证计划

### 6.1 本地验证

```sh
cd homie
./scripts/dev.sh --full --no-launch --smoke
```

期望：

- 输出 `.app` 路径。
- codesign 通过。
- 临时 Engine 启动成功。
- 随包 CLI `status` 返回成功。
- 临时目录被清理。

### 6.2 回归验证

```sh
cd homie
./scripts/dev.sh --settings remote
```

现有快速 UI dev 路径仍可工作。

### 6.3 Package 验证

```sh
HOMIE_DIST_DIR=/private/tmp/homie-dist ./homie/scripts/package.sh
homie/scripts/smoke-dev-bundle.sh /private/tmp/homie-dist/homie.app
```

### 6.4 风险控制

| 风险 | 控制 |
|------|------|
| dev smoke 变成第二套 release package | 首阶段只构建本机架构，不做 universal/notary/DMG/update zip |
| smoke 污染真实用户状态 | 强制临时 `HOMIE_APP_SUPPORT`、临时 socket，验证真实 app support 目录无写入 |
| 与 `package-release-phases` 重复 | smoke 检查项与后续 package verify 对齐；共用检查函数可后置 |
| CI 不稳定拖慢开发 | 先记录本地 evidence；CI 接入作为稳定后任务 |
| 自动 launch 接管真实 daemon | `--full --smoke` 默认不 launch UI，不继承 `HOMIE_SOCKET` |

## 7. 验收标准

1. 用户可以用一个命令生成可真实验证的本机 Homie `.app`。
2. full dev bundle 包含 Engine、holder、askpass、CLI、MCP、manifests。
3. smoke 不使用真实 Application Support，不污染已有会话。
4. 现有快速 dev 路径不被破坏。
5. OpenSpec alignment 明确 full dev bundle 与 release package 的边界，避免重复发布逻辑。
6. Beads `homie-ceg` 更新为已验证状态后才可关闭。

## 8. Beads 追踪

- Beads: `homie-ceg`
- change_id: `full-dev-bundle-smoke`
- 类型: refactor
- 优先级: P0

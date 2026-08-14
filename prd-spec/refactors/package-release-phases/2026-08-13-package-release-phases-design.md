# Package/Release 阶段化设计文档

## 1. 概述

### 1.1 问题/动机

Homie `homie/scripts/package.sh` 当前承担了完整发布包链路：

- Rust app / MCP universal build；
- cargo-packager 组装 app；
- license inventory；
- Swift CLI build；
- Rust Engine/holder/askpass universal build；
- remote helper catalog；
- manifest copy；
- nested signing；
- app signing；
- 可选 notarization；
- 可选 DMG；
- update zip。

这个脚本已经包含大量正确的发布知识，但它是线性 shell 流程，失败往往发生在长构建之后，且难以只运行某个 phase。Waku 的 `scripts/release.ts` 提供了更明确的 release orchestration：参数校验、工具校验、远端发布前置检查、产物结构验证、helper 自测、DMG mount 验证和 publish 分段。Homie 可以保留 shell 或逐步引入更强脚本语言，但应先把 package/release 拆成可组合阶段。

### 1.2 目标

1. 将 package/release 流程拆成可单独执行和验证的 phase。
2. 将长构建前能发现的问题提前到 preflight。
3. 支持本地高频验证：`--local-arm64`、`--skip-build`、`--verify-only`。
4. 增强产物自测，避免“打包成功但运行时资源缺失”。
5. 保留现有 release 行为作为默认路径，降低迁移风险。

### 1.3 非目标

- 不立即替换为 Bun/TypeScript。
- 不改变 signing/notarization 证书策略。
- 不改变 remote helper catalog 协议。
- 不改变 updater 下载 artifact 格式。

## 2. 现状分析

| 当前阶段 | 位置 | 问题 |
|----------|------|------|
| 工具检查 | `package.sh` 前段 | 只检查部分工具，部分发布条件后置 |
| universal build | `package.sh` Rust 多 target | 重，失败晚 |
| Swift CLI build | 打包中段 | 无独立 skip/verify |
| remote helpers | 调用 `build-remote-helpers.sh` | 失败会浪费前面构建成本 |
| signing/notary | 打包后段 | 选项分散在环境变量 |
| bundle verification | 当前有 codesign 和 CI bundle structure | 本地无法 verify-only 复用 |

## 3. 方案设计

### 3.1 Phase 划分

建议将当前 `package.sh` 逻辑拆成以下 phase：

| Phase | 职责 | 可单独运行 |
|-------|------|------------|
| preflight | 工具、target、证书/notary env、版本、输出路径、磁盘空间、remote helper 构建条件 | 是 |
| build-app | 构建 `homie-app` 和 `homie-mcp` | 是 |
| bundle-app | cargo-packager 组装基础 app | 是 |
| build-cli | Swift CLI build | 是 |
| build-engine | Engine/holder/askpass build | 是 |
| build-remote-helpers | remote helper catalog | 是 |
| assemble-resources | copy CLI/Engine/manifests/licenses/sidecar | 是 |
| sign | nested signing + app signing | 是 |
| verify | plist、核心二进制、manifest、remote helper、codesign、临时 Engine smoke | 是 |
| notarize | app zip / DMG notarization | 是 |
| dmg | DMG staging/mount/sign/verify | 是 |
| update-zip | updater zip | 是 |

### 3.2 CLI 形态

保守方案：继续使用 `package.sh`，增加参数：

```sh
homie/scripts/package.sh --phase preflight
homie/scripts/package.sh --phase verify --app <path>
homie/scripts/package.sh --local-arm64
homie/scripts/package.sh --skip-build
homie/scripts/package.sh --create-dmg
```

更清晰方案：新增 `homie/scripts/package.ts` 或 `release.ts` 作为 orchestrator，shell 脚本保留为底层 phase。考虑到仓库当前没有 Bun 依赖，第一阶段推荐 shell phase 化，不引入新运行时。

### 3.3 Preflight 前置内容

preflight 至少检查：

- macOS 版本与 target 架构；
- `cargo`、`rustup`、Swift、`cargo-packager`；
- universal targets 是否已安装；
- remote helper target 构建工具：`zig`、`cargo-zigbuild`；
- signing identity / ad-hoc / notarization env 是否自洽；
- `HOMIE_DIST_DIR` 可写；
- `CARGO_TARGET_DIR` 不指向危险共享目录；
- 预计磁盘空间最低值；
- version 从 `crates/homie-app/Cargo.toml` 读取成功。

### 3.4 Verify-only

`verify` phase 应可对已存在 app 运行：

1. `plutil -lint`。
2. 核心二进制存在且可执行。
3. nested binary 架构符合期望。
4. manifest 数量等于 source。
5. remote helper manifest 存在且 artifact 数量符合 target 列表。
6. `codesign --verify --deep --strict`。
7. 可选临时 Engine smoke。

### 3.5 与 full dev bundle 的关系

`full-dev-bundle-smoke` 的 smoke 脚本应复用 `verify` phase 中的核心检查。package verify 和 dev smoke 不应维护两套 bundle 结构知识。

## 4. 实施步骤

1. 在不改变默认行为的前提下，为 `package.sh` 增加参数解析。
2. 把现有逻辑分为函数：`preflight`、`build_app`、`bundle_app`、`build_cli`、`build_engine`、`build_remote_helpers`、`assemble_resources`、`sign_app`、`verify_app`、`create_dmg`、`notarize_artifacts`。
3. 默认无参数仍执行完整旧流程。
4. 新增 `--phase verify --app <path>`。
5. 新增 `--local-arm64`，只构建本机架构并跳过 universal/remote helper 三平台要求。
6. 将 CI bundle job 改为使用 verify phase。
7. 更新 `homie/PACKAGING.md`。

## 5. 涉及文件

- `homie/scripts/package.sh`
- `homie/scripts/build-remote-helpers.sh`
- `homie/scripts/dev.sh`
- `homie/PACKAGING.md`
- `.github/workflows/ci.yml`
- `homie/scripts/release.sh`
- `homie/crates/homie-updater/src/*`

## 6. 验证计划

### 6.1 行为兼容

```sh
HOMIE_DIST_DIR=/private/tmp/homie-package-full homie/scripts/package.sh
```

默认完整流程仍成功。

### 6.2 Phase 验证

```sh
homie/scripts/package.sh --phase preflight
homie/scripts/package.sh --phase verify --app /private/tmp/homie-package-full/homie.app
homie/scripts/package.sh --local-arm64 --phase build-app
```

### 6.3 CI 验证

- bundle job 继续通过。
- release publishing guard 测试继续通过。

## 7. 验收标准

1. 默认 `package.sh` 行为保持兼容。
2. `preflight` 能在长构建前发现工具/target/signing/notary 的配置错误。
3. `verify-only` 能验证任意已生成 app。
4. CI bundle job 复用 phase 化 verify。
5. Beads `homie-d5w` 更新为已验证状态后才可关闭。

## 8. Beads 追踪

- Beads: `homie-d5w`
- change_id: `package-release-phases`
- 类型: refactor
- 优先级: P1

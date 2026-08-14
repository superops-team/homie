# Agent Manifest 单源化功能验证 Case

## 1. 验证目标

本验证 Case 面向 `agent-manifest-single-source` 变更，目标是证明：

1. `homie/crates/homie-engine/manifests/` 是唯一人工维护源。
2. `Sources/HomieCore/Resources/manifests/` 是由 Rust source 生成的 Swift resource mirror。
3. 任何 Swift/Rust manifest 内容漂移都会被本地 check 和 CI 阻断。
4. Swift CLI/Core、Rust Engine、打包产物继续使用一致 catalog。
5. 用户 override 目录不参与内置 manifest 同步，不改变现有 override 语义。

## 2. Case 清单

### FC-01: 当前双源漂移可被检测

- 优先级：P0
- 覆盖需求：目标 1、目标 2、目标 4；Spec Review P1/P2 漂移问题
- 前置环境：仓库根目录，保留当前双源不一致状态或构造临时不一致。
- 执行命令：

```sh
scripts/check-agent-manifest-drift.sh
```

- 预期结果：
  - 当 Swift mirror 与 Rust source 不一致时，命令非 0 退出。
  - 输出至少包含 drift 文件路径和修复提示 `scripts/sync-agent-manifests.sh`。
- 通过标准：
  - 能稳定发现任一 manifest 内容漂移，不只是数量变化。
- 证据路径：
  - `docs/verification/agent-manifest-single-source/fc-01-drift-detected.log`
- 失败处理：
  - 若命令只检查数量或未输出具体 drift 文件，回到实现修复 check 脚本。

### FC-02: 同步脚本能从 Rust source 生成 Swift mirror

- 优先级：P0
- 覆盖需求：目标 1、目标 2、实施步骤 2
- 前置环境：仓库根目录；允许修改工作树。
- 执行命令：

```sh
scripts/sync-agent-manifests.sh
scripts/check-agent-manifest-drift.sh
```

- 预期结果：
  - sync 后 `Sources/HomieCore/Resources/manifests/` 文件集合与 `homie/crates/homie-engine/manifests/` 一致。
  - drift check 0 退出。
- 通过标准：
  - `diff -qr Sources/HomieCore/Resources/manifests homie/crates/homie-engine/manifests` 无差异，或 check 脚本的规范化比较无差异。
- 证据路径：
  - `docs/verification/agent-manifest-single-source/fc-02-sync-green.log`
- 失败处理：
  - 若 sync 不删除 Swift 多余文件或不复制新增文件，修复同步逻辑后重跑。

### FC-03: 手工改 Swift mirror 必须被本地 check 阻断

- 优先级：P0
- 覆盖需求：目标 3、验收标准 3
- 前置环境：完成 FC-02 后的 clean mirror。
- 执行命令：

```sh
cp Sources/HomieCore/Resources/manifests/codex.json /private/tmp/homie-codex-manifest.backup.json
python3 - <<'PY'
import json
from pathlib import Path
path = Path("Sources/HomieCore/Resources/manifests/codex.json")
data = json.loads(path.read_text())
data["agent"]["shortLabel"] = "codex-drift"
path.write_text(json.dumps(data, indent=2) + "\n")
PY
scripts/check-agent-manifest-drift.sh
cp /private/tmp/homie-codex-manifest.backup.json Sources/HomieCore/Resources/manifests/codex.json
scripts/check-agent-manifest-drift.sh
```

- 预期结果：
  - 修改 Swift mirror 后 check 非 0。
  - 恢复后 check 0。
- 通过标准：
  - 行为字段 drift 被检测到。
  - 测试结束后工作树不保留临时 drift。
- 证据路径：
  - `docs/verification/agent-manifest-single-source/fc-03-manual-swift-drift-blocked.log`
- 失败处理：
  - 若修改 `shortLabel` 不触发失败，说明 check 不是内容级比较，必须修复。

### FC-04: Swift AgentCatalog 继续加载生成 mirror

- 优先级：P0
- 覆盖需求：兼容 Swift CLI/Core；Spec Review P1 存量影响
- 前置环境：完成 FC-02 后。
- 执行命令：

```sh
swift test --package-path . --filter HomieCoreTests
```

- 预期结果：
  - HomieCoreTests 通过。
  - `catalogLoadsBundledManifests`、`catalogResolvesUserTypedNames`、`envScrubPrefixesComeFromManifests` 等现有 catalog 测试不回退。
- 通过标准：
  - Swift resource mirror 可正常被 `AgentCatalog.shared` 读取。
- 证据路径：
  - `docs/verification/agent-manifest-single-source/fc-04-swift-core-tests.log`
- 失败处理：
  - 若 Swift resource bundle 路径断裂，回到实现修复 Package.swift/resource mirror。

### FC-05: Swift CLI/MCP agent vocabulary 不回退

- 优先级：P1
- 覆盖需求：MCP/CLI 动态 agent catalog 不回退
- 前置环境：完成 FC-02 后。
- 执行命令：

```sh
swift test --package-path . --filter HomieCLITests
swift test --package-path . --filter HomieProtocolTests
```

- 预期结果：
  - CLI 语法测试通过。
  - CLI/MCP 仍能通过 `AgentCatalog.shared.launchable` 获取 manifest-only agents。
- 通过标准：
  - 新 agent manifest 不会因同步镜像改造退化为硬编码列表。
- 证据路径：
  - `docs/verification/agent-manifest-single-source/fc-05-swift-cli-protocol-tests.log`
- 失败处理：
  - 若 CLI/MCP 解析失效，修复 Swift mirror 或 AgentCatalog 加载路径。

### FC-06: Rust Engine manifest decode 与 runtime catalog 不回退

- 优先级：P0
- 覆盖需求：Rust source 作为唯一人工源；运行时不回退
- 前置环境：完成 FC-02 后。
- 执行命令：

```sh
cd homie
cargo test -p homie-engine --lib detect::tests::every_bundled_manifest_decodes
cargo test -p homie-engine --lib agent::tests::every_shipped_manifest_declares_an_authority
cargo test -p homie-engine --test mcp_tools
```

- 预期结果：
  - Rust Engine manifest decode、agent authority、MCP manifest tools 测试通过。
- 通过标准：
  - Rust source catalog 可解析并继续服务 runtime/MCP。
- 证据路径：
  - `docs/verification/agent-manifest-single-source/fc-06-rust-engine-manifest-tests.log`
- 失败处理：
  - 若 manifest decode 失败，回到 Rust source manifest 修复；不得通过修改 Swift mirror 绕过。

### FC-07: 打包产物仍从 Rust source catalog 复制

- 优先级：P1
- 覆盖需求：打包路径不回退；验收标准 4
- 前置环境：完成 FC-02 后；具备本机 package 所需工具。
- 执行命令：

```sh
HOMIE_DIST_DIR=/private/tmp/homie-agent-manifest-package homie/scripts/package.sh
app=/private/tmp/homie-agent-manifest-package/homie.app
test -f "${app}/Contents/Resources/bin/manifests/codex.json"
source_count="$(find homie/crates/homie-engine/manifests -name '*.json' -type f | wc -l | tr -d ' ')"
bundle_count="$(find "${app}/Contents/Resources/bin/manifests" -name '*.json' -type f | wc -l | tr -d ' ')"
test "${source_count}" = "${bundle_count}"
codesign --verify --deep --strict "${app}"
```

- 预期结果：
  - package 成功。
  - bundle manifest 数量等于 Rust source。
  - app codesign 验证通过。
- 通过标准：
  - bundle 不依赖 Swift mirror 作为发布源。
- 证据路径：
  - `docs/verification/agent-manifest-single-source/fc-07-package-manifest-source.log`
- 失败处理：
  - 若 package 环境工具缺失，记录为环境阻塞；若 manifest 数量或来源错误，回到实现修复 package。

### FC-08: User overrides 不参与内置同步和 drift check

- 优先级：P1
- 覆盖需求：非目标 user override 目录不改变
- 前置环境：完成 FC-02 后。
- 执行命令：

```sh
tmp_home="$(mktemp -d /private/tmp/homie-override-home.XXXXXX)"
mkdir -p "${tmp_home}/Library/Application Support/Homie/manifests/overrides"
cp homie/crates/homie-engine/manifests/shell.json "${tmp_home}/Library/Application Support/Homie/manifests/overrides/custom-shell.json"
HOME="${tmp_home}" scripts/check-agent-manifest-drift.sh
rm -rf "${tmp_home}"
```

- 预期结果：
  - drift check 0 退出。
  - check 不扫描或同步用户 override。
- 通过标准：
  - 内置 source/mirror 同步与用户 override 完全隔离。
- 证据路径：
  - `docs/verification/agent-manifest-single-source/fc-08-user-overrides-ignored.log`
- 失败处理：
  - 若 check 扫描 HOME override 并失败，修复脚本边界。

### FC-09: 文档不再指向 Swift mirror 作为人工源

- 优先级：P1
- 覆盖需求：文档一致性
- 前置环境：实现完成后。
- 执行命令：

```sh
rg -n "Sources/HomieCore/Resources/manifests" \
  README.md CONTRIBUTING.md docs/GETTING_STARTED.md docs/README.md \
  Sources homie scripts
```

- 预期结果：
  - README/CONTRIBUTING/用户文档不再把 `Sources/HomieCore/Resources/manifests` 描述为人工编辑源。
  - 若源码注释仍出现该路径，必须明确说明它是生成镜像，而非权威源。
- 通过标准：
  - 新贡献者能从文档明确知道只改 Rust manifest source。
- 证据路径：
  - `docs/verification/agent-manifest-single-source/fc-09-doc-source-path-scan.log`
- 失败处理：
  - 修正文档和注释后重跑。

### FC-10: 全量本地质量门禁

- 优先级：P1
- 覆盖需求：整体回归
- 前置环境：所有实现和文档更新完成。
- 执行命令：

```sh
./scripts/check.sh
```

- 预期结果：
  - shell script 检查、Swift tests、Rust fmt/clippy/tests、license policy 全部通过。
- 通过标准：
  - 本次变更没有破坏既有 contributor gate。
- 证据路径：
  - `docs/verification/agent-manifest-single-source/fc-10-full-check.log`
- 失败处理：
  - 按失败模块回到实现修复；不得降低 check 覆盖。

## 3. 覆盖矩阵

| PRD/Review 需求 | 覆盖 Case |
|-----------------|-----------|
| Rust manifest 为唯一人工源 | FC-01, FC-02, FC-06, FC-07 |
| Swift manifest 为生成镜像 | FC-02, FC-03, FC-04, FC-05 |
| CI/local gate 阻断 drift | FC-01, FC-03, FC-10 |
| 文档指向正确路径 | FC-09 |
| Swift CLI/Core 不回退 | FC-04, FC-05 |
| Rust Engine runtime 不回退 | FC-06 |
| 打包产物使用 Rust source | FC-07 |
| user overrides 不改变 | FC-08 |
| 全量回归 | FC-10 |

## 4. 执行顺序

1. FC-01：先证明当前 drift 可被目标 check 捕获。
2. FC-02：同步生成镜像并进入 green 状态。
3. FC-03：验证手工 drift 被阻断。
4. FC-04、FC-05：验证 Swift 侧不回退。
5. FC-06：验证 Rust Engine 不回退。
6. FC-08、FC-09：验证边界和文档。
7. FC-07：验证 package。
8. FC-10：全量门禁。

## 5. 证据留存规范

- 所有命令输出写入 `docs/verification/agent-manifest-single-source/*.log`。
- 若某个 Case 因本机缺少工具不能执行，必须记录：命令、缺失工具、是否影响 P0/P1 准出、替代验证是否可接受。
- P0 Case 不允许无证据跳过。

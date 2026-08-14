# Agent Manifest 单源化设计文档

## 1. 概述

### 1.1 问题/动机

Waku 对比 review 发现 Homie 当前存在明确的 agent manifest 双源漂移：

- 用户文档仍指导新增 agent 修改 `Sources/HomieCore/Resources/manifests/`。
- Rust Engine 运行时实际从 `homie/crates/homie-engine/manifests/` 加载内置 catalog。
- `homie/scripts/package.sh` 打包时复制的是 Rust Engine manifest 目录。
- `diff -qr Sources/HomieCore/Resources/manifests homie/crates/homie-engine/manifests` 显示 20 个 manifest 文件全部不同。

这些差异不只是格式差异，已经包含行为字段：`resume.style`、`deny`、`approve`、`envScrubPrefixes`、`agent.id` 等。贡献者按 README 修改 Swift manifest 时，发布包不会采用该修改；Swift CLI/Core 若读取旧 manifest，也可能展示与 Engine 不一致的 agent 能力。

### 1.2 目标

1. 确立 `homie/crates/homie-engine/manifests/` 为唯一权威 agent manifest 源。
2. 将 Swift manifest 副本降级为由 Rust manifest 生成的镜像，避免双源人工维护。
3. 更新文档、脚本和测试，使“添加 agent 是数据变更”仍成立，但路径唯一。
4. 在 CI 中加入漂移检测，防止后续重新引入双源不一致。

### 1.3 非目标

- 不重新设计 manifest schema。
- 不把所有 agent 转成 typed driver。
- 不改变 user override 目录：`~/Library/Application Support/Homie/manifests/overrides`。
- 不为旧 Swift manifest 行为提供兼容层。

## 2. 现状分析

| 位置 | 当前事实 | 风险 |
|------|----------|------|
| `README.md` | 指向 `Sources/HomieCore/Resources/manifests/` | 贡献者改错目录 |
| `Sources/HomieCore/AgentDescriptor.swift` | 注释把 Swift resource manifest 描述为 agent block 来源 | Swift/Rust 边界语义漂移 |
| `Sources/HomieCore/AgentCatalog.swift` | 仍可从 Swift resource bundle 加载 manifest | CLI/Core 可能读取旧 catalog |
| `homie/crates/homie-engine/src/detect/mod.rs` | Rust 注释声明 `crates/homie-engine/manifests` 是 Rust-owned catalog | 与 README 冲突 |
| `homie/crates/homie-engine/src/bin/homied-rs.rs` | daemon 加载 `exe_dir/manifests`、Rust source fallback、user overrides | 运行时以 Rust 目录为准 |
| `homie/scripts/package.sh` | 打包复制 Rust Engine manifest，且只校验 Rust source/bundle 数量 | Swift manifest 不参与发布 |

Waku 参考价值：Waku 没有跨语言重复 manifest 目录；provider 能力集中在 Rust model/driver 边界，减少了数据源漂移面。Homie 不需要照搬 Waku 的 typed provider 路线，但 agent catalog 应先做到单源。

## 3. 方案设计

### 3.1 推荐方案

采用“Rust Engine manifest 为唯一人工源 + Swift resource manifest 作为生成镜像”的方案：

1. 保留 `homie/crates/homie-engine/manifests/`。
2. 保留 `Sources/HomieCore/Resources/manifests/`，但将其定义为生成镜像，不能人工修改。
3. 新增同步脚本从 Rust manifest 源复制/规范化生成 Swift resource bundle。
4. Swift CLI/Core 继续离线读取 `HomieCore` resource bundle，避免本次变更扩大到 CLI 与 daemon 查询协议。
5. README、CONTRIBUTING、开发文档统一指向 Rust Engine manifest 路径。
6. 新增 CI gate：
   - `scripts/check-agent-manifest-drift.sh` 必须证明 Swift 生成镜像与 Rust 权威源一致；
   - 若 drift 存在，提示运行同步脚本，而不是让两套 manifest 各自演进。

### 3.2 备选方案

| 方案 | 优点 | 缺点 |
|------|------|------|
| 保留 Swift manifest 但生成化 | 兼容 Swift resource bundle，变更小，能立刻阻断漂移 | 仍有生成流程和校验成本 |
| 删除 Swift manifest，Swift 只走 daemon catalog | 最彻底，漂移面最小 | CLI 离线能力需要改造，超过本次最小实现 |
| 继续双源，只加文档说明 | 实现最小 | 不能解决实际漂移，高风险 |

推荐第一阶段采用生成化：Rust 目录是唯一人工源，Swift 目录是同步产物。删除 Swift resource bundle 或改为 daemon 查询可作为后续独立重构，不进入本次 P0 交付。

## 4. 实施步骤

1. 盘点 Swift 侧真实使用 manifest 的路径：
   - `Sources/HomieCore/AgentCatalog.swift`
   - `Sources/homie-cli/*`
   - Swift tests
2. 新增同步脚本：
   - `scripts/sync-agent-manifests.sh`：从 `homie/crates/homie-engine/manifests/` 同步到 `Sources/HomieCore/Resources/manifests/`；
   - `scripts/check-agent-manifest-drift.sh`：校验两目录一致，失败时输出 drift 文件和修复命令。
3. 更新文档：
   - `README.md`
   - `docs/GETTING_STARTED.md`
   - `CONTRIBUTING.md` 如有相关说明
4. 更新脚本和 CI：
   - 新增 manifest drift check；
   - 把 drift check 放入 `scripts/check.sh` 和 `.github/workflows/ci.yml`。
5. 将 `Sources/HomieCore/Resources/manifests/` 视为生成镜像，并在目录 README 或脚本提示中声明不要人工编辑。
6. 补充测试：
   - Rust manifest decode 全量通过；
   - Swift CLI/Core 可继续读取生成镜像；
   - 打包后 `Contents/Resources/bin/manifests` 与 Rust source 数量和关键文件一致。

## 5. 涉及文件

- `README.md`
- `CONTRIBUTING.md`
- `scripts/check.sh`
- `scripts/sync-agent-manifests.sh`
- `scripts/check-agent-manifest-drift.sh`
- `.github/workflows/ci.yml`
- `Sources/HomieCore/AgentCatalog.swift`
- `Sources/HomieCore/AgentDescriptor.swift`
- `Sources/HomieCore/ResourceBundle.swift`
- `Sources/HomieCore/Resources/manifests/`
- `homie/crates/homie-engine/manifests/`
- `homie/crates/homie-engine/src/detect/mod.rs`
- `homie/scripts/package.sh`
- `homie/scripts/verify-remote-refactor.sh`

## 6. 验证计划

### 6.1 静态验证

- `rg "Sources/HomieCore/Resources/manifests" README.md docs Sources homie scripts` 不再出现作为权威人工源路径的说明。
- CI 中 manifest drift check 通过；手工修改 Swift 镜像能让 check 失败。
- `homie/scripts/package.sh` 仍只从 Rust Engine source catalog 打包。

### 6.2 单元测试

- Rust manifest decode 测试覆盖所有 manifest。
- Swift AgentCatalog 测试继续可加载 resource bundle，并验证其内容与 Rust source 镜像一致。

### 6.3 打包验证

- `homie/scripts/package.sh` 后：
  - `Contents/Resources/bin/manifests/codex.json` 存在；
  - bundled manifest 数量等于 Rust source；
  - 不存在未声明的 Swift manifest 运行时来源。

## 7. 验收标准

1. 只有一个人工维护的内置 agent manifest 源目录：`homie/crates/homie-engine/manifests/`。
2. README 和开发文档指向的路径与真实 Engine/package 路径一致。
3. CI 能阻断 Swift/Rust manifest 漂移。
4. 20 个现有 agent 的运行行为不因目录收敛而回退。
5. Beads `homie-rc2` 更新为已验证状态后才可关闭。

## 8. Beads 追踪

- Beads: `homie-rc2`
- change_id: `agent-manifest-single-source`
- 类型: refactor
- 优先级: P0

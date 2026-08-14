# Spec Review Report

## 1. 总体结论

- 可行性：高
- 最大风险：Swift CLI/Core 当前真实依赖 `HomieCore` resource bundle 中的 manifest。若直接删除 `Sources/HomieCore/Resources/manifests/`，会把本次 P0 从“漂移治理”扩大成“CLI catalog 查询架构重写”。
- 推荐方向：采用“Rust Engine manifest 为唯一人工源 + Swift manifest 作为生成镜像 + CI drift gate”的最小实现。后续若要完全删除 Swift resource bundle，应单独规划协议/CLI 查询改造。

## 2. 问题清单

| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|
| P1 | 最小化实现 | 原 PRD 将“删除 Swift manifest”和“生成 Swift manifest”并列为推荐路径，没有收敛第一阶段落地方式。 | 执行者可能直接删除 Swift resource bundle，导致 `AgentCatalog.shared`、CLI 解析、MCP 工具 schema 等路径断裂。 | 已整改：第一阶段明确保留 Swift resource bundle，但作为由 Rust manifest 生成的镜像；不再人工维护。 |
| P1 | 存量影响 | PRD 已指出 Swift CLI/Core 可能读取旧 catalog，但没有把 `Package.swift` resource copy、`AgentCatalog.manifestURLs`、`HomieMCP.Tools` 等依赖列为必须验证路径。 | 迁移时可能只修文档和打包，漏掉 Swift 单元测试和 CLI 命令解析。 | 在功能验证 Case 和 OpenSpec 中显式覆盖 Swift build、HomieCoreTests、HomieCLITests。 |
| P2 | 可测试性 | PRD 提到 CI drift gate，但没有明确 sync/check 脚本名称和失败行为。 | 不同实现者可能写成只比较数量，不能阻断行为字段漂移。 | 已整改：新增 `scripts/sync-agent-manifests.sh` 与 `scripts/check-agent-manifest-drift.sh`，要求校验内容一致并输出修复命令。 |
| P2 | 运行风险 | PRD 对 user overrides 保持不变，但未强调 source catalog、Swift mirror、user override 的加载优先级不能混淆。 | 可能误把用户 overrides 同步进内置镜像或 drift check。 | 在后续 Case 中加入 user override 不参与内置同步、运行时 override 路径保持不变的验证。 |
| P2 | 文档一致性 | README/CONTRIBUTING 当前仍指向 `Sources/HomieCore/Resources/manifests/`。 | 新贡献者继续改错目录。 | OpenSpec 需包含 README/CONTRIBUTING/Package.swift 注释更新任务。 |

P0/P1 处理状态：

- P0：未发现。
- P1：已通过 PRD 修改收敛方案；剩余依赖路径将在功能验证 Case 和 OpenSpec 中强制覆盖。

## 3. 整改后的完善方案

### 3.1 目标与范围

本变更只解决内置 agent manifest 双源漂移：

- 唯一人工维护源：`homie/crates/homie-engine/manifests/`。
- Swift 资源目录：`Sources/HomieCore/Resources/manifests/`，保留为生成镜像。
- 用户 override：`~/Library/Application Support/Homie/manifests/overrides`，不参与同步，不改变语义。

### 3.2 非目标

- 不删除 Swift `AgentCatalog`。
- 不改变 Swift CLI 离线解析 agent names 的能力。
- 不改变 manifest schema。
- 不改变 Runtime/Engine 对 user overrides 的加载方式。

### 3.3 设计原则

1. 最小化实现：先阻断漂移，不重写 CLI/daemon 协议。
2. 单向同步：只允许 Rust source -> Swift mirror。
3. 可失败门禁：手动改 Swift mirror 必须在 CI 中失败。
4. 行为不回退：20 个现有 agent 的 Swift descriptor 与 Rust manifest 保持一致。

### 3.4 核心方案

1. 新增同步脚本：
   - `scripts/sync-agent-manifests.sh`
   - 从 `homie/crates/homie-engine/manifests/` 同步到 `Sources/HomieCore/Resources/manifests/`。
2. 新增 drift check：
   - `scripts/check-agent-manifest-drift.sh`
   - 校验两个目录文件集合与内容一致。
   - 失败时输出 drift 文件和 `scripts/sync-agent-manifests.sh` 修复提示。
3. 接入门禁：
   - `scripts/check.sh`
   - `.github/workflows/ci.yml`
4. 更新文档：
   - README/CONTRIBUTING 指向 Rust source。
   - Package.swift 和 Swift 注释说明资源目录是生成镜像。
5. 测试：
   - Rust manifest decode 继续通过。
   - Swift `AgentCatalog.shared` 能加载生成镜像。
   - CLI/MCP agent name 解析不回退。

### 3.5 兼容与风险控制

- 旧 Swift manifest 目录不立即删除，降低 CLI/Core 断裂风险。
- 打包仍使用 Rust source catalog，不改变 release bundle 结构。
- drift check 不扫描用户 overrides，避免误伤用户配置。

## 4. 分层任务拆解

| 层级 | 任务 | 交付物 | 依赖 | 优先级 |
|------|------|--------|------|--------|
| 文档 | 更新 PRD 推荐方案为生成镜像 | PRD 修订 | 无 | P0 |
| 脚本 | 新增 sync 脚本 | `scripts/sync-agent-manifests.sh` | PRD | P0 |
| 脚本 | 新增 drift check 脚本 | `scripts/check-agent-manifest-drift.sh` | sync 脚本 | P0 |
| CI | 接入本地 check 与 GitHub Actions | `scripts/check.sh`、`.github/workflows/ci.yml` | drift check | P0 |
| Swift | 更新注释并验证 resource mirror | Swift docs/tests | sync 脚本 | P1 |
| 文档 | 更新 README/CONTRIBUTING | 文档补丁 | sync/check 语义确定 | P1 |
| 验证 | 执行功能验证 Case | 验证报告 | 实现完成 | P0 |

## 5. 测试规划

| 类型 | 覆盖点 | 关键用例 | 执行阶段 |
|------|--------|----------|----------|
| 单元/脚本 | sync 后目录一致 | 删除/修改 Swift mirror 后 check 失败，sync 后通过 | TDD RED/GREEN |
| Rust | Rust source manifest 可解析 | `cargo test -p homie-engine` 相关 manifest tests | 实现后 |
| Swift | Swift mirror 可被 AgentCatalog 加载 | `swift test --package-path . --filter HomieCoreTests` | 实现后 |
| CLI | CLI agent name 解析不回退 | `swift test --package-path . --filter HomieCLITests` | 实现后 |
| 打包 | release bundle 仍使用 Rust source | `homie/scripts/package.sh` 或 package verify | E2E |
| 回归 | user overrides 不参与同步 | 临时 HOME/Application Support 下 override 不被 sync/check 读取 | 功能验证 |

## 6. 开发排期

| 阶段 | 顺序 | 工作项 | 风险与缓冲 | 验收物 |
|------|------|--------|------------|--------|
| S1 | 先 | PRD 整改与功能验证 Case | 方案必须先收敛 | 修订 PRD + Case 清单 |
| S2 | 次 | OpenSpec plan/tasks/alignment | 任务必须映射 Case | OpenSpec 三件套 |
| S3 | 次 | TDD 编写 drift check RED | 先证明 drift 能失败 | 失败用例 |
| S4 | 次 | 实现 sync/check + 文档更新 | 避免误扫 overrides | GREEN |
| S5 | 后 | 功能验证与两轮 review | 覆盖 Swift/Rust/packaging | 验证报告 |

## 7. 待确认问题

- 无阻塞待确认问题。第一阶段已明确采用生成镜像方案；删除 Swift resource bundle 不进入本次 dev-loop。

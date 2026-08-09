# Packaging & Updater 组件规格

## 1. 组件定位

`packaging-updater` 定义 Homie macOS app bundle、签名、公证、DMG、update zip、update feed、手动更新 flow、helper swap、回滚和 packaged performance gate。它是 Reference parity release 准出的发布合同。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- 功能验证: FC-017, FC-018

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | release script | 构建、签名、公证、生成 feed |
| 上游 | `homie-app` | 检查更新、下载、触发 restart-to-update |
| 下游 | codesign/spctl/notarytool | 验证 trust |
| 下游 | release host | 发布 DMG/update zip/feed |

## 4. 职责边界

负责：

- `.app` bundle assembly。
- universal/architecture matrix。
- Developer ID signing、hardened runtime、notarization、stapling。
- DMG 和 update zip。
- update feed JSON。
- updater trust check。
- helper swap 和 rollback。
- packaged perf gate。

不负责：

- runtime session 迁移。
- provider credential。
- release host 运维细节。

## 5. 核心接口

```rust
pub trait Updater {
    async fn check(&self) -> Result<UpdateCheckResult, UpdateError>;
    async fn download(&self, version: Version) -> Result<StagedUpdate, UpdateError>;
    fn verify(&self, staged: &StagedUpdate) -> Result<(), UpdateError>;
    fn restart_to_install(&self, staged: StagedUpdate) -> Result<(), UpdateError>;
}
```

## 6. 数据模型

```rust
pub struct UpdateFeedEntry {
    pub version: Version,
    pub url: String,
    pub sha256: String,
    pub minimum_system_version: Option<String>,
}
```

Trust inputs:

- Team ID。
- bundle id。
- codesign status。
- spctl assessment。
- version match。
- HTTPS host allowlist。

## 7. 运行模型与状态机

```text
idle -> checking -> update_available -> downloading -> staged -> restart_requested -> installing -> relaunched
idle -> checking -> up_to_date
downloading/staged/installing -> failed -> recover current bundle
```

更新不得自动重启 live app；下载和安装必须由用户动作触发。

## 8. 安全与权限

- 不安装未通过 codesign/spctl/version/bundle/team 检查的 bundle。
- 不执行 feed 中任意 URL；host 必须在 allowlist。
- helper 只操作自身 bundle 路径和 staged update path。
- update log 不包含 credential。

## 9. 可观测性

- update.check_started / completed / failed。
- update.download_progress。
- update.verify_failed。
- update.install_started / completed / rolled_back。
- perf_gate.measured。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| app 目录不可写 | 下载前 fail |
| verify failed | 删除 staged update，保留当前 app |
| install interrupted | helper restore previous bundle |
| perf gate failed | release blocked |

## 11. 测试计划与验收引用

- FC-017: packaging, updater, release trust。
- FC-018: full local quality gate。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M20-F001, M20-F002 |
| Required Diri test mapping | updater trust/install/rollback, packaged launch, DMG, notarization, perf gates |
| Pre-implementation gaps | release pipeline gates |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- Requirements: FR-15, FR-16
- Beads: `homie-t3u`

本组件当前状态是 `partial`。ad-hoc codesign、单架构 tarball、纯 trust decision 和 `PERF_GATE=not_run` 不能作为 release parity 证据。

### 12.1 依赖闭包

发布产物必须显式包含并验证：

- universal Homie app executable；
- Homie CLI；
- runtime daemon；
- holder；
- browser/test sidecar 及其 runtime/assets；
- bundled agent manifests/icons/resources；
- updater helper；
- node/service artifacts（若该 release channel 包含 remote node）。

禁止依赖开发机全局 Node、Python、shell package、未记录 dylib 或用户 HOME 文件。

### 12.2 Release Pipeline

顺序固定为：

```text
build arm64+x86_64
  -> assemble universal bundle
  -> sign nested code and app with hardened runtime
  -> verify codesign/spctl
  -> notarize and staple
  -> build DMG and update zip
  -> compute SHA256
  -> generate signed/immutable feed metadata
  -> install/launch/update/rollback smoke
  -> packaged performance gate
```

- 缺少 Developer ID/notary credential 时状态为 `blocked`，不得用 ad-hoc 签名替代 pass。
- feed/download 必须使用 HTTPS host allowlist 和 SHA256。
- helper 只能操作自身 bundle 和已验证 stage path。
- install failure 必须原子恢复 previous bundle。
- update 由用户动作触发，且必须协调 runtime daemon/live sessions。

### 12.3 性能门禁

至少测量 cold/warm launch、first frame、runtime connect、session attach/replay、terminal repaint、idle memory、event lag 和 update check。每项必须有明确预算、测量设备/OS/build、样本数和原始结果摘要。

### 12.4 完成门禁

- 两架构 slice 和 universal bundle 都能启动。
- codesign、spctl、notarization、stapling、DMG mount/install 通过。
- 从真实本地/测试 feed 完成 check/download/verify/install/relaunch/rollback。
- clean macOS user 环境不依赖开发机工具链。
- packaged performance gate 实际运行且通过，不存在 `not_run`。

## 13. Wave 1A Daemon Closure 修订

权威来源：

- PRD: `prd-spec/features/diri-runtime-daemon-client-transport/2026-08-08-diri-runtime-daemon-client-transport-design.md`
- OpenSpec: `openspec/changes/diri-runtime-daemon-client-transport/`
- Beads: `homie-nep`

- 开发和 package assemble 必须构建 `homie-runtime-daemon`。
- app bundle 内 daemon 使用固定相对位置，app 在启动时解析为 canonical absolute path 后交给 `RuntimeLauncher`。
- launcher 不通过环境变量或 PATH 查找 daemon。
- Wave 1A package smoke 至少验证 bundled daemon 存在、可执行、可 hello/state snapshot，并与 app client 返回同一 instance id。
- universal signing/notarization、updater coordination 和 clean-user 完整 release gate 仍由 T-501 负责；Wave 1A 不得据此声明完整 packaging parity。

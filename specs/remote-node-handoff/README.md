# Remote Node & Handoff 组件规格

## 1. 组件定位

`remote-node-handoff` 定义 Homie 的 remote hosts、first-party node、node accounts、remote spawn、prefs sync、repo locate、session move/fork/handoff、fleet usage 和 companion access。它必须遵守 Homie virtual key 和 credential custody，不复制 provider raw key。

## 2. 来源需求映射

- PRD: `prd-spec/features/reference-parity-v1/2026-08-05-reference-parity-v1-design.md`
- OpenSpec: `openspec/changes/reference-parity-v1/`
- 功能验证: FC-015, FC-018

## 3. 上游与下游关系

| 方向 | 组件 | 关系 |
|------|------|------|
| 上游 | `homie-app` settings | 管理 host/node/companion config |
| 上游 | `homie-runtime` | remote spawn/handoff |
| 上游 | `homie-cli` | node account、handoff、status |
| 下游 | SSH fallback | install/recovery/compat path |
| 下游 | Homie node | first-party remote execution service |
| 下游 | `virtual-key-credentials` | credential policy |

## 4. 职责边界

负责：

- host catalog：id、name、ssh、default cwd、node endpoint、token file、node id。
- node hello/capability/token auth。
- node account add/login/status/default/list。
- remote spawn 和 same-repo locate。
- prefs sync，不同步 credential。
- handoff move/fork：preflight、checkpoint、transfer、quarantine restore、provider-native resume/fork、lease commit。
- fleet usage merge。
- companion access：Tailscale endpoint、pairing token、pairing URL。

不负责：

- provider raw key 存储。
- UI 渲染。
- local PTY implementation。

## 5. 核心接口

```rust
pub trait RemoteNodeService {
    async fn hello(&self, endpoint: NodeEndpoint) -> Result<NodeHello, NodeError>;
    async fn spawn_remote(&self, request: RemoteSpawnRequest) -> Result<SessionRecord, NodeError>;
    async fn handoff(&self, request: HandoffRequest) -> Result<HandoffReceipt, NodeError>;
}
```

## 6. 数据模型

```rust
pub struct HostEntry {
    pub id: HostId,
    pub name: Option<String>,
    pub ssh: String,
    pub default_cwd: Option<String>,
    pub node: Option<HostNodeConfig>,
}

pub struct HostNodeConfig {
    pub endpoint: String,
    pub token_file: PathBuf,
    pub node_id: Option<String>,
}
```

### 6.1 `host.locate_repo` 第一阶段合同

`host.locate_repo` 对齐 Diri 的协议字段和三态结果：

```rust
pub struct HostLocateRepoParams {
    pub host: Option<String>,
    pub origin_url: Option<String>, // JSON: originURL
    pub session_id: Option<SessionId>, // JSON: sessionID
}

pub struct HostLocateRepoResult {
    pub path: Option<String>,
    pub origin_url: Option<String>, // JSON: originURL
}
```

结果语义：

| 输入/状态 | 输出 | 调用方含义 |
|-----------|------|------------|
| origin 可推导且候选 checkout 命中 | `path + originURL` | 同仓路径可用 |
| origin 可推导但候选未命中 | `originURL` only | 目标 host 未克隆 |
| cwd/session 无 git origin | `{}` | 回退 host 默认目录 |

实现边界：

- `homie-remote` 只读 `.git/config` 或 linked worktree `gitdir` 的 `config`，不 shell 到 `git`。
- `homie-client` 优先使用 `homie-storage` 的 `projects.remote_origin` 事实源匹配同仓 checkout，再 fallback 到本地候选目录 `.git/config`。
- 本阶段不声明真实 SSH/node 远端扫描完成；完整 REM-003 仍需 remote node E2E。

## 7. 运行模型与状态机

Remote spawn:

```text
select host -> validate token/capabilities -> locate cwd/repo -> spawn session -> stream events
```

Handoff:

```text
preflight -> source checkpoint -> transfer missing blobs -> target quarantine restore -> provider resume/fork -> lease commit
```

## 8. 安全与权限

- node token file owner-only。
- checkpoint excludes `.git`, provider homes, SSH material, `.env*`, credential files, build/dependency dirs, symlinks, oversized/special files。
- provider raw key 不进入 checkpoint、manifest、transfer、logs。
- public TCP unsupported；bind loopback 或 Tailscale。
- companion token owner-only and revocable。

## 9. 可观测性

- node.hello。
- node.spawn_started/completed/failed。
- handoff.preflight_failed。
- handoff.checkpoint_created。
- handoff.transfer_completed。
- handoff.committed。
- fleet_usage.merged。

## 10. 失败与恢复

| 场景 | 行为 |
|------|------|
| preflight 失败 | abort，source 不变 |
| transfer 失败 | quarantine 清理，source 不变 |
| restore 失败 | target quarantine 保留诊断，source 不变 |
| commit 后失败 | 通过新 move 反向恢复，不 destructive rollback |
| node unreachable | local state 标记 unreachable，不阻塞本地 session |

## 11. 测试计划与验收引用

- FC-015: remote host, node, accounts, handoff。
- FC-018: full local quality gate。

## Diri Parity Mapping

This section is mandatory for Diri-to-Homie parity planning and fixes the review gap recorded in `docs/verification/diri-module-inventory/bingo-component-spec-review-report.md`.

| Field | Value |
|-------|-------|
| Owned feature atoms | M06-F002, M10-F002, M11 remote client, M18-F001, M18-F002, M19-F002 |
| Required Diri test mapping | RemoteAccessTests, RemoteSpawnTests, PrefsSyncTests, host repo locate fixtures |
| Pre-implementation gaps | node protocol/account/handoff matrix |

Rules:

- A PRD/OpenSpec change touching this component must cite the owned feature atom ids above.
- Implementation is not complete until the required Diri test mapping has an equivalent Homie verification gate.
- If a new Diri source/test is discovered for this component, update `docs/research/diri-module-inventory.md` and this section before coding.

## 12. Diri 7ba3407 重基线修订

权威来源：

- PRD: `prd-spec/features/diri-7ba3407-parity-rebaseline/2026-08-08-diri-7ba3407-parity-rebaseline-design.md`
- 能力矩阵: `docs/research/diri-7ba3407-capability-matrix.md`
- Requirements: FR-12, FR-13, FR-15, FR-16
- Beads: `homie-t3u`

本组件当前状态是 `partial`。host/node DTO fixtures、repo locate、prefs sync argv 和 companion config 只证明局部合同，不证明 node server、remote spawn、account runtime 或 handoff 已存在。

### 12.1 First-party Node 合同

node 必须提供：

- authenticated hello、capabilities、version 和 node identity；
- runtime/session spawn、attach、events 和 cancellation；
- account add/login/status/default/list；
- provider call 或 Homie virtual-key proxy adaptation；
- usage query；
- checkpoint/blob manifest/transfer；
- move/fork prepare、restore、commit 和 receipt；
- service install/start/stop/upgrade contract。

本地无认证 UDS 不能直接暴露为 TCP。network listener 只允许 loopback、显式 Tailscale 地址或用户批准的安全配置，并使用 owner-only token 或更强认证。

### 12.2 Handoff 一致性

```text
preflight
  -> freeze/checkpoint source
  -> hash and transfer missing blobs
  -> restore into target quarantine
  -> provider-native resume/fork
  -> validate target session
  -> commit lease
  -> release or retain source according to move/fork
```

- commit 前任何失败不得破坏 source。
- checkpoint 排除 credential、provider home、SSH、`.env*`、`.git`、build/dependency、symlink、special 和 oversized files。
- target restore 只写 quarantine；validation 通过后原子发布。
- duplicate/replayed request 必须通过 operation id/lease 幂等处理。

### 12.3 完成门禁

- loopback node integration 覆盖 auth、spawn、attach、account、usage 和 shutdown。
- two-node fixture 覆盖 move、fork、incremental blob、transfer interruption、restore failure 和 lease replay。
- remote agent config/checkpoint/protocol/evidence 的 raw-key scan 为零。
- app settings、CLI 和 runtime remote spawn 使用真实 listener，不只切换 preference。
- node service 与 package/update pipeline 有安装、升级和回滚证据。

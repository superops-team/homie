# Diri host.locate_repo 对齐设计文档

```yaml
change_id: diri-host-locate-repo
beads: homie-37j
target_rows:
  - REM-003
  - API-001
  - API-003
feature_atoms:
  - M07-F001
  - M12-F001
  - M18-F001
```

## 1. 概述

### 1.1 问题/背景

`REM-003` 当前仍为 `partial`。Homie 已实现 `host.sync_prefs` 的第一阶段 include-list 和 argv model，但 Diri 的 `host.locate_repo` 还没有在 Homie 中形成可调用、可测试的等价能力。

Diri 的 `host.locate_repo` 用于按 git origin URL 在目标 host 上寻找同仓 checkout。它支持直接传 `originURL`，也支持通过 `sessionID` 从已有 session 的 cwd 推导 origin。UI 在远端 spawn 前依赖该结果决定是否直接进入同仓路径、提示未克隆，或回退到 host 默认目录。

### 1.2 目标

- 在 Homie 协议中固化 Diri 等价的 `HostLocateRepoParams` 和 `HostLocateRepoResult`，保持 `originURL`、`sessionID` JSON 拼写。
- 在 `homie-remote` 中实现本地 repo origin 发现、候选路径匹配和结果分类。
- 在 `homie-client` 中提供 `host.locate_repo` 请求处理，优先从 Homie storage 的 project facts 中匹配同仓 checkout。
- 在 `homie-cli` 中提供 `homie host locate-repo`，可用本地 fixture 验证命中、未克隆和无 origin。
- 更新 `REMOTE-003` parity lock 证据，但不把真实远端 SSH/node E2E 标成完成。

## 2. 用户场景

### 场景 1: 远端同仓 spawn 前定位路径

**Given** 用户选中一个已有本地 session，且该 session 的 repo 有 origin。  
**When** Homie 请求 `host.locate_repo`。  
**Then** Homie 返回目标 host 上同 origin 的 checkout 路径，远端 spawn 可以默认进入该路径。

### 场景 2: 目标 host 未克隆同仓库

**Given** Homie 能推导出 origin URL，但候选路径里没有同 origin checkout。  
**When** 请求 `host.locate_repo`。  
**Then** 结果返回 `originURL` 但没有 `path`，调用方可展示未克隆提示。

### 场景 3: 当前 session/cwd 没有 git origin

**Given** 当前路径不是 git repo，或没有 origin remote。  
**When** 请求 `host.locate_repo`。  
**Then** 结果不包含 `path` 和 `originURL`，调用方回退到 host 默认目录。

## 3. 功能需求

### FR-1: 协议字段对齐

`HostLocateRepoParams` 必须包含可选 `host`、`origin_url`、`session_id`，序列化为 Diri 的 `host`、`originURL`、`sessionID`。`HostLocateRepoResult` 必须包含可选 `path`、`originURL`。

### FR-2: Origin 发现

当调用方只提供 cwd/session 时，Homie 必须从 repo 的 `.git/config` 读取 `remote "origin"` 的 URL。无 `.git`、无 config、无 origin 时返回 no-origin，不 shell 到真实远端。

### FR-3: 候选路径匹配

Homie 必须在候选 checkout 路径中查找 origin URL 相同的 repo。命中时返回路径和 origin；未命中但有 origin 时只返回 origin。

### FR-4: Storage-backed host locate

`homie-client` 必须把 `host.locate_repo` 映射到真实 runtime/storage 数据：直接 `originURL` 查询时按 project `remote_origin` 匹配；`sessionID` 查询时从 session workspace 推导 origin，再匹配 project facts。

### FR-5: CLI 可验证入口

`homie host locate-repo` 必须支持：

- `--data-dir`
- `--origin-url`
- `--cwd`
- `--session-id`
- `--candidate <path>` 可重复

输出 JSON 使用 `path` 和 `originURL`。

## 4. 实现方案

### 4.1 协议层

在 `homie-proto` 新增 `HostLocateRepoParams` 与 `HostLocateRepoResult`，并把 `host.locate_repo` 纳入已存在 method catalog 的真实 DTO 覆盖。

### 4.2 remote 领域层

在 `homie-remote` 新增：

- `discover_repo_origin(cwd: &Path) -> Result<Option<String>, LocateRepoError>`
- `locate_repo_by_origin(origin_url: &str, candidates: &[PathBuf]) -> Result<HostLocateRepoResult, LocateRepoError>`
- `locate_repo(cwd: Option<&Path>, origin_url: Option<&str>, candidates: &[PathBuf]) -> Result<HostLocateRepoResult, LocateRepoError>`

实现只读 `.git/config`，不调用 `git`，避免测试和远端环境依赖。

### 4.3 client/CLI 层

`HomieClient::locate_repo` 调用 remote 领域逻辑。候选路径来自 storage 的 project roots；CLI 可通过 `--candidate` 注入 fixture 路径，便于本地 E2E。

## 5. 边界情况

| 场景 | 处理方式 |
|------|---------|
| 只传 `sessionID` 但找不到 session | 返回 protocol/runtime error，不伪造 no-origin |
| cwd 不是 git repo | 返回 `{}` |
| cwd 有 origin 但候选无匹配 | 返回 `{"originURL": "..."}` |
| 候选路径不是 git repo | 忽略该候选 |
| `.git` 是 linked worktree 文件 | 本阶段读取 gitdir 指向的 config；若不存在则返回 no-origin |
| origin URL 含敏感凭据 | 不写入日志；测试和 evidence 使用脱敏/示例域名 |

## 6. 涉及文件

- `crates/homie-proto/src/lib.rs`
- `crates/homie-remote/src/lib.rs`
- `crates/homie-remote/tests/host_locate_repo.rs`
- `crates/homie-client/src/lib.rs`
- `crates/homie-cli/src/main.rs`
- `crates/homie-cli/tests/host_locate_repo_cli.rs`
- `specs/remote-node-handoff/README.md`
- `docs/research/diri-parity-lock.md`

## 7. 验收标准

- `cargo test -p homie-proto`
- `cargo test -p homie-remote --test host_locate_repo -- --nocapture`
- `cargo test -p homie-cli --test host_locate_repo_cli -- --nocapture`
- `cargo check -p homie-remote -p homie-client -p homie-cli`
- `cargo clippy -p homie-remote -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

## 8. Beads 跟踪

- Beads: `homie-37j`
- 父级分组: `homie-h7n.5`
- 完成后只关闭 `homie-37j`；`homie-h7n.5` 仍需等待 remote node、usage、update、package、perf 的剩余 parity。

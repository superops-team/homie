# Diri Worktree CLI Runtime 对齐设计文档

```yaml
change_id: diri-worktree-cli-runtime
beads: homie-ye8
target_rows:
  - GIT-002
  - API-003
  - UI-007
feature_atoms:
  - M07-F002
  - M12-F001
```

## 1. 概述

### 1.1 问题/背景

Homie 当前只有 worktree overview 和 cleanup suggestion model，缺少 Diri 的 `worktree.create/list/remove` runtime/CLI 路径。Diri Swift CLI 已提供 `dirijor worktree list/create/remove`，通过 daemon 调 `git worktree`，并用真实 git fixture 验证 linked worktree detection。

### 1.2 目标

- 在 Homie runtime 中实现真实 `git worktree list/create/remove`。
- 在 `homie-client` 增加对应方法。
- 在 `homie-cli` 增加 `homie worktree list/create/remove`。
- 用真实临时 git repo E2E 验证 create/list/remove。
- 更新 parity lock，但不把 UI worktree sheet 标成完成。

## 2. 用户场景

### 场景 1: 列出 repo worktrees

**Given** 本地 repo 已初始化并有至少 main checkout。  
**When** 用户运行 `homie worktree list --repo <repo> --json`。  
**Then** Homie 返回 git worktree porcelain 解析出的 worktree 列表。

### 场景 2: 创建 feature worktree

**Given** 本地 repo 已有提交。  
**When** 用户运行 `homie worktree create --repo <repo> --branch feature/demo --json`。  
**Then** Homie 调用真实 `git worktree add -b feature/demo <sibling-path>` 并返回新 worktree info。

### 场景 3: 删除 worktree

**Given** 已创建 feature worktree。  
**When** 用户运行 `homie worktree remove --repo <repo> --path <worktree> --force --json`。  
**Then** Homie 调用真实 `git worktree remove`，路径被删除，后续 list 不再返回该 worktree。

## 3. 功能需求

### FR-1: Worktree 数据模型

Homie 必须提供 `WorktreeInfo`、`WorktreeCreateRequest`、`WorktreeListRequest`、`WorktreeRemoveRequest`，字段与 Diri 的 `path/branch/isBare/isDetached/isPrunable` 语义一致。

### FR-2: Runtime git worktree 操作

Runtime 必须通过 `/usr/bin/git` 调用 `worktree list --porcelain`、`worktree add -b`、`worktree remove`。调用必须禁用 stdin prompt、固定基础 env，并返回 safe error。

### FR-3: CLI 操作

CLI 必须提供：

- `homie worktree list --data-dir <dir> --repo <repo> [--json]`
- `homie worktree create --data-dir <dir> --repo <repo> --branch <branch> [--base <rev>] [--json]`
- `homie worktree remove --data-dir <dir> --repo <repo> --path <worktree> [--force] [--json]`

### FR-4: 真实 E2E

测试必须初始化真实 git repo、创建提交、调用 Homie CLI 创建/列出/删除 worktree，不允许只测字符串解析。

## 4. 非目标

- 不做 UI worktree sheet 完整交互。
- 不做默认 branch 自动生成。
- 不做 worktree cleanup suggestion UI E2E。
- 不把 `GIT-002` 或 `UI-007` 标为 implemented。

## 5. 边界情况

| 场景 | 处理方式 |
|------|---------|
| repo 不是 git repo | 返回 git safe error |
| branch 已存在或 worktree path 冲突 | 返回 git safe error |
| branch 包含 `/` | sibling path slug 使用 Diri 规则压成 dash |
| remove dirty worktree without force | 交给 git 拒绝并返回 safe error |
| `/usr/bin/git` 不存在 | 返回 I/O error |

## 6. 涉及文件

- `crates/homie-proto/src/lib.rs`
- `crates/homie-runtime/src/lib.rs`
- `crates/homie-runtime/tests/worktree_git.rs`
- `crates/homie-client/src/lib.rs`
- `crates/homie-cli/src/main.rs`
- `crates/homie-cli/tests/worktree_cli.rs`
- `docs/research/diri-parity-lock.md`

## 7. 验收标准

- `cargo test -p homie-runtime --test worktree_git -- --nocapture`
- `cargo test -p homie-cli --test worktree_cli -- --nocapture`
- `cargo check -p homie-proto -p homie-runtime -p homie-client -p homie-cli`
- `cargo clippy -p homie-runtime -p homie-client -p homie-cli --all-targets -- -D warnings`
- scoped `git diff --check`
- `make parity-lock`

## 8. Beads 跟踪

- Beads: `homie-ye8`
- 父级分组: `homie-h7n.3` / `homie-h7n.1`

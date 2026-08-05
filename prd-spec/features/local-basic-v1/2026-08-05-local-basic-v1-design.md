# Local Basic V1 设计文档

## 1. 概述

### 1.1 背景

Homie 已完成 Rust workspace 和 SQLite storage bootstrap，但还缺少一个可被用户实际执行的“基础版本”闭环：默认 Codex runtime/profile 初始化、session 记录创建、CLI 状态查看、安装和打包。为了避免一次性实现完整 GPUI/runtime/LLM proxy，本切片只补齐本地基础可用版本，作为后续 Codex runtime、LLM proxy 和桌面 UI 的基座。

### 1.2 目标

- 提供默认 Codex runtime descriptor、LLM profile、permission profile、agent profile 的 SQLite seed。
- 提供 CLI 命令创建 session 记录，用于验证 profile/runtime/storage 关系。
- 提供 CLI 命令查看 runtime/storage status 和 session list。
- 提供安装脚本，把 `homie` CLI 安装到指定 prefix。
- 提供打包脚本，生成可分发 tarball。
- 提供 `make smoke`、`make full-check` 和 `make package`，确保编译、安装、打包、使用路径通过。

### 1.3 非目标

- 不启动真实 Codex process。
- 不实现 PTY/runtime socket。
- 不实现 GPUI app。
- 不实现 LLM proxy 或 virtual key 签发。
- 不实现 MCP server proxy。

## 2. 用户场景

### 场景 1: 初始化并检查本地数据

**Given** 用户拿到仓库。
**When** 运行 `homie doctor --data-dir <tmp> --json`。
**Then** SQLite 被创建并完成 schema migration，默认 Codex profile 可被 seed。

### 场景 2: 创建基础 session 记录

**Given** 数据目录已经初始化。
**When** 运行 `homie session create --data-dir <tmp> --workspace <path> --title "Smoke"`。
**Then** Homie 使用默认 Codex agent profile 写入一条 session 记录。

### 场景 3: 查看 session list

**Given** 已创建 session 记录。
**When** 运行 `homie session list --data-dir <tmp> --json`。
**Then** 输出包含该 session、runtime `codex`、status `created`。

### 场景 4: 安装和打包

**Given** 项目在本机可构建。
**When** 运行 `scripts/package/package.sh` 和 `scripts/dev/install-local.sh --prefix <tmp>`。
**Then** tarball 存在，安装后的 `<prefix>/bin/homie doctor --data-dir <tmp> --json` 可运行。

## 3. 功能需求

### FR-1: 默认配置 seed

`homie-storage` 增加 `seed_defaults()`：

- provider: `provider_local_placeholder`
- llm profile: `llm_default`
- runtime descriptor: `runtime_codex`
- permission profile: `perm_default`
- agent profile: `agent_codex_default`, enabled + default

说明：

- provider 使用 placeholder base URL 和 secret ref，不包含真实 key。
- seed 必须幂等。
- 若用户已有同 id 配置，不覆盖。

### FR-2: Session repository

`homie-storage` 增加最小 session API：

```rust
create_session(workspace, title) -> SessionSummary
list_sessions() -> Vec<SessionSummary>
```

要求：

- create_session 使用 enabled default agent profile。
- default profile 不存在或 disabled 时返回稳定错误。
- session status 初始为 `created`。
- session 记录 runtime_id、llm_profile_id、permission_profile_id。

### FR-3: CLI commands

新增命令：

```text
homie doctor [--data-dir <path>] [--json]
homie runtime status [--data-dir <path>] [--json]
homie session create --data-dir <path> --workspace <path> [--title <title>] [--json]
homie session list --data-dir <path> [--json]
```

`doctor` 应执行 migration + seed + health check。

### FR-4: 安装脚本

新增：

```text
scripts/dev/install-local.sh --prefix <path>
```

行为：

- `cargo build --release -p homie-cli`
- 安装 binary 到 `<prefix>/bin/homie`
- 输出安装路径
- 安装后可运行 `homie doctor`

### FR-5: 打包脚本

新增：

```text
scripts/package/package.sh
```

行为：

- release build `homie-cli`
- 生成 `dist/homie-<version>-<target>.tar.gz`
- tarball 内包含 `bin/homie`、`README.md`、`LICENSE`
- 不包含 `.beads`、`target`、SQLite 数据库、secret 文件

### FR-6: 门禁

扩展 Makefile：

```text
make smoke
make package
make full-check
```

要求：

- `make smoke` 运行 doctor/session create/session list。
- `make package` 生成 tarball。
- `make full-check` 包含 pre-commit + smoke + package。

## 4. 实现方案

- 在 `homie-storage` 中实现 seed 和 session repository。
- 在 `homie-cli` 中扩展 clap subcommands。
- 新增 shell scripts，保持 POSIX shell 兼容。
- Makefile 调用 scripts，避免重复命令。

## 5. 组件 spec 影响

| 组件 | 是否影响 | 原因 |
|------|----------|------|
| `specs/storage-indexing/README.md` | 是 | 增加 seed/session repository |
| `specs/runtime-supervisor/README.md` | 否 | 不启动 runtime process |
| `specs/agent-adapter-contract/README.md` | 否 | 仅 seed Codex descriptor，不实现 adapter |
| `specs/packaging-updater/README.md` | 部分 | 只做 CLI tarball，不做 app bundle/updater |

## 6. 测试计划

- storage unit/integration：seed idempotent、create/list session、no default profile error。
- CLI functional：doctor、runtime status、session create/list。
- packaging smoke：install-local 后 binary 可运行；package tarball 存在且不含忽略文件。
- gates：`make full-check` 通过。

## 7. 验收标准

- `make full-check` 通过。
- `scripts/dev/install-local.sh --prefix <tmp>` 安装成功。
- 安装后的 `homie doctor --data-dir <tmp> --json` 成功。
- `scripts/package/package.sh` 生成 tarball。
- tarball 可解包并运行 `bin/homie doctor --data-dir <tmp> --json`。

## 8. Beads 追踪

- Beads issue: `homie-54y`
- change_id: `local-basic-v1`
- spec-id: `prd-spec/features/local-basic-v1/2026-08-05-local-basic-v1-design.md`

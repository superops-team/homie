# Local Basic V1 功能验证 Case

```yaml
change_id: local-basic-v1
report_type: functional-cases
status: pass
beads: homie-54y
source_prd: prd-spec/features/local-basic-v1/2026-08-05-local-basic-v1-design.md
```

## Case 清单

### FC-001: doctor 初始化默认配置

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-1, FR-3 |
| 命令 | `TMPDIR=$(mktemp -d) && cargo run -q -p homie-cli -- doctor --data-dir "$TMPDIR" --json` |
| 预期 | exit=0；JSON status=ok；schemaVersion=1；defaultAgentProfile=agent_codex_default |
| 证据 | `docs/verification/local-basic-v1/artifacts/fc-001-doctor.json` |

### FC-002: session create/list

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-2, FR-3 |
| 命令 | `TMPDIR=$(mktemp -d) && cargo run -q -p homie-cli -- doctor --data-dir "$TMPDIR" --json && cargo run -q -p homie-cli -- session create --data-dir "$TMPDIR" --workspace "$PWD" --title Smoke --json && cargo run -q -p homie-cli -- session list --data-dir "$TMPDIR" --json` |
| 预期 | create 输出 session；list 包含同一 session；runtimeId=runtime_codex；status=created |
| 证据 | `docs/verification/local-basic-v1/artifacts/fc-002-session.json` |

### FC-003: runtime status

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-3 |
| 命令 | `TMPDIR=$(mktemp -d) && cargo run -q -p homie-cli -- runtime status --data-dir "$TMPDIR" --json` |
| 预期 | exit=0；status=ok；runtimeProcess=not_running；databaseReady=true |
| 证据 | `docs/verification/local-basic-v1/artifacts/fc-003-runtime-status.json` |

### FC-004: install-local

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-4 |
| 命令 | `PREFIX=$(mktemp -d) && scripts/dev/install-local.sh --prefix "$PREFIX" && "$PREFIX/bin/homie" doctor --data-dir "$(mktemp -d)" --json` |
| 预期 | 安装成功；安装后的 binary 可运行 doctor |
| 证据 | `docs/verification/local-basic-v1/artifacts/fc-004-install-local.txt` |

### FC-005: package tarball

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-5 |
| 命令 | `scripts/package/package.sh` |
| 预期 | `dist/homie-*.tar.gz` 存在；tarball 不包含 `.beads`、`target`、SQLite、secret |
| 证据 | `docs/verification/local-basic-v1/artifacts/fc-005-package.txt` |

### FC-006: full gate

| 字段 | 内容 |
|------|------|
| 覆盖需求 | FR-6 |
| 命令 | `make full-check` |
| 预期 | pre-commit、smoke、package 全部通过 |
| 证据 | `docs/verification/local-basic-v1/artifacts/fc-006-full-check.txt` |

## 覆盖矩阵

| PRD 需求 | 覆盖 Case |
|----------|-----------|
| FR-1 默认配置 seed | FC-001 |
| FR-2 Session repository | FC-002 |
| FR-3 CLI commands | FC-001, FC-002, FC-003 |
| FR-4 安装脚本 | FC-004 |
| FR-5 打包脚本 | FC-005 |
| FR-6 门禁 | FC-006 |

# Code Review Report: Diri host.locate_repo

```yaml
change_id: diri-host-locate-repo
beads: homie-37j
status: pass
reviewed_at: 2026-08-08
```

## 1. 审查范围

- 文件/模块：`homie-proto`, `homie-remote`, `homie-client`, `homie-cli`, `homie-storage`, `specs/remote-node-handoff`, `docs/research/diri-parity-lock`。
- 变更类型：新增 Diri `host.locate_repo` DTO、repo origin discovery、storage-backed client handler、CLI fixture E2E。
- 调用链：CLI/client protocol -> `HomieClient::locate_repo` -> storage project facts -> remote locate helper。
- 参考规则：AGENTS.md workflow、`docs/development/standards.md`、`docs/development/quality-gates.md`。

## 2. 旧问题复核

| ID/标题 | 位置 | 状态 | 依据 |
|---|---|---|---|
| RED-001 missing DTO | `homie-proto` | fixed | added `HostLocateRepoParams` / `HostLocateRepoResult` and proto test |
| RED-002 missing remote API | `homie-remote` | fixed | added origin discovery/matching and 4 tests |
| RED-003 missing CLI command | `homie-cli` | fixed | added `host locate-repo` and CLI integration test |

## 3. Findings

| 严重度 | 类别 | 位置 | 证据与影响 | 处置 |
|---|---|---|---|---|
| medium | Correctness | `crates/homie-client/src/lib.rs` | First GREEN version used project root candidates but still relied on candidate `.git/config`, so storage `remote_origin` facts were not authoritative. | fixed: `HomieClient::locate_repo` now first matches `projects.remote_origin`; test target clone has no `.git/config`. |
| low | Scope | `docs/research/diri-parity-lock.md` | This slice does not execute real remote SSH/node lookup. Marking REM-003 implemented would overclaim. | fixed: parity row remains `partial` and notes real remote E2E pending. |

## 4. 对抗式复盘

- 反例：candidate path exists but has no `.git/config`; storage says it is the matching project. Expected: client still returns path. Covered by `client_dispatches_host_locate_repo_from_project_facts`.
- 反例：source cwd has no origin. Expected: empty result, not an error. Covered by `returns_empty_result_when_cwd_has_no_origin`.
- 反例：linked worktree `.git` is a file. Expected: follow `gitdir` and read config. Covered by `follows_linked_worktree_gitdir_for_origin`.

## 5. 修复摘要

- Added Diri-compatible `originURL/sessionID` protocol DTOs.
- Added read-only repo origin discovery and candidate matching in `homie-remote`.
- Added storage-backed `HomieClient::locate_repo` and protocol dispatch.
- Added `homie host locate-repo`.
- Updated component spec and parity lock evidence.

## 6. 验证结果

| 命令 | 结果 | 说明 |
|---|---|---|
| `cargo test -p homie-proto host_locate_repo_round_trips_diri_spelling -- --nocapture` | pass | protocol spelling |
| `cargo test -p homie-remote --test host_locate_repo -- --nocapture` | pass | origin discovery/match/no-origin |
| `cargo test -p homie-client client_dispatches_host_locate_repo_from_project_facts -- --nocapture` | pass | storage-backed handler |
| `cargo test -p homie-cli --test host_locate_repo_cli -- --nocapture` | pass | CLI E2E |
| `cargo check -p homie-proto -p homie-remote -p homie-client -p homie-cli` | pass | build |
| `cargo clippy -p homie-remote -p homie-client -p homie-cli --all-targets -- -D warnings` | pass | lint |
| scoped `git diff --check` | pass | whitespace |
| `make parity-lock` | pass | remaining partial rows retained |

## 7. 剩余风险

- No real SSH/node remote locate execution yet.
- No app remote spawn consumption yet.

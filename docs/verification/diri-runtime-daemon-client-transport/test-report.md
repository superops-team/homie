# Wave 1A 最终测试报告

```yaml
change_id: diri-runtime-daemon-client-transport
beads: homie-nep
date: 2026-08-08
scope_status: pass
workspace_status: partial
```

## 1. 判定

Wave 1A 的 protocol、daemon、async client、CLI/MCP、GPUI bridge、package closure 和 cross-entry 测试通过。完整 workspace 仍有 3 个已知 T-102 holder/PTY 失败，因此按 PRD 的诚实准出规则记为 `partial`，不将 RT-001、RT-006、RT-007 提升为 implemented。

## 2. 静态门禁

| 命令 | 结果 |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo check --workspace --all-targets` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `git diff --check` | pass |

## 3. Wave 1A 功能门禁

| 范围 | 结果 |
|---|---|
| runtime unit tests | 133/133 pass |
| `server_control` | 36/36 pass |
| `server_streams` | 20/20 pass |
| `daemon_process` | 12/12 pass |
| `runtime_dispatcher` | 17/17 pass |
| client lib/launcher/recovery/request/streams/facade | 63/63 pass |
| app tests, including event resubscribe/terminal offset/dispatch recovery | 19/19 pass |
| holder launch rollback | 1/1 pass |
| cross-entry real-daemon E2E | 1/1 pass |
| `make package-shell-test` | pass |
| `make smoke` | pass: `PACKAGED_RUNTIME_SMOKE=pass`, `HELLO_STATE_SNAPSHOT=pass` |

最终 package smoke 使用当前代码重新构建并嵌套签名 app、CLI、daemon 和 holder。`GUI_LAUNCH=not_run`，`NOTARIZATION=not_required`；完整 GUI 与公证仍由后续 package/release change 负责。

## 4. Workspace 与 T-102 分类

`cargo test --workspace --all-targets -- --test-threads=1` 的 Wave 1A 及后续非 T-102 suites 均通过；当前最终代码的 focused holder suite 结果为：

```text
cargo test -p homie-runtime --test session_lifecycle -- --test-threads=1
10 passed; 3 failed
```

失败项：

| 测试 | Actual | Expected | 归属 |
|---|---|---|---|
| `runtime_holder_stat_tracks_resize_and_log_offsets` | initial `log_offset=16` | `0` | T-102 prompt/epoch semantics |
| `runtime_reopen_can_adopt_holder_and_continue_session` | `detached` | `running` | T-102 holder adoption |
| `runtime_spawn_shell_uses_live_pty` | reopened `detached` | `running` | T-102 holder adoption |

这些失败在本 change 中没有通过伪造状态或放宽产品断言消除。失败测试遗留的 temp holder 已清理，最终检查为：

```text
test_daemon_count=0
test_holder_count=0
```

## 5. 外部环境限制

完整 workspace 命令的 Rust 断言之外，Trae sandbox 还拒绝访问以下既有外部状态：

```text
~/.codex/state_5.sqlite
~/.codex/state_5.sqlite-wal
~/.codex/state_5.sqlite-shm
~/.bytesec/commit_hook/commit_result.json
```

该外层 gate 记为 `blocked`，不等同于 Rust 测试失败，也不作为 Wave 1A pass 的替代证据。

## 6. 结论

- Wave 1A implementation gate：`pass`
- Full workspace：`partial`
- T-102：`blocked`，由 `diri-agent-session-runtime` 后续 change 关闭
- 测试进程清理：`pass`

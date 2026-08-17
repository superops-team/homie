# Engine Control Wire/Runtime Split 功能验证 Case

## 1. 验证目标

面向 `engine-control-wire-runtime-split` 首切片（S1 wire codec 抽取），证明：

- 只抽 wire 编解码纯函数（write_message/decode/encode/poisoned/resolve_on_path/
  migrate_control_error/io_control_error）；
- 新模块 `control/wire.rs` 不依赖 registry/session/GPUI/socket-loop；
- 行为由普通 Rust focused tests 覆盖；
- wire shape（method 名、参数、返回 JSON）完全不变；
- `control.rs` 只保留 routing + handler + runtime，行数下降。

## FC-01: 基线测试全绿

```bash
cargo test -p homie-engine --lib
```

通过标准：抽取前 264 passed / 0 failed 为基线。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-01-baseline.log`

## FC-02: wire.rs 纯函数抽取完成且无重依赖

```bash
test -s homie/crates/homie-engine/src/control/wire.rs
if rg -n "Registry|Session|ControlServer|spawn|bind\(|UnixListener" \
  homie/crates/homie-engine/src/control/wire.rs; then exit 1; fi
echo "wire.rs has no registry/session/socket-loop dependency"
```

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-02-pure-module.log`

## FC-03: wire 函数 focused tests

```bash
cargo test -p homie-engine control::wire -- --nocapture
```

通过标准：覆盖 decode round-trip / decode 缺省空对象 / decode shape 错误 → bad_request /
encode round-trip / io_error 映射 / migrate_error 映射 / resolve_on_path。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-03-wire-tests.log`

## FC-04: 全量行为不变

```bash
cargo test -p homie-engine
```

通过标准：抽取后全部测试（lib 272 + 集成测试）全绿，无新增失败。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-04-full-tests.log`

## FC-05: 静态门禁与范围守卫

```bash
cargo fmt -p homie-engine -- --check
cargo check -p homie-engine
git diff --name-only -- homie/crates/homie-engine/src/control
```

通过标准：fmt 干净、无 warning、只改动 control 模块内预期文件。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-05-static-gates.log`

---

# S2 codec 投影（history_entry_to_wire / worktree_to_wire）

## FC-06: codec.rs 纯函数抽取完成且无重依赖

```bash
test -s homie/crates/homie-engine/src/control/codec.rs
rg -n "Registry|Session|ControlServer|spawn|bind\(|UnixListener|Mutex|Arc" \
  homie/crates/homie-engine/src/control/codec.rs
```

通过标准：文件存在；`rg` 无 Registry/Session/ControlServer/spawn/bind/UnixListener/Mutex/Arc 命中。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-06-codec-pure.log`

## FC-07: codec 投影 focused tests

```bash
cargo test -p homie-engine control::codec -- --nocapture
```

通过标准：覆盖 kind 映射（ClaudeCode→CLAUDE_CODE、Codex→CODEX）、标量字段保留、
时间戳毫秒换算、created_at/title 缺失保留、worktree 字段保留、bare/缺 branch 保留。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-07-codec-tests.log`

## FC-08: 全量行为不变

```bash
cargo test -p homie-engine
```

通过标准：抽取后全部测试（lib 278 + 集成测试）全绿，0 failed。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-08-full-tests.log`

## FC-09: 静态门禁与范围守卫

```bash
cargo fmt -p homie-engine -- --check
cargo check -p homie-engine
```

通过标准：fmt 干净、无 warning。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-09-static-gates.log`

---

# S3 runtime 生命周期抽取（bind 循环、订阅句柄、连接守卫、空闲关停、远程恢复）

## FC-10: runtime.rs 抽取完成且无 transport 泄漏

```bash
# runtime.rs 拥有生命周期符号，control.rs 不再定义这些 impl 方法
bash /tmp/fc10.sh
```

通过标准：`bind` / `daemon_shutdown` / `impl Drop for ControlServer` /
`SubscriptionHandle` / `ActiveConnectionGuard` / `spawn_remote_restore` /
`idle_shutdown_refusal` / `daemon_prepare_shutdown` / `retire_legacy_remote_sessions` /
`restore_remote_bindings` / `legacy_remote_marker` 全部由 `runtime.rs` 拥有；
`control.rs` 不再定义它们；`runtime.rs` 不含 `serve`/`handle_line`/`dispatch`。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-10-runtime-pure.log`

## FC-11: runtime focused tests

```bash
cargo test -p homie-engine control::runtime::tests -- --nocapture
```

通过标准：`idle_shutdown_requires_exactly_the_requesting_client_and_no_session`、
`dropping_an_event_subscription_stops_its_detached_thread` 两个测试全绿。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-11-runtime-tests.log`

## FC-12: 全量行为不变

```bash
cargo test -p homie-engine
```

通过标准：抽取后全部测试（lib 278 + 集成测试）全绿，0 failed。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-12-full-tests.log`

## FC-13: 静态门禁与范围守卫

```bash
cargo fmt -p homie-engine -- --check
cargo check -p homie-engine
```

通过标准：fmt 干净、无 warning（移除残留 unused `Ordering` import）。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-13-static-gates.log`

---

# S4-a handler 机械下沉（ControlServer 只保留路由表）

## FC-14: handlers.rs 抽取完成且 transport 层留在 control.rs

```bash
wc -l homie/crates/homie-engine/src/control.rs
rg -n "fn (session_spawn|session_list|host_|worktree_|session_|governor_|client_set_active|browser_call|hello|resolve_host)\b" \
  homie/crates/homie-engine/src/control.rs
rg -n "fn (serve|handle_line|dispatch)\b" homie/crates/homie-engine/src/control/handlers.rs
```

通过标准：`control.rs` < 800 行（目标 ~460）；control.rs 不再定义任何 handler 方法；
handlers.rs 不含 serve/handle_line/dispatch。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-14-handler-sink.log`

## FC-15: 全量行为不变

```bash
cargo test -p homie-engine
```

通过标准：抽取后全部测试（lib 278 + 集成测试）全绿，0 failed。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-15-full-tests.log`

## FC-16: 静态门禁与范围守卫

```bash
cargo fmt -p homie-engine -- --check
cargo check -p homie-engine
git status --porcelain
```

通过标准：fmt 干净、无 warning；仅改动 control 模块内文件
（control.rs / control/handlers.rs / control/tests.rs）。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-16-static-gates.log`

---

# S4-b 领域逻辑下沉（worktree_overview → crate::git）

## FC-17: worktree_overview 领域逻辑下沉到 crate::git

```bash
rg -n "stale_suggestion" homie/crates/homie-engine/src/control/handlers.rs
rg -n "pub fn worktree_overview" homie/crates/homie-engine/src/git.rs
```

通过标准：`worktree_overview` 的 staleness join + git subprocess 领域逻辑由
`crate::git::worktree_overview` 拥有；handler 只剩 lock registry + 收集 records/roots +
调用 + encode。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-17-worktree-sink.log`

## FC-18: 全量行为不变

```bash
cargo test -p homie-engine
```

通过标准：抽取后全部测试（lib 278 + 集成测试）全绿，0 failed。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-18-full-tests.log`

## FC-19: 静态门禁

```bash
cargo fmt -p homie-engine -- --check
cargo check -p homie-engine
```

通过标准：fmt 干净、无 warning。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-19-static-gates.log`

---

# S4-b 领域逻辑下沉（prompt-injection → control/inject）

## FC-20: prompt-injection 领域下沉到 control/inject.rs

```bash
rg -n "fn (prepare_agent_input|accept_claude_workspace_trust|inject_initial_prompt|wait_until_ready|screen_settled|wait_for_echo|verification_probe|is_claude_workspace_trust_screen)" homie/crates/homie-engine/src/control/inject.rs
rg -n "fn (prepare_agent_input|inject_initial_prompt|wait_until_ready|wait_for_echo|verification_probe)" homie/crates/homie-engine/src/control/handlers.rs
rg -n "^[^/]*\b(ControlServer|UnixListener|UnixStream|super::)\b" homie/crates/homie-engine/src/control/inject.rs
wc -l homie/crates/homie-engine/src/control/handlers.rs homie/crates/homie-engine/src/control/inject.rs
```

通过标准：`prepare_agent_input` 及其全部辅助函数（含 Claude workspace-trust 自动确认、
initial prompt 注入、echo 验证、就绪等待、屏幕稳定等待）由 `control/inject.rs` 拥有；
handlers.rs 不再定义它们；inject.rs 无 transport/runtime 依赖；handlers.rs 从 1,977 行降至
1,582 行。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-20-inject-sink.log`

## FC-21: 全量行为不变

```bash
cargo test -p homie-engine
```

通过标准：抽取后全部测试（lib 278 + 集成测试）全绿，0 failed。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-21-full-tests.log`

## FC-22: 静态门禁

```bash
cargo fmt -p homie-engine -- --check
cargo check -p homie-engine
git status --porcelain
```

通过标准：fmt 干净、无 warning；仅改动 control 模块内文件。

证据路径：`docs/verification/engine-control-wire-runtime-split/fc-22-static-gates.log`

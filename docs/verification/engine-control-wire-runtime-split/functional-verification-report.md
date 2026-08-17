# Engine Control Wire/Runtime Split 功能验证报告

## 1. 结论

`engine-control-wire-runtime-split` 首切片（S1 wire codec 抽取）功能验证通过。

已完成：

- 新增 `homie/crates/homie-engine/src/control/wire.rs`（170 行）。
- 从 `control.rs` 抽出 7 个 wire 编解码纯函数：
  `write_message` / `poisoned` / `decode` / `encode` / `resolve_on_path` /
  `migrate_control_error` / `io_control_error`。
- 新增 8 个普通 Rust focused tests 覆盖编解码与错误映射。
- `control.rs` 从 3,802 行降至 3,739 行；wire shape 完全不变。

## 2. Case 执行结果

| Case | 状态 | 证据 |
|---|---|---|
| FC-01 基线测试全绿 | pass | 264 passed / 0 failed（抽取前） |
| FC-02 wire.rs 无重依赖 | pass | `rg` 无 Registry/Session/ControlServer/spawn/bind 命中 |
| FC-03 wire focused tests | pass | 8 个 `control::wire` 测试全绿 |
| FC-04 全量行为不变 | pass | 272 lib + 集成测试全绿，0 failed |
| FC-05 静态门禁 | pass | `cargo fmt --check` 干净、`cargo check` 无 warning |

## 3. 关键证据

- `control/wire.rs` 无 `Registry|Session|ControlServer|spawn|bind(|UnixListener` 依赖。
- `cargo test -p homie-engine`：272 lib tests + 各集成测试套件全部 `0 failed`。
- `cargo fmt -p homie-engine -- --check`：clean。
- `cargo check -p homie-engine`：无 warning（移除了 unused `Write` import）。
- 行数：`control.rs` 3,802 → 3,739；`control/wire.rs` 新增 170 行。

## 4. 残余风险

- 本切片只抽 wire codec；S2 codec 投影（history_entry_to_wire/worktree_to_wire）、
  S3 runtime 生命周期、S4 handler 下沉尚未执行，留待后续切片。
- wire shape 由现有集成测试（control_socket/session/pty）间接保障，未新增 golden fixture；
  协议 golden fixture 属 `protocol-contract-golden-fixtures` 职责。

---

# S2 codec 投影 功能验证

## 1. 结论

`engine-control-wire-runtime-split` 第二切片（S2 codec 投影抽取）功能验证通过。

已完成：

- 新增 `homie/crates/homie-engine/src/control/codec.rs`（122 行）。
- 从 `control.rs` 抽出 2 个 proto↔domain 投影纯函数：
  `history_entry_to_wire` / `worktree_to_wire`。
- 新增 6 个普通 Rust focused tests 覆盖 kind 映射、标量保留、时间戳换算与 worktree 字段。
- `control.rs` 从 3,739 行降至 3,714 行；wire shape 完全不变。

## 2. Case 执行结果

| Case | 状态 | 证据 |
|---|---|---|
| FC-06 codec.rs 无重依赖 | pass | `rg` 无 Registry/Session/ControlServer/spawn/bind/Mutex/Arc 命中 |
| FC-07 codec focused tests | pass | 6 个 `control::codec` 测试全绿 |
| FC-08 全量行为不变 | pass | 278 lib + 集成测试全绿，0 failed |
| FC-09 静态门禁 | pass | `cargo fmt --check` 干净、`cargo check` 无 warning |

## 3. 关键证据

- `control/codec.rs` 无重依赖；仅依赖 `homie_proto::{AgentKind, DateMillis}` 与 domain 类型。
- `cargo test -p homie-engine`：278 lib tests + 集成测试全部 `0 failed`。
- 投影函数体一字未改，仅移动 + `pub(super)` 可见性限定在 `control` 模块内。
- 行数：`control.rs` 3,739 → 3,714；`control/codec.rs` 新增 122 行。

## 4. 残余风险

- S3 runtime 生命周期、S4 handler 下沉尚未执行，留待后续切片（同一 change_id）。

---

# S3 runtime 生命周期抽取 功能验证

## 1. 结论

`engine-control-wire-runtime-split` 第三切片（S3 runtime 生命周期抽取）功能验证通过。

已完成：

- 新增 `homie/crates/homie-engine/src/control/runtime.rs`（424 行）。
- 从 `control.rs` 抽出生命周期协调逻辑：
  `bind` / `spawn_remote_restore` / `retire_legacy_remote_sessions` /
  `legacy_remote_marker` / `restore_remote_bindings` / `daemon_prepare_shutdown` /
  `daemon_shutdown_if_idle` / `daemon_shutdown` / `impl Drop for ControlServer` /
  `SubscriptionHandle` / `ActiveConnectionGuard` / `idle_shutdown_refusal`。
- 两个相关测试（`idle_shutdown_requires_exactly_...`、`dropping_an_event_subscription_...`）
  随迁入 `runtime.rs`，并改用 `SubscriptionHandle::new(...)` 构造。
- `control.rs` 从 3,714 行降至 3,328 行；wire shape 与生命周期行为完全不变。

## 2. Case 执行结果

| Case | 状态 | 证据 |
|---|---|---|
| FC-10 runtime.rs 无 transport 泄漏 | pass | 生命周期符号全部归 runtime.rs；control.rs 不再定义；无 serve/handle_line/dispatch |
| FC-11 runtime focused tests | pass | 2 个 `control::runtime::tests` 测试全绿 |
| FC-12 全量行为不变 | pass | 278 lib + 集成测试全绿，0 failed |
| FC-13 静态门禁 | pass | `cargo fmt --check` 干净、`cargo check` 无 warning |

## 3. 关键证据

- `control/runtime.rs` 拥有全部生命周期符号，无 transport 泄漏。
- `SubscriptionHandle::new(stop, handle)` 构造器 `pub(super)`；`ActiveConnectionGuard::new`
  `pub(super)`，仅 `control` 模块内可见。
- `daemon_*` 方法 `pub(super)`，`control.rs` 的 `dispatch` 可访问，外部不可见。
- `cargo test -p homie-engine`：278 lib tests + 各集成测试套件全部 `0 failed`。
- 移除 `control.rs` 残留 unused `Ordering` import，`cargo check` 0 warning。
- 行数：`control.rs` 3,714 → 3,328；`control/runtime.rs` 新增 424 行。

## 4. 残余风险

- S4 handler 下沉（session_*/host_*/worktree_* 下沉到 registry/session/remote manager）
  尚未执行，体量最大、blast radius 大，建议单独切片确认，目标 `control.rs < 800` 行。

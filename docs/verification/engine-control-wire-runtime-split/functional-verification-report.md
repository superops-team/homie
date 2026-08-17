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

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

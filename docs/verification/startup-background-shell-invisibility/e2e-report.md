# E2E 测试报告

## 1. 执行结果

| 项目 | 命令 | 状态 | 证据 |
|------|------|------|------|
| App build | `cargo build --manifest-path homie/Cargo.toml -p homie-app --bin homie` | pass | `e2e-app-build.log` |
| Engine build | `cargo build --manifest-path homie/Cargo.toml -p homie-engine --bin homied-rs --bin homie-holder` | pass | `e2e-engine-build.log` |
| Heavy rc startup smoke | `bash run-heavy-rc-smoke.sh` | pass | `e2e-heavy-rc.log` |
| Startup exec probe | `bash run-startup-exec-probe.sh` | pass | `e2e-exec-probe.log` |

## 2. 结论

- Rust Engine 可以构建。
- GPUI app 可以构建。
- daemon 启动到 socket ready 不执行 heavy interactive shell rc。
- daemon 普通启动不触发 `ssh`、`node`、`rsync`、`gh`、`lsof`、`open`、`osascript` wrapper。

## 3. 残余风险

- 本 E2E 不打开真实 GUI 窗口进行截图验证；本需求核心是后台进程无感，已通过 daemon 启动和 exec probe 覆盖。
- PATH lazy refresh 目前接在用户打开 new-agent picker 的路径上，后续可在 UI 中增加明确“刷新中/最近刷新时间”提示，但不属于 P0。

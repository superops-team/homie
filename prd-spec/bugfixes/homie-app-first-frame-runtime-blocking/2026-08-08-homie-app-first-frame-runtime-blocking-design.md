# Homie App First-frame Runtime Blocking 修复设计

```yaml
change_id: homie-app-first-frame-runtime-blocking
beads: homie-7tb
target_rows:
  - UI-001
```

## 1. 背景

在刷新 Homie 当前 UI 截图证据时，`cargo run -p homie-app` 与 packaged `Homie.app` 均出现窗口可枚举但截图不可用/全屏捕获黑屏的问题。诊断中确认 `HomieWorkbench::load` 和 `Render::render` 能执行，但发现两个真实风险：

- holder IPC `request` 没有读写超时，如果 holder socket 接受连接但不返回，app 首帧或后续刷新可能阻塞。
- `Render::render` 内同步调用 `sync_terminal_geometry`，把 runtime holder I/O 放进 GPUI render 路径。

截图环境本身仍有 WindowServer shield/frontmost 失真问题，不能把新截图作为通过证据。本 bugfix 只修代码层的首帧阻塞风险。

## 2. 目标

- holder request 设置短读写超时，避免无界等待。
- app render path 不直接调用 runtime resize/session I/O。
- 保持现有 app/runtime 测试通过。
- 明确本修复不关闭 UI parity。

## 3. 非目标

- 不修复本机 macOS 截图权限/WindowServer shield。
- 不用 rejected screenshot 作为 UI parity 证据。
- 不改变 runtime session 生命周期语义。

## 4. 验收

- `cargo check -p homie-app`
- `cargo test -p homie-app --test app_shell_copy_regression -- --nocapture`
- `cargo test -p homie-runtime --test session_lifecycle -- --nocapture`
- `cargo clippy -p homie-app -p homie-runtime --all-targets -- -D warnings`
- `make ui-screenshot-gate`
- `loopx --registry .loopx/registry.json check --scan-root /Users/bytedance/workspace/github/homie`
- `make parity-lock`

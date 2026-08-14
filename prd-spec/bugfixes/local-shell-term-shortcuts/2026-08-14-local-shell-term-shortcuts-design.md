# 本地 Shell 任务快捷键不完整问题修复设计文档

## 1. 概述

### 1.1 问题

用户在 Homie 中新建 `shell` 类型任务后，按 `Ctrl+L` 这类常见 shell/readline/zle
快捷键无法完成预期行为。以 `Ctrl+L` 为例，用户期望清屏，但实际屏幕没有按完整 shell
终端行为刷新。

该问题影响范围不限于 `Ctrl+L`。任何依赖终端能力数据库、readline/zle 终端类型、全屏 TUI 控制序列或 shell line editor 的快捷键，都可能因为本地 shell PTY 环境缺少 `TERM` 而表现不完整。

### 1.2 诊断结论

本地诊断使用临时 `HOMIE_KEY_TRACE=/private/tmp/homie-key-trace.log` 对客户端和 Engine 输入链路加 trace。结论：

1. App 侧 `TerminalPane::handle_key_down` 收到了 `Ctrl+L`：

```text
stage=enter key="l" key_char=None mods={platform:false, control:true, ...}
```

2. App 侧编码为 shell 期望的 Form Feed：

```text
stage=encoded ... bytes=[12]
```

3. Engine 侧 `Session::write_input` 收到并写入 held PTY transport：

```text
engine stage=enter session=<id> bytes=[12]
engine stage=wrote-held session=<id> bytes=[12]
```

4. 对应 raw PTY output log 中没有出现清屏输出序列：

```text
b'\x0c' -1
b'\x1b[2J' -1
b'\x1b[H' -1
b'\x1bc' -1
b'\x1b[3J' -1
```

5. 运行中的本地 shell 子进程环境中未观察到 `TERM`：

```text
ps eww -p <zsh-pid> | tr ' ' '\n' | rg '^(TERM|COLORTERM|SHELL|PATH|ZDOTDIR)='
# 无 TERM 输出
```

因此问题不在 GPUI key adapter、`homie-term` encoder、TerminalPane 焦点、SessionAttachment 或 Engine write path；问题在本地 shell/generic PTY launch environment 没有补齐 `TERM`。

## 2. 根因

`homie/crates/homie-engine/src/control.rs` 的本地 `session_spawn` 路径在处理 `shell` 或显式 `argv` 时，会创建 `PtySpec` 并继承 daemon 进程环境：

```rust
let mut spec = crate::pty::PtySpec::new(argv.clone(), &cwd_path);
spec.env = inherited;
spec.env.retain(|(key, _)| key != "NO_COLOR");
spec
```

但 GUI app / daemon 启动环境通常没有 `TERM`。PTY 子进程虽然连接到了伪终端，但 shell line editor 仍依赖 `TERM`/terminfo 来决定清屏、光标移动、全屏 TUI 等控制序列。缺少 `TERM` 时，`Ctrl+L` 可能只重绘提示符或表现不完整。

对比远程 spawn 路径，代码已经显式设置了：

```rust
spec.env.retain(|(key, _)| key != "TERM");
spec.env.push(("TERM".into(), "xterm-256color".into()));
```

本地 shell/generic argv 路径缺少同等处理。

## 3. 用户场景

### 场景 1: 本地 shell 中 Ctrl+L 清屏

**Given** 用户在 Homie 中新建本地 `shell` 类型任务
**When** 用户在 shell terminal 中按 `Ctrl+L`
**Then** shell 收到 `0x0c` 并基于 `TERM=xterm-256color` 输出清屏/重绘控制序列，Homie UI 更新为清屏后的视图

### 场景 2: 本地 shell 启动 TUI 程序

**Given** 用户在 Homie shell task 中运行依赖终端能力的程序，例如 `less`、`vim`、`top` 或 shell completion UI
**When** 程序读取 `$TERM` 或 terminfo
**Then** 它能看到有效终端类型，而不是空值或 `dumb`

### 场景 3: 旧 session 不被误认为已修复

**Given** 用户已有修复前创建的 shell session
**When** 代码修复后不重启该 session
**Then** 旧进程环境不会被 retroactively 修改；验收必须基于新建 shell session

## 4. 功能需求

### FR-1: 本地 shell/generic argv PTY 必须设置 TERM

本地 `session_spawn` 中所有走 `PtySpec::new(argv, cwd)` 的 shell/generic argv 路径必须确保：

- 移除继承环境中的旧 `TERM`；
- 添加 `TERM=xterm-256color`；
- 保持移除 `NO_COLOR` 的现有行为。

### FR-2: 行为必须与远程 launch environment 保持一致

本地 shell/generic argv 路径与远程 launch 路径对 `TERM` 的处理应一致。后续若要改默认终端类型，应通过一个共享 helper 或明确 specs 同步两个路径。

### FR-3: 不改变 agent manifest launch 行为

本次只修复 `shell` 和 explicit/generic argv 的 PTY 环境。已有 manifest agent 的 `descriptor.spawn_spec(...)` 行为不在本次范围内，除非测试证明它同样缺少 `TERM` 且属于同一根因。

### FR-4: 验证必须覆盖真实 shell 快捷键路径

验证不能只检查 `TerminalPane` 编码。必须至少覆盖：

- `TerminalPane` 编码 `Ctrl+L -> [12]`；
- Engine `Session::write_input` 能写入 PTY；
- 新建 shell session 的环境包含 `TERM=xterm-256color`；
- raw output log 在 `Ctrl+L` 后出现清屏相关控制序列，或用等价的 headless/PTY 测试证明 shell 收到 `TERM` 后产生清屏行为。

## 5. 实现方案

### 5.1 新增本地 PTY 环境 helper

在 `homie-engine/src/control.rs` 或附近模块新增小 helper：

```rust
fn prepare_local_pty_environment(mut inherited: Vec<(String, String)>) -> Vec<(String, String)> {
    inherited.retain(|(key, _)| key != "TERM" && key != "NO_COLOR");
    inherited.push(("TERM".into(), "xterm-256color".into()));
    inherited
}
```

如远程路径也可复用，应优先抽到更通用 helper，避免本地/远程分叉。

### 5.2 应用到 explicit argv / shell 路径

修改本地 `session_spawn` 中 `None if !argv.is_empty()` 分支：

```rust
let mut spec = crate::pty::PtySpec::new(argv.clone(), &cwd_path);
spec.env = prepare_local_pty_environment(inherited);
spec
```

### 5.3 测试覆盖

建议新增测试：

1. 本地 shell spawn spec 会设置 `TERM=xterm-256color`。
2. 本地 generic command spawn spec 会设置 `TERM=xterm-256color`。
3. 如果 inherited 中已有 `TERM=dumb`，最终只保留 `TERM=xterm-256color`。
4. 继续保留 `NO_COLOR` 被移除。
5. 端到端/功能验证：新建 shell session 后执行 `printf '%s\n' \"$TERM\"` 或等价 control/socket 检查，应返回 `xterm-256color`。

## 6. 涉及文件

- `homie/crates/homie-engine/src/control.rs`
- `homie/crates/homie-engine/tests/control_socket.rs` 或相关 Engine test
- `specs/engine-session-runtime.md`
- `docs/verification/local-shell-term-shortcuts/*`
- `openspec/changes/local-shell-term-shortcuts/*`

## 6.1 受影响长期规格

- `specs/engine-session-runtime.md`: 新增 Engine session runtime 合同，固化 shell/generic argv PTY 子进程环境必须移除继承 `TERM`/`NO_COLOR` 并设置 `TERM=xterm-256color`。

## 7. 验证计划

### 7.1 单元测试

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-engine local_shell -- --nocapture
cargo test --manifest-path homie/Cargo.toml -p homie-engine term -- --nocapture
```

### 7.2 功能验证

使用新建 shell session 验证：

```bash
echo $TERM
```

预期：

```text
xterm-256color
```

然后按 `Ctrl+L`，确认 raw output log 或 UI 表现显示清屏行为。

### 7.3 回归验证

- shell session 正常启动；
- generic command session 正常启动；
- manifest agent session 不受影响；
- 远程 session 不受影响；
- `git diff --check` 通过。

## 8. 边界情况

| 场景 | 处理方式 |
|------|----------|
| inherited env 已有 `TERM=dumb` | 覆盖为 `xterm-256color` |
| inherited env 没有 `TERM` | 添加 `xterm-256color` |
| 旧 shell session | 不 retroactively 修改；用户需新建 session 验证 |
| remote session | 保持现有逻辑；若抽 helper，必须保证结果一致 |
| manifest agent | 默认不改变，除非明确走 explicit argv 分支 |

## 9. 验收标准

1. 新建本地 `shell` session 中 `$TERM == xterm-256color`。
2. 新建本地 shell 中 `Ctrl+L` 能产生完整清屏行为。
3. 本地 generic argv session 也具备 `TERM=xterm-256color`。
4. `NO_COLOR` 仍不会进入 PTY child environment。
5. 远程 session TERM 处理不回退。
6. Targeted Engine tests 和功能验证 Case 全部通过。

## 10. Beads 追踪

- Beads: `homie-mff`
- change_id: `local-shell-term-shortcuts`
- 类型: bugfix
- 优先级: P1
- source: `shell-key-trace-diagnosis`

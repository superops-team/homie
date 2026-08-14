# 清理 Dev 顶栏链接和开发标识展示设计文档

## 1. 背景

用户在修复版 dev app 中验证 shell 快捷键时，顶部区域出现两类干扰信息：

- 居中的 `DEV · dev/local-shell-term-shortcuts@...` 浮层；
- terminal header 右侧的 `github.com`、`chatgpt.com` 等 artifact/link chip。

这些信息对当前终端工作流不是必要信息，会让页面显得拥挤，并影响用户对 terminal 内容的关注。

## 2. 目标

- 移除窗口中间顶部的 dev build marker。
- 移除 terminal header 顶部的 artifact/link chip 展示。
- 移除右侧 Inspector Artifacts 中的普通 link artifact 展示，例如 `github.com`、`chatgpt.com`。
- 保留 dev bundle 的内部身份、bundle id、窗口标题和 daemon hash 刷新能力。
- 保留 session artifact/link 数据与 inspector 后续能力，不从数据层删除。

## 3. 非目标

- 不改变 shell/PTY/daemon 行为。
- 不删除 artifact 数据采集、PR status 数据或 inspector artifact 页面。
- 不重设计整个 toolbar。
- 不改变 sidebar 的 session 列表和 New Agent 行为。

## 4. 需求

### FR-1: Dev marker 不再覆盖主界面

dev build 仍可通过窗口标题、bundle id、进程路径识别，但主界面不显示居中的 `DEV` 浮层。

### FR-2: Terminal header 不显示链接 chip

terminal header 不显示 artifact/link chip，包括普通链接 host、PR、preview port、checks/comments chip。顶部只保留会话标题、分支/host 等与当前 terminal 直接相关的信息，以及必要的 inspector/sidebar 控件。

### FR-3: 数据能力保留

`SessionRecord.artifacts`、`pull_requests`、`listening_ports` 等数据不被清空，后续 inspector 或其它入口仍可使用这些数据。

### FR-4: Inspector Artifacts 不展示普通链接

右侧 Inspector 的 Artifacts tab 不展示 `ArtifactKind::Link` 和 `ArtifactKind::Unknown`。Pull request、Linear issue、Preview 和本地端口仍保留展示。

## 5. 涉及文件

- `homie/crates/homie-app/src/root.rs`
- `homie/crates/homie-app/src/terminal_pane.rs`
- `homie/crates/homie-app/src/inspector.rs`

## 6. 验证计划

```bash
cargo fmt --check
cargo test --manifest-path homie/Cargo.toml -p homie-app dev_build
cargo test --manifest-path homie/Cargo.toml -p homie-app terminal_pane
cargo test --manifest-path homie/Cargo.toml -p homie-app inspector
HOMIE_ENGINE_PATH=... ./scripts/dev.sh
```

人工验收：

1. 顶部不再显示 `DEV · ...` 浮层。
2. terminal header 不再显示 `github.com`、`chatgpt.com` 等 link chip。
3. Inspector Artifacts 不再显示 `github.com`、`chatgpt.com` 这类普通 link artifact。
4. 新建 shell 后 `Ctrl+L` 修复仍可验证。

## 7. Beads

- Beads: `homie-2ct`
- change_id: `clean-dev-titlebar-chrome`
- 类型: bugfix
- 优先级: P1

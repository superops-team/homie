# 发布就绪报告 — terminal-state-module-split

## 变更概述

将 `homie/crates/homie-terminal-state/src/lib.rs`（约 1,168 行）机械拆分为 `mirror` / `screen` / `wire` 三个聚焦子模块，`lib.rs` 只做 re-export。公共 API 与运行时行为完全不变。

- change_id：`terminal-state-module-split`
- Beads：`homie-ubu.8`
- 类型：refactor（机械拆分，删除旧单文件，不做向后兼容）

## 模块划分

```text
homie-terminal-state/src/
├── lib.rs      mod 声明 + pub use re-export（24 行）
├── mirror.rs   GridMirror + MirrorError + validate_grid（183 行）
├── screen.rs   HeadlessScreen + ScreenSnapshot + Geometry + Collector + history（610 行）
├── wire.rs     alacritty <-> wire 单元映射（169 行）
└── tests.rs    旧内联测试迁出（192 行）
```

依赖方向：`screen → wire`（`wire_cell`/`sgr`/`emulator_cell`/`find`）；`lib → {mirror, screen}`；`mirror`/`wire` 无逆向依赖。

## 交付切片 S1–S4

| 切片 | 内容 | 状态 |
|------|------|------|
| S1 | 抽取 mirror.rs（GridMirror + MirrorError + validate_grid） | 完成 |
| S2 | 抽取 wire.rs（单元映射辅助） | 完成 |
| S3 | 抽取 screen.rs（HeadlessScreen 状态机 + 投影） | 完成 |
| S4 | 抽取 tests.rs + lib.rs re-export 收尾 | 完成 |

## 验证证据

| 验证项 | 命令 | 结果 |
|--------|------|------|
| 全 workspace 编译 | `cargo check --workspace` | 通过，0 error / 0 warning |
| 格式检查 | `cargo fmt --all --check` | 通过 |
| terminal-state 单测 | `cargo test -p homie-terminal-state` | **14 passed / 0 failed / 0 ignored** |

- 14 个 terminal-state 相关测试原样迁移到 `tests.rs`，行为等价，全部通过。
- 公共 API 兼容性：`cargo check --workspace` 编译通过，证明 `homie-engine`/`homie-remote` 等下游对 `GridMirror`/`MirrorError`/`HeadlessScreen`/`ScreenSnapshot` 的调用签名与可达性不变。

## specs/ 评估

- `homie-terminal-state` 公共 API（`GridMirror`/`MirrorError`/`HeadlessScreen`/`ScreenSnapshot`）未改变。
- `specs/engine-session-runtime.md` 只界定 `homie-engine` 的会话监督与 screen state 权威，不约束 `homie-terminal-state` 的内部模块布局；`specs/gpui-shell.md` 中的「Terminal pane」指 `homie-app/src/terminal_pane`，与本 engine 级 headless 模拟 crate 无关。
- 结论：无需更新任何 `specs/` 文档。

## 已知限制与后续

- 全 workspace 测试中，`homie-remote --test holder_e2e::environment_capture_frames_its_payload_and_scrubs_ssh_state` 因「login environment capture timed out」失败。该测试通过 `bash -l` 捕获登录环境，属环境/时序敏感，与本机械拆分无关（`homie-terminal-state` 自身 14/14 测试全绿，`cargo check --workspace` 0 警告）。
- 机械重构未改变任何运行时行为，公共 API 不变，删除旧文件、不做向后兼容。

## 结论

所有验收标准（C1–C5）均已满足，验证证据齐备，可发布。

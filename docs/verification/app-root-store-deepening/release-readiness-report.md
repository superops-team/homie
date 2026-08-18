# 发布就绪报告 — app-root-store-deepening

## 变更概述

将 `homie/crates/homie-app/src/root.rs`（约 2,130 行）机械拆分为 `root/` 下的聚焦子模块目录，把 shortcut 策略、seam 几何、auxiliary terminal 编排、render 方法下沉为职责单一的子模块；同时评估 store session/project 投影是否已收敛为单一 source of truth。公共 API 与运行时行为完全不变。

- change_id：`app-root-store-deepening`
- Beads：`homie-ubu.7`
- 类型：refactor（机械拆分，删除旧单文件，不做向后兼容）

## 模块划分

```text
root/
├── mod.rs          facade（RootView 结构体 + Focusable + 核心编排 + Render，约 1,190 行）
├── shortcuts.rs    NewSessionShortcut 枚举 + new_session_shortcut + session_navigation_delta 纯策略（41 行）
├── seams.rs        advance_seam 纯函数 + 三个拖拽边缘 marker（62 行）
├── auxiliary.rs    open_auxiliary_terminal / sync_auxiliary_terminal 编排（147 行）
├── view.rs         render 方法 + preview_control/preview_hint 渲染辅助（679 行）
└── tests.rs        旧内联测试迁出（37 行）
```

依赖方向：`auxiliary/view → mod`；`shortcuts/seams` 为纯策略无逆向依赖；`mod → {shortcuts, seams, auxiliary, view}`。

store 投影保持单点于 `store/projection.rs`（356 行），`store/mod.rs`（2,434 行）只做 `SessionStore/StoreRuntime` 编排。

## 交付切片 S1–S6

| 切片 | 内容 | 状态 |
|------|------|------|
| S1 | 抽取 root/shortcuts.rs 纯策略 | 完成 |
| S2 | 抽取 root/seams.rs seam 动画 + 拖拽边缘 marker | 完成 |
| S3 | 抽取 root/auxiliary.rs auxiliary terminal 编排 | 完成 |
| S4 | 抽取 root/view.rs render 方法 + 渲染辅助 | 完成 |
| S5 | 抽取 root/tests.rs + facade 收尾 | 完成 |
| S6 | 评估 store 投影单点化（F8） | 完成（结论：职责正交，无需改动） |

## 验证证据

| 验证项 | 命令 | 结果 |
|--------|------|------|
| 编译检查 | `cargo check -p homie-app` | 通过，0 error / 0 warning |
| 格式检查 | `cargo fmt --check` | 通过 |
| 全量测试 | `cargo test -p homie-app` | **303 passed / 0 failed / 1 ignored** |
| root 单测 | `cargo test -p homie-app root::tests` | **2 passed / 0 failed** |

- 2 个 root 相关测试（`command_t_launches_the_configured_default_agent` / `session_navigation_requires_command_option_arrows`）原样迁移到 `tests.rs`，行为等价，全部通过。
- 公共 API 兼容性：`main.rs` / `store` / 其余模块经 `cargo check -p homie-app` 编译通过，证明 `RootView`、`NewSessionShortcut`、`advance_seam` 等对外签名与可达性不变。

## store 投影单点化评估（F8）

核查结果：

- engine `registry.rs::session_project_id` 负责**稳定项目身份**：对 `root` 路径做 FNV-1a 哈希并截断 48 位，返回 `ProjectId`；远程主机按 `ssh\0{host}\0{root}` 命名空间隔离。这是身份/哈希层。
- app `store/projection.rs` 负责**UI 呈现投影**：构建 `SidebarProjection` / `SidebarProject` / `SidebarRow`，组织会话分组、折叠、置顶、rail 缩进等展示结构。这是呈现层。

结论：二者职责正交——engine 拥有「身份」，app 拥有「UI 投影」，不存在需要消除的重复投影。store session/project 投影已在 `store/projection.rs` 单点收敛，F8 无需代码改动。

## specs/gpui-shell.md 评估

- `RootView` 公共 API 未改变：仍拥有 child entity 组合、全局 action 路由、focus fallback、resize/drag shield、app service 引用与窄编排状态。
- 拆分未改变 `specs/gpui-shell.md` 的合同与 Ownership Rules；只是把「RootView 不堆积业务逻辑」落实为聚焦子模块。
- 结论：`specs/gpui-shell.md` 无需更新。

## 已知限制与后续

- 无已知限制。
- 机械重构未改变任何运行时行为，公共 API 不变，删除旧文件、不做向后兼容。
- `root/view.rs`（679 行）仍偏大，后续若 render 职责继续膨胀可按 section 再拆；当前已符合「逻辑与渲染分离」目标，不在本次范围。

## 结论

所有验收标准（C1–C7）均已满足，验证证据齐备，可发布。

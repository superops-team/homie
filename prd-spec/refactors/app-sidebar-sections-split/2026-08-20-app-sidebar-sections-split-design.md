# PRD — app-sidebar-sections-split

## 背景

`homie/crates/homie-app/src/sidebar/sections.rs`（1,202 行）是侧边栏区块渲染的 God Module：单个
`impl Sidebar` 块同时承载新 agent 行（`new_agent_row`）、顶栏（`top_bar`）、空状态（`empty_state`）、
项目区块（`project_section`）、会话行（`session_row`）、折叠箭头（`disclosure`）、归档桶
（`archived_bucket`）、归档行（`archived_row`）、更新徽标（`update_pill`）与账户页脚
（`account_footer`），单文件远超 800 行阈值，阅读与变更成本极高，违背仓库「组件模块化、关注点清晰」
原则。

## 目标

将 `sections.rs` 机械拆分为目录化聚焦子模块，按区块类型对齐关注点，公共 API 与运行时行为零变更，
引用方零改动，单文件 < 800 行。

## 非目标

- 不改变任何区块渲染逻辑、交互行为或视觉样式。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动 `Sidebar` 结构体、任何方法签名或可见性。
- 不合并或重命名方法。

## 用户场景

1. 开发者定位「顶栏/空状态」渲染逻辑时，直接进入 `chrome.rs`。
2. 开发者定位「项目区块」时，聚焦在 `project.rs`。
3. 开发者定位「会话行/折叠箭头」时，聚焦在 `session.rs`。
4. 开发者定位「归档桶/归档行」时，聚焦在 `archive.rs`。
5. 开发者定位「更新徽标/账户页脚」时，聚焦在 `footer.rs`。

## 模块划分方案

```text
sidebar/sections/
├── mod.rs             facade：use super::*; + 模块声明
├── chrome.rs          顶栏与空状态：new_agent_row/top_bar/empty_state
├── project.rs         项目区块：project_section
├── session.rs         会话行与折叠：session_row/disclosure
├── archive.rs         归档：archived_bucket/archived_row
└── footer.rs          页脚：update_pill/account_footer
```

## 可见性设计

- `sidebar/mod.rs` 中 `mod sections;` 声明不变，`crate::sidebar::sections` 模块路径不变。
- 10 个方法均为 `pub(crate) fn`，迁移后保持 `pub(crate)` 可见性不变，经 `impl Sidebar` 在子模块
  实现，crate 内调用方经 `Sidebar` 类型访问，路径零改动。
- 各子模块以 `use super::*;` 引入 `Sidebar` 与渲染依赖，`impl Sidebar` 跨子模块实现。
- 无 `pub` 可见性泄漏，无生产代码语义变更。

## 影响面

- 仅 `sidebar/sections.rs` → `sidebar/sections/` 目录化迁移，生产代码与其它模块零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo check -p homie-app --offline` 全绿。
- `cargo fmt --all --check` 通过。
- `cargo clippy -p homie-app --all-targets --offline` 0 警告。
- `cargo build -p homie-app --offline` 通过。
- `cargo test -p homie-app --offline` 全绿（沙箱内 2 个 `daemon_launch` socket bind EPERM 属预期）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（1,202 行）拆为 5 子模块 + facade。
- C2：公共 API 不变，引用方零改动（`git status` 确认仅改 `sections.rs` → `sections/` 目录 + 文档）。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：10 个方法逐字迁移，`pub(crate)` 可见性保持不变。
- C6：release readiness 证据写入 `docs/verification/app-sidebar-sections-split/`。

## Beads

- `homie-e7i`

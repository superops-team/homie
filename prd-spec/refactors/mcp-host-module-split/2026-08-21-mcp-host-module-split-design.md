# PRD — mcp-host-module-split

## 背景

`homie/crates/homie-engine/src/mcp/host.rs`（863 行）是 MCP 工具执行的注册表宿主单文件模块，
同时承载三类关注点：调用方会话谱系与关系判定（`Relation`/`relation_word`/`Lineage` 及
`new`/`record`/`children_of`/`descendants_of`/`ancestors_of`/`relation_to`/`frame`）、
注册表宿主与工具分发（`RegistryHost` 结构体 + 首个 `impl` + `impl ToolHost` 的 ~20 工具大
`match`）、以及宿主辅助方法（`require_caller`/`browser_call`/`spawn_agent`/
`spawn_agent_remote`/`wait_for`/`wait_for_children`/`logs_dir`）。
单文件超过 800 行阈值，且三类关注点彼此独立，阅读与变更成本高，违背仓库「组件模块化、关注点
清晰」原则。

## 目标

将 `host.rs` 按关注点拆分为目录化聚焦子模块，公共 API 与运行时行为零变更，引用方零改动，
单文件 < 800 行。

## 非目标

- 不改变任何 MCP 工具执行的运行时行为。
- 不新增抽象、不新增配置、不做向后兼容层。
- 不改动任何函数/结构体签名语义。
- 不合并或重命名函数、结构体或常量。

## 用户场景

1. 开发者定位「调用方会话谱系与 send_prompt 溯源」时，聚焦在 `lineage.rs`。
2. 开发者定位「注册表宿主 + 工具分发」时，聚焦在 `registry.rs`。
3. 开发者定位「MCP 宿主对外入口与常量」时，聚焦在 `mod.rs`。

## 模块划分方案

```text
mcp/host/
├── mod.rs       facade：模块文档 + 子模块声明 + `pub use` 再导出 + SESSION_ID_ENV 常量
├── lineage.rs   Relation / relation_word / Lineage + new / record / children_of /
│                descendants_of / ancestors_of / relation_to / frame
└── registry.rs  RegistryHost 结构体 + 首个 impl（new/with_holder/with_caller/registry）+
                 impl ToolHost（~20 工具 match）+ 第二个 impl（require_caller/browser_call/
                 spawn_agent/spawn_agent_remote/wait_for/wait_for_children/logs_dir）+
                 required_str/opt_strings/render_report/status_word + 4 个常量
```

## 可见性设计

- `RegistryHost` 为公共类型，经 `mod.rs` 的 `pub use` 再导出（`pub use registry::RegistryHost`），
  `mcp/mod.rs` 的 `pub use host::RegistryHost` 保持不变，公共 API 不变。
- `SESSION_ID_ENV` 为 `pub const`，仅 `host.rs` 内部使用（`inject.rs` 有独立同名常量，外部无
  `host::SESSION_ID_ENV` 引用），保留在 `mod.rs` 以维持潜在外部路径可达性。
- `Relation`/`relation_word`/`Lineage` 及其方法因被 `registry.rs` 跨模块调用，从私有提升为
  `pub(crate)`。
- `delivers_verbatim`/`descendants_of`/`ancestors_of` 仅 `lineage.rs` 内部使用，保持私有。
- `required_str`/`opt_strings`/`render_report`/`status_word` 及 4 个常量仅 `registry.rs` 使用，
  保持私有并随宿主下放到 `registry.rs`。
- 各子模块以显式 `use` 引入所需依赖。

## 影响面

- 仅 `mcp/host.rs` 的类型/函数/常量拆分为 3 个聚焦子模块，生产代码与其它模块零改动。
- 无持久化/schema/协议变更。

## 测试计划

- `cargo fmt --all --check` 通过。
- `cargo check -p homie-engine --offline` 全绿。
- `cargo clippy -p homie-engine --all-targets --offline` 0 警告。
- `cargo build -p homie-engine --offline` 通过。
- `cargo check --workspace --offline` 全绿。
- `cargo test -p homie-engine --offline` 全绿（unit 303 passed + 集成测试全绿；4 个
  socket/transport 用例在沙箱内因权限失败，非沙箱环境全绿）。
- `wc -l` 每文件 < 800 行。

## 验收标准

- C1：目录化成功，旧单文件（863 行）拆为 3 子模块 + facade。
- C2：公共 API 不变，引用方零改动。
- C3：单文件 < 800 行。
- C4：编译、fmt、clippy、test 全绿。
- C5：类型/函数/常量逐字迁移，公共可见性不变，私有辅助仅内部提升为 `pub(crate)`。
- C6：release readiness 证据写入 `docs/verification/mcp-host-module-split/`。

## Beads

- `homie-p68`

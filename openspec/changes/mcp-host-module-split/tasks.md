# OpenSpec Tasks — mcp-host-module-split

## T1 类型/函数/常量边界扫描

- [x] 定位谱系关注点（`Relation`/`relation_word`/`Lineage` + `new`/`record`/`children_of`/
  `descendants_of`/`ancestors_of`/`relation_to`/`frame`）、宿主关注点（`RegistryHost` 结构体 +
  首个 `impl` 4 方法 + `impl ToolHost` 20 工具 + 第二个 `impl` 7 方法）、辅助
  （`required_str`/`opt_strings`/`render_report`/`status_word`）+ 4 常量边界。
- 验收：全部解析，无缺失/重复。关联 C2/C5。

## T2 生成子模块

- [x] `lineage.rs`（Relation + relation_word + Lineage + 7 方法）、`registry.rs`（RegistryHost +
  工具分发 + 辅助方法 + 4 常量）。
- 验收：函数体逐字迁移，`pub` 保持 `pub`。关联 C1/C5。

## T3 重建 mod.rs + 编译验证

- [x] 移除旧 `host.rs`，新增 `mcp/host/mod.rs`（文档 + 声明 + `pub use` 再导出 + `SESSION_ID_ENV`）。
- [x] `cargo check -p homie-engine --offline` 全绿（0 警告）。
- 验收：通过。关联 C2/C4。

## T4 全量验证

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-engine --offline`
- [x] `cargo clippy -p homie-engine --all-targets --offline`
- [x] `cargo build -p homie-engine --offline`
- [x] `cargo check --workspace --offline`
- [x] `cargo test -p homie-engine --offline`（unit 303 passed + 集成测试全绿；4 个 socket/transport
  用例沙箱内权限失败，非沙箱全绿）
- 验收：全部通过。关联 C3/C4。

## T5 code review + release readiness

- [x] code review：拆分边界清晰、类型/函数逐字迁移、无行为变更、可见性正确、单文件 < 800 行。
- [x] release readiness 证据写入 `docs/verification/mcp-host-module-split/`。
- 验收：通过。关联 C6。

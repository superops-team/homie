# OpenSpec Tasks — engine-control-handlers-split

## T1 方法边界扫描

- [x] 定位 44 个方法 + `new_record` 自由函数边界，跳过 doc 注释、多行签名、嵌套闭包。
- 验收：全部解析，无缺失/重复。关联 C2/C5。

## T2 生成子模块

- [x] `agent.rs`（5 方法）、`governor.rs`（3 方法）、`handshake.rs`（2 方法）、`host.rs`（6 方法）、
  `migrate.rs`（1 方法）、`resume.rs`（5 方法）、`session.rs`（16 方法）、`spawn.rs`（5 方法）、
  `worktree.rs`（4 方法）。
- 验收：方法体逐字迁移，`pub(super)` → `pub(crate)`。关联 C1/C5。

## T3 重建 mod.rs + 编译验证

- [x] 移除旧 `handlers.rs`，新增 `handlers/mod.rs`（共享 imports + 子模块声明 + `new_record`）。
- [x] `cargo check -p homie-engine --offline` 全绿。
- 验收：通过。关联 C2/C4。

## T4 全量验证

- [x] `cargo fmt --all --check`
- [x] `cargo check -p homie-engine --offline`
- [x] `cargo clippy -p homie-engine --all-targets --offline`
- [x] `cargo build -p homie-engine --offline`
- [x] `cargo check --workspace --offline`
- [x] `cargo test -p homie-engine --offline`（303 passed / 0 failed / 3 ignored）
- 验收：全部通过。关联 C3/C4。

## T5 code review + release readiness

- [x] code review：拆分边界清晰、方法逐字迁移、无行为变更、可见性正确、单文件 < 800 行。
- [x] release readiness 证据写入 `docs/verification/engine-control-handlers-split/`。
- 验收：通过。关联 C6。

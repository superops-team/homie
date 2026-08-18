# OpenSpec 对齐校验报告 — terminal-state-module-split

## 1. 需求 → 任务映射

| PRD 需求项 | OpenSpec Task | 验证 Case |
|-----------|---------------|-----------|
| 抽取 mirror（grid mirror + 校验） | S1 | C1 |
| 抽取 wire 单元映射 | S2 | C2 |
| 抽取 screen 状态机 + 投影 | S3 | C3 |
| 抽取内联测试 + lib re-export 收尾 | S4 | C4 |
| 公共 API 与行为不变 | 全 S | C5 |

## 2. 功能验证 Case 覆盖矩阵

| Case | 描述 | 覆盖需求 | 执行命令 |
|------|------|---------|---------|
| C1 | mirror 抽取后行为等价 | S1 | `cargo test -p homie-terminal-state` |
| C2 | wire 映射等价 | S2 | `cargo test -p homie-terminal-state` |
| C3 | screen 状态机等价 | S3 | `cargo test -p homie-terminal-state` |
| C4 | 测试迁移 + lib re-export | S4 | `cargo check --workspace && cargo fmt --all --check && cargo test -p homie-terminal-state` |
| C5 | 公共 API 签名/可达性 | 全 S | `cargo check --workspace`（homie-engine/homie-remote 等下游编译通过即证） |

## 3. 一致性结论

- 每个 Task 均有明确验收标准 + 关联验证 Case。
- 无重叠、无遗漏；PRD 需求（按职责拆分、单文件不再 1,168 行、lib 仅 re-export、公共 API 与行为不变）均被 C1–C5 覆盖。
- 拆解 100% 贴合 PRD，零漏项、零错配。

## 4. 风险与缓解

- 风险：机械移动引入 `pub(crate)` 可见性/`use` 路径错误 → 缓解：每片 `cargo check` 即时反馈。
- 风险：`screen.rs` 依赖 `wire.rs` 的单元映射函数跨模块访问 → 缓解：`wire_cell`/`sgr`/`emulator_cell`/`find` 改 `pub(crate)`。
- 风险：`tests.rs` 双重嵌套 `mod tests` 导致 `super::*` 取不到类型 → 缓解：展平为单层，`use super::*` 顶层引入。

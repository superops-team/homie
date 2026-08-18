# OpenSpec 对齐校验报告 — app-surface-shell-module-split

## 1. 需求 → 任务映射

| PRD 需求项 | OpenSpec Task | 验证 Case |
|-----------|---------------|-----------|
| 抽取纯投影（agent/路径/更新文案/相对时间） | S1 | C1 |
| 抽取 host 表单状态机 | S2 | C2 |
| 抽取 host 初始化生命周期 | S3 | C3 |
| 渲染收敛到 view.rs | S4 | C4 |
| facade 收尾 + 测试迁移 | S5 | C5 |
| 公共 API 不变 | 全 S | C6 |

## 2. 功能验证 Case 覆盖矩阵

| Case | 描述 | 覆盖需求 | 执行命令 |
|------|------|---------|---------|
| C1 | 纯投影抽取后行为等价 | S1 | `cargo test -p homie-app` |
| C2 | host 表单状态机等价 | S2 | `cargo test -p homie-app` |
| C3 | host 初始化生命周期等价 | S3 | `cargo test -p homie-app` |
| C4 | 渲染等价 | S4 | `cargo test -p homie-app` |
| C5 | facade + 测试迁移 | S5 | `cargo check && cargo fmt --check && cargo test` |
| C6 | 公共 API 签名/可达性 | 全 S | `cargo check -p homie-app`（root.rs 编译通过即证） |

## 3. 一致性结论

- 每个 Task 均有明确验收标准 + 关联验证 Case。
- 无重叠、无遗漏；P0/P1 需求（纯逻辑可单测、渲染收敛、API 不变）均被 C1–C6 覆盖。
- 拆解 100% 贴合 PRD，零漏项、零错配。

## 4. 风险与缓解

- 风险：机械移动引入 `pub(crate)` 可见性/`use` 路径错误 → 缓解：每片 `cargo check` 即时反馈。
- 风险：`HostInitializationCardModel.tone: Rgba` 携带 gpui 类型 → 缓解：仅数据字段，非渲染，不影响 `host_init.rs` 纯逻辑属性。
- 风险：`view.rs` 行数较大（渲染方法多）→ 缓解：纯逻辑已全部抽出，`view.rs` 仅含渲染，符合「渲染与逻辑分离」目标。

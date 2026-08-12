# Code Review Round 1

## 1. 范围

审查对象：

- Rust Engine startup environment changes；
- `environment.refresh_path` control/client/app trigger；
- Swift daemon/holder target cleanup；
- docs/scripts 中 Swift daemon legacy 清理；
- 功能验证脚本与报告。

## 2. 显性问题检查

| 类别 | 结论 | 证据 |
|------|------|------|
| 编译 | pass | `cargo check --manifest-path homie/Cargo.toml --workspace` 通过；`swift build` 通过 |
| 格式 | pass | `cargo fmt --manifest-path homie/Cargo.toml --all -- --check` 通过 |
| 启动 eager shell | pass | Rust startup 改为 `startup_environment(app_support)`，不调用 shell capture |
| Swift daemon target | pass | `Package.swift` 已移除 daemon/holder/client/detection/git targets |
| 产品 legacy 文案 | pass | FC-08 产品范围扫描无命中 |
| 验证脚本 | pass | FC-03/FC-04 均执行通过 |

## 3. 发现与修复记录

| 问题 | 影响 | 处理 |
|------|------|------|
| `HOMIED_PATH` 作为 Rust Engine override 名称仍带旧 daemon 语义 | 容易让后续开发误解为旧 daemon fallback | 已改为 `HOMIE_ENGINE_PATH` |
| new-agent picker 每次打开都可能刷新 PATH | 这是用户触发路径，但仍可能频繁执行 shell | 已在 `environment.refresh_path` 中加入 cache TTL；重复打开复用 fresh cache |
| 产品文档仍描述 Swift daemon/porting 状态 | 与 Rust-only daemon 决策冲突 | 已删除迁移期 docs，并改写 README/CONTRIBUTING/UPDATING/ROADMAP |
| Rust 源码注释继续引用 Swift daemon as baseline | 可能被理解成 legacy 仍然是架构基线 | 已改成 reference/prior implementation 或删除具体引用 |

## 4. 覆盖矩阵复核

- FC-01/FC-02 覆盖 Rust startup 和 PATH 策略。
- FC-03/FC-04 覆盖实际启动体验与 exec probe。
- FC-05/FC-06 覆盖 Swift target cleanup 和 Swift build。
- FC-07 覆盖 Rust compile/format。
- FC-08 覆盖产品 legacy 文案和旧 target 回流。

## 5. 首轮结论

首轮显性问题已处理，无阻塞项。进入二轮隐性风险复核。

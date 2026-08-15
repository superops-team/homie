# GPUI Large Module Test Boundaries Functional Verification Report

## 1. 结论

`gpui-large-module-test-boundaries` 首阶段功能验证通过。

已完成：

- 新增 `homie/crates/homie-app/src/sidebar/picker_logic.rs`。
- 从 `sidebar/view.rs` 抽出 new-agent picker 的纯 helper：
  - remote target normalization；
  - active repo resolution decision；
  - shortcut label decision。
- 添加普通 Rust focused tests。
- 保持 GPUI render 结构和 click handlers 在 `view.rs`。

## 2. Case 执行结果

| Case | 状态 | 证据 |
|---|---|---|
| FC-01 PRD/spec 和 review 风险已收敛 | pass | `fc-01-spec-review.log` |
| FC-02 OpenSpec 三件套完整并覆盖 Case | pass | `fc-02-openspec-alignment.log` |
| FC-03 picker logic module is pure Rust | pass | `fc-03-pure-module.log` |
| FC-04 picker logic focused tests | pass | `fc-04-picker-tests.log` |
| FC-05 静态门禁和范围守卫 | pass | `fc-05-static-gates.log` |

## 3. 关键证据

- `picker_logic.rs` has no `Window|Context|Entity|cx.|div(` matches.
- `cargo test --manifest-path homie/Cargo.toml -p homie-app picker_logic -- --nocapture` passed 4 tests.
- `cargo fmt --manifest-path homie/Cargo.toml --all -- --check` passed.
- `git diff --check` passed.

## 4. 残余风险

- 本切片只抽一个 helper module，没有覆盖 terminal/inspector/root 的纯逻辑拆分。
- 没有跑真实 app visual/manual regression；本切片不改变 GPUI tree，只移动 pure helper logic。

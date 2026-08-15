# GPUI 大模块纯逻辑边界功能验证 Case

## 1. 验证目标

本验证 Case 面向 `gpui-large-module-test-boundaries` 首阶段，目标是证明：

- 只抽一个行为单元：Sidebar new-agent picker 纯 helper；
- 新 helper 不依赖 GPUI `Window/Context/Entity/cx.` 或 element 构造；
- 行为由普通 Rust focused tests 覆盖；
- 不重复修改已完成 GPUI child slices；
- UI render 结构保持不变，只调用 helper。

## FC-01: PRD/spec 和 review 风险已收敛

```bash
test -s docs/verification/gpui-large-module-test-boundaries/spec-review-report.md
rg -n "首阶段关闭口径|Sidebar new-agent picker|不重复|纯逻辑|Window|Context|Entity" \
  prd-spec/refactors/gpui-large-module-test-boundaries/2026-08-13-gpui-large-module-test-boundaries-design.md \
  docs/verification/gpui-large-module-test-boundaries/spec-review-report.md
```

证据路径：`docs/verification/gpui-large-module-test-boundaries/fc-01-spec-review.log`

## FC-02: OpenSpec 三件套完整并覆盖 Case

```bash
test -s openspec/changes/gpui-large-module-test-boundaries/plan.md
test -s openspec/changes/gpui-large-module-test-boundaries/tasks.md
test -s openspec/changes/gpui-large-module-test-boundaries/alignment-report.md
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05" \
  openspec/changes/gpui-large-module-test-boundaries/tasks.md \
  openspec/changes/gpui-large-module-test-boundaries/alignment-report.md
```

证据路径：`docs/verification/gpui-large-module-test-boundaries/fc-02-openspec-alignment.log`

## FC-03: picker logic module is pure Rust

```bash
test -s homie/crates/homie-app/src/sidebar/picker_logic.rs
if rg -n "Window|Context|Entity|cx\.|div\(" homie/crates/homie-app/src/sidebar/picker_logic.rs; then
  exit 1
fi
echo "picker_logic.rs has no GPUI context/entity/window/element dependency"
```

证据路径：`docs/verification/gpui-large-module-test-boundaries/fc-03-pure-module.log`

## FC-04: picker logic focused tests

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app picker_logic -- --nocapture
```

通过标准：覆盖 remote/local target resolution、repo-preservation decision、shortcut label decision。

证据路径：`docs/verification/gpui-large-module-test-boundaries/fc-04-picker-tests.log`

## FC-05: 静态门禁和范围守卫

```bash
bash -n scripts/*.sh homie/scripts/*.sh
cargo fmt --manifest-path homie/Cargo.toml --all -- --check
git diff --check
git diff --name-only -- homie/crates/homie-app/src/sidebar
git status --short -- homie/crates/homie-app/src/sidebar
```

通过标准：只显示 sidebar 模块内预期文件。

证据路径：`docs/verification/gpui-large-module-test-boundaries/fc-05-static-gates.log`

# GPUI UtilitySurfaces 首个生命周期切片功能验证 Case

## 1. 验证范围

本验证只覆盖 `homie-yon` / `gpui-utility-surfaces-first-slice`：

- History load/resume task ownership；
- Worktrees refresh/cleanup task ownership；
- generation guard；
- close/dismiss 后旧结果不回写；
- targeted tests 和静态门禁。

## 2. Case 清单

### FC-01: PRD/spec 明确最小切片边界

- 覆盖需求：PRD 1.2、1.3、7。
- 执行命令：

```bash
rg -n "不拆分|不重构 `RootView`|不新增 `homie-ui`|generation|history_task|worktrees_task" prd-spec/refactors/gpui-utility-surfaces-first-slice/2026-08-14-gpui-utility-surfaces-first-slice-design.md
```

- 预期结果：命令退出码 0，输出包含非目标和 task/generation 要求。
- 证据路径：`docs/verification/gpui-utility-surfaces-first-slice/fc-01-prd-scope.log`

### FC-02: OpenSpec 任务与 Case 对齐

- 覆盖需求：OpenSpec 拆解。
- 执行命令：

```bash
test -s openspec/changes/gpui-utility-surfaces-first-slice/plan.md
test -s openspec/changes/gpui-utility-surfaces-first-slice/tasks.md
test -s openspec/changes/gpui-utility-surfaces-first-slice/alignment-report.md
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06|FC-07|FC-08" openspec/changes/gpui-utility-surfaces-first-slice/tasks.md openspec/changes/gpui-utility-surfaces-first-slice/alignment-report.md
```

- 预期结果：OpenSpec 文件存在，且引用所有功能验证 Case。
- 证据路径：`docs/verification/gpui-utility-surfaces-first-slice/fc-02-openspec-alignment.log`

### FC-03: UtilitySurfaces 持有 History/Worktrees lifecycle tasks

- 覆盖需求：PRD 3.1、3.3。
- 执行命令：

```bash
rg -n "history_generation|history_task|worktrees_generation|worktrees_task" homie/crates/homie-app/src/surface_shell.rs
```

- 预期结果：命令退出码 0，输出包含四个字段和相关使用点。
- 证据路径：`docs/verification/gpui-utility-surfaces-first-slice/fc-03-task-fields.log`

### FC-04: clean worktree 可读取 homie-ui 图标 assets

- 覆盖需求：PRD 1.1、1.2、7.7。
- 执行命令：

```bash
git check-ignore -v homie/crates/homie-ui/assets/icons/activity.svg || true
git status --short -- homie/crates/homie-ui/assets/icons | wc -l
test -s homie/crates/homie-ui/assets/icons/activity.svg
```

- 预期结果：`git check-ignore` 不再显示 `Icon?` 忽略命中；图标 assets 在当前变更中可追踪；`activity.svg` 存在且非空。
- 证据路径：`docs/verification/gpui-utility-surfaces-first-slice/fc-04-icon-assets.log`

### FC-05: History/Worktrees 操作不再直接 detach lifecycle task

- 覆盖需求：PRD 3.3、7.1。
- 执行命令：

```bash
rg -n "open_history|refresh_worktrees|resume_history|confirm_cleanup|\\.detach\\(\\)" homie/crates/homie-app/src/surface_shell.rs
```

- 预期结果：History/Worktrees lifecycle path 不再出现 `cx.spawn(...).detach()`；其他明确 app/service lifetime 的 detach 不属于本 Case。
- 证据路径：`docs/verification/gpui-utility-surfaces-first-slice/fc-05-no-lifecycle-detach.log`

### FC-06: stale History result 不覆盖 newer state

- 覆盖需求：PRD 3.2、7.2。
- 执行命令：

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app stale_history_result -- --nocapture
```

- 预期结果：测试通过。
- 证据路径：`docs/verification/gpui-utility-surfaces-first-slice/fc-06-stale-history.log`

### FC-07: stale Worktrees result 不覆盖 newer state

- 覆盖需求：PRD 3.2、7.2。
- 执行命令：

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app stale_worktrees_result -- --nocapture
```

- 预期结果：测试通过。
- 证据路径：`docs/verification/gpui-utility-surfaces-first-slice/fc-07-stale-worktrees.log`

### FC-08: 关闭 History/Worktrees 后旧结果不回写可见 surface

- 覆盖需求：PRD 3.2、3.3、7.3。
- 执行命令：

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app closed_utility_surface_ignores_late_results -- --nocapture
```

- 预期结果：测试通过。
- 证据路径：`docs/verification/gpui-utility-surfaces-first-slice/fc-08-close-guard.log`

### FC-09: targeted utility surfaces 测试和静态门禁通过

- 覆盖需求：PRD 6。
- 执行命令：

```bash
(cd homie && cargo fmt --check)
cargo test --manifest-path homie/Cargo.toml -p homie-app surface_shell::tests -- --nocapture
git diff --check
git diff --name-only -- homie/crates/homie-ui/src homie/crates/homie-engine homie/crates/homie-client
```

- 预期结果：前三个命令退出码 0；最后一个命令无输出。`homie/crates/homie-ui/assets/icons` 是本轮前置构建阻塞修复，允许出现在 diff 中。
- 证据路径：`docs/verification/gpui-utility-surfaces-first-slice/fc-09-targeted-gates.log`

## 3. 覆盖矩阵

| PRD/Spec 需求 | Case |
|---------------|------|
| 最小切片边界 | FC-01 |
| OpenSpec 对齐 | FC-02 |
| clean worktree 图标 assets | FC-04 |
| task 字段持有 | FC-03, FC-05 |
| generation guard | FC-06, FC-07 |
| close/dismiss 后不回写 | FC-08 |
| targeted tests / fmt / scope | FC-09 |

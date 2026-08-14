# GPUI UtilitySurfaces 首个生命周期切片功能验证报告

## 1. 结论

`gpui-utility-surfaces-first-slice` 功能验证通过。本轮完成：

- 修正 clean worktree 缺失 `homie-ui` SVG assets 的编译阻塞；
- 为 `UtilitySurfaces` 的 History/Worktrees 操作增加 task 字段持有；
- 为 History/Worktrees 操作增加 generation guard；
- 关闭 History/Worktrees surface 时清理对应 lifecycle task；
- 增加 stale result 和 close guard focused tests。

## 2. Case 结果

| Case | 目标 | 状态 | 证据 |
|------|------|------|------|
| FC-01 | PRD/spec 明确最小切片边界 | pass | `fc-01-prd-scope.log` |
| FC-02 | OpenSpec 任务与 Case 对齐 | pass | `fc-02-openspec-alignment.log` |
| FC-03 | UtilitySurfaces 持有 History/Worktrees lifecycle tasks | pass | `fc-03-task-fields.log` |
| FC-04 | clean worktree 可读取 homie-ui 图标 assets | pass | `fc-04-icon-assets.log` |
| FC-05 | History/Worktrees 操作不再直接 detach lifecycle task | pass | `fc-05-no-lifecycle-detach.log` |
| FC-06 | stale History result 不覆盖 newer state | pass | `fc-06-stale-history.log` |
| FC-07 | stale Worktrees result 不覆盖 newer state | pass | `fc-07-stale-worktrees.log` |
| FC-08 | 关闭 History/Worktrees 后旧结果不回写可见 surface | pass | `fc-08-close-guard.log` |
| FC-09 | targeted tests 和静态门禁 | pass | `fc-09-targeted-gates.log` |

## 3. 说明

FC-05 的扫描日志仍会显示 `prepare_host` 中的 `.detach()`。该路径属于
Settings/Remote Host initialization，并且已有 operation guard；它不在本切片范围内。
本切片覆盖的 `open_history`、`resume_history`、`refresh_worktrees`、`confirm_cleanup`
已改为字段持有 task。

## 4. 未覆盖范围

- RootView 拆分未执行。
- UtilitySurfaces 文件拆分未执行。
- homie-ui semantic primitives 未实现。
- render projection 外移未执行。
- 视觉平台矩阵未执行。

这些均由 parent program 的其他 child beads 追踪。

# GPUI 视觉与平台偏好验证门禁设计文档

## 1. 概述

### 1.1 问题

Homie GPUI 视觉/交互变更需要可复用的验证矩阵，覆盖 preview scenario、light/dark、reduced motion、keyboard-only 和真实 app launch。当前缺少统一入口，容易把 `cargo check` 当作视觉证明。

### 1.2 目标

1. 新增视觉验证 runbook。
2. 新增 `homie/scripts/visual-gate.sh` 作为统一入口。
3. 支持 `--dry-run` 输出应执行的 preview/app launch 命令，便于 CI 或无 GUI 环境验证门禁计划。
4. 不直接修改 GPUI UI 行为。

### 1.3 非目标

- 不引入截图 diff 工具。
- 不强制当前环境打开 GUI。
- 不改变 `dev.sh` 构建行为。

## 2. 验证

```bash
homie/scripts/visual-gate.sh --dry-run
homie/scripts/visual-gate.sh --dry-run --scenario stress --appearance dark --reduced-motion
bash -n homie/scripts/visual-gate.sh
git diff --check
```

## 3. Beads

- Beads: `homie-mpc`
- change_id: `gpui-visual-platform-gates`

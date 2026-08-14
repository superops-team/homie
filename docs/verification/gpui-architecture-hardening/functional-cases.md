# GPUI 架构硬化第一阶段功能验证 Case

## 1. 验证范围

本验证 Case 只覆盖 `homie-4lu` 第一阶段：合同基线、证据 inventory、
OpenSpec 拆解、child beads 和 worktree shared target 规则。Phase 2-5 的 GPUI
代码重构不在本轮功能验证范围内。

## 2. Case 清单

### FC-01: PRD/spec 已收敛为 program + Phase 0/1 关闭边界

- 覆盖需求：PRD 1.2、1.3、1.4、2.4、7.1、7.2。
- 前置环境：当前分支 `dev/gpui-architecture-hardening-prd`。
- 执行命令：

```bash
rg -n "program-level|Phase 0/1|第一阶段关闭条件|child beads|Program-level 完成条件" prd-spec/refactors/gpui-architecture-hardening/2026-08-14-gpui-architecture-hardening-design.md
```

- 预期结果：命令退出码为 0，输出包含 program-level 定位、Phase 0/1 关闭边界、child beads 和 Program-level 完成条件。
- 通过标准：PRD 不再将 Phase 2-5 代码整改作为 `homie-4lu` 关闭条件。
- 证据路径：`docs/verification/gpui-architecture-hardening/fc-01-prd-scope.log`
- 失败处理：回到 PRD 修改，补齐边界描述。

### FC-02: `AGENTS.md` worktree shared target 规则存在

- 覆盖需求：PRD 3.2、7.2.1。
- 执行命令：

```bash
rg -n "Worktree Build Cache Rules|homie-worktrees/.shared/homie-target|ln -s" AGENTS.md
```

- 预期结果：命令退出码为 0，输出包含规则标题、共享 target 路径和 symlink 命令。
- 通过标准：规则在 `AGENTS.md` 中存在；PRD 只引用 `AGENTS.md` 为权威入口。
- 证据路径：`docs/verification/gpui-architecture-hardening/fc-02-agents-worktree-cache.log`
- 失败处理：补齐 `AGENTS.md` 规则或 PRD 引用。

### FC-03: `AGENTS.md` 要求读取的 docs 已补齐

- 覆盖需求：PRD 3.1、7.2.2。
- 执行命令：

```bash
test -s docs/architecture/project-layout.md
test -s docs/development/standards.md
test -s docs/development/quality-gates.md
test -s docs/research/rust-package-selection.md
```

- 预期结果：所有命令退出码为 0。
- 通过标准：四个 docs 文件存在且非空。
- 证据路径：`docs/verification/gpui-architecture-hardening/fc-03-required-docs.log`
- 失败处理：补齐缺失文档。

### FC-04: GPUI 长期 component specs 已补齐

- 覆盖需求：PRD 3.1、7.2.3。
- 执行命令：

```bash
test -s specs/gpui-shell.md
test -s specs/gpui-interaction-contract.md
test -s specs/ui-components.md
rg -n "RootView|UtilitySurfaces|task|subscription|render|a11y|stable id|Button|ListRow|Dialog" specs/gpui-shell.md specs/gpui-interaction-contract.md specs/ui-components.md
```

- 预期结果：文件存在且关键词扫描退出码为 0。
- 通过标准：三个 specs 覆盖 shell、interaction、component contract。
- 证据路径：`docs/verification/gpui-architecture-hardening/fc-04-gpui-specs.log`
- 失败处理：补齐 specs 内容。

### FC-05: review inventory 使用结构化 schema 并覆盖 P1-P10

- 覆盖需求：PRD Phase 0、7.2.4。
- 执行命令：

```bash
test -s docs/verification/gpui-architecture-hardening/review-inventory.md
rg -n "finding_id|current_symptom|target_contract|owner_task|verification_command|evidence_path" docs/verification/gpui-architecture-hardening/review-inventory.md
for id in P1 P2 P3 P4 P5 P6 P7 P8 P9 P10; do rg -n "^\\| $id \\|" docs/verification/gpui-architecture-hardening/review-inventory.md; done
```

- 预期结果：所有命令退出码为 0。
- 通过标准：inventory 表格包含规定列，且 P1-P10 均有一行。
- 证据路径：`docs/verification/gpui-architecture-hardening/fc-05-review-inventory.log`
- 失败处理：补齐 inventory。

### FC-06: OpenSpec plan/tasks/alignment 已创建并关联功能验证 Case

- 覆盖需求：PRD 1.3、4、6.1、7.2.5。
- 执行命令：

```bash
test -s openspec/changes/gpui-architecture-hardening/plan.md
test -s openspec/changes/gpui-architecture-hardening/tasks.md
test -s openspec/changes/gpui-architecture-hardening/alignment-report.md
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06|FC-07|FC-08" openspec/changes/gpui-architecture-hardening/tasks.md openspec/changes/gpui-architecture-hardening/alignment-report.md
```

- 预期结果：文件存在且功能验证 Case 被引用。
- 通过标准：每个 OpenSpec task 都有验收标准和 Case 映射，alignment report 无未覆盖 P1 项。
- 证据路径：`docs/verification/gpui-architecture-hardening/fc-06-openspec-alignment.log`
- 失败处理：补齐 OpenSpec 或对齐报告。

### FC-07: 后续 child beads 已创建并链接到 `homie-4lu`

- 覆盖需求：PRD Phase 1、7.2.6。
- 执行命令：

```bash
bd children homie-4lu
bd list --json | rg "gpui-lifecycle-task-ownership|gpui-utility-surfaces-first-slice|gpui-ui-primitives-a11y|gpui-render-path-purity|gpui-visual-platform-gates"
```

- 预期结果：命令退出码为 0，输出包含五个 child change。
- 通过标准：至少五个 child beads 存在，并通过 parent-child 或 dependency 关系关联到 `homie-4lu`。
- 证据路径：`docs/verification/gpui-architecture-hardening/fc-07-child-beads.log`
- 失败处理：创建缺失 child beads 并建立依赖关系。

### FC-08: Active Homie worktrees 共享同一个 Cargo target

- 覆盖需求：PRD 3.2、6.5、7.2.7。
- 执行命令：

```bash
git worktree list --porcelain
for wt in /Users/bytedance/workspace/github/homie /Users/bytedance/workspace/github/homie-gpui-audit-20260814 /Users/bytedance/workspace/github/homie-worktrees/diri-agent-session-runtime /Users/bytedance/workspace/github/homie-worktrees/diri-sidebar-core-slice /Users/bytedance/workspace/github/homie-worktrees/diri-storage-core-facts; do realpath "$wt/homie/target"; done | sort -u
git status --short --ignored=matching homie/target
```

- 预期结果：`realpath` 去重后只有 `/Users/bytedance/workspace/github/homie-worktrees/.shared/homie-target`；`git status` 将 `homie/target` 显示为 ignored。
- 通过标准：active worktrees 共享同一个 target，symlink 未进入 tracked files。
- 证据路径：`docs/verification/gpui-architecture-hardening/fc-08-worktree-target.log`
- 失败处理：修复缺失 symlink 或本地 exclude。

### FC-09: 当前 change 未包含 Phase 2-5 GPUI 代码重构

- 覆盖需求：PRD 1.4、7.2。
- 执行命令：

```bash
git diff --name-only -- homie/crates/homie-app homie/crates/homie-ui
```

- 预期结果：命令无输出。
- 通过标准：当前闭环没有修改 GPUI app/runtime 代码。
- 证据路径：`docs/verification/gpui-architecture-hardening/fc-09-no-code-scope-creep.log`
- 失败处理：移除越界代码改动，或新建 child change 承接。

### FC-10: 静态文档质量检查

- 覆盖需求：PRD 6.2。
- 执行命令：

```bash
git diff --check
```

- 预期结果：退出码为 0。
- 通过标准：没有 trailing whitespace 或 diff 格式问题。
- 证据路径：`docs/verification/gpui-architecture-hardening/fc-10-diff-check.log`
- 失败处理：修复空白或格式问题。

## 3. 覆盖矩阵

| PRD/Spec 需求 | 验证 Case |
|---------------|-----------|
| program-level 定位和 Phase 0/1 边界 | FC-01 |
| Worktree Build Cache Rules | FC-02, FC-08 |
| 必读 docs 补齐 | FC-03 |
| GPUI specs 补齐 | FC-04 |
| review inventory schema | FC-05 |
| OpenSpec 拆解和对齐 | FC-06 |
| child beads 追踪 | FC-07 |
| 当前 change 不越界实现 Phase 2-5 | FC-09 |
| 静态质量 | FC-10 |

## 4. 执行计划

按 FC-01 到 FC-10 顺序执行。FC-03、FC-04、FC-05、FC-06、FC-07 在对应产物生成后执行；FC-10 作为最终静态门禁执行。

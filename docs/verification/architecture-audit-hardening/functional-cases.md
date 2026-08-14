# Brooks 架构审计治理功能验证 Case

## 1. 执行范围

- change_id: `architecture-audit-hardening`
- Beads: `homie-om7`
- 范围：Phase 0 planning loop，不包含 Phase 1-4 代码重构。
- 验证目标：证明 PRD/spec 已修复 review 问题，且后续 OpenSpec 可从文档直接执行。

## 2. Case 清单

### FC-01: Brooks findings 全部可追踪

**命令：**

```bash
rg -n "2\\.1|2\\.2|2\\.3|2\\.4|Symptom|Source|Consequence|Remedy" \
  prd-spec/refactors/architecture-audit-hardening/2026-08-14-architecture-audit-hardening-design.md
```

**通过标准：**

- 四个 Brooks finding 均存在；
- 每个 finding 保留 Symptom / Source / Consequence / Remedy。

### FC-02: parent PRD 与 child Beads 边界清晰

**命令：**

```bash
rg -n "关闭口径|homie-om7|不直接承诺|child Beads|Phase 1-4" \
  prd-spec/refactors/architecture-audit-hardening/2026-08-14-architecture-audit-hardening-design.md
```

**通过标准：**

- 明确 `homie-om7` 只关闭 Phase 0；
- 明确 Phase 1-4 需要 child Beads。

### FC-03: 存量 PRD/spec 关系明确

**命令：**

```bash
rg -n "gpui-architecture-hardening|gpui-large-module-test-boundaries|protocol-contract-golden-fixtures|specs/gpui-shell|specs/engine-session-runtime" \
  prd-spec/refactors/architecture-audit-hardening/2026-08-14-architecture-audit-hardening-design.md
```

**通过标准：**

- PRD 显式说明复用、补充或约束关系；
- Phase 4 明确复用 protocol fixtures PRD。

### FC-04: 功能验证 Case 覆盖完整

**命令：**

```bash
rg -n "FC-01|FC-02|FC-03|FC-04|FC-05|FC-06|FC-07|FC-08" \
  prd-spec/refactors/architecture-audit-hardening/2026-08-14-architecture-audit-hardening-design.md \
  docs/verification/architecture-audit-hardening/functional-cases.md
```

**通过标准：**

- PRD 与本 Case 文档均包含 FC-01 到 FC-08。

### FC-05: OpenSpec 结构完整

**命令：**

```bash
test -s openspec/changes/architecture-audit-hardening/plan.md
test -s openspec/changes/architecture-audit-hardening/tasks.md
test -s openspec/changes/architecture-audit-hardening/alignment-report.md
```

**通过标准：**

- OpenSpec 三个文件均存在且非空。

### FC-06: OpenSpec 覆盖 Brooks finding 与 Case

**命令：**

```bash
rg -n "Finding|2\\.1|2\\.2|2\\.3|2\\.4|FC-01|FC-02|FC-03|FC-04|FC-05|FC-06|FC-07|FC-08" \
  openspec/changes/architecture-audit-hardening/plan.md \
  openspec/changes/architecture-audit-hardening/tasks.md \
  openspec/changes/architecture-audit-hardening/alignment-report.md
```

**通过标准：**

- OpenSpec 明确映射四个 finding；
- OpenSpec 任务引用 FC-01 到 FC-08。

### FC-07: 文档静态门禁

**命令：**

```bash
git diff --check \
  prd-spec/refactors/architecture-audit-hardening/2026-08-14-architecture-audit-hardening-design.md \
  docs/verification/architecture-audit-hardening \
  openspec/changes/architecture-audit-hardening
```

**通过标准：**

- 无 trailing whitespace；
- 无 conflict marker。

### FC-08: Phase 0 不包含代码实现

**命令：**

```bash
git diff --name-only -- \
  homie/crates \
  Sources \
  Tests
```

**通过标准：**

- 输出为空；本轮只交付 PRD/OpenSpec/evidence。

## 3. 覆盖矩阵

| PRD 要求 | Case |
|----------|------|
| Brooks findings 可追踪 | FC-01 |
| parent PRD 与 child Beads 边界清晰 | FC-02 |
| 存量 PRD/spec 关系明确 | FC-03 |
| 功能验证 Case 完整 | FC-04 |
| OpenSpec 结构完整 | FC-05 |
| OpenSpec 覆盖 finding 和 Case | FC-06 |
| 文档静态门禁 | FC-07 |
| Phase 0 不包含代码实现 | FC-08 |

## 4. 执行计划

执行顺序：FC-01 → FC-02 → FC-03 → FC-04 → FC-05 → FC-06 → FC-07 → FC-08。

失败处理：

- FC-01 到 FC-04 失败：回到 PRD 修订。
- FC-05 到 FC-06 失败：回到 OpenSpec 修订。
- FC-07 失败：修复格式。
- FC-08 失败：移除非 Phase 0 范围代码改动，或单独创建 child Bead。

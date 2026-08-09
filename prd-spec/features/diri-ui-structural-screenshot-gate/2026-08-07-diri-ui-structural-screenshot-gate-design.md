# Diri UI Structural Screenshot Gate 设计文档

```yaml
change_id: diri-ui-structural-screenshot-gate
beads: homie-nnp
target_rows:
  - UI-001
  - UI-002
  - UI-003
  - UI-004
  - UI-005
  - UI-006
  - UI-007
  - UI-008
  - UI-009
```

## 1. 背景

`docs/verification/diri-ui-screenshot/visual-verification-report.md` 已保存 Diri reference screenshot 和 Homie screenshot，并且 `make ui-screenshot-gate` 能拒绝黑屏/空 PNG。但当前 gate 只证明“截图存在且非空”，没有证明 Homie 与 Diri 都呈现工作台结构，也没有自动检查 sidebar、terminal/workbench、inspector 三栏结构。

这会让后续 UI parity 继续存在一个风险：单张 Homie 截图可能被误当成 Diri side-by-side parity 证据。

## 2. 目标

- 将 `make ui-screenshot-gate` 升级为结构化截图门禁。
- 对 Homie 和 Diri PNG 同时解码，计算稳定结构指标。
- 验证三栏工作台结构：left sidebar、center terminal/workbench、right inspector 之间存在亮度/边界差异。
- 验证截图报告明确记录 structural comparison gate，避免只保留人工描述。
- 本阶段只推进 UI parity 证据门禁，不把 UI-001..UI-009 标为 implemented。

## 3. 非目标

- 不做逐像素视觉 diff。
- 不要求 Homie 和 Diri 截图分辨率一致。
- 不替代真实 GPUI 交互 E2E。
- 不修改产品 UI 布局、颜色或交互逻辑。

## 4. 用户场景

### 场景 1: 拒绝静态或错误截图

**Given** Homie 截图为空、黑屏、尺寸过小或缺少 Diri reference  
**When** 执行 `make ui-screenshot-gate`  
**Then** gate 必须失败，并输出具体缺失或结构指标失败原因。

### 场景 2: 证明 Diri/Homie 都是工作台结构

**Given** Homie 和 Diri 截图均存在  
**When** 执行 `make ui-screenshot-gate`  
**Then** gate 必须验证两张截图都具有 left/center/right 分区差异，以及能代表 sidebar/inspector 分隔线的 vertical edge peaks。

### 场景 3: 报告可审计

**Given** structural gate 已执行  
**When** 查看 visual verification report  
**Then** 报告必须包含 structural comparison gate、Homie metrics、Diri metrics、side-by-side limitations 和后续 E2E 缺口。

## 5. 需求

| ID | 需求 | 优先级 |
|----|------|--------|
| FR-1 | PNG decoder 必须解析 IHDR/IDAT，并恢复 scanline filters，不能只看文件大小。 | P0 |
| FR-2 | 对每张截图输出 width、height、nonzero raw bytes、left/center/right luma、luma gaps、vertical edge peaks。 | P0 |
| FR-3 | Homie 与 Diri 均需通过尺寸、非空、三栏亮度差、边界峰值数量门禁。 | P0 |
| FR-4 | visual report 必须包含 structural gate 的执行说明和指标摘要。 | P0 |
| FR-5 | gate 失败时必须给出具体 label 和指标值，便于修复截图或 UI。 | P1 |

## 6. 影响文件

- `scripts/quality/check-ui-screenshot-evidence.py`
- `docs/verification/diri-ui-screenshot/visual-verification-report.md`
- `docs/verification/diri-ui-structural-screenshot-gate/`
- `openspec/changes/diri-ui-structural-screenshot-gate/`

## 7. 功能验证

- `make ui-screenshot-gate`
- `python3 scripts/quality/check-ui-screenshot-evidence.py`
- `git diff --check -- scripts/quality/check-ui-screenshot-evidence.py docs/verification/diri-ui-screenshot docs/verification/diri-ui-structural-screenshot-gate prd-spec/features/diri-ui-structural-screenshot-gate openspec/changes/diri-ui-structural-screenshot-gate`
- `make parity-lock`

## 8. 验收标准

- `make ui-screenshot-gate` 通过并打印 Homie/Diri structural screenshot metrics。
- `visual-verification-report.md` 记录 structural comparison gate，而不是只说明单张截图非空。
- release readiness 明确标注 UI-001..UI-009 仍为 `partial`，真实交互 E2E 仍是后续任务。

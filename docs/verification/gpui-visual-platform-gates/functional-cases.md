# GPUI 视觉平台门禁功能验证 Case

## FC-01: PRD 和 runbook 存在

```bash
test -s prd-spec/refactors/gpui-visual-platform-gates/2026-08-14-gpui-visual-platform-gates-design.md
test -s docs/qa/gpui-visual-platform-gates.md
```

## FC-02: visual-gate dry-run 输出默认命令

```bash
homie/scripts/visual-gate.sh --dry-run
```

## FC-03: visual-gate 支持 stress/dark/reduced-motion/settings

```bash
homie/scripts/visual-gate.sh --dry-run --scenario stress --appearance dark --reduced-motion --settings remote
```

## FC-04: 脚本语法和静态门禁

```bash
bash -n homie/scripts/visual-gate.sh
git diff --check
```

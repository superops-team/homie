# Bingo Spec Review Task

你是 Bingo，请基于 review-spec 技能的标准，对 Homie 当前每个 `specs/*/README.md` 功能模块逐个做只读审查。

严格要求：

- 不要修改任何文件。
- 不要执行代码实现。
- 逐个 review 每个组件 spec。
- 审查是否能支撑 `docs/research/diri-module-inventory.md` 中对应 Diri 模块的真实落地。
- 按 review-spec 维度输出：上下文一致性、落地性、语义清晰性、SDD/TDD 适配、最小实现、存量影响、运行风险、可扩展性、架构一致性。
- 重点指出 P0/P1 问题，尤其是会导致“看起来对齐但实际没有对齐 Diri”的问题。
- 给出每个 spec 的整改建议、需要补的 PRD/OpenSpec/验证 case。

请输出 Markdown，结构如下：

```markdown
# Bingo Component Spec Review

## Overall Verdict
- 可行性：
- 最大风险：
- 推荐方向：

## Per-Spec Findings

### <spec path>
| 优先级 | 维度 | 问题 | 影响 | 整改建议 |
|---|---|---|---|---|

## Cross-Spec Gaps
| 优先级 | 问题 | 涉及 spec | 整改建议 |
|---|---|---|---|

## Implementation Readiness
| spec | 是否可直接进入 PRD/OpenSpec | 前置补充 |
|---|---|---|

## Recommended Next Actions
```


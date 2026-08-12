# startup-background-shell-invisibility 对齐报告

## 1. 对齐输入

- PRD：`prd-spec/bugfixes/startup-background-shell-invisibility/2026-08-12-startup-background-shell-invisibility-design.md`
- Spec review：`docs/verification/startup-background-shell-invisibility/spec-review-report.md`
- 功能验证 Case：`docs/verification/startup-background-shell-invisibility/functional-cases.md`
- OpenSpec Plan：`openspec/changes/startup-background-shell-invisibility/plan.md`
- OpenSpec Tasks：`openspec/changes/startup-background-shell-invisibility/tasks.md`
- Beads：`homie-f21`

## 2. PRD 需求到 Case 与 Task 映射

| PRD 需求 | 功能验证 Case | OpenSpec Tasks | 对齐状态 |
|----------|----------------|----------------|----------|
| FR-1 启动首帧前禁止交互 login shell | FC-01、FC-03、FC-04 | T1、T2、T3、T9、T10 | 已覆盖 |
| FR-2 PATH 捕获 lazy/cached | FC-01、FC-02、FC-03 | T1、T2、T3、T4、T10 | 已覆盖 |
| FR-3 后台任务统一分级 | FC-04 | T5、T9、T10 | 已覆盖 |
| FR-4 shell/exec 静默契约 | FC-01、FC-03、FC-04 | T1、T3、T5、T9、T10 | 已覆盖 |
| FR-5 readiness 不阻塞启动 | FC-02、FC-04 | T3、T4、T10 | 已覆盖 |
| FR-6 remote/browser 按需启动 | FC-04 | T5、T9、T10 | 已覆盖 |
| FR-7 可观测但不打扰 | FC-02、FC-03、FC-04 | T3、T5、T9、T10、T12 | 已覆盖 |
| FR-8 Rust 唯一 daemon/supervisord | FC-01、FC-05、FC-06、FC-08 | T1、T6、T7、T8、T10、T12 | 已覆盖 |
| FR-9 删除 Swift daemon legacy | FC-05、FC-06、FC-08 | T6、T7、T8、T10、T12 | 已覆盖 |

## 3. Spec Review 整改对齐

| Review Finding | 整改位置 | OpenSpec 覆盖 |
|----------------|----------|---------------|
| Swift daemon legacy 不能保留 | PRD FR-8/FR-9、架构决策 | T6、T7、T8 |
| 范围需要分阶段 | PRD 方案设计、review report | Plan P0-A/P0-B/P1 |
| README/CONTRIBUTING/Package.swift 仍声明 Swift daemon | PRD FR-9、影响范围、验收标准 | T8、FC-08 |
| 启动无感必须可测 | 功能验证 Case FC-03/FC-04 | T9、T10 |
| PATH refresh 策略待确认 | PRD Open Questions | T3 中保留默认非交互 lazy 策略 |

## 4. 待确认问题处理

### Q1: 是否允许首次打开 agent picker 执行非交互 PATH refresh？

当前 OpenSpec 默认策略：允许 lazy、非交互 `shell -l -c 'printenv PATH'`，禁止启动阶段执行，禁止 `-i`，必须 timeout/cancel/log。若用户后续要求完全禁用 shell PATH refresh，T3 可收敛为纯 fallback/cache/manual path。

### Q2: interactive rc 用户如何处理？

当前 OpenSpec 默认策略：首版显示检测中/不可用 + 可操作错误/刷新入口，不在 P0 内做复杂授权 UI。显式授权按钮可作为 P1 诊断/设置增强。

## 5. 门禁结论

- PRD 每个 P0/P1 需求均至少有一个功能验证 Case 覆盖。
- 每个 OpenSpec task 均有明确验收与关联 Case。
- Swift daemon 删除不作为兼容迁移，而是架构清理任务。
- 可以进入 Step 5 SDD/TDD 实现阶段。

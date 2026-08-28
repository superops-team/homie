# OpenSpec Tasks — llm-gateway-tier3-evidence-hardening

| Task | 描述 | 验收标准 | 关联验证 Case |
|------|------|---------|---------------|
| T1 | 审阅 gateway 源码，产出合并 failure model（`failure-model.md`），逐条标注伤害方式→捕获层→证据编号 | 8 类伤害方式全覆盖，每条有捕获层 | FC-02/FC-03/FC-04/FC-06 |
| T2 | 更新 `specs/llm-gateway.md` §10 增加 Failure Model 段（R3） | §10 含 Failure Model，与 T1 一致 | FC-05 |
| T3 | 执行基线 + 对抗验证（`cargo test -p homie-gateway` 含集成）并留存日志 | 44/44 通过，退出码 0 | FC-01/FC-03 |
| T4 | 执行手动变异 M1–M5（每个被杀死后恢复）+ 负向对照（FC-02） | 5/5 杀死，恢复后全绿，工作树干净 | FC-02/FC-04 |
| T5 | 执行手动逐行覆盖率核对 + 产出三份报告（functional-verification、release-readiness、code-review round-1/2） | 安全关键行 100% 核对或 known limit 显式 | FC-05/FC-06 |

执行顺序：T1 → T2 → T3 → T4 → T5（串行，不可逆）。

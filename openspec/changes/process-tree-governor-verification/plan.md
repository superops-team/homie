# Plan: process-tree-governor-verification

## 目标

以 TDD（RED→GREEN→REFACTOR）为 `main` 上已移植的 `process_tree.rs` 与 `governor.rs`
补验证，先证明当前实现安全，暴露缺陷则记录到 `gaps.md` 另行修复。

## 范围

- 新增 `homie/crates/homie-engine/tests/process_tree.rs` 集成测试。
- 补 governor eligibility（`idle_since`）+ 休眠策略单测。
- 编写失败模型 `docs/verification/process-tree-governor-verification/failure-model.md`。

## 非范围

- 不改 `process_tree.rs` / `governor.rs` 实现（缺陷另立项）。
- 不做真实 agent 端到端休眠。

## 顺序

1. RED：先写测试，运行证明缺口/失败。
2. 记录失败模型与对抗性推理。
3. GREEN：仅当测试暴露真实缺陷时另行立项修复；本 change 以验证为主，若实现已正确则测试直接 GREEN。
4. REFACTOR：清理测试 helper，确保无 flake。
5. 记录证据到 `docs/verification/process-tree-governor-verification/`。

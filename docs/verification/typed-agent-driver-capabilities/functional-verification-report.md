# Typed Agent Driver Capability Functional Verification Report

## 1. 结论

`typed-agent-driver-capabilities` 首阶段功能验证通过。

已完成：

- Rust `DriverCapabilities` wire DTO。
- Swift `DriverCapabilities` / `SessionCapabilitiesResult` DTO。
- Engine `driver` module with default unsupported behavior and fake driver contract tests。
- Read-only `session.capabilities` control method。
- Rust client helper `session_capabilities`。
- Focused Swift/Rust wire tests。

未做：

- 不接真实 Codex/Claude/OpenCode provider。
- 不新增 steer/cancel/model action methods。
- 不改 MCP tool surface。
- 不改 UI。

## 2. Case 执行结果

| Case | 状态 | 证据 |
|---|---|---|
| FC-01 PRD/spec 和 review 风险已收敛 | pass | `fc-01-spec-review.log` |
| FC-02 OpenSpec 三件套完整并覆盖 Case | pass | `fc-02-openspec-alignment.log` |
| FC-03 capability DTO 和 fake driver contract | pass | `fc-03-driver-contract.log` |
| FC-04 `session.capabilities` 查询路径 | pass | `fc-04-session-capabilities.log` |
| FC-05 wire compatibility 和 Swift/Rust method vocabulary | pass | `fc-05-wire-compatibility.log` |
| FC-06 静态门禁和范围守卫 | pass | `fc-06-static-gates.log` |

## 3. 关键证据

- `driver::tests::unsupported_driver_returns_stable_errors_and_no_capabilities` passed。
- `driver::tests::fake_driver_declares_capabilities_without_storing_prompt_text` passed。
- `control::tests::session_capabilities_are_read_only_and_default_to_unsupported` passed。
- `session_capabilities_wire_shape_is_camel_case_and_default_false` passed in Rust。
- `sessionCapabilitiesWireShape` passed in Swift。
- `cargo fmt --manifest-path homie/Cargo.toml --all -- --check` passed。
- `git diff --check` passed。

## 4. 残余风险

- This slice exposes a read-only query only. Real provider support and typed actions still require separate child changes.
- Capability values for real manifest-backed sessions currently default to unsupported/all false by design.

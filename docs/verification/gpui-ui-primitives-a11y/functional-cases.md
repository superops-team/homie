# GPUI UI primitives 首个切片功能验证 Case

## FC-01: PRD 边界清晰

```bash
rg -n "Button|close-settings|非目标|disabled" prd-spec/refactors/gpui-ui-primitives-a11y/2026-08-14-gpui-ui-primitives-a11y-design.md
```

## FC-02: Button primitive 已导出

```bash
rg -n "pub struct Button|pub enum ButtonVariant|pub enum ButtonSize|pub use components::.*Button" homie/crates/homie-ui/src/components.rs homie/crates/homie-ui/src/lib.rs
```

## FC-03: close-settings 使用 Button

```bash
rg -n "Button::new\\(\"close-settings\"|close-settings" homie/crates/homie-app/src/surface_shell.rs
```

## FC-04: Button tests 通过

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-ui button -- --nocapture
```

## FC-05: close settings 回归测试通过

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app settings_content_close_control_dismisses_the_surface -- --nocapture
```

## FC-06: 静态门禁通过

```bash
(cd homie && cargo fmt --check)
git diff --check
git diff --name-only -- homie/crates/homie-engine homie/crates/homie-client
```

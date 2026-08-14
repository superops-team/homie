# GPUI render purity 首个切片功能验证 Case

## FC-01: PRD 边界清晰

```bash
rg -n "shortcut rank|非目标|sidebar_projection" prd-spec/refactors/gpui-render-path-purity/2026-08-14-gpui-render-path-purity-design.md
```

## FC-02: shortcut rank 纯函数存在并被 render 调用

```bash
rg -n "fn shortcut_ranks|shortcut_ranks\\(" homie/crates/homie-app/src/sidebar/view.rs
```

## FC-03: shortcut rank tests 通过

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app shortcut_rank -- --nocapture
```

## FC-04: sidebar 回归和静态门禁

```bash
cargo test --manifest-path homie/Cargo.toml -p homie-app sidebar -- --nocapture
(cd homie && cargo fmt --check)
git diff --check
git diff --name-only -- homie/crates/homie-ui homie/crates/homie-engine homie/crates/homie-client
```

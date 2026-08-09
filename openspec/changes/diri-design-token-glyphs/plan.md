# Diri Design Token/Glyph OpenSpec Plan

> Change ID: `diri-design-token-glyphs`
> Beads: `homie-8x8`

## Summary

Add brand, icon/status glyph catalog, and gallery model to `homie-ui` so Diri design-system parity has reusable data beyond scalar tokens.

## Verification

| Case | Command |
|------|---------|
| FC-DDTG-001 | `cargo test -p homie-ui --tests -- --nocapture` |
| FC-DDTG-002 | `cargo clippy -p homie-ui --all-targets -- -D warnings` |
| FC-DDTG-003 | `make parity-lock` |


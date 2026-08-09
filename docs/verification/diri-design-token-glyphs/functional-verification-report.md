# Diri Design Token/Glyph Functional Verification Report

```yaml
change_id: diri-design-token-glyphs
beads: homie-8x8
status: pass
validated_at: 2026-08-07
```

## Summary

This slice advances `UI-009`:

- `homie-ui` now exposes `HOMIE_BRAND`.
- `STATUS_GLYPHS` and `status_glyph` provide reusable status glyph data.
- `DESIGN_GALLERY` provides a stable gallery data source for later screenshot gates.

`UI-009` remains `partial` until icon asset rendering and screenshot gate evidence are complete.

## Functional Cases

| Case | Command | Result |
|------|---------|--------|
| FC-DDTG-001 | `cargo test -p homie-ui --tests -- --nocapture` | pass |
| FC-DDTG-002 | `cargo clippy -p homie-ui --all-targets -- -D warnings` | pass |


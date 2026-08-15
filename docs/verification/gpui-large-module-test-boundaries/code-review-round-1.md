# GPUI Large Module Test Boundaries Code Review Round 1

## 1. Scope

Reviewed files:

- `homie/crates/homie-app/src/sidebar/picker_logic.rs`
- `homie/crates/homie-app/src/sidebar/mod.rs`
- `homie/crates/homie-app/src/sidebar/view.rs`
- `docs/verification/gpui-large-module-test-boundaries/*`
- `openspec/changes/gpui-large-module-test-boundaries/*`

## 2. Findings

| Severity | Finding | Result |
|---|---|---|
| P0 | Slice could become a sidebar rewrite | pass: only four helper functions were moved; render tree and event handlers remain in `view.rs` |
| P0 | New helper could depend on GPUI context | pass: static scan has no `Window`, `Context`, `Entity`, `cx.`, or `div(` matches |
| P1 | Behavior could drift during extraction | pass: focused tests cover remote target normalization, repo resolution, and shortcut label selection |
| P1 | Could overlap completed shortcut-rank slice | pass: `shortcut_ranks` and related tests were not moved or changed |
| P1 | Could touch UI primitives or lifecycle code | pass: no `homie-ui`, RootView, UtilitySurfaces, terminal, or inspector files changed for this slice |

## 3. Verification Reviewed

| Evidence | Result |
|---|---|
| `fc-03-pure-module.log` | pure-module static gate passed |
| `fc-04-picker-tests.log` | 4 focused tests passed |
| `fc-05-static-gates.log` | shell syntax, rustfmt, diff checks passed |

## 4. Conclusion

No P0/P1 code issues found in round 1.

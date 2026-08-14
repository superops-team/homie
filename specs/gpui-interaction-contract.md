# GPUI Interaction Contract

## 1. Purpose

This spec defines Homie's durable interaction rules for GPUI controls,
overlays, keyboard routing, focus, accessibility and platform preferences.

## 2. Stable Identity

Interactive or stateful elements must use stable IDs.

Preferred ID shape:

```text
("<role>", domain_id)
("<role>", enum_variant)
("<role>", stable_action_id)
```

Avoid:

- list index for reorderable/filterable/searchable lists;
- localized display text;
- random values created during render;
- duplicate static IDs in loops.

## 3. Keyboard And Focus

Every actionable UI path must have a keyboard path unless the interaction is
intrinsically pointer-only and documented as such.

Rules:

- Opening a dialog or overlay moves focus to a meaningful target.
- Closing a dialog or overlay returns focus to the source when it still exists.
- Escape dismisses transient surfaces from innermost to outermost.
- Enter/Space activate controls according to their role.
- Tab/Shift-Tab traverse actionable controls predictably.
- Focus-visible styling must not be clipped by parent overflow.

## 4. Accessibility

For each interactive primitive, provide when supported by the pinned GPUI API:

- role;
- accessible name;
- selected/expanded/disabled/loading state;
- click or activation action;
- disabled behavior that blocks pointer, keyboard and accessibility activation.

A raw `div().on_click(...)` is not sufficient for a semantic button, row, tab or
dialog action unless the caller documents why no primitive fits.

## 5. Pointer Behavior

- Respond visually on press/down.
- Commit action on click/release according to control semantics.
- Preserve drag offset for direct manipulation.
- Use `occlude()` or equivalent behavior to prevent pointer events from leaking
  through modal surfaces.
- Dismiss popovers through a defined outside-click policy.

## 6. Disabled, Loading, Empty And Error States

Controls and surfaces must represent:

- default;
- hover;
- pressed;
- focused;
- selected or expanded;
- disabled;
- loading;
- empty;
- error.

Do not encode state only by subtle alpha or color. Add text, icon, stroke,
shape, or position when the state changes behavior.

## 7. Platform Preferences

Homie must respect available platform/application preferences:

| Preference | Expected UI Response |
|------------|----------------------|
| reduce motion | snap or short fade; no continuous animation frame demand |
| reduce transparency | opaque surfaces preserving hierarchy |
| increase contrast | stronger text, border and focus separation |
| differentiate without color | add icon, label, shape, stroke, or pattern |

If a platform preference is not currently wired, the spec or verification report
must mark it as unverified rather than implying support.

## 8. Overlay Contract

Dialogs, popovers, sheets and command surfaces must define:

1. source/anchor identity;
2. initial focus;
3. Escape behavior;
4. outside-click behavior;
5. scroll/pointer occlusion policy;
6. focus return behavior;
7. resize or anchor-loss behavior;
8. evidence path for runtime validation when behavior is user-visible.

## 9. Test Requirements

Use the smallest meaningful test:

- pure ID/state/policy rules: unit tests;
- focus/action/key routing: `#[gpui::test]`;
- real IME, menu, window and material behavior: runtime validation.

For disabled controls, test all activation paths: pointer, keyboard and
accessibility action where supported.

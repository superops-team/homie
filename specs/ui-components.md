# Homie UI Component Contract

## 1. Purpose

This spec defines the minimum contract for reusable GPUI components in
`homie-ui`. It prevents every app surface from hand-rolling hover, click,
keyboard, focus, disabled and accessibility behavior.

## 2. Component Levels

| Level | Use When | Example |
|-------|----------|---------|
| Token | Shared value, no behavior | colors, radius, type, spacing |
| RenderOnce component | Value-like reusable visual or local interaction | loading indicator, divider, static surface |
| Stateful entity | Owns focus, tasks, subscriptions, internal selection, or async work | complex editor, async picker |
| App surface | Product workflow with domain state | sidebar, settings, inspector |

Do not create an entity for decorative wrappers.

## 3. Required Primitive Set

The first semantic primitive set should include:

- `Button`
- `IconButton`
- `ListRow`
- `Dialog`
- `Tab`
- `TextField` or `TextInputAdapter`

## 4. Shared Primitive Requirements

Every interactive primitive must define:

- stable ID input;
- semantic variant;
- visual size;
- enabled/disabled state;
- selected/expanded/loading state when applicable;
- pointer behavior;
- keyboard behavior;
- focus-visible behavior;
- accessible name and role where supported;
- theme token use;
- disabled activation blocking;
- tests.

## 5. Button

Variants:

- primary;
- secondary;
- quiet;
- destructive;
- toolbar.

Required behavior:

- click and keyboard activation;
- disabled state blocks activation;
- visible focus;
- loading state prevents duplicate action when configured;
- accessible name must be explicit when icon-only.

## 6. IconButton

Required behavior:

- fixed hit target independent of icon glyph size;
- tooltip or accessible name for unfamiliar icons;
- disabled and selected state when applicable;
- no text overflow concerns.

Use existing icon assets or SF Symbols through the platform-specific path already
used by the app. Do not introduce a second icon system without PRD/spec.

## 7. ListRow

Required behavior:

- domain stable row ID;
- selected and multi-selected state;
- hover and focus-visible state;
- keyboard activation and list navigation where the list owns navigation;
- optional trailing actions that do not steal row drag/click semantics;
- row identity must not be based on index for reorderable/filterable lists.

## 8. Dialog

Required behavior:

- title or accessible name;
- focus scope and initial focus;
- Escape behavior;
- outside-click policy;
- focus return target;
- scroll/resize behavior;
- disabled background interaction via occlusion.

## 9. Tab

Required behavior:

- selected state;
- keyboard navigation;
- stable ID from enum/domain value, not localized label;
- selected state perceivable without color alone.

## 10. TextField / TextInputAdapter

Editable text is platform-sensitive. Any reusable text input must preserve:

- cursor movement;
- selection;
- Unicode scalar/grapheme behavior expected by existing editor helpers;
- copy/cut/paste behavior;
- placeholder distinction from input text;
- focus and keyboard routing.

Prefer adapting existing `query_editor` behavior before adding a new text system.

## 11. Theme And Tokens

Components should use `SemanticColors`, `Typo`, `Radius` and shared fill/stroke
roles. Component callers should choose semantic variants rather than raw colors
where possible.

## 12. Test Requirements

Each primitive needs tests for:

- default render-relevant state;
- hover/press/selected/disabled state policy where pure-testable;
- keyboard activation;
- disabled activation blocking;
- stable ID construction;
- accessibility label/state generation where supported.

Runtime visual evidence is required when changing dimensions, material, focus
rings, text clipping, or platform-specific appearance.

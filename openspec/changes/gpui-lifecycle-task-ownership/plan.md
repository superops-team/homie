# GPUI Lifecycle Task Ownership Plan

## Scope

This change only hardens RootView child entity subscription ownership.

## In Scope

- Hold terminal/sidebar/launcher/navigation/inspector subscriptions in
  `RootView::_subscriptions`.
- Preserve event handling behavior.
- Add verification evidence.

## Out Of Scope

- Non-subscription `.detach()` calls.
- UtilitySurfaces History/Worktrees task lifecycle.
- UI primitives, render purity, visual gates.

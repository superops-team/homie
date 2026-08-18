# GPUI Shell Contract

## 1. Purpose

This spec defines durable ownership boundaries for the Homie GPUI desktop shell.
It exists so future GPUI changes do not depend on chat context or one-off review
notes.

## 2. Current Shell Components

| Component | Current File | Durable Responsibility |
|-----------|--------------|------------------------|
| App startup | `homie/crates/homie-app/src/main.rs` | runtime setup, menus, window creation, app services |
| Root shell | `homie/crates/homie-app/src/root.rs` | top-level entity composition, global action routing, focus fallback |
| Workbench layout | `homie/crates/homie-app/src/workbench.rs` | pure split and pane sizing policy |
| Sidebar | `homie/crates/homie-app/src/sidebar/` | session/project navigation, selection, sidebar-specific popovers |
| Terminal pane | `homie/crates/homie-app/src/terminal_pane/` | terminal rendering and input bridge |
| Utility surfaces | `homie/crates/homie-app/src/surface_shell/` | history, worktrees, settings, remote host editor overlays |
| Inspector | `homie/crates/homie-app/src/inspector.rs` | artifact/review/code inspector surfaces |
| Shared UI | `homie/crates/homie-ui/` | semantic tokens and reusable GPUI components |

## 3. Ownership Rules

### 3.1 RootView

`RootView` may own:

- child entity handles;
- global action handlers;
- focus fallback;
- top-level resize/drag shields;
- app service references;
- narrow shell orchestration state.

`RootView` should not own:

- product-specific settings form state;
- history scanning state;
- worktree cleanup operation state;
- remote host editor state;
- reusable button/list/dialog behavior;
- long-lived service loops that can live in a bridge/controller.

### 3.2 Workbench Layout

Layout math should live in pure structs where possible. A layout object should
accept available dimensions and return stable presentation values. It must not
read or write GPUI context, store state, filesystem, network, or process state.

### 3.3 Utility Surfaces

Each utility surface that owns async work, focus, or error/loading state should
be eligible for its own entity or controller. A shared overlay shell may handle
backdrop, focus scope, Escape/outside dismiss and focus restoration.

### 3.4 Service Event Bridges

Store/update/usage watchers should be concentrated in bridge/controller code
when they grow beyond trivial invalidation. The bridge should translate service
events into entity updates without taking over render policy.

## 4. Task And Subscription Contract

- UI lifecycle tasks are fields on their owner.
- Replacing a task cancels stale work when cancellation is supported.
- Uncancellable async operations must use generation or operation IDs before
  applying results.
- Core `Subscription` handles are held by the owner whose lifetime defines the
  relationship.
- `.detach()` requires app-lifetime or service-lifetime justification.

## 5. Render Contract

Render methods:

- read prepared state;
- derive bounded presentation values;
- build element trees;
- attach handlers with stable captures.

Render methods must not:

- start tasks or subscriptions;
- access disk, network or processes;
- perform unbounded parsing/sorting/filtering;
- mutate domain state;
- hold long-running write locks.

## 6. Module Extraction Gate

A new shell module should be created only when it satisfies all of:

1. It hides a specific complexity.
2. It has stable input/output or clear lifecycle ownership.
3. It can be tested independently.
4. It reduces the caller interface rather than increasing parameter sprawl.
5. It preserves dependency direction.

Candidate module names in PRD/spec documents are suggestions, not mandatory
file structures.

## 7. First Recommended Code Slice

The first code-bearing child change should target `UtilitySurfaces history/worktrees task lifecycle`.
This slice is lower risk than RootView restructuring and can establish the task
generation/stale-result pattern used by later surfaces.

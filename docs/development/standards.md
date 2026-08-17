# Homie Development Standards

## 1. Scope

These standards apply to Homie Rust, Swift, GPUI, scripts, documentation, and
verification work. Repository-specific rules in `AGENTS.md` remain the top-level
authority.

## 2. Workflow Standards

Meaningful changes must use:

1. Beads issue with stable `change_id`.
2. Chinese PRD/spec under `prd-spec/`.
3. Durable component contract updates under `specs/` when boundaries change.
4. OpenSpec plan/tasks/alignment under `openspec/changes/<change-id>/`.
5. Evidence under `docs/verification/<change-id>/`.

Small documentation-only changes may use the lightweight path, but process
rules that affect future development still need clear source and Beads linkage.

## 3. Rust And GPUI Module Standards

### 3.1 Entity Ownership

Use a GPUI `Entity<T>` when the value:

- changes independently;
- owns focus, tasks, or subscriptions;
- emits events;
- is observed by multiple owners;
- needs focused GPUI tests.

Use `RenderOnce` or plain element composition for value-like reusable
components with no independent lifetime.

### 3.2 Root And Shell Boundaries

`RootView` should remain the top-level composition root and global action
router. It should not accumulate business logic for history, worktrees,
settings, remote host editing, notification policy, or long-running service
loops.

Before adding fields to `RootView`, check whether the state belongs to:

- a child entity;
- a pure layout state object;
- a service event bridge;
- a controller with a narrow lifecycle;
- the shared store/runtime.

### 3.3 Render Purity

Render paths must stay deterministic and cheap.

Do not do this in `render`:

- filesystem, network, process, or sleep operations;
- task or subscription creation;
- domain state mutation;
- long-held write locks;
- unbounded sorting/filtering/parsing;
- random ID creation.

Render may read already prepared state, derive bounded presentation values, and
build element trees with stable handlers.

### 3.4 Task And Subscription Ownership

- Lifecycle-bound UI tasks must be stored in fields and cancelled by replacement
  or entity drop.
- Repeated user-triggered operations must use generation, operation id, or
  revision checks before applying async results.
- `.detach()` is only for deliberate app-lifetime or service-lifetime work with
  observed error handling.
- Important `cx.subscribe` / `cx.observe` handles must be stored in an owner
  field, not detached by default.

### 3.5 Interaction And Accessibility

Interactive UI should use semantic primitives from `homie-ui` when available.
If a raw `div().on_click(...)` is used for a button, row, tab, or menu action,
the code must justify why no primitive fits.

Every interactive element needs:

- stable ID;
- role and accessible name where GPUI support is available;
- keyboard activation;
- visible focus;
- hover, pressed, selected, disabled and loading state as applicable;
- disabled behavior that blocks pointer, keyboard and accessibility activation;
- adequate hit target.

### 3.6 Stable IDs

Prefer domain identity plus local role:

```text
("session-row", session.id)
("settings-tab", SettingsTab::Remote)
```

Avoid list indexes for reorderable, filterable, searchable, or virtualized
collections. Avoid localized text and random values as element IDs.

## 4. Swift Standards

- Swift targets are CLI/protocol/core/MCP glue unless a PRD/spec says otherwise.
- Do not reintroduce Swift daemon, holder, PTY, git, or detection runtime paths.
- Protocol changes touching both Swift and Rust must include shared fixtures or
  a documented parity check.

## 5. Worktree And Build Cache Standards

- Follow `AGENTS.md` Worktree Build Cache Rules.
- New Homie worktrees must symlink `homie/target` to the shared project target
  before running Cargo.
- Do not commit machine-local symlinks, target directories, `.build`, or cache
  directories.

## 6. Testing Standards

Choose the smallest meaningful test first:

- pure reducer/layout/token logic: unit tests;
- GPUI entity state/focus/actions: `#[gpui::test]`;
- app launch, menus, windows, materials, visuals: real app launch and evidence;
- CLI/protocol parity: Swift/Rust tests and fixtures;
- packaging/release: script logs and bundle verification.

Do not claim visual or runtime correctness from `cargo check` alone.

### 6.1 TDD Loop (RED → GREEN → REFACTOR)

"SDD/TDD" is not a single step. Each behavior follows a loop:

1. **RED** — write the test for one behavior and run it first. Watch it fail.
   The failure must be an assertion failure, not an import / compile error. If
   the module does not exist yet, create a stub that raises (e.g.
   `unimplemented!()`) so the test fails on behavior. A new test that passes
   immediately is either vacuous or already covered: prove which by breaking
   the implementation with a one-off throwaway mutant, watching the test fail,
   then restoring — and record it as pre-existing behavior kept as regression
   armor.
2. **GREEN** — write the least code that makes the failing test pass, then run
   the full suite, not just the new test.
3. **REFACTOR** — while green, improve names and structure. Behavioral
   assertions are frozen: implementation refactors touch no test files. Any
   change to an assertion is a behavior change and must go back to SPEC.

### 6.2 Anti-Gaming Rules (absolute)

The gauntlet only creates trust if it cannot be gamed:

1. Never weaken a test to make it pass (no broadened assertions, added skips,
   raised tolerances, or deleted failing tests).
2. Never edit a test and the implementation in the same step to reach green.
3. Never mock the unit under test, or mock so much that only the mocks are
   exercised. Mock boundaries (network, clock, filesystem), not logic.
4. Never chase the coverage number. A test added only to touch lines, with no
   meaningful assertion, is gaming.
5. Never report a layer you did not run. A skipped layer must state its reason.
6. A failing gauntlet blocks done — you are not finished while any layer fails.

### 6.3 Calibration (Tier 1 / 2 / 3)

Scale effort to blast radius, and record which tier was chosen:

- **Tier 1 — trivial** (typo, comment, config value): full suite + lint. No new
  test required, but state why the change is untestable or already covered.
- **Tier 2 — normal** (bug fix, small feature): full loop. Bug fixes MUST start
  with a RED test that reproduces the bug.
- **Tier 3 — high stakes** (money, auth, data loss, concurrency, public API,
  credentials and virtual-key custody): start with a **failure model** — list
  the specific ways this change can hurt (race, partial write, hostile input,
  overflow, unbounded growth, failed rollback, credential leak) and add a layer
  per mode that actually catches it (race/stress tests, fuzzing, rollback
  rehearsal, latency benchmarks, API-compat checks, contract tests,
  logging/metric assertions). Then run: full loop + property-based tests +
  mutation testing + one adversarial pass (explicitly try to break your own
  implementation with hostile inputs). Failure modes deliberately not covered
  go in EVIDENCE as known limits.

For credential custody, virtual key issuance, LLM proxying, orchestration,
concurrency, and data-loss-adjacent code, treat Tier 3 as the default.

## 7. Documentation Standards

- PRD/spec files are Chinese and live under `prd-spec/`.
- Durable contracts live under `specs/`.
- Verification evidence lives under `docs/verification/<change-id>/`.
- OpenSpec tasks must reference functional verification cases where applicable.

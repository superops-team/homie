## Context

Homie is a Rust + GPUI reconstruction of Diri. Previous work created many DTOs, parsers, models, UI surfaces and scoped tests, but the current product still uses an in-process runtime client, fixed shell spawning, partial UI actions and placeholder release paths. The previous planning model split work into more than 50 micro changes, making local evidence look complete while the end-to-end product remained incomplete.

This change fixes the planning and contract layer. It does not implement all runtime behavior. The source of truth is:

- `diri/` at commit `7ba3407`;
- the Chinese PRD for this change;
- `docs/research/diri-7ba3407-capability-matrix.md`;
- long-lived component specs;
- this OpenSpec change.

Constraints:

- macOS is the parity release target.
- Rust owns runtime/domain/storage; GPUI owns desktop rendering; Swift or macOS-specific Rust owns thin native integration.
- SQLite is the durable local fact source; offset logs own high-volume output.
- provider raw keys remain under Homie credential custody.
- no backward-compatibility layer is required for current incorrect internal APIs.
- existing user changes in the dirty worktree must not be reverted.

## Goals / Non-Goals

**Goals:**

- freeze a complete, testable Diri `7ba3407` capability baseline;
- establish truthful status and evidence rules;
- define a long-term daemon/client/service architecture;
- split implementation into dependency-ordered vertical waves;
- map all PRD requirements to component contracts, OpenSpec tasks and verification;
- separate Diri parity from Homie-only control-plane extensions.

**Non-Goals:**

- implement all waves in this change;
- mechanically port Diri source line by line;
- preserve production in-process runtime or UI direct-storage behavior;
- add Windows/Linux release support;
- choose child-wave details that require focused package/security research;
- rewrite historical evidence.

## Decisions

### Decision 1: Pin one immutable baseline

The baseline SHALL be the embedded repository commit `7ba3407`, not a branch or moving upstream reference.

Rationale:

- every requirement and test can cite immutable source;
- future Diri changes cannot silently alter acceptance;
- status changes become reviewable diffs.

Alternative considered: continuously track Diri main. Rejected because it prevents a stable completion definition and mixes parity with upstream change intake.

### Decision 2: Use one capability matrix for completion truth

`docs/research/diri-7ba3407-capability-matrix.md` SHALL be the only module-level completion source. Historical inventory and verification files remain evidence inputs, not completion authorities.

Rationale:

- avoids conflicting status across PRD, Beads and release reports;
- forces code, product wiring and verification to be evaluated together;
- gives final release automation one machine-readable target.

Alternative considered: keep the current row matrix and add more rows. Rejected because it missed baseline capabilities, accepted invalid statuses and did not detect regressions in implemented rows.

### Decision 3: Deliver vertical waves, not independent micro slices

Implementation SHALL use the wave IDs in the PRD. Each wave produces a working end-to-end increment with its own Bead, PRD, OpenSpec and evidence.

Rationale:

- integration is proven as part of each delivery;
- dependencies are explicit;
- a DTO/parser-only change cannot close a product capability.

Alternative considered: continue the existing 55+ micro changes. Rejected because it caused document and evidence fragmentation without a green workspace or complete workflow.

### Decision 4: Runtime is an independent process

The target architecture SHALL have an independent runtime daemon and an endpoint-based `homie-client`. Production client code SHALL NOT construct `RuntimeSupervisor` or open storage.

Rationale:

- matches Diri crash-survival and multi-entry architecture;
- creates one owner for live PTY/session/event state;
- allows app, CLI and MCP to observe the same runtime.

Alternative considered: retain an embedded runtime and add a daemon later. Rejected because it preserves two ownership models and makes recovery semantics untestable.

### Decision 5: Public catalogs must be executable

Protocol method catalogs, CLI command discovery and MCP `tools/list` SHALL only publish executable capabilities for the active runtime. Unsupported capabilities SHALL be absent from discovery and direct unknown calls SHALL use method-not-found errors.

Rationale:

- prevents false parity from descriptor-only code;
- makes automation clients capability-aware;
- removes ambiguous `-32000` execution/method errors.

Alternative considered: advertise future tools with an unsupported result. Rejected because discovery is a behavioral contract, not a roadmap.

### Decision 6: Use dual OpenSpec artifacts

Each change SHALL contain:

- `proposal.md`, `design.md`, `specs/**/spec.md`, `tasks.md` for the OpenSpec CLI;
- `plan.md`, `tasks.md`, `alignment-report.md` for the repository workflow.

Rationale:

- current repository rules and installed OpenSpec schema differ;
- producing both avoids another incomplete 1/4 artifact state;
- a single `tasks.md` remains the shared execution checklist.

Alternative considered: customize OpenSpec schema immediately. Rejected for Wave 0 because schema migration is process infrastructure beyond the approved parity planning scope.

### Decision 7: Separate Diri parity and Homie extensions

Context, memory, task and orchestrator integration SHALL have an independent extension status. Their failure blocks Homie control-plane release claims but does not redefine which Diri capabilities are required.

Rationale:

- avoids using Homie-specific models to hide missing Diri behavior;
- preserves the product direction without contaminating parity accounting.

### Decision 8: No compatibility layer for incorrect internal boundaries

When a wave replaces an internal shortcut, the shortcut SHALL be removed. Tests use explicit test harnesses rather than production fallbacks.

Rationale:

- repository rules reject backward compatibility by default;
- dual runtime/storage paths would reintroduce state divergence;
- current product has no stable external contract requiring preservation.

### Decision 9: Evidence is typed and scope-aware

Gate status SHALL be one of `pass`, `blocked`, `not_run`, `partial`, `fail`. A wave pass SHALL NOT imply overall parity pass.

Rationale:

- eliminates ambiguous statuses such as `pass_with_scope_limit`;
- makes blockers and unexecuted gates visible;
- allows automated final completeness checks.

## Risks / Trade-offs

- **[Large program scope]** -> Keep Wave 0 documentation-only and require independent child change approval before code.
- **[Runtime rewrite can destabilize current shell flow]** -> Build protocol/client tests first, use a deterministic daemon fixture, then remove the embedded path only after cross-entry E2E is green.
- **[Dirty worktree makes attribution difficult]** -> Add isolated change paths and targeted component amendments; do not reformat or rewrite unrelated user files.
- **[Historical evidence conflicts with new status]** -> Preserve historical reports and record explicit supersession; do not retroactively present them as current pass.
- **[External release credentials unavailable]** -> Mark notarization gates blocked; never substitute ad-hoc signing.
- **[Remote/provider tests could require secrets]** -> Use loopback fake node and fake provider for mandatory integration; real services remain opt-in smoke.
- **[Dual OpenSpec formats add documentation overhead]** -> Keep one task checklist and generate alignment from stable FR/task IDs.
- **[Component contracts may over-specify future details]** -> Child PRDs choose only details required by their wave; Wave 0 records boundaries and invariants, not implementation classes.

## Migration Plan

1. Create and review the Wave 0 PRD, capability matrix and component amendments.
2. Correct parity statuses contradicted by current tests or architecture.
3. Validate OpenSpec artifacts and requirement coverage.
4. Create Wave 1A Bead/PRD/OpenSpec before editing runtime/client code.
5. Implement daemon/client transport beside the current code only inside a feature branch/worktree and test harness.
6. Switch app/CLI/MCP to the endpoint client in one controlled change.
7. Delete production embedded runtime construction and direct UI storage dependencies.
8. Continue Wave 1B through Wave 5B in dependency order.
9. Run final parity gate; archive superseded planning only after all references point to the new baseline.

Rollback:

- Wave 0 is documentation-only and can be reverted without product data migration.
- Child waves must define their own rollback. Internal API compatibility fallbacks are not an accepted rollback strategy.
- Storage changes use forward-only migrations and require backup/quarantine plans in their child PRD.

## Open Questions

These are mandatory child-wave decisions, not blockers for approving this rebaseline:

1. Wave 1A must choose the exact data-channel framing and peer authentication after focused package/security research.
2. Wave 4B must choose and document the secret envelope format, KDF, AEAD and key source.
3. Wave 5A must set numeric performance budgets against an identified release machine/OS matrix.
4. Wave 5A must decide whether remote node service artifacts ship in the desktop release or a separate signed package.

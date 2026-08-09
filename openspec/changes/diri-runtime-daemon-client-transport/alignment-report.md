# PRD / Component Spec / OpenSpec Alignment Report

```yaml
change_id: diri-runtime-daemon-client-transport
status: pass
beads: homie-nep
parent_change_id: diri-7ba3407-parity-rebaseline
baseline_commit: 7ba3407
```

## 1. Sources

- PRD: `prd-spec/features/diri-runtime-daemon-client-transport/2026-08-08-diri-runtime-daemon-client-transport-design.md`
- Proposal: `openspec/changes/diri-runtime-daemon-client-transport/proposal.md`
- Design: `openspec/changes/diri-runtime-daemon-client-transport/design.md`
- Tasks: `openspec/changes/diri-runtime-daemon-client-transport/tasks.md`
- Capabilities:
  - `runtime-daemon-lifecycle`
  - `multiplexed-runtime-transport`
  - `async-runtime-client`
  - `shared-runtime-consumers`

## 2. Requirement Alignment

| PRD | OpenSpec requirement coverage | Tasks | Verification |
|-----|-------------------------------|-------|--------------|
| FR-01 Data Dir paths | runtime-daemon-lifecycle: runtime paths, singleton | 4, 11 | launcher + daemon process permissions/singleton |
| FR-02 Explicit launcher | runtime-daemon-lifecycle: explicit launcher | 4, 17 | launcher tests + app startup |
| FR-03 daemon lifecycle | runtime-daemon-lifecycle: startup, shutdown, recovery | 5, 11, 19 | actor + real process + restart E2E |
| FR-04 binary frame | multiplexed-runtime-transport: fixed preface/frame | 2, 3 | byte fixtures + hostile codec |
| FR-05 kind/payload | multiplexed-runtime-transport: payload/kind/stream rules | 2, 3, 7 | proto contract + server protocol |
| FR-06 capability truth | multiplexed-runtime-transport: Hello exact capabilities | 6, 7 | registry equality + Hello |
| FR-07 correlation/errors | multiplexed-runtime-transport and async-runtime-client | 7, 12, 14 | out-of-order, timeout, disconnect, error mapping |
| FR-08 event gap recovery | multiplexed-runtime-transport + async-runtime-client | 9, 13, 19 | ring replay, gap snapshot, restart |
| FR-09 terminal stream | multiplexed-runtime-transport + async-runtime-client | 10, 13, 17 | replay/grid/live + reopen + app bridge |
| FR-10 bounded queues | multiplexed-runtime-transport + async-runtime-client | 8, 12, 13, 19 | fairness, overflow, stream isolation |
| FR-11 RuntimeActor/dispatcher | runtime-daemon-lifecycle | 5, 6, 9, 10 | actor ordering/backpressure + handler registry |
| FR-12 pure async client | async-runtime-client | 12, 13, 14 | client tests + dependency/source negative gates |
| FR-13 CLI/MCP | shared-runtime-consumers | 15, 16 | CLI and MCP focused suites |
| FR-14 GPUI app | shared-runtime-consumers | 17 | runtime bridge + first frame + compile |
| FR-15 shortcut deletion | async-runtime-client + shared-runtime-consumers | 14, 15, 16, 17 | compile and source/dependency negative scans |
| FR-16 UDS security | runtime-daemon-lifecycle + multiplexed-runtime-transport | 3, 7, 11, 19 | peer UID, permissions, limits, safe logs |

Every PRD FR maps to at least one normative OpenSpec requirement, executable task group, and verification gate.

## 3. Component Contract Alignment

| Component spec | OpenSpec capability | Enforced boundary |
|----------------|---------------------|-------------------|
| `runtime-client-transport` | multiplexed-runtime-transport, async-runtime-client | one UDS, fixed frame, bounded client, no implicit launcher |
| `runtime-supervisor` | runtime-daemon-lifecycle | daemon/actor single owner, holder-safe shutdown |
| `desktop-shell` | shared-runtime-consumers | two-worker Tokio bridge, non-blocking first frame |
| `mcp-automation` | shared-runtime-consumers | async client, control bridge, error truth |
| `observability` | all | safe transport events and honest evidence |
| `packaging-updater` | shared-runtime-consumers | daemon included at fixed bundle path |

## 4. Scope Boundary Alignment

| Deferred scope | Owner | Wave 1A guard |
|----------------|-------|----------------|
| manifest-driven agent spawn | T-102 | existing shell spawn only; no agent claims |
| holder detached regression | T-102 | retained as blocker; no status fabrication |
| complete app/CLI storage ownership | T-103 | settings/doctor/usage isolated from live runtime |
| remote/node/Tailscale | later remote wave | local UDS only |
| styled grid/visual parity | T-202 | full default-style grid over final wire contract |
| signing/notarization/updater | T-501 | daemon closure only |

## 5. Capability Truth Audit

The design registry includes current production behavior used by app/CLI/MCP:

- session lifecycle/snapshot/status/artifact/port/lineage/history/diff;
- worktree list/create/remove/overview;
- host locate;
- hook report;
- event wait;
- daemon shutdown;
- event and terminal streams.

Future proto constants remain unadvertised. Tasks 6.1-6.5 make handler/capability equality executable.

## 6. Alignment Decision

`pass`

No PRD requirement is missing from OpenSpec, no OpenSpec capability exceeds the approved Wave 1A product scope, and deferred T-102/T-103/T-202/T-501 work is explicitly fenced. Implementation remains blocked until the spec review and release-readiness evidence gates are approved.

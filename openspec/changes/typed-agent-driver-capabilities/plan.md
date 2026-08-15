# Typed Agent Driver Capability Plan

## 1. Scope

First slice for `typed-agent-driver-capabilities`.

## 2. In Scope

- Add capability DTOs to `homie-proto`.
- Add Swift protocol DTOs/method vocabulary for the read-only query.
- Add `homie-engine::driver` abstraction with default unsupported behavior and fake driver tests.
- Add read-only Engine method `session.capabilities`.
- Add Engine tests for missing session, manifest-only session and fake-driver capability query.
- Record evidence under `docs/verification/typed-agent-driver-capabilities/`.

## 3. Out Of Scope

- Real Codex/Claude/OpenCode provider driver.
- `session.steer`, `session.cancel_turn`, `agent.models` or other control actions.
- MCP tool surface changes.
- UI behavior changes.
- Replacing manifest, PTY, holder, output log, screen reducer or session persistence authority.

## 4. Design

The first slice exposes capability discovery without executing any typed control action.

Wire additions:

```text
session.capabilities
```

Params:

```text
{ "sessionID": "s_..." }
```

Result:

```text
{
  "sessionID": "s_...",
  "capabilities": {
    "prompt": false,
    "cancelTurn": false,
    "steerMessage": false,
    "respondPermission": false,
    "modelDiscovery": false,
    "nativeResumeCursor": false,
    "rollback": false,
    "fork": false,
    "usageEvents": false,
    "backgroundWork": false
  }
}
```

All real manifest-backed sessions return unsupported/default false in this slice.
Only fake-driver tests exercise non-default capability values.

## 5. Authority Rules

- Session lifecycle authority remains holder/PTY/output log/screen reducer.
- Capability query is read-only.
- No typed event changes visible session status in this slice.
- No provider payload or prompt content is recorded by driver tests.

## 6. Evidence

- Spec review: `docs/verification/typed-agent-driver-capabilities/spec-review-report.md`
- Functional cases: `docs/verification/typed-agent-driver-capabilities/functional-cases.md`
- Functional verification: `docs/verification/typed-agent-driver-capabilities/functional-verification-report.md`
- Code review: `docs/verification/typed-agent-driver-capabilities/code-review-round-1.md`, `code-review-round-2.md`
- Release readiness: `docs/verification/typed-agent-driver-capabilities/release-readiness-report.md`

## 7. Risks

| Risk | Control |
|---|---|
| Query expands into action API | Only `session.capabilities` is in scope |
| Real provider assumptions leak into abstraction | Use fake driver only |
| Status authority changes | Query is read-only and status tests assert no mutation |
| Wire drift | Add Swift/Rust DTO tests |

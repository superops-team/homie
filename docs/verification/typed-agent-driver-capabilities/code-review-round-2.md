# Typed Agent Driver Capability Code Review Round 2

## 1. Scope

Second-pass review focused on hidden authority, persistence, and protocol risks.

## 2. Hidden Risk Review

| Risk | Result | Evidence / Handling |
|---|---|---|
| Capability state could become persisted session truth | pass: no `SessionRecord` field was added; query computes capabilities on demand |
| New wire method could imply action support | pass: only read-only `session.capabilities` exists; no steer/cancel/model action methods added |
| Real sessions might accidentally expose fake capabilities | pass: fake capabilities are keyed only by internal `__fake_driver__` id used in tests; real manifest sessions default false |
| Query could alter visible status | pass: no status reducer interaction; test asserts record equality before/after query |
| Swift/Rust DTO mismatch could break clients | pass: both sides define same method name and camelCase capability fields; focused tests pass |
| MCP/UI behavior could change unintentionally | pass: no MCP or UI files changed for this slice |

## 3. Not Changed

- No provider adapter was added.
- No MCP tool was changed.
- No UI behavior changed.
- No session persistence schema changed.
- No PTY, holder, output log or screen reducer authority changed.

## 4. Conclusion

No P0/P1 hidden risks remain for this first slice.

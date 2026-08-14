# Local Shell TERM Shortcuts Plan

## Scope

Fix local shell/generic argv PTY environment so shell shortcuts and terminal
capabilities work in newly spawned local shell sessions.

## In Scope

- Add a local PTY environment helper in `homie-engine`.
- Apply it to local explicit argv / shell spawn path.
- Reuse the same shell/generic PTY environment helper for the remote
  non-binary shell/generic path to prevent TERM policy drift.
- Record the durable Engine session runtime contract in `specs/`.
- Add Engine tests for TERM and NO_COLOR behavior.
- Add a real shell-session test that prints `$TERM`.

## Out Of Scope

- Existing shell sessions.
- TerminalPane key adapter changes.
- Remote session behavior changes.
- Manifest agent spawn behavior unless it already uses the explicit argv path.

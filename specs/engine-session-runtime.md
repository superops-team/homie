# Engine Session Runtime Contract

## 1. Purpose

This spec defines durable runtime contracts for Homie's Rust engine session
launch path. It covers behavior that affects local and remote agent processes,
PTY child environments, and the control protocol.

## 2. Authority

`homie-engine` is the daemon/runtime authority for:

- `session.spawn` request handling;
- local PTY process launch;
- remote helper launch request construction;
- session supervision and screen state;
- environment normalization for PTY children.

The GPUI app and Swift clients may request sessions and send input, but they do
not decide the final child-process environment.

## 3. Shell PTY Environment

Every newly spawned local or remote shell/generic argv PTY session must receive
a terminal-capable environment:

- inherited `TERM` is removed;
- inherited `NO_COLOR` is removed;
- final `TERM` is set to `xterm-256color`;
- unrelated inherited variables such as `PATH` are preserved.

This applies to manifests with no binary, including first-class shell sessions
and generic command sessions that are launched through explicit argv. Existing
live sessions are not retroactively modified.

Manifest-backed agent launches use the manifest descriptor environment path.
That path already asserts terminal color capability and may additionally set
agent-specific variables such as `COLORTERM` or manifest `env` overrides.

## 4. Verification Contract

Changes affecting shell/generic PTY environment must include:

1. a helper-level regression proving `TERM` replacement and `NO_COLOR` removal;
2. an engine spawn regression proving `session.spawn` produces a shell process
   with `TERM=xterm-256color`;
3. a socket or equivalent real-control-path regression when user-visible shell
   behavior is affected.

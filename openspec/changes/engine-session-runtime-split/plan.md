# Engine Session Runtime Split Plan

## 1. Scope

Implement the P0 refactor that (a) splits `session.rs` (2,888 lines) into cohesive
submodules and (b) sinks spawn/resume/migrate business logic from
`control/handlers.rs` into the session/registry domain, leaving handlers as thin
decode→domain→encode adapters. This is the "session internal deepening" deferred by
`engine-registry-session-split` and the "spawn/resume/migrate sinking" residual
documented by `engine-control-wire-runtime-split`.

## 2. In Scope

- Split `homie/crates/homie-engine/src/session.rs` into
  `session/{lifecycle,screen,pty,status}.rs` + `session/mod.rs`, each file < 800 lines.
- Re-export `Session`/`SessionView` public types unchanged from `session/mod.rs`.
- Sink resume spec construction (`resume_spec`, `remote_resume_spec`) and the
  spawn-spec assembly path into the session/registry domain.
- Thin the spawn/resume/migrate handlers to decode→domain→encode.
- Add focused unit tests proving each split seam preserves behavior.

## 3. Out Of Scope

- `ControlMessage` wire shape / method names / JSON semantics changes.
- `Session` public method signature or behavior changes (pure relocation).
- New persistence backends (SQLite/rocksdb — `persistence-incremental-state` phase 2).
- Session state-machine semantic rework (status reducer moves verbatim).
- Real provider typed driver integration (`typed-agent-driver-capabilities`).
- `homie-proto/src/control.rs` protocol definition changes.

## 4. Design

Follow `prd-spec/refactors/engine-session-runtime-split/2026-08-19-engine-session-runtime-split-design.md`.

Session split slices:

1. S1 `session/status.rs` — status reducer: `feed_signal`/`claude_hook`/
   `observe_prompt_input`/`capture_prompt_title`/`status` + related helpers.
2. S2 `session/screen.rs` — screen/grid: `SessionView`/`PromptInputState`/`Shared`/
   `RemoteGridState`/`GridSignature`/`GridWake` + `grid_update_if_changed`/
   `screen_lines`/`read_scrollback*`/`scroll`.
3. S3 `session/pty.rs` — PTY I/O: `Transport` + `write_raw`/`send_text`/`paste_text`/
   `submit_input`/`write_input`/`resize`/`read_output`/`screen_size`/`child_pid`.
4. S4 `session/lifecycle.rs` — lifecycle: spawn/adopt/attach/resume/migrate + spec
   structures (`SessionSpec`/`HolderConfig`/`RemoteSessionSpec`/`RemoteAdoptSpec`/
   `RemoteLaunchCleanup`/`DeferredLaunch`/`LaunchHandoff`/`DeferredState`).
5. S5 `session/mod.rs` — `Session` public type + re-exports + submodule assembly.

Handler sinking slices:

6. S6 — extract resume spec construction into `session/lifecycle` (or a new
   `session/launch.rs`), parameterized by a launch context (injection, socket path,
   logs dir, holder) instead of `ControlServer` fields.
7. S7 — thin `session_spawn`/`session_spawn_remote`/`session_resume`/
   `session_resume_from_history`/`session_migrate` to decode→domain→encode.

Each slice keeps `cargo test -p homie-engine` green.

## 5. Evidence

- `docs/verification/engine-session-runtime-split/spec-review-report.md`
- `docs/verification/engine-session-runtime-split/functional-cases.md`
- `docs/verification/engine-session-runtime-split/functional-verification-report.md`
- `docs/verification/engine-session-runtime-split/failure-model.md` (PTY lifecycle Tier 3)
- `docs/verification/engine-session-runtime-split/code-review-round-*.md`
- `docs/verification/engine-session-runtime-split/release-readiness-report.md`

## 6. Risks

| Risk | Control |
|---|---|
| Behavior regresses during relocation | Existing green suite as refactor baseline; move slice-by-slice |
| Hidden coupling between Session fields and submodules | `pub(crate)`/`pub(super)` sharing; `Session` stays single public type |
| PTY process lifecycle regression (Tier 3) | Failure model: process residue, partial launch, resume race + stress evidence |
| Handler sinking breaks wire shape | golden fixture / protocol_contract unchanged gate |
| Scope creep into state-machine rewrite | Only move responsibilities, no new behavior |

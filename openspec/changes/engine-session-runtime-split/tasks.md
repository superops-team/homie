# Engine Session Runtime Split Tasks

## T1: Spec review and functional cases

- Deliverables: `docs/verification/engine-session-runtime-split/spec-review-report.md`,
  `docs/verification/engine-session-runtime-split/functional-cases.md`,
  `openspec/changes/engine-session-runtime-split/*`.
- Acceptance: `specs/engine-session-runtime.md` boundary evaluated; contract confirmed
  unchanged; split slices enumerated.

## T2: Split status reducer (S1)

- Deliverables: `session/status.rs`, `session/mod.rs`.
- Acceptance: `feed_signal`/`claude_hook`/`observe_prompt_input`/`capture_prompt_title`/
  `status` move verbatim; existing status tests green.

## T3: Split screen/grid (S2)

- Deliverables: `session/screen.rs`.
- Acceptance: `SessionView`/`GridSignature`/`GridWake` + scrollback move; screen tests green.

## T4: Split PTY I/O (S3)

- Deliverables: `session/pty.rs`.
- Acceptance: `Transport` + read/write/resize move; pty tests green.

## T5: Split lifecycle (S4)

- Deliverables: `session/lifecycle.rs`.
- Acceptance: spawn/adopt/attach/resume + spec structures move; lifecycle tests green.

## T6: Finalize session/mod.rs (S5)

- Deliverables: `session/mod.rs`.
- Acceptance: `session.rs` replaced by `session/`; each submodule < 800 lines;
  `Session`/`SessionView` public API unchanged.

## T7: Sink resume spec construction (S6)

- Deliverables: `session/lifecycle.rs` (or `session/launch.rs`) launch-context helpers.
- Acceptance: `resume_spec`/`remote_resume_spec` business logic moves to domain;
  handler calls domain entry; thin-adapter unit tests added.

## T8: Thin handlers (S7)

- Deliverables: `control/handlers.rs`.
- Acceptance: spawn/resume/migrate handlers decode→domain→encode; no private field
  pokes; `control/handlers.rs` reduced; control tests green.

## T9: Final verification and review

- Deliverables: `docs/verification/engine-session-runtime-split/*` incl. failure model
  (PTY Tier 3) and mutation/adversarial evidence.
- Acceptance: `cargo test -p homie-engine` green; `cargo fmt --check`; `cargo clippy -D warnings`;
  release readiness report exists.

## Status (2026-08-19)

- T1 ✅ spec review + functional cases 已写（`docs/verification/engine-session-runtime-split/`）。
- T2–T6 ✅ session 拆分完成，单文件均 < 800 行，`cargo check`/`test`/`clippy`/`fmt` 全绿。
- T7 ✅ resume spec 下沉至 `session/launch.rs`，`LaunchContext` 参数化，handler 薄适配。
- T8 ✅ `session_spawn` → `session::spawn_spec`、`session_spawn_remote` → `session::remote_spawn_spec`
  下沉完成；`session_resume`/`session_resume_from_history` 已薄化。`session_migrate` 的迁移阶段
  （WIP commit / push / hard-sync）属编排而非 spawn-spec 组装，保留在 handler，其 resume 调用已下沉。
  `control/handlers.rs` 由 1,461 行降至 1,179 行，无私有字段 pokes。
- T9 ✅ code-review round 已完成（`docs/verification/engine-session-runtime-split/code-review-report.md`，
  两轮审查 + 去重修复）。统一打 tag 待用户确认后再执行。

# Local Shell TERM Shortcuts Tasks

## T1: PRD and functional cases

- Verification Cases: FC-01, FC-02

## T2: Add local PTY environment helper

- Deliverables: `homie/crates/homie-engine/src/control.rs`
- Acceptance:
  - helper removes inherited `TERM` and `NO_COLOR`;
  - helper sets `TERM=xterm-256color`.
- Verification Cases: FC-03

## T3: Apply helper to local shell/generic argv spawn

- Deliverables: `homie/crates/homie-engine/src/control.rs`
- Acceptance:
  - local explicit argv path uses helper.
- Verification Cases: FC-04

## T4: Keep local and remote shell/generic TERM policy aligned

- Deliverables: `homie/crates/homie-engine/src/control.rs`
- Acceptance:
  - remote non-binary shell/generic path reuses the same shell PTY environment helper;
  - no manifest-backed agent launch behavior is changed.
- Verification Cases: FC-04, FC-06

## T5: Record durable Engine runtime contract

- Deliverables: `specs/engine-session-runtime.md`
- Acceptance:
  - spec states shell/generic PTY sessions remove inherited `TERM` and `NO_COLOR`;
  - spec states final `TERM=xterm-256color`.
- Verification Cases: FC-01, FC-02

## T6: Add real shell TERM regression

- Deliverables: Engine tests.
- Verification Cases: FC-05

## T7: Run static gates

- Verification Cases: FC-06

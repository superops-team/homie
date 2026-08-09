# Functional Cases: Diri host.locate_repo

```yaml
change_id: diri-host-locate-repo
beads: homie-37j
```

## FC-DHLR-001: Diri protocol spelling

- Command: `cargo test -p homie-proto host_locate_repo_round_trips_diri_spelling -- --nocapture`
- Expected:
  - Params serialize to `host`, `originURL`, `sessionID`.
  - Empty result deserializes to no path and no origin.
  - Found result deserializes from `path` plus `originURL`.

## FC-DHLR-002: Origin discovery and match

- Command: `cargo test -p homie-remote --test host_locate_repo -- locates_matching_candidate_by_origin --nocapture`
- Expected:
  - Source cwd origin is read from `.git/config`.
  - Candidate with same origin returns `path` and `originURL`.

## FC-DHLR-003: Not cloned classification

- Command: `cargo test -p homie-remote --test host_locate_repo -- returns_origin_without_path_when_repo_is_not_cloned --nocapture`
- Expected:
  - Source cwd has origin.
  - Candidates have no matching origin.
  - Result contains `originURL` and no `path`.

## FC-DHLR-004: No origin classification

- Command: `cargo test -p homie-remote --test host_locate_repo -- returns_empty_result_when_cwd_has_no_origin --nocapture`
- Expected:
  - Non-git cwd or repo without origin returns empty result.

## FC-DHLR-005: CLI fixture E2E

- Command: `cargo test -p homie-cli --test host_locate_repo_cli -- --nocapture`
- Expected:
  - `homie host locate-repo --cwd <source> --candidate <clone>` returns matched path.
  - `--origin-url <url> --candidate <other>` returns origin without path.

## FC-DHLR-006: Quality and parity gates

- Commands:
  - `cargo check -p homie-remote -p homie-client -p homie-cli`
  - `cargo clippy -p homie-remote -p homie-client -p homie-cli --all-targets -- -D warnings`
  - `git diff --check -- crates/homie-remote crates/homie-proto crates/homie-client crates/homie-cli specs/remote-node-handoff/README.md prd-spec/features/diri-host-locate-repo openspec/changes/diri-host-locate-repo docs/verification/diri-host-locate-repo docs/research/diri-parity-lock.md`
  - `make parity-lock`
- Expected:
  - All commands pass.
  - `REM-003` remains `partial` until real remote E2E exists, but evidence includes `host.locate_repo`.

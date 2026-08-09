# Code Review Report: Diri Proto Node Account Login Fixtures

```yaml
change_id: diri-proto-node-account-login
beads: homie-05q
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Missing DTO | `crates/homie-proto/src/lib.rs` | Node account/login/provider-call DTOs were absent. | fixed: added account, installation, login, and provider call structs. |
| medium | Wire contract | DTO serde | Diri uses camelCase with many optional fields omitted when absent. | fixed and tested. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| low | Scope | proto crate | DTOs must not imply account storage or login runtime. | accepted: runtime remains pending. |
| low | Provider map | account catalog | Provider default map keys must serialize lowercase. | pass: fixture covers defaults. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-proto node_account_login_match_diri_wire -- --nocapture` | pass |
| `cargo test -p homie-proto --tests` | pass |
| `cargo check -p homie-proto` | pass |
| `cargo clippy -p homie-proto --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |

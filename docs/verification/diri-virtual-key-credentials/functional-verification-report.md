# Functional Verification Report

```yaml
change_id: diri-virtual-key-credentials
beads: homie-e1s
dev_loop_step: 7
status: pass
source_cases: docs/verification/diri-virtual-key-credentials/functional-cases.md
```

## 1. Summary

All P0/P1 functional cases designed for the first Diri/Homie virtual key credential parity slice passed through real `homie-llm` public interfaces or document gates.

## 2. Results

### FC-DVKC-001

| Field | Value |
|-------|-------|
| Status | pass |
| Command | `cargo test -p homie-llm issued_virtual_key_validates_only_for_matching_scope -- --nocapture` |
| Exit code | 0 |
| Evidence | Matching scope validated; wrong session returned `ScopeMismatch` |

### FC-DVKC-002

| Field | Value |
|-------|-------|
| Status | pass |
| Command | `cargo test -p homie-llm revoked_expired_and_unknown_virtual_keys_are_rejected -- --nocapture` |
| Exit code | 0 |
| Evidence | Expired key returned `Expired`; revoked key returned `Revoked`; unknown key returned `NotFound`; rendered error did not contain the virtual key secret |

### FC-DVKC-003

| Field | Value |
|-------|-------|
| Status | pass |
| Command | `cargo test -p homie-llm scope_denied_covers_profile_provider_and_model -- --nocapture` |
| Exit code | 0 |
| Evidence | Wrong agent profile and provider returned `ScopeMismatch`; disallowed model returned `ModelNotAllowed` |

### FC-DVKC-004

| Field | Value |
|-------|-------|
| Status | pass |
| Command | `cargo test -p homie-llm managed_proxy_config_serializes_without_raw_provider_key -- --nocapture` |
| Exit code | 0 |
| Evidence | Managed config JSON contained local proxy URL and `hv_` virtual key; did not contain fake raw provider key, `Authorization`, `secretRef`, or `providerApiKey` |

### FC-DVKC-005

| Field | Value |
|-------|-------|
| Status | pass |
| Command | `cargo test -p homie-llm raw_provider_key_is_rejected_for_cross_module_destinations -- --nocapture` |
| Exit code | 0 |
| Evidence | Remote node, MCP tool, managed agent config, and log/event destinations rejected fake raw provider key; virtual-key proxy config payload was allowed; error strings did not include fake raw provider key |

### FC-DVKC-006

| Field | Value |
|-------|-------|
| Status | pass |
| Commands | `rg -n "Cross-Spec Mandatory Gates|Raw Provider Key Forbidden Matrix|Diri Behavior Parity" specs/virtual-key-credentials/README.md`; `rg -n "FC-DVKC-00[1-6]" openspec/changes/diri-virtual-key-credentials/tasks.md openspec/changes/diri-virtual-key-credentials/alignment-report.md`; `rg -n "FR-[1-5]" openspec/changes/diri-virtual-key-credentials/alignment-report.md` |
| Exit code | 0 for all commands |
| Evidence | Component spec includes required sections; OpenSpec tasks and alignment report include FC-DVKC-001 through FC-DVKC-006 and FR-1 through FR-5 |

### FC-DVKC-007

| Field | Value |
|-------|-------|
| Status | pass_with_note |
| Commands | `cargo test -p homie-llm`; `cargo fmt --package homie-llm -- --check`; `cargo check -p homie-llm`; `cargo clippy -p homie-llm --all-targets -- -D warnings`; `cargo check --workspace`; `git diff --check -- <lane paths>` |
| Exit code | 0 for listed focused and workspace check commands |
| Note | `cargo fmt --all -- --check` failed in out-of-scope `crates/homie-storage/tests/diri_storage_indexing.rs`; this file was already outside the allowed write scope and was not modified |

## 3. Coverage

| Requirement | Result |
|-------------|--------|
| FR-1 Component spec gates | pass |
| FR-2 Virtual key lifecycle | pass |
| FR-3 Managed agent config secretless | pass |
| FR-4 Raw key propagation denial | pass |
| FR-5 Secretless errors/evidence | pass_with_note |

## 4. Failure Handling

No functional case failed after implementation. The only remaining quality note is an out-of-scope repository-level rustfmt failure in `crates/homie-storage/tests/diri_storage_indexing.rs`; it is not caused by this lane and was not changed per the user's write-scope constraint.


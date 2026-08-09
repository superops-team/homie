# SDD/TDD Task Report

```yaml
change_id: diri-virtual-key-credentials
beads: homie-e1s
dev_loop_step: 5_and_6
status: pass
openspec_tasks:
  - T-001
  - T-002
  - T-003
  - T-004
  - T-005
```

## 1. Summary

Implemented the first Diri/Homie credential parity security slice:

- Strengthened `specs/virtual-key-credentials/README.md` with Diri/Homie adaptation, cross-spec mandatory gates, raw provider key forbidden matrix, and first-stage implementation contract.
- Added `ManagedLlmProxyConfig` as the agent-visible secretless LLM proxy config.
- Added `CredentialDestination` and `CredentialPropagationPolicy` to fail closed when raw provider keys are about to enter remote node, MCP, managed agent config, or log/event payloads.
- Extended virtual key tests to cover unknown key rejection, profile/provider/model scope denial, secretless managed config serialization, and no raw key propagation.

## 2. RED

Command:

```bash
cargo test -p homie-llm --test virtual_key
```

Result:

- Exit code: 101.
- Expected failure:
  - unresolved imports `CredentialDestination`, `CredentialPropagationPolicy`, `ManagedLlmProxyConfig`;
  - missing `VirtualKeyError::RawProviderKeyForbidden`;
  - missing `serde_json` dev dependency for the new serialization contract test.

This confirmed the new tests were exercising behavior not yet implemented.

## 3. GREEN

Implemented:

- `crates/homie-llm/src/lib.rs`
  - `ManagedLlmProxyConfig`
  - `CredentialDestination`
  - `CredentialPropagationPolicy`
  - `VirtualKeyError::RawProviderKeyForbidden`
  - safe `Debug` for `IssuedVirtualKey`, `ManagedLlmProxyConfig`, and `CredentialPropagationPolicy`
  - serde support for safe virtual-key config fields
- `crates/homie-llm/Cargo.toml`
  - `serde_json` as a dev dependency from workspace.
- `crates/homie-llm/tests/virtual_key.rs`
  - 5 public contract tests covering lifecycle, scope, config serialization, and raw-key propagation denial.

Focused command:

```bash
cargo test -p homie-llm --test virtual_key
```

Result:

- Exit code: 0.
- 5 tests passed:
  - `issued_virtual_key_validates_only_for_matching_scope`
  - `revoked_expired_and_unknown_virtual_keys_are_rejected`
  - `scope_denied_covers_profile_provider_and_model`
  - `managed_proxy_config_serializes_without_raw_provider_key`
  - `raw_provider_key_is_rejected_for_cross_module_destinations`

## 4. Task Results

| Task | Result | Evidence |
|------|--------|----------|
| T-001 | pass | Spec sections added: Diri behavior parity, cross-spec mandatory gates, raw key forbidden matrix |
| T-002 | pass | Lifecycle and scope tests pass |
| T-003 | pass | Managed config serialization test passes |
| T-004 | pass | Raw provider key propagation denial test passes |
| T-005 | pass_with_note | Focused and compile gates pass; workspace fmt has a pre-existing out-of-scope failure |

## 5. Supervision Log

- Kept edits inside the user-approved write scope.
- Did not modify `crates/homie-remote`, MCP files, runtime files, or non-lane specs.
- Kept the implementation minimal and did not add HTTP proxy, secret envelope, storage, or remote behavior.
- First code-review pass removed raw provider key storage from `CredentialPropagationPolicy`; the policy is now zero-state and receives raw key material only for one validation call.
- Second code-review pass removed virtual key secret leakage from `Debug` output for `IssuedVirtualKey` and `ManagedLlmProxyConfig`.
- Recorded the workspace fmt failure as out-of-scope instead of formatting another lane's file.


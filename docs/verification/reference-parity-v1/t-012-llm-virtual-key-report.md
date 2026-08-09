# T-012 LLM Proxy Virtual Key Usage Report

```yaml
change_id: reference-parity-v1
openspec_task: T-012
beads: homie-eyi
status: pass
functional_cases:
  - FC-013
  - FC-018
```

## 1. Summary

T-012 implemented the first LLM security slice: virtual key issue, validation, scope enforcement, expiry and revoke behavior.

Implemented:

- `homie-llm` crate.
- `VirtualKeyScope`.
- `VirtualKeyRequest`.
- `IssuedVirtualKey`.
- `VirtualKeyClaims`.
- `InMemoryVirtualKeyStore`.
- `VirtualKeyError`.

This is not a provider proxy implementation yet. It establishes the virtual-key contract that later proxy and runtime work must use.

## 2. RED

Added failing virtual key tests:

- `crates/homie-llm/tests/virtual_key.rs`

The tests require:

- Matching session/profile/provider/model scope validates.
- Wrong session scope fails.
- Expired keys fail.
- Revoked keys fail.
- Disallowed model fails.

## 3. GREEN

Implemented:

- `crates/homie-llm/Cargo.toml`
- `crates/homie-llm/src/lib.rs`
- workspace registration in `Cargo.toml`

## 4. Verification

Focused command:

```bash
cargo test -p homie-llm
```

Result:

- Exit code: 0
- Virtual key tests: 3 passed
- Doc tests: 0 tests

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/LLM/proto/storage tests passed.

Safety checks:

```bash
rg -n -i "<old-reference-name-pattern>" .
git diff --check
```

Result:

- Old reference name scan: no matches.
- Markdown/patch whitespace check: pass.

## 5. Remaining Scope

Still deferred:

- OpenAI-compatible HTTP proxy.
- Provider routing and streaming.
- Usage/cost writes into SQLite.
- Secret envelope and persistent virtual key repository.
- Runtime managed-agent env injection.


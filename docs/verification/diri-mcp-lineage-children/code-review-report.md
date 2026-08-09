# Code Review Report: Diri MCP Lineage Children

```yaml
change_id: diri-mcp-lineage-children
beads: homie-s82
status: pass
reviewed_at: 2026-08-08
```

## Findings

| Severity | Category | Location | Evidence and impact | Status |
|----------|----------|----------|---------------------|--------|
| medium | Correctness | `crates/homie-client/src/lib.rs` | Client methods called storage APIs directly, requiring explicit error mapping to keep ClientError layering. | fixed: storage errors map through RuntimeError. |
| low | Scope | parity lock | Direct children support is not full Diri lineage permission enforcement. | accepted: API-005 remains partial. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test mcp_lineage_children_cli -- --nocapture` | pass |
| `cargo check -p homie-storage -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-storage -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |


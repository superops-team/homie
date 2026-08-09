# E2E Report

```yaml
change_id: diri-virtual-key-credentials
beads: homie-e1s
dev_loop_step: 10
status: pass_for_first_stage
scope: crate_level_security_contract
```

## 1. Scope

This first-stage E2E is not a network proxy or remote-node E2E. The implemented slice has no HTTP endpoint, no remote node server, and no MCP runtime wiring by design.

The end-to-end path verified here is the security contract inside `homie-llm`:

```text
issue virtual key
  -> build managed agent proxy config
  -> serialize agent-visible config without raw provider key
  -> validate/reject scoped key use
  -> reject raw provider key propagation to remote/MCP/agent/log destinations
```

## 2. Commands

| Command | Exit code | Result |
|---------|-----------|--------|
| `cargo test -p homie-llm` | 0 | pass |
| `cargo check -p homie-llm` | 0 | pass |
| `cargo clippy -p homie-llm --all-targets -- -D warnings` | 0 | pass |
| `cargo check --workspace` | 0 | pass |

## 3. Cross-Check Against Functional Cases

| Functional case | E2E coverage |
|-----------------|--------------|
| FC-DVKC-001 | Matching scope issue/validate path covered |
| FC-DVKC-002 | Expired, revoked and unknown key rejection path covered |
| FC-DVKC-003 | Profile/provider/model denial path covered |
| FC-DVKC-004 | Agent-visible config serialization path covered |
| FC-DVKC-005 | Remote/MCP/agent/log destination rejection path covered |
| FC-DVKC-006 | Document alignment gate covered |
| FC-DVKC-007 | Focused quality gates covered; workspace fmt note remains out of scope |

## 4. Not Run

| Scenario | Status | Reason |
|----------|--------|--------|
| OpenAI-compatible HTTP proxy request | not_run | Full proxy is out of scope for this foundation-security slice |
| Remote node handoff carrying scoped proxy credential | not_run | Remote node files/specs are outside this lane write scope |
| MCP stdio tool with LLM access | not_run | MCP automation files/specs are outside this lane write scope |
| Real provider credential request | not_run | Tests must not require or expose real provider credentials |

## 5. Conclusion

The first-stage crate-level security E2E passes. Full remote/MCP/proxy E2E remains a follow-up for their respective lanes after they consume this L0 credential contract.


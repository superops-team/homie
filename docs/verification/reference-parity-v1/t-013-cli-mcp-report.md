# T-013 CLI Hook Notify MCP Report

```yaml
change_id: reference-parity-v1
openspec_task: T-013
beads: homie-e54
status: pass
functional_cases:
  - FC-014
```

## 1. Summary

T-013 implemented a minimal CLI automation contract slice.

Implemented:

- `homie hook <event>` fail-open placeholder that prints `{}`.
- `homie notify ...` fail-open placeholder.
- `homie mcp-tools` listing the required tool names.
- `homie mcp-call --tool list_agents`.
- `homie mcp-call --tool whoami`.
- CLI grammar tests for automation commands.

This is not a live daemon MCP bridge yet. It establishes CLI grammar and fail-open command behavior for later runtime/MCP integration.

## 2. RED

Added parser tests inside:

- `crates/homie-cli/src/main.rs`

The tests require hook, notify, mcp-tools and mcp-call command parsing.

## 3. GREEN

Implemented:

- CLI command variants and handlers in `crates/homie-cli/src/main.rs`.

## 4. Verification

Focused command:

```bash
cargo test -p homie-cli
cargo run -q -p homie-cli -- mcp-tools
cargo run -q -p homie-cli -- hook SessionStart
cargo run -q -p homie-cli -- mcp-call --tool list_agents
```

Result:

- Exit code: 0 for all commands.
- `mcp-tools` prints the required tool list.
- `hook SessionStart` prints `{}`.
- `mcp-call --tool list_agents` prints `{ "ok": [] }`.

Workspace regression command:

```bash
cargo test --workspace
```

Result:

- Exit code: 0
- Homie agents/app/CLI/context/LLM/memory/orchestrator/proto/runtime/storage/task/term/UI tests passed.

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

- Real stdio MCP server loop.
- Tool identity and permission enforcement.
- Runtime-backed tool execution.
- Browser/test_run implementations.


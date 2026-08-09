# Code Review Report: Diri MCP Worktree Tools

```yaml
change_id: diri-mcp-worktree-tools
beads: homie-4wg
status: pass
reviewed_at: 2026-08-08
```

## Round 1: Explicit Defect Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| high | Missing behavior | `crates/homie-cli/src/main.rs` | Worktree tools were advertised in MCP descriptors but returned unsupported. | fixed: `create_worktree`, `list_worktrees`, and `remove_worktree` dispatch to `HomieClient::worktree_*`. |
| high | Path realism | `mcp_worktree_tools_cli.rs` | Tests must prove real git worktree side effects rather than static JSON. | fixed: test initializes a real repo, creates a real worktree, lists it, removes it, and checks filesystem state. |
| medium | Parameter contract | MCP payload parser | Diri MCP contract uses `repo` and `path`; internal DTO uses `repo_path`/`worktree_path`. | fixed: MCP boundary converts Diri arguments to DTO fields. |

## Round 2: Hidden Risk Review

| Severity | Category | Location | Finding | Status |
|----------|----------|----------|---------|--------|
| medium | Duplicate logic | `create_worktree` branch | Reimplementing git commands in MCP would drift from CLI/runtime. | pass: implementation only calls `HomieClient`. |
| medium | Error semantics | missing param tests | Unsupported errors should become invalid params once a tool is implemented. | pass: missing `repo` and missing `path` return `-32602`. |
| low | Scope | parity lock | This does not complete app worktree sheet E2E. | accepted: UI/GIT rows remain `partial`. |

## Verification

| Command | Result |
|---------|--------|
| `cargo test -p homie-cli --test mcp_worktree_tools_cli -- --nocapture` | pass |
| `cargo test -p homie-cli --test worktree_cli -- --nocapture` | pass |
| `cargo check -p homie-client -p homie-cli` | pass |
| `cargo clippy -p homie-client -p homie-cli --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |
| scoped `git diff --check` | pass |
| `make parity-lock` | pass |

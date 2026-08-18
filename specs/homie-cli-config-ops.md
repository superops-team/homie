# Homie CLI Config Ops Contract

## 1. Purpose

This spec defines the durable contracts for Homie's CLI configuration surface:
the `config` command grammar, the shared local config file format, `doctor` /
`fix` semantics, and the `homie` skill. It is the long-lived authority for any
change that affects how a human or an AI agent inspects, records, diagnoses, or
repairs Homie's LLM gateway configuration.

## 2. Authority

The `homie` CLI (Swift, `Sources/homie-cli/Homie.swift`) is the user-facing
entry point. It reads and writes one canonical local config file, shared with
the Rust `homie-gateway` binary, and reads the gateway's SQLite store read-only.
The Rust `homie-engine::inject::injection_args()` remains the single source of
truth for agent config injection; the CLI never reimplements it.

## 3. Config File Contract

- Canonical file: `homie.local.json`, JSON, single source shared by Swift CLI
  and Rust gateway.
- Default path `~/.config/homie/homie.local.json`; overridable by `HOMIE_CONFIG`
  environment variable.
- It is git-ignored (`homie.local.*`, `*.local.json`) and written atomically
  with owner-only `0600` permissions.
- Schema:

```jsonc
{
  "gateway":   { "listen": "127.0.0.1:7338", "masterKey": null },
  "upstream":  { "baseUrl": "https://api.openai.com/v1", "apiKey": "sk-..." },
  "models":    { "codex": "gpt-5.2-codex", "claude": "claude-sonnet-4-5" }
}
```

- `apiKey` and `masterKey` are secrets; they live only in this ignored file,
  never in git, logs, or agent-visible config.

## 4. Command Contract

### 4.1 `homie config show` / `config get`

- `show` prints gateway listen address, upstream `baseUrl`, masked `apiKey`
  (`sk-***<last4>` or `***`), model mapping, and the virtual key list with
  `last_used` (read-only from the gateway SQLite).
- `get <key-path>` prints one raw value (e.g. `upstream.baseUrl`).
- Secrets are always masked in output.

### 4.2 `homie config set`

- Settable keys: `upstream.baseUrl`, `upstream.apiKey`, `gateway.listen`,
  `gateway.masterKey`, `models.codex`, `models.claude`.
- Secrets may be provided via `--api-key-from-stdin` or environment to avoid
  shell history; never force a plain `--api-key` value onto argv.
- Writes are atomic and owner-only.

### 4.3 `homie config agent <codex|claude>`

- Emits the exact injection result produced by `homie-engine::inject::injection_args()`
  for the selected agent: Codex `-c` overrides, or Claude `ANTHROPIC_BASE_URL` /
  `ANTHROPIC_AUTH_TOKEN` environment.
- Output is JSON by default for scriptability; a human-readable form is
  available via a non-JSON flag.
- The CLI delegates to the Rust `homie-gateway inject --agent <agent>` to
  guarantee parity with real spawn-time injection.

### 4.4 `homie doctor`

- Retains the original three checks (daemon socket, `claude`/`codex` binaries,
  state file).
- Adds: gateway reachability, upstream credential presence/validity, virtual
  key effectiveness, and agent config pointing to the gateway (not a real
  provider).
- Each check reports `✓`/`✗`; any failure yields a non-zero exit code.

### 4.5 `homie fix`

- A finite, idempotent set of repair actions: port conflict, missing upstream
  credential, config drift (missing/corrupt file), and gateway-not-running.
- Each action probes first; if healthy, it is skipped. It never silently fills
  a real credential, and it never auto-spawns the daemon (to avoid lifecycle
  ambiguity).

## 5. Injection Parity Contract

The `config agent` preview and the actual spawn-time injection MUST derive from
the same Rust function (`injection_args()`). A unit test must assert the two
outputs are shape-equal. This prevents configuration drift between what an
agent previews and what it actually receives.

## 6. Virtual Key Read Contract

The CLI reads the gateway SQLite store read-only to surface virtual key status.
The CLI never writes to that store; key lifecycle (create/delete) is the
gateway's exclusive responsibility.

## 7. Security And Recovery

- Real keys and virtual keys live only in ignored files and local SQLite.
- CLI output and errors are masked; no secret is echoed.
- A corrupt `homie.local.json` is surfaced with a pointer to `homie fix`, which
  rebuilds a minimal valid file while preserving recoverable fields.

## 8. Skill Contract

The `homie` skill lives at `homie/.agents/skills/homie/SKILL.md` and documents
the CLI commands above, the masking/secret rules, and the stdin/env secret
entry path. It must instruct agents never to persist or echo real keys.

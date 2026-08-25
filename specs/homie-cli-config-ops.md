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
the Rust daemon (`homied-rs`), and reads the gateway's SQLite store read-only.
The Rust `homie-engine::inject` module remains the single source of truth for
agent config injection; the CLI never reimplements it (and no longer exposes an
injection preview — see §4.3).

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
  "models":    { "codex": "gpt-5.2-codex" }
}
```

- `models` is optional and defaults to `{}`. Empty or whitespace-only model
  values are equivalent to absent entries.
- The CLI must not create an empty `models.codex` placeholder. Setting
  `models.codex` to an empty or whitespace-only value removes that override so
  the agent keeps using its own default model.
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
  `gateway.masterKey`, `models.codex`.
- `models.codex` is an override, not a required startup setting: a non-empty
  value writes the override, while an empty or whitespace-only value deletes it.
- Secrets may be provided via `--api-key-from-stdin` or environment to avoid
  shell history; never force a plain `--api-key` value onto argv.
- Writes are atomic and owner-only.

### 4.3 `homie config agent` (removed)

- The `config agent` injection preview is **removed**. It previously delegated to
  the `homie-gateway inject` subcommand, which no longer exists now that the LLM
  gateway is embedded in the daemon and injection happens daemon-internally at
  spawn time.
- Claude Code receives no gateway injection at all (it uses its native Anthropic
  credentials; Homie manages only its hooks + MCP orchestration).
- Codex injection (gateway virtual key + model routing) is produced by
  `homie-engine::inject` at spawn time; there is no standalone preview surface.

### 4.4 `homie doctor`

- Retains the original three checks (daemon socket, `claude`/`codex` binaries,
  state file).
- Adds: daemon-embedded gateway reachability, upstream credential
  presence/validity, virtual key effectiveness, and agent config pointing to
  the local gateway (not a real provider).
- Each check reports `✓`/`✗`; any failure yields a non-zero exit code.

### 4.5 `homie fix`

- A finite, idempotent set of repair actions: port conflict, missing upstream
  credential, config drift (missing/corrupt file), and gateway-not-running.
- Each action probes first; if healthy, it is skipped. It never silently fills
  a real credential, and it never auto-spawns the daemon (to avoid lifecycle
  ambiguity).

## 5. Injection Parity Contract

Spawn-time injection is produced exclusively by `homie-engine::inject` inside
the daemon (Codex only). There is no separate CLI preview surface, so there is
no drift risk between a preview and real spawn-time injection. A unit test must
assert that a Codex spawn with gateway routing enabled receives the virtual key
and model routing produced by the inject module.

## 6. Virtual Key Read Contract

The CLI reads the gateway SQLite store read-only to surface virtual key status.
The CLI never writes to that store; key lifecycle (create/delete) is the
daemon's exclusive responsibility.

## 7. Security And Recovery

- Real keys and virtual keys live only in ignored files and local SQLite.
- CLI output and errors are masked; no secret is echoed.
- A corrupt `homie.local.json` is surfaced with a pointer to `homie fix`, which
  rebuilds a minimal valid file while preserving recoverable fields.

## 8. Skill Contract

The `homie` skill lives at `homie/.agents/skills/homie/SKILL.md` and documents
the CLI commands above, the masking/secret rules, and the stdin/env secret
entry path. It must instruct agents never to persist or echo real keys.

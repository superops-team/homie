# LLM Gateway Contract

> change_id: `llm-gateway-daemon-embed` · Beads: `homie-6md`

## 1. Purpose

This spec defines durable contracts for Homie's local LLM gateway: the virtual
key model, the wire protocol it serves to managed agents, upstream forwarding,
and usage recording. It is the long-lived authority for any change that affects
credential custody, virtual key issuance, or LLM proxying.

## 2. Authority

The LLM gateway is **embedded in the daemon** (`homied-rs`), not a standalone
process. It is implemented by the `homie-gateway` **library crate** (no binary),
which `homie-engine` links. The daemon hosts a single loopback HTTP listener that
serves the OpenAI-compatible proxy to managed agents (Codex and other
OpenAI-compatible agents).

It:

- serves an OpenAI Responses API (`POST /v1/responses`);
- authenticates callers with a virtual key (or the gateway master key);
- forwards requests to a configured OpenAI-compatible upstream provider;
- records per-virtual-key usage locally.

It binds `127.0.0.1` only. It never exposes the upstream provider key to the
caller. Anthropic Messages (`POST /v1/messages`) is **removed**; Claude Code is
no longer routed through the gateway and uses its native Anthropic credentials.

## 3. Virtual Key Model

- A virtual key is a random `sk-<uuid><uuid>` string issued per agent/session.
- **Issuance is embedded in the daemon**: at spawn time, for an agent whose
  manifest opts into gateway routing, the daemon calls the key store's `create`
  and injects the `sk-…` into the agent env. There is no separate `/admin/keys`
  HTTP issuance surface.
- Keys are stored locally with `id`, optional `label`, `created_at`,
  `last_used_at`. Only a SHA-256 hash of the key is stored (never the raw key);
  the raw key is returned exactly once at creation and never logged.
- `create`, `delete`, `list`, and `accept` are the only operations.
- Keys persist across daemon restarts (SQLite), unlike the upstream reference's
  in-memory map.
- A revoked key returns `401` and is never forwarded.

## 4. Auth Contract

- A request may present the key as `Authorization: Bearer <key>` or `x-api-key:
  <key>`; `Bearer` takes precedence.
- A configured master key is also accepted by either header.
- No master key configured + a non-loopback bind is a hard error. Loopback-only
  bind without a master key is permitted (local trust boundary) but must warn.
- `401` responses use the standard error body and never leak the upstream key.

## 5. Protocol Contract

- `POST /v1/responses`: OpenAI Responses wire shape, forwarded to the upstream,
  streaming (SSE) preserved.
- The upstream provider key is attached server-side only; caller payload keys
  are ignored.

## 6. Upstream Forwarding

- Single OpenAI-compatible upstream: `base_url` + `api_key` from local ignored
  config (`homie.local.json` under `~/.config/homie/`, overridable with
  `HOMIE_CONFIG` / `HOMIE_CONFIG_DIR`). JSON is used (not TOML) so the Swift CLI
  and this Rust daemon read and write the same bytes.
- Upstream errors are surfaced with a sanitized body; no upstream key or full
  sensitive prompt is echoed into logs.

## 7. Model Routing

- `homie.local.json` carries an optional `models` map (`codex` → upstream model
  name). It is `#[serde(default)]`; absent is valid.
- Before forwarding, the gateway rewrites the request body's top-level `model`
  string by route key: `POST /v1/responses` → `models["codex"]`.
- Override semantics: rewrite only when the `models` entry exists and its value
  is non-empty after trimming whitespace; otherwise pass the original `model`
  through unchanged. A non-JSON body or a
  missing/non-string top-level `model` is passed through unchanged (never an
  error).
- Empty model overrides from old or hand-edited configs are treated as
  unconfigured so a managed agent can still use its own default model.
- Usage recording uses the rewritten model, reflecting the model actually routed
  upstream.
- The daemon is the single source of truth for model routing; spawn-time
  injection does not separately set the model.

## 8. Usage Contract

- Each forwarded request records, per virtual key: `model`, `occurred_at`,
  `input_tokens`, `output_tokens` (estimated via `homie-usage::openai_estimate`
  when billed usage is absent).
- Usage is local and append-only; it is estimates, never authoritative billing.

## 9. Policy And Quota

- `homie.local.json` carries an optional `policy` section; it is
  `#[serde(default)]` and absent means no rate limiting or quota is applied
  (backward compatible with existing deployments).
- `policy.rate_limit.requests_per_minute` (per virtual key, minute-grained
  sliding window) is enforced in-memory before forwarding. Exceeding it returns
  `429` with a `rate_limit_error` body and does not forward upstream.
- `policy.quota.daily_token_limit` (per virtual key, per natural day) is enforced
  by aggregating `SUM(input_tokens + output_tokens)` over `gateway_usage`.
  Exceeding it returns `429` with a `quota_error` body and does not forward.
- A value of `0` for `daily_token_limit` or `requests_per_minute` is treated as
  "not configured" (no enforcement), avoiding accidental total lockout.
- Denied requests are recorded in a local `gateway_audit` table (event, key_id,
  occurred_at, reason); they are not written to `gateway_usage`, which records
  only actually-forwarded requests.
- Denial bodies and audit detail never contain the upstream key, master key,
  raw virtual key, model, or sensitive prompt.

## 10. Security And Recovery

- Upstream keys and virtual keys live only in local ignored files and local
  SQLite; never in git, logs, or agent-visible config.
- Daemon restart restores virtual keys from SQLite.
- Port conflicts fail the daemon with a clear error; the port is configurable.

## 11. Credential Source

- `homie.local.json` may carry an optional `credentialSource` field; it is
  `#[serde(default)]` and absent means `static` (read `upstream.apiKey`), fully
  backward compatible.
- `credentialSource: "node"` makes the daemon resolve the upstream credential
  dynamically from `homie-node` instead of a static key. The daemon never reads
  provider auth files directly; credential extraction/refresh is owned by
  `homie-node`.
- `homie-node` exposes library functions (`homie_node::credentials`) — a
  restricted, in-process credential resolver: `resolve_default_codex_credential`
  and `resolve_codex_api_key`. It returns only a short-lived upstream token
  (`kind`, `base_url`, `token`) for a given account profile. It never returns
  refresh tokens and never exposes arbitrary file reads. This is library
  embedding, not a cross-process RPC; the daemon links `homie-node` as a crate
  dependency.
- Phase 1 resolves only the Codex API-key mode (`OPENAI_API_KEY` in the
  profile-scoped `config_home/auth.json`). Codex ChatGPT-login token refresh is
  Phase 2 and out of scope for this contract revision. (Claude OAuth was removed
  along with Anthropic protocol support.)
- Resolved short-lived tokens are held in memory only: never written to SQLite,
  never logged, never sent to managed agents.
- Fallback semantics: when `credentialSource == "node"`, a failed resolve
  (node unreachable, not authenticated, or unsupported mode) falls back to the
  static `upstream.apiKey` if configured; otherwise the daemon returns a clear
  `503` configuration-error body that leaks no key or account data.
- Failed resolve attempts are recorded in `gateway_audit` with reason
  `credential_resolve_failed` (no token material in the audit detail).

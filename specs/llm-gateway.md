# LLM Gateway Contract

## 1. Purpose

This spec defines durable contracts for Homie's local LLM gateway: the virtual
key model, the wire protocols it serves to managed agents, upstream forwarding,
and usage recording. It is the long-lived authority for any change that affects
credential custody, virtual key issuance, or LLM proxying.

## 2. Authority

`homie-gateway` is the local HTTP gateway that managed agents (Codex, Claude
Code) call instead of a real provider. It:

- serves an OpenAI Responses API (`POST /v1/responses`) to Codex;
- serves an Anthropic Messages API (`POST /v1/messages`) to Claude Code;
- authenticates callers with a virtual key (or the gateway master key);
- forwards requests to a configured OpenAI-compatible upstream provider;
- records per-virtual-key usage locally.

It binds `127.0.0.1` only. It never exposes the upstream provider key to the
caller.

## 3. Virtual Key Model

- A virtual key is a random `sk-<uuid><uuid>` string issued per agent/session.
- Keys are stored locally with `id`, optional `label`, `created_at`,
  `last_used_at`. Only a SHA-256 hash of the key is stored (never the raw key);
  the raw key is returned exactly once at creation and never logged.
- `create`, `delete`, `list`, and `accepts` are the only operations.
- Keys persist across gateway restarts (SQLite), unlike the upstream reference's
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
- `POST /v1/messages`: Anthropic Messages wire shape, mapped to the upstream
  provider, streaming (SSE) preserved.
- The upstream provider key is attached server-side only; caller payload keys
  are ignored.

## 6. Upstream Forwarding

- Single OpenAI-compatible upstream: `base_url` + `api_key` from local ignored
  config (`homie.local.json` under `~/.config/homie/`, overridable with
  `HOMIE_CONFIG` / `HOMIE_CONFIG_DIR`). JSON is used (not TOML) so the Swift CLI
  and this Rust binary read and write the same bytes.
- Upstream errors are surfaced with a sanitized body; no upstream key or full
  sensitive prompt is echoed into logs.

## 7. Model Routing

- `homie.local.json` carries an optional `models` map (`codex` / `claude` → upstream model
  name). It is `#[serde(default)]`; absent or partial is valid.
- Before forwarding, the gateway rewrites the request body's top-level `model` string by
  route key: `POST /v1/responses` → `models["codex"]`, `POST /v1/messages` → `models["claude"]`.
- Override semantics: rewrite only when the corresponding `models` entry exists; otherwise
  pass the original `model` through unchanged. A non-JSON body or a missing/non-string
  top-level `model` is passed through unchanged (never an error).
- Usage recording uses the rewritten model, reflecting the model actually routed upstream.
- The gateway is the single source of truth for model routing; spawn-time injection does not
  separately set the model.

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
- Gateway restart restores virtual keys from SQLite.
- Port conflicts fail the gateway with a clear error; the port is configurable.

## 11. Credential Source

- `homie.local.json` may carry an optional `credentialSource` field; it is
  `#[serde(default)]` and absent means `static` (read `upstream.apiKey`), fully
  backward compatible.
- `credentialSource: "node"` makes the gateway resolve the upstream credential
  dynamically from `homie-node` instead of a static key. The gateway never reads
  provider auth files directly; credential extraction/refresh is owned by
  `homie-node`.
- `homie-node` exposes library functions (`homie_node::credentials`) — a
  restricted, in-process credential resolver: `resolve_default_codex_credential`
  and `resolve_codex_api_key`. It returns only a short-lived upstream token
  (`kind`, `base_url`, `token`) for a given account profile. It never returns
  refresh tokens and never exposes arbitrary file reads. This is library
  embedding, not a cross-process RPC; the gateway links `homie-node` as a crate
  dependency.
- Phase 1 resolves only the Codex API-key mode (`OPENAI_API_KEY` in the
  profile-scoped `config_home/auth.json`). Claude OAuth and Codex ChatGPT-login
  token refresh are Phase 2 and out of scope for this contract revision.
- Resolved short-lived tokens are held in memory only: never written to SQLite,
  never logged, never sent to managed agents.
- Fallback semantics: when `credentialSource == "node"`, a failed resolve
  (node unreachable, not authenticated, or unsupported mode) falls back to the
  static `upstream.apiKey` if configured; otherwise the gateway returns a clear
  `503` configuration-error body that leaks no key or account data.
- Failed resolve attempts are recorded in `gateway_audit` with reason
  `credential_resolve_failed` (no token material in the audit detail).

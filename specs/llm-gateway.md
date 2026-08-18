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

## 7. Usage Contract

- Each forwarded request records, per virtual key: `model`, `occurred_at`,
  `input_tokens`, `output_tokens` (estimated via `homie-usage::openai_estimate`
  when billed usage is absent).
- Usage is local and append-only; it is estimates, never authoritative billing.

## 8. Security And Recovery

- Upstream keys and virtual keys live only in local ignored files and local
  SQLite; never in git, logs, or agent-visible config.
- Gateway restart restores virtual keys from SQLite.
- Port conflicts fail the gateway with a clear error; the port is configurable.

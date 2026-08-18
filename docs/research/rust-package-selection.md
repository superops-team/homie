# Homie Rust Package Selection

## 1. Purpose

This document records the current Rust package choices relevant to Homie
architecture work. Read it before adding or replacing dependencies.

## 2. Current Core Choices

| Area | Package | Current Role | Notes |
|------|---------|--------------|-------|
| Native UI | `gpui` | Desktop app UI framework | Pinned to Zed revision in `homie/Cargo.toml`; GPUI is pre-1.0, so local pinned API is the authority |
| Platform backend | `gpui_platform`, patched `gpui_macos` | Windowing, macOS rendering, platform integration | Local patch avoids Command Line Tools-only Metal shader build failures and renderer issues |
| Async runtime | `tokio` | Daemon/client async runtime, app-side background orchestration | Use explicit task ownership for UI lifecycle work |
| HTTP server | `axum` | Local LLM gateway HTTP server (owner crate `homie-gateway`) | `json` feature; loopback-only bind |
| HTTP client | `reqwest` | Gateway upstream forwarding (owner crate `homie-gateway`) | `json` + `stream` features; SSE pass-through |
| Terminal engine | `alacritty_terminal` | Terminal state/emulation foundation | Keep PTY and terminal state outside GPUI render logic |
| File watching | `notify` | Transcript and filesystem invalidation | Debounce and reconcile before UI updates |
| Matching/search | `nucleo-matcher` | Fuzzy matching | Prefer shared helpers over ad hoc matching |
| Persistence | `rusqlite` | Local database support | Use clear schema/index contracts before expanding storage |
| Serialization | `serde`, `serde_json`, `plist` | Protocol and config serialization | Keep Rust/Swift wire contracts aligned with fixtures where possible |
| Security | `sha2`, `zeroize`, `getrandom` | Hashing, secret cleanup, IDs/randomness | Never log real credentials or virtual key secrets |
| Images/assets | `image` | Raster image handling | Avoid decoding large assets in render |

## 3. GPUI Version Policy

`homie/Cargo.toml` pins GPUI to a Zed Git revision. Do not copy current upstream
or examples blindly. Before using a GPUI API:

1. Search the current checkout for a compiling local example.
2. Check `homie/Cargo.lock` and `homie/Cargo.toml` for the pinned revision.
3. Prefer local patterns in `homie-app` and `homie-ui`.
4. If upgrading GPUI, create a separate PRD/spec and record migration risks.

## 4. Dependency Addition Policy

Before adding a package:

- inspect existing workspace dependencies;
- confirm the package is actively maintained;
- confirm license compatibility;
- define the owner crate and feature flags;
- add tests around the integration point;
- update this document if the package becomes part of the architecture.

Do not add a dependency to avoid writing a small pure helper unless the package
meaningfully reduces long-term complexity or provides proven domain logic.

## 5. UI And Accessibility Libraries

Homie currently relies on GPUI and platform APIs for UI and accessibility. New
UI primitives should first be implemented in `homie-ui` using the pinned GPUI
API. Do not add a parallel UI framework or webview-based component stack for
desktop surfaces.

## 6. Async And Blocking Work

Use Tokio and GPUI task APIs intentionally:

- blocking I/O and CPU-heavy work should not run in GPUI render paths;
- UI lifecycle tasks should be held by the owning entity;
- app-lifetime service loops should document why they are detached;
- stale async results should be rejected with generation or operation IDs.

## 7. Platform Notes

Homie currently targets macOS desktop packaging and local development first.
Cross-platform Rust crates should not assume macOS-only APIs unless guarded by
`cfg(target_os = "macos")` and documented in the relevant spec.

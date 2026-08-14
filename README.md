# homie

[![CI](https://github.com/cristicretu/homie/actions/workflows/ci.yml/badge.svg)](https://github.com/cristicretu/homie/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![GitHub release](https://img.shields.io/github/v/release/cristicretu/homie)](https://github.com/cristicretu/homie/releases/latest)

Native macOS orchestrator for coding agents. Run Claude Code, Codex, Cursor, Gemini and plain
shells in parallel — across git worktrees or on remote hosts — each with a live status
(working / needs-you / done) and tmux-like persistence: closing the app never kills a session,
and a daemon restart brings conversations back.

![homie](docs/images/homie.png)

## Install

```sh
brew install --cask cristicretu/homie/homie
```

Or download the latest DMG from [Releases](https://github.com/cristicretu/homie/releases/latest),
open it, and drag homie to Applications. Either way it is the same universal build (Apple
silicon and Intel), signed and notarized. homie updates itself from there.

The tap has to be named in full — a bare `homie` resolves only against Homebrew's default
taps. The cask lives in [cristicretu/homebrew-homie](https://github.com/cristicretu/homebrew-homie)
rather than `homebrew-cask`, which requires a notability threshold homie does not meet yet.

macOS 15 or newer.

## 60-second tour

1. Add a project directory and create a session for Claude Code, Codex, another
   supported agent, or a plain shell.
2. Start several sessions, ideally in separate git worktrees when they edit the
   same repository.
3. Watch the sidebar instead of every terminal: it shows which agents are
   working, waiting for you, or done.
4. Quit and reopen homie. The daemon keeps each PTY alive and replays the session
   when you return.

The [getting-started guide](docs/GETTING_STARTED.md) covers remote hosts, MCP
orchestration, diagnostics, local data, and uninstalling.

## What it does

- **Many agents at once.** Each session is a real terminal with a real PTY. Group them by
  project, split them across git worktrees, or run them on a remote host over ssh+tmux.
- **Status you can trust.** homie reads what an agent actually painted on its screen and tells
  you which ones are working, which are waiting on you, and which are done — so you can watch
  ten sessions without reading ten terminals.
- **Sessions outlive the app.** A background daemon owns the PTYs. Quit homie, reopen it, and
  everything is still there.
- **Agents can orchestrate agents.** An MCP server lets a running agent spawn another one,
  watch it, read its output, and answer its prompts.

First-class status detection and resume are Claude Code and Codex. Cursor and Gemini run with
partial support, and anything else runs as a terminal with running/exited status.

## Architecture

Two processes, one wire protocol:

- **`homie`** — the desktop app: Rust + [GPUI](https://github.com/zed-industries/zed). Owns the
  window, sidebar, terminal renderer, command palette, and usage accounting. Lives in
  [`homie/`](homie/).
- **`homied-rs`** — a headless Rust Engine, launched by the app and outliving it. Owns PTYs and
  child agent processes, an offset-addressed output log per session (for detach and replay), a
  headless terminal emulator for status detection, the session registry and persistence,
  worktrees, and the control socket.

`homie` is a small CLI: the MCP shim injected into agents, the hook and notify forwarders, and
`status`/`doctor`. `homie-holder` owns the PTY master so sessions survive an Engine restart.
All background process supervision is Rust-owned; Swift code is limited to shared protocol/CLI
support and platform glue where it is still needed.

## Adding an agent

Agent support is data, not code. Each agent is one JSON file in
`homie/crates/homie-engine/manifests/` describing how to spawn it, how to resume,
which keys approve or deny a prompt, and the screen rules that decide whether it
is working, waiting, or done. Copy the closest existing manifest and adjust it,
then run `scripts/sync-agent-manifests.sh` so the Swift CLI/Core resource mirror
stays aligned — no Swift or Rust required. This is the easiest way to
contribute.

## Building from source

Needs both toolchains: Rust (pinned in `homie/rust-toolchain.toml`) and Swift 6 with the Xcode
command-line tools. The first Rust build compiles GPUI from a pinned Zed revision and takes a
while.

```sh
swift build                                # Swift CLI/protocol support
(cd homie && cargo build)                  # Rust app + Engine
(cd homie && cargo run -p homie-app)        # run the app from source

homie/scripts/package.sh                    # full bundle
homie/scripts/install-local.sh
```

Run the same core checks as CI with one command:

```sh
./scripts/check.sh
```

[`homie/PACKAGING.md`](homie/PACKAGING.md) covers signing and notarization,
[`homie/UPDATING.md`](homie/UPDATING.md) the updater and release flow,
[`homie/NODE.md`](homie/NODE.md) running agents on a remote VPS node.

The [documentation index](docs/README.md) links user and engineering guides.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Bug reports, fixes, docs, and new agent
manifests are all welcome. New contributors can start with
[`good first issue`](https://github.com/cristicretu/homie/labels/good%20first%20issue)
or [`help wanted`](https://github.com/cristicretu/homie/labels/help%20wanted).

Questions belong in [Discussions](https://github.com/cristicretu/homie/discussions),
reproducible bugs in [Issues](https://github.com/cristicretu/homie/issues), and
vulnerabilities in [private security reports](SECURITY.md). See the
[roadmap](ROADMAP.md), [support guide](SUPPORT.md), [privacy notice](PRIVACY.md),
and [governance](GOVERNANCE.md) for project expectations.

## License

Homie's original source is Apache 2.0. Builds also contain third-party software
under its own licenses; see [LICENSE](LICENSE), [NOTICE](NOTICE), and the
machine-checked dependency policy in [`license-policy.json`](license-policy.json).

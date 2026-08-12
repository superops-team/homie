# Getting started

## Install

Homie requires macOS 15 or newer. Install the signed, notarized universal build:

```sh
brew install --cask cristicretu/homie/homie
```

Alternatively, download the latest DMG from [GitHub Releases](https://github.com/cristicretu/homie/releases/latest),
open it, and drag Homie to Applications. The app checks the same release feed for
updates; it never installs one until you click restart.

## Your first session

1. Open Homie and add a project directory.
2. Create a session and choose Claude Code, Codex, another supported agent, or a
   plain shell.
3. Work in the embedded terminal. The sidebar summarizes whether each agent is
   working, waiting for input, or done.
4. Quit and reopen Homie. The background daemon owns the PTY, so the session and
   its terminal history remain available.

Claude Code and Codex have first-class status detection and resume support.
Other agents may offer partial detection; every agent can still run as a normal
terminal.

## Parallel work with worktrees

Create separate sessions with separate git worktrees when agents may edit the
same repository. This keeps branches and working trees isolated while Homie
gives you one place to monitor them. Treat each worktree as a normal checkout:
commit or move changes before deleting it.

## Agent orchestration

Homie ships a CLI and MCP server. A running agent can create or inspect Homie
sessions when you explicitly configure the MCP server. That capability is
powerful: it can launch local processes with your user privileges. Only expose
it to agents you trust and review requested actions.

## Remote hosts

Remote sessions use SSH and tmux; Homie does not run a hosted relay. Start with
the [remote-node guide](../homie/NODE.md), use a dedicated non-admin account when
possible, and avoid forwarding credentials the remote job does not need.

## Diagnostics

Run this when the CLI is available:

```sh
homie doctor
```

Daemon logs and session state live under
`~/Library/Application Support/Homie`. Logs may contain terminal output,
paths, or secrets printed by a process, so redact them before filing an issue.
See [SUPPORT.md](../SUPPORT.md) for the details to include.

## Local data and uninstalling

After stopping Homie and its daemon, remove the app and these directories if you
also want to remove its local state:

```text
~/Library/Application Support/Homie
~/Library/Application Support/homie
~/Library/Caches/homie/updates
```

Read [PRIVACY.md](../PRIVACY.md) and the [security model](SECURITY-MODEL.md)
before using Homie with sensitive repositories or remote hosts.

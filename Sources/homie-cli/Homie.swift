import ArgumentParser
import Darwin
import HomieCore
import HomieProtocol
import Foundation

/// `homie <resource> <action> [target] [options]`.
///
/// The resource groups (`session`, `worktree`, `events`) are the automation
/// surface — everything the MCP tools can do, plus the event stream, reachable
/// from a shell. The flat commands below them are integration points that
/// predate the grammar and are invoked by other programs, not by people:
/// `hook`/`notify` are wired into the agents' own config files, so their
/// argv is fixed by those contracts.
/// `status` stays as a top-level alias of `session list` for the same reason.
@main
struct Homie: AsyncParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "homie",
        abstract: "Homie agent-orchestrator CLI.",
        subcommands: [
            Session.self, Worktree.self, Artifacts.self, Events.self,
            Status.self, Ports.self, Forward.self,
            Hook.self, Notify.self, Doctor.self,
            Config.self, Fix.self,
        ]
    )
}

// MARK: - hook

struct Hook: ParsableCommand {
    static let configuration = CommandConfiguration(
        abstract: "Forward a Claude Code hook event to the daemon (fail-open)."
    )

    @Argument(help: "The Claude hook event name (e.g. Stop, SessionStart).")
    var event: String

    func run() {
        // CRITICAL: fail-open. Any error still prints "{}" and exits 0, fast.
        let output = (try? performHook()) ?? "{}"
        print(output)
    }

    private func performHook() throws -> String {
        let stdinData = CLISupport.readStdin(cap: 1 << 20, timeoutMs: 500)
        let payload = CLISupport.parsePayload(stdinData)
        let params = HookReportParams(
            kind: "claude-hook",
            homieSessionID: CLISupport.sessionID(),
            event: event,
            payload: payload
        )
        let conn = try DaemonConn.connect()
        defer { conn.close() }
        let response = try conn.request(Method.hookReport, params: params)

        // SessionStart may carry a homie-assigned title to inject.
        if event == "SessionStart", let title = response["sessionTitle"]?.stringValue {
            let out = JSONValue.object([
                "hookSpecificOutput": .object([
                    "hookEventName": .string("SessionStart"),
                    "sessionTitle": .string(title),
                ])
            ])
            return CLISupport.encodeCompact(out)
        }
        return "{}"
    }
}

// MARK: - notify

struct Notify: ParsableCommand {
    static let configuration = CommandConfiguration(
        abstract: "Codex notify target: forward a notify payload to the daemon (fail-open)."
    )

    @Argument(parsing: .remaining, help: "The notify payload; the JSON string is the last argument.")
    var args: [String] = []

    func run() {
        // Fail-open: any error is swallowed, always exit 0.
        try? performNotify()
    }

    private func performNotify() throws {
        guard let jsonString = args.last else { return }
        let payload = CLISupport.parsePayload(Data(jsonString.utf8))
        let params = HookReportParams(
            kind: "codex-notify",
            homieSessionID: CLISupport.sessionID(),
            event: nil,
            payload: payload
        )
        let conn = try DaemonConn.connect()
        defer { conn.close() }
        _ = try conn.request(Method.hookReport, params: params)
    }
}

// MARK: - status

/// Predates the resource grammar and is in people's muscle memory and scripts,
/// so it keeps its exact output. Implemented as a thin alias over
/// `session list` rather than a copy, so the table only has one definition.
struct Status: ParsableCommand {
    static let configuration = CommandConfiguration(
        abstract: "Show a table of active sessions (alias of `session list`)."
    )

    @OptionGroup var output: OutputOptions

    func run() throws {
        // `status` has always listed every record the daemon knows about;
        // `session list` hides archived ones by default. Keep the old view.
        try SessionListing.run(includeArchived: true, statusPrefix: nil, output: output)
    }
}

// MARK: - doctor

struct Doctor: ParsableCommand {
    static let configuration = CommandConfiguration(
        abstract: "Check the daemon socket, agent binaries, and state file."
    )

    func run() throws {
        var daemonOK = false

        // 1. Socket + hello round-trip.
        let socketPath = DaemonConn.socketPath()
        if FileManager.default.fileExists(atPath: socketPath) {
            do {
                let conn = try DaemonConn.connect()
                defer { conn.close() }
                let params = HelloParams(build: "homie-cli/0.1.0")
                let result = try conn.request(Method.hello, params: params)
                let hello = try result.decoded(as: HelloResult.self)
                print("✓ daemon reachable (build \(hello.build), pid \(hello.pid), proto \(hello.proto))")
                daemonOK = true
            } catch {
                print("✗ daemon unreachable (\(error))")
            }
        } else {
            print("✗ daemon socket missing at \(socketPath)")
        }

        // 2. Agent binaries on PATH.
        for binary in ["claude", "codex"] {
            if let path = CLISupport.which(binary) {
                print("✓ \(binary) found at \(path)")
            } else {
                print("✗ \(binary) not found on PATH")
            }
        }

        // 3. State file.
        let statePath = HomiePaths.stateFile.path
        if FileManager.default.fileExists(atPath: statePath) {
            print("✓ state file present at \(statePath)")
        } else {
            print("✗ state file missing at \(statePath)")
        }

        // 4-7. LLM gateway checks.
        let gatewayOK = runGatewayChecks()

        if !daemonOK || !gatewayOK {
            throw ExitCode.failure
        }
    }

    /// Checks the LLM gateway: reachability, upstream credentials, virtual-key
    /// effectiveness, and agent routing. Returns false if any check fails.
    private func runGatewayChecks() -> Bool {
        let config = HomieConfigStore.read()
        let listen = config?.gateway.listen ?? HomieConfigStore.defaultListen
        let (host, port) = GatewayProbe.splitListen(listen)
            ?? ("127.0.0.1", UInt16(7338))
        var ok = true

        // 4. Gateway reachability.
        if GatewayProbe.gatewayRunning(host: host, port: port) {
            print("✓ gateway reachable at \(host):\(port)")
        } else {
            print("✗ gateway not reachable at \(host):\(port)")
            ok = false
        }

        // 5. Upstream credentials present.
        let baseUrl = config?.upstream.baseUrl ?? ""
        let apiKey = config?.upstream.apiKey ?? ""
        if !baseUrl.isEmpty && !apiKey.isEmpty {
            print("✓ upstream configured (\(baseUrl), apiKey \(HomieConfigStore.mask(apiKey)))")
        } else {
            print("✗ upstream credential missing (baseUrl or apiKey empty)")
            ok = false
        }

        // 6. Virtual key effectiveness.
        let keys = HomieConfigStore.virtualKeys()
        if !keys.isEmpty {
            print("✓ \(keys.count) virtual key(s) issued")
        } else {
            print("✗ no virtual keys issued (daemon not initialized)")
            ok = false
        }

        // 7. Agent routing points at loopback gateway, not a public provider.
        let loopback = host == "127.0.0.1" || host == "localhost" || host.hasPrefix("127.")
        if loopback {
            print("✓ agent routing points at local gateway (\(host):\(port))")
        } else {
            print("✗ gateway listen is not loopback (\(host):\(port)); agents may route to a public provider")
            ok = false
        }

        return ok
    }
}

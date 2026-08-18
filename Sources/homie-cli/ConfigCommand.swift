import ArgumentParser
import Foundation
import HomieProtocol

/// `homie config` — view and edit Homie's LLM gateway configuration. All secret
/// output is masked; secrets are recorded only via stdin or environment.
struct Config: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "config",
        abstract: "View and edit Homie's LLM gateway configuration.",
        subcommands: [ConfigShow.self, ConfigGet.self, ConfigSet.self, ConfigAgent.self]
    )
}

// MARK: - show

struct ConfigShow: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "show",
        abstract: "Show gateway config with secrets masked."
    )

    func run() throws {
        guard HomieConfigStore.fileExists(), let config = HomieConfigStore.read() else {
            print("no config at \(HomieConfigStore.configPath)")
            print("run `homie config set` to create one")
            return
        }
        print("gateway.listen   \(config.gateway.listen)")
        print("upstream.baseUrl \(config.upstream.baseUrl)")
        print("upstream.apiKey  \(HomieConfigStore.mask(config.upstream.apiKey))")
        print("gateway.masterKey \(HomieConfigStore.mask(config.gateway.masterKey))")
        print("models.codex     \(config.models["codex"] ?? "")")
        print("models.claude    \(config.models["claude"] ?? "")")

        let keys = HomieConfigStore.virtualKeys()
        if keys.isEmpty {
            print("virtual keys     none (gateway not initialized)")
        } else {
            print("virtual keys     \(keys.count)")
            for key in keys {
                let used = key.lastUsedAt.map { " last_used=\($0)" } ?? " never-used"
                let label = key.label.map { " (\($0))" } ?? ""
                print("  \(key.id)\(label)\(used)")
            }
        }
    }
}

// MARK: - get

struct ConfigGet: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "get",
        abstract: "Print one config value (raw; secrets stay masked)."
    )

    @Argument(help: "Dot path, e.g. upstream.baseUrl, gateway.listen, models.codex.")
    var keyPath: String

    func run() throws {
        let config = HomieConfigStore.read() ?? HomieConfigStore.empty
        let value: String
        switch keyPath {
        case "gateway.listen": value = config.gateway.listen
        case "upstream.baseUrl": value = config.upstream.baseUrl
        case "upstream.apiKey": value = HomieConfigStore.mask(config.upstream.apiKey)
        case "gateway.masterKey": value = HomieConfigStore.mask(config.gateway.masterKey)
        case "models.codex": value = config.models["codex"] ?? ""
        case "models.claude": value = config.models["claude"] ?? ""
        default:
            FileHandle.standardError.write(Data("homie: unknown key path: \(keyPath)\n".utf8))
            throw ExitCode.failure
        }
        print(value)
    }
}

// MARK: - set

struct ConfigSet: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "set",
        abstract: "Set gateway config. Secrets via --api-key-from-stdin / --master-key-from-stdin."
    )

    @Option(help: "Gateway listen address, e.g. 127.0.0.1:7338.")
    var listen: String?

    @Option(name: .customLong("base-url"), help: "Upstream OpenAI-compatible base URL.")
    var baseUrl: String?

    @Flag(name: .customLong("api-key-from-stdin"), help: "Read upstream apiKey from stdin.")
    var apiKeyFromStdin = false

    @Flag(name: .customLong("master-key-from-stdin"), help: "Read gateway masterKey from stdin.")
    var masterKeyFromStdin = false

    @Option(name: .customLong("model-codex"), help: "Codex model id.")
    var modelCodex: String?

    @Option(name: .customLong("model-claude"), help: "Claude model id.")
    var modelClaude: String?

    func run() throws {
        var config = HomieConfigStore.read() ?? HomieConfigStore.empty
        if let listen { config.gateway.listen = listen }
        if let baseUrl { config.upstream.baseUrl = baseUrl }
        if let modelCodex { config.models["codex"] = modelCodex }
        if let modelClaude { config.models["claude"] = modelClaude }

        if apiKeyFromStdin {
            config.upstream.apiKey = ConfigSet.readSecret(fromStdin: true, envVar: "HOMIE_UPSTREAM_API_KEY") ?? config.upstream.apiKey
        } else if let fromEnv = ConfigSet.readSecret(fromStdin: false, envVar: "HOMIE_UPSTREAM_API_KEY") {
            config.upstream.apiKey = fromEnv
        }
        if masterKeyFromStdin {
            config.gateway.masterKey = ConfigSet.readSecret(fromStdin: true, envVar: "HOMIE_MASTER_KEY") ?? config.gateway.masterKey
        } else if let fromEnv = ConfigSet.readSecret(fromStdin: false, envVar: "HOMIE_MASTER_KEY") {
            config.gateway.masterKey = fromEnv
        }

        try HomieConfigStore.write(config)
        print("wrote \(HomieConfigStore.configPath)")
    }

    /// Reads a secret from stdin when `fromStdin` is set, otherwise from `envVar`.
    static func readSecret(fromStdin: Bool, envVar: String) -> String? {
        if fromStdin {
            let data = CLISupport.readStdin(cap: 1 << 16, timeoutMs: 5000)
            let s = String(decoding: data, as: UTF8.self)
                .trimmingCharacters(in: .whitespacesAndNewlines)
            return s.isEmpty ? nil : s
        }
        if let v = ProcessInfo.processInfo.environment[envVar], !v.isEmpty { return v }
        return nil
    }
}

// MARK: - agent

struct ConfigAgent: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "agent",
        abstract: "Preview the exact injection a codex/claude spawn receives."
    )

    @Argument(help: "Agent: codex or claude.")
    var agent: String

    @Flag(name: .long, help: "Emit human-readable lines instead of JSON.")
    var text = false

    func run() throws {
        guard ["codex", "claude"].contains(agent.lowercased()) else {
            FileHandle.standardError.write(Data("homie: unknown agent \(agent)\n".utf8))
            throw ExitCode.failure
        }
        guard let binary = ConfigAgent.gatewayBinary() else {
            FileHandle.standardError.write(
                Data("homie: homie-gateway binary not found; cannot preview injection\n".utf8))
            throw ExitCode.failure
        }

        let process = Process()
        process.executableURL = URL(fileURLWithPath: binary)
        process.arguments = ["inject", "--agent", agent.lowercased()]
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = FileHandle.nullDevice
        do { try process.run() } catch {
            FileHandle.standardError.write(Data("homie: failed to run gateway: \(error)\n".utf8))
            throw ExitCode.failure
        }
        let out = pipe.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()
        guard process.terminationStatus == 0, let raw = String(data: out, encoding: .utf8) else {
            FileHandle.standardError.write(Data("homie: gateway inject failed\n".utf8))
            throw ExitCode.failure
        }

        if text {
            print(ConfigAgent.humanize(raw))
        } else {
            print(raw.trimmingCharacters(in: .whitespacesAndNewlines))
        }
    }

    /// Locate the Rust `homie-gateway` binary: installed bin dir, then a sibling
    /// of the CLI (dev builds), then PATH.
    static func gatewayBinary() -> String? {
        let candidates = [
            HomiePaths.binDir.appendingPathComponent("homie-gateway").path,
            URL(fileURLWithPath: CommandLine.arguments[0])
                .deletingLastPathComponent().appendingPathComponent("homie-gateway").path,
        ]
        for path in candidates where FileManager.default.isExecutableFile(atPath: path) {
            return path
        }
        return CLISupport.which("homie-gateway")
    }

    /// Minimal human-readable rendering of the JSON injection preview.
    static func humanize(_ raw: String) -> String {
        guard let data = raw.data(using: .utf8),
            let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
            let agent = obj["agent"] as? String
        else { return raw }
        var lines = ["agent: \(agent)"]
        if let args = obj["args"] as? [String], !args.isEmpty {
            lines.append("args:")
            lines.append(contentsOf: args.map { "  \($0)" })
        }
        if let env = obj["env"] as? [[String]] {
            lines.append("env:")
            lines.append(contentsOf: env.map { "  \($0[0])=\($0[1])" })
        }
        return lines.joined(separator: "\n")
    }
}

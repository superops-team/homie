import ArgumentParser
import Foundation
import HomieProtocol

/// `homie config` — view and edit Homie's LLM gateway configuration. All secret
/// output is masked; secrets are recorded only via stdin or environment.
struct Config: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "config",
        abstract: "View and edit Homie's LLM gateway configuration.",
        subcommands: [ConfigShow.self, ConfigGet.self, ConfigSet.self]
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

    func run() throws {
        var config = HomieConfigStore.read() ?? HomieConfigStore.empty
        if let listen { config.gateway.listen = listen }
        if let baseUrl { config.upstream.baseUrl = baseUrl }
        if let modelCodex { config.models["codex"] = modelCodex }

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

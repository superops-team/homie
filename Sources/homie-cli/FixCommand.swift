import ArgumentParser
import Foundation

/// `homie fix` — a finite, idempotent set of repairs. Every action probes first
/// and is skipped when healthy. It never silently fills a real credential and
/// never auto-spawns the daemon.
struct Fix: ParsableCommand {
    static let configuration = CommandConfiguration(
        abstract: "Idempotently repair common gateway config problems."
    )

    func run() throws {
        // 1. Config drift: missing or corrupt file.
        let configExists = HomieConfigStore.fileExists()
        let config = HomieConfigStore.read()
        if !configExists || config == nil {
            let why = configExists ? "corrupt" : "missing"
            print("fix: config \(why) at \(HomieConfigStore.configPath); rebuilding minimal")
            try HomieConfigStore.write(HomieConfigStore.empty)
        } else {
            print("skip: config present and valid")
        }

        var current = HomieConfigStore.read() ?? HomieConfigStore.empty

        // 2. Missing upstream apiKey (guide, never fill).
        if current.upstream.apiKey.isEmpty {
            print("fix: upstream apiKey missing — set with `homie config set --api-key-from-stdin`")
        } else {
            print("skip: upstream apiKey set")
        }

        let (host, port) = GatewayProbe.splitListen(current.gateway.listen)
            ?? ("127.0.0.1", UInt16(7338))

        // 3. Gateway not running (report, never auto-spawn).
        if GatewayProbe.gatewayRunning(host: host, port: port) {
            print("skip: gateway running")
        } else {
            print("fix: gateway not running — start with `homie-gateway` (not auto-spawned)")
        }

        // 4. Port conflict: a foreign listener holds the configured port.
        if !GatewayProbe.gatewayRunning(host: host, port: port),
            GatewayProbe.portOccupied(host: host, port: port)
        {
            if let free = GatewayProbe.findFreePort(startingAt: port) {
                current.gateway.listen = "\(host):\(free)"
                try HomieConfigStore.write(current)
                print("fix: port \(port) occupied by foreign listener; moved listen to \(host):\(free)")
            } else {
                print("fix: port \(port) occupied and no free port found; set manually with `homie config set --listen`")
            }
        } else if GatewayProbe.gatewayRunning(host: host, port: port) {
            print("skip: no port conflict (gateway owns the port)")
        } else {
            print("skip: no port conflict (port free)")
        }
    }
}

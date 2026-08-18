import Darwin
import Foundation
import HomieProtocol

/// Shared network probes for the gateway: reachability (`/healthz`), port
/// occupancy, and free-port discovery. Used by both `doctor` and `fix`.
enum GatewayProbe {
    /// Ephemeral session: no on-disk URL cache, so the loopback health probe
    /// never emits Foundation's network-cache diagnostics to stderr.
    private static let ephemeral = URLSession(configuration: .ephemeral)

    /// Splits `"127.0.0.1:7338"` into host and port. IPv4 loopback is the only
    /// listen form the gateway accepts.
    static func splitListen(_ listen: String) -> (host: String, port: UInt16)? {
        guard let idx = listen.lastIndex(of: ":") else { return nil }
        let host = String(listen[..<idx])
        let portStr = String(listen[listen.index(after: idx)...])
        guard let port = UInt16(portStr) else { return nil }
        return (host, port)
    }

    /// True when the gateway answers `GET /healthz` with `ok` (short timeout).
    static func gatewayRunning(host: String, port: UInt16) -> Bool {
        guard let url = URL(string: "http://\(host):\(port)/healthz") else { return false }
        var request = URLRequest(url: url)
        request.timeoutInterval = 2
        let semaphore = DispatchSemaphore(value: 0)
        let flag = AtomicFlag()
        let task = Self.ephemeral.dataTask(with: request) { data, _, error in
            if error == nil, let data = data,
                String(decoding: data, as: UTF8.self)
                    .trimmingCharacters(in: .whitespacesAndNewlines) == "ok"
            {
                flag.set()
            }
            semaphore.signal()
        }
        task.resume()
        _ = semaphore.wait(timeout: .now() + 3)
        task.cancel()
        return flag.isSet
    }

    /// True when something accepts TCP on the port (gateway or foreign).
    static func portOccupied(host: String, port: UInt16) -> Bool {
        guard let conn = try? DaemonConn.connectTCP(host: host, port: port, connectTimeout: 1)
        else { return false }
        conn.close()
        return true
    }

    /// True when binding the loopback port fails with `EADDRINUSE`.
    static func bindFails(_ port: UInt16) -> Bool {
        let fd = socket(AF_INET, SOCK_STREAM, 0)
        if fd < 0 { return false }
        defer { close(fd) }
        var addr = sockaddr_in()
        addr.sin_family = sa_family_t(AF_INET)
        addr.sin_port = port.bigEndian
        addr.sin_addr.s_addr = inet_addr("127.0.0.1")
        let result = withUnsafePointer(to: &addr) {
            $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                bind(fd, $0, socklen_t(MemoryLayout<sockaddr_in>.stride))
            }
        }
        return result != 0
    }

    /// Returns a free loopback port at or after `startingAt` (wrapping at 65535).
    static func findFreePort(startingAt port: UInt16) -> UInt16? {
        var candidate = port
        for _ in 0..<128 {
            candidate = candidate == 65535 ? 1024 : candidate + 1
            if !bindFails(candidate) { return candidate }
        }
        return nil
    }
}


/// Thread-safe boolean used by the async health probe. A bare captured `var`
/// trips Swift 6 sendability; a lock makes the intent explicit.
private final class AtomicFlag: @unchecked Sendable {
    private let lock = NSLock()
    private var value = false

    func set() {
        lock.lock(); value = true; lock.unlock()
    }

    var isSet: Bool {
        lock.lock(); defer { lock.unlock() }
        return value
    }
}

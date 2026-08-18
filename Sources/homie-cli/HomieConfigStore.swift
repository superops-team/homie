import Foundation

#if canImport(SQLite3)
import SQLite3
#endif

/// The canonical local gateway config (`homie.local.json`), shared byte-for-byte
/// with the Rust `homie-gateway` binary. The Swift CLI reads and writes the same
/// JSON schema so a value set here is what the gateway loads at startup.
struct HomieLocalConfig: Codable, Equatable {
    var gateway: GatewaySection
    var upstream: UpstreamSection
    var models: [String: String]
}

struct GatewaySection: Codable, Equatable {
    var listen: String
    var masterKey: String?
}

struct UpstreamSection: Codable, Equatable {
    var baseUrl: String
    var apiKey: String
}

/// A read-only virtual-key row from the gateway SQLite store.
struct VirtualKeyRow: Equatable {
    let id: String
    let label: String?
    let lastUsedAt: Int64?
}

/// Atomic read/write of `homie.local.json`, secret masking, and read-only
/// access to the gateway's virtual-key SQLite. The CLI never writes SQLite —
/// key lifecycle belongs to the gateway.
enum HomieConfigStore {
    static let defaultListen = "127.0.0.1:7338"

    // MARK: paths

    static var configPath: String {
        if let p = ProcessInfo.processInfo.environment["HOMIE_CONFIG"], !p.isEmpty {
            return p
        }
        return FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".config/homie/homie.local.json").path
    }

    static var dbPath: String {
        URL(fileURLWithPath: configPath)
            .deletingLastPathComponent()
            .appendingPathComponent("gateway.sqlite3").path
    }

    // MARK: config object

    static var empty: HomieLocalConfig {
        HomieLocalConfig(
            gateway: GatewaySection(listen: defaultListen, masterKey: nil),
            upstream: UpstreamSection(baseUrl: "", apiKey: ""),
            models: ["codex": "", "claude": ""]
        )
    }

    /// Loads the config, returning `nil` when the file is absent or unparsable.
    /// Callers distinguish "absent" from "corrupt" via `fileExists` themselves.
    static func read() -> HomieLocalConfig? {
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: configPath)) else {
            return nil
        }
        return try? JSONDecoder().decode(HomieLocalConfig.self, from: data)
    }

    static func fileExists() -> Bool {
        FileManager.default.fileExists(atPath: configPath)
    }

    /// Writes atomically with owner-only 0600 permissions.
    static func write(_ config: HomieLocalConfig) throws {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try encoder.encode(config)
        let url = URL(fileURLWithPath: configPath)
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        let tmp = URL(fileURLWithPath: configPath + ".tmp")
        try data.write(to: tmp, options: [.atomic])
        try FileManager.default.setAttributes(
            [.posixPermissions: 0o600], ofItemAtPath: tmp.path)
        _ = try FileManager.default.replaceItemAt(
            url, withItemAt: tmp, backupItemName: nil, options: [])
    }

    // MARK: masking

    /// Never echo a real secret. Shows `***` for short/empty values and
    /// `***<last4>` otherwise.
    static func mask(_ secret: String?) -> String {
        guard let s = secret, !s.isEmpty else { return "***" }
        guard s.count > 4 else { return "***" }
        return "***\(s.suffix(4))"
    }

    // MARK: SQLite (read-only)

    /// Lists virtual keys from the gateway SQLite (id, label, last_used_at),
    /// read-only. Returns `[]` when the DB is absent or unreadable — the
    /// gateway has simply not initialized yet.
    static func virtualKeys() -> [VirtualKeyRow] {
        #if canImport(SQLite3)
        var db: OpaquePointer?
        guard sqlite3_open_v2(dbPath, &db, SQLITE_OPEN_READONLY, nil) == SQLITE_OK else {
            return []
        }
        defer { sqlite3_close(db) }

        var stmt: OpaquePointer?
        let sql = "SELECT id, label, last_used_at FROM gateway_api_keys ORDER BY created_at"
        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else {
            return []
        }
        defer { sqlite3_finalize(stmt) }

        var rows: [VirtualKeyRow] = []
        while sqlite3_step(stmt) == SQLITE_ROW {
            let id = String(cString: sqlite3_column_text(stmt, 0))
            let label = sqlite3_column_text(stmt, 1).map { String(cString: $0) }
            let lastUsed: Int64? =
                sqlite3_column_type(stmt, 2) == SQLITE_NULL
                ? nil : sqlite3_column_int64(stmt, 2)
            rows.append(VirtualKeyRow(id: id, label: label, lastUsedAt: lastUsed))
        }
        return rows
        #else
        return []
        #endif
    }
}

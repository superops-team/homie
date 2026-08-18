import ArgumentParser
import Foundation
import Testing

@testable import homie_cli

private func parse<T: ParsableCommand>(_ argv: [String], as type: T.Type) throws -> T {
    let command = try Homie.parseAsRoot(argv)
    return try #require(command as? T)
}

// MARK: - masking (security: never leak a real key)

@Test func maskShowsOnlyLastFourCharacters() {
    #expect(HomieConfigStore.mask("sk-abcdef123456") == "***3456")
    #expect(HomieConfigStore.mask("sk-1234") == "***1234")
}

@Test func maskCollapsesShortEmptyAndNilToStars() {
    #expect(HomieConfigStore.mask(nil) == "***")
    #expect(HomieConfigStore.mask("") == "***")
    #expect(HomieConfigStore.mask("abc") == "***")
}

// MARK: - JSON schema parity with the Rust gateway (camelCase)

@Test func configEncodesTheSharedCamelCaseSchema() throws {
    let config = HomieLocalConfig(
        gateway: GatewaySection(listen: "127.0.0.1:7338", masterKey: "master"),
        upstream: UpstreamSection(baseUrl: "https://api.openai.com/v1", apiKey: "sk-secret"),
        models: ["codex": "gpt-5.2-codex", "claude": "claude-sonnet-4-5"]
    )
    let data = try JSONEncoder().encode(config)
    let obj = try #require(try JSONSerialization.jsonObject(with: data) as? [String: Any])
    let gateway = try #require(obj["gateway"] as? [String: Any])
    let upstream = try #require(obj["upstream"] as? [String: Any])
    let models = try #require(obj["models"] as? [String: String])

    #expect(gateway["listen"] as? String == "127.0.0.1:7338")
    #expect(gateway["masterKey"] as? String == "master")
    #expect(upstream["baseUrl"] as? String == "https://api.openai.com/v1")
    #expect(upstream["apiKey"] as? String == "sk-secret")
    #expect(models["codex"] == "gpt-5.2-codex")
}

@Test func configDecodesTheSharedSchemaBack() throws {
    let raw = #"{"gateway":{"listen":"127.0.0.1:7338","masterKey":null},"upstream":{"baseUrl":"https://api.example.com/v1","apiKey":"sk-x"},"models":{"codex":"m1"}}"#
    let config = try JSONDecoder().decode(HomieLocalConfig.self, from: Data(raw.utf8))
    #expect(config.gateway.listen == "127.0.0.1:7338")
    #expect(config.gateway.masterKey == nil)
    #expect(config.upstream.baseUrl == "https://api.example.com/v1")
    #expect(config.upstream.apiKey == "sk-x")
    #expect(config.models["codex"] == "m1")
}

// MARK: - command grammar

@Test func configSubcommandsParse() throws {
    _ = try parse(["config", "show"], as: ConfigShow.self)
    let get = try parse(["config", "get", "upstream.baseUrl"], as: ConfigGet.self)
    #expect(get.keyPath == "upstream.baseUrl")

    let set = try parse(
        ["config", "set", "--base-url", "https://api.openai.com/v1", "--model-codex", "m1"],
        as: ConfigSet.self
    )
    #expect(set.baseUrl == "https://api.openai.com/v1")
    #expect(set.modelCodex == "m1")
    #expect(set.apiKeyFromStdin == false)

    let agent = try parse(["config", "agent", "codex"], as: ConfigAgent.self)
    #expect(agent.agent == "codex")
    #expect(agent.text == false)
}

@Test func fixCommandParses() throws {
    _ = try parse(["fix"], as: Fix.self)
}

@Test func gatewayListenSplitsHostAndPort() {
    let split = GatewayProbe.splitListen("127.0.0.1:7338")
    #expect(split?.host == "127.0.0.1")
    #expect(split?.port == 7338)
    #expect(GatewayProbe.splitListen("no-port") == nil)
}

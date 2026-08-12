// swift-tools-version: 6.0
import PackageDescription

// Shared Swift support for Homie's CLI and protocol tooling. The daemon and
// process supervisor live in Rust (`homie/crates/homie-engine`).
let package = Package(
    name: "homie",
    platforms: [.macOS(.v15)],
    products: [
        .library(name: "HomieProtocol", targets: ["HomieProtocol"]),
        .library(name: "HomieCore", targets: ["HomieCore"]),
        .executable(name: "homie", targets: ["homie-cli"]),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-argument-parser", from: "1.5.0"),
    ],
    targets: [
        // MARK: Shared
        // Agent manifests live here because the CLI and protocol tooling need
        // the descriptor half. Runtime detection is owned by the Rust Engine.
        .target(name: "HomieCore", resources: [.copy("Resources/manifests")]),
        .target(name: "HomieProtocol", dependencies: ["HomieCore"]),
        .target(name: "HomieMCP", dependencies: ["HomieProtocol", "HomieCore"]),

        // MARK: Executables
        .executableTarget(
            name: "homie-cli",
            dependencies: [
                "HomieProtocol", "HomieCore", "HomieMCP",
                .product(name: "ArgumentParser", package: "swift-argument-parser"),
            ]
        ),

        // MARK: Tests
        .testTarget(name: "HomieProtocolTests", dependencies: ["HomieProtocol"]),
        .testTarget(name: "HomieCoreTests", dependencies: ["HomieCore"]),
        .testTarget(
            name: "HomieCLITests",
            dependencies: ["homie-cli", "HomieCore", "HomieProtocol"]
        ),
    ]
)

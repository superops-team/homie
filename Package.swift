// swift-tools-version: 6.0
import PackageDescription

// The engine behind `homie`, the Rust + GPUI desktop app in `homie/`. This
// package builds the daemon (`homied`), the PTY holder, and the `homie`
// CLI; `homie/scripts/package.sh` copies all three into `homie.app`.
let package = Package(
    name: "homie",
    platforms: [.macOS(.v15)],
    products: [
        .library(name: "HomieProtocol", targets: ["HomieProtocol"]),
        .library(name: "HomieCore", targets: ["HomieCore"]),
        .library(name: "HomieClient", targets: ["HomieClient"]),
        .library(name: "HomieDetection", targets: ["HomieDetection"]),
        .executable(name: "homied", targets: ["homied"]),
        .executable(name: "homied-holder", targets: ["homied-holder"]),
        .executable(name: "homie", targets: ["homie-cli"]),
    ],
    dependencies: [
        .package(url: "https://github.com/migueldeicaza/SwiftTerm", from: "1.13.0"),
        .package(url: "https://github.com/apple/swift-argument-parser", from: "1.5.0"),
    ],
    targets: [
        // MARK: Shared
        // Agent manifests live here (not in HomieDetection) because every
        // layer needs the `agent` descriptor half — the CLI and the protocol
        // depend on HomieCore but not on the detection engine. Detection
        // reads the `rules` half out of the same files.
        .target(name: "HomieCore", resources: [.copy("Resources/manifests")]),
        .target(name: "HomieProtocol", dependencies: ["HomieCore"]),
        .target(name: "HomieClient", dependencies: ["HomieProtocol", "HomieCore"]),
        .target(name: "HomieDetection", dependencies: ["HomieCore"]),

        // MARK: Daemon side
        .target(name: "CHomiePTY"),
        .target(name: "HomieHolderKit", dependencies: ["CHomiePTY"]),
        .target(name: "HomieGit", dependencies: ["HomieCore"]),
        .target(name: "HomieMCP", dependencies: ["HomieProtocol", "HomieCore"]),
        .target(
            name: "HomieDaemonKit",
            dependencies: [
                "HomieProtocol", "HomieCore", "HomieDetection", "HomieGit",
                "CHomiePTY", "HomieHolderKit",
                .product(name: "SwiftTerm", package: "SwiftTerm"),
            ],
            linkerSettings: [.linkedLibrary("sqlite3")]
        ),

        // MARK: Executables
        .executableTarget(name: "homied", dependencies: ["HomieDaemonKit"]),
        .executableTarget(name: "homied-holder", dependencies: ["HomieHolderKit"]),
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
        .testTarget(name: "HomieDetectionTests", dependencies: ["HomieDetection"]),
        .testTarget(
            name: "HomieDaemonKitTests",
            dependencies: ["HomieDaemonKit", "HomieHolderKit", "HomieClient", "homied-holder"]
        ),
        .testTarget(
            name: "HomieCLITests",
            dependencies: ["homie-cli", "HomieCore", "HomieProtocol"]
        ),
    ]
)

// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "AshenFrontierMac",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(name: "AshenFrontierMac", targets: ["AshenFrontierMac"])
    ],
    targets: [
        .target(
            name: "AshenFrontierBridge",
            path: "Sources/AshenFrontierBridge",
            publicHeadersPath: "include"
        ),
        .executableTarget(
            name: "AshenFrontierMac",
            dependencies: ["AshenFrontierBridge"],
            path: "Sources/AshenFrontierMac",
            resources: [
                .copy("Shaders")
            ],
            linkerSettings: [
                .unsafeFlags(["-L", "../../target/release", "-lashen_frontier"])
            ]
        ),
    ]
)

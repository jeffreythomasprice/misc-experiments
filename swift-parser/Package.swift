// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "experiment",
    targets: [
        .executableTarget(name: "experiment"),
        .testTarget(name: "experiment-test", dependencies: ["experiment"]),
    ]
)

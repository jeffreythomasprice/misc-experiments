// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "swift-experiment",
    dependencies: [
        .package(url: "https://github.com/vapor/vapor.git", from: "4.96.0"),
        .package(url: "https://github.com/vapor/leaf.git", from: "4.3.0"),
    ],
    targets: [
        .executableTarget(
            name: "swift-experiment",
            dependencies: [
                .product(name: "Vapor", package: "vapor"),
                .product(name: "Leaf", package: "leaf"),
            ]
        )
    ]
)

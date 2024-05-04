// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "swift-experiment",
    dependencies: [
        .package(url: "https://github.com/vapor/vapor.git", from: "4.92.3"),
        .package(url: "https://github.com/vapor/leaf.git", from: "4.3.0"),
        .package(url: "https://github.com/vapor/jwt.git", from: "4.2.2"),
        .package(url: "https://github.com/stephencelis/SQLite.swift.git", from: "0.14.1"),
    ],
    targets: [
        .executableTarget(
            name: "swift-experiment",
            dependencies: [
                .product(name: "Vapor", package: "vapor"),
                .product(name: "Leaf", package: "leaf"),
                .product(name: "JWT", package: "jwt"),
                .product(name: "SQLite", package: "SQLite.swift"),
            ]
        )
    ]
)

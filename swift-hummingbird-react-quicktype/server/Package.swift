// swift-tools-version: 6.0
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Experiment",
    dependencies: [
        .package(url: "https://github.com/apple/swift-log", from: "1.6.2"),
        .package(url: "https://github.com/hummingbird-project/hummingbird.git", from: "2.0.0"),
        .package(url: "https://github.com/thebarndog/swift-dotenv.git", from: "2.0.0"),
    ],
    targets: [
        .executableTarget(
            name: "Experiment",
            dependencies: [
                .product(name: "Logging", package: "swift-log"),
                .product(name: "Hummingbird", package: "Hummingbird"),
                .product(name: "HummingbirdRouter", package: "Hummingbird"),
                .product(name: "SwiftDotenv", package: "swift-dotenv"),
            ]
        )
    ]
)

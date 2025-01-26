// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Experiment",
    dependencies: [
        .package(url: "https://github.com/apple/swift-log", from: "1.6.2"),
        .package(url: "https://github.com/thebarndog/swift-dotenv.git", from: "2.0.0"),
        .package(url: "https://github.com/soto-project/soto.git", from: "7.3.0"),
    ],

    targets: [
        .executableTarget(
            name: "Experiment",
            dependencies: [
                .product(name: "Logging", package: "swift-log"),
                .product(name: "SwiftDotenv", package: "swift-dotenv"),
                .product(name: "SotoECS", package: "soto"),
                .product(name: "SotoSTS", package: "soto"),
            ]
        )
    ]
)

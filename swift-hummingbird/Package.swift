// swift-tools-version: 6.0
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Experiment",
    dependencies: [
        .package(url: "https://github.com/apple/swift-log", from: "1.6.2"),
        .package(url: "https://github.com/hummingbird-project/hummingbird.git", from: "2.0.0"),
        .package(url: "https://github.com/hummingbird-project/swift-mustache", from: "2.0.0"),
        .package(url: "https://github.com/thebarndog/swift-dotenv.git", from: "2.0.0"),
        .package(url: "https://github.com/vapor/jwt-kit.git", from: "5.0.0"),
        .package(url: "https://github.com/hummingbird-project/hummingbird-postgres.git", from: "0.5.2"),
        .package(url: "https://github.com/vapor/postgres-nio.git", from: "1.23.0"),
        .package(url: "https://github.com/swift-server/swift-service-lifecycle.git", from: "2.0.0"),
    ],
    targets: [
        .executableTarget(
            name: "Experiment",
            dependencies: [
                .product(name: "Logging", package: "swift-log"),
                .product(name: "Hummingbird", package: "Hummingbird"),
                .product(name: "HummingbirdRouter", package: "Hummingbird"),
                .product(name: "Mustache", package: "swift-mustache"),
                .product(name: "SwiftDotenv", package: "swift-dotenv"),
                .product(name: "JWTKit", package: "jwt-kit"),
                .product(name: "HummingbirdPostgres", package: "hummingbird-postgres"),
                .product(name: "PostgresMigrations", package: "hummingbird-postgres"),
                .product(name: "PostgresNIO", package: "postgres-nio"),
                .product(name: "ServiceLifecycle", package: "swift-service-lifecycle"),
            ]
        ),
        .testTarget(
            name: "ExperimentTests",
            dependencies: [
                .byName(name: "Experiment")
            ]
        ),
    ]
)

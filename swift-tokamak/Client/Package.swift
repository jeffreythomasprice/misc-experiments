// swift-tools-version:5.8
import PackageDescription
let package = Package(
    name: "Client",
    platforms: [.macOS(.v11), .iOS(.v13)],
    products: [
        .executable(name: "Client", targets: ["Client"])
    ],
    dependencies: [
        .package(url: "https://github.com/TokamakUI/Tokamak", from: "0.11.0")
    ],
    targets: [
        .executableTarget(
            name: "Client",
            dependencies: [
                "ClientLibrary",
                .product(name: "TokamakShim", package: "Tokamak")
            ]),
        .target(
            name: "ClientLibrary",
            dependencies: []),
        .testTarget(
            name: "ClientLibraryTests",
            dependencies: ["ClientLibrary"]),
    ]
)
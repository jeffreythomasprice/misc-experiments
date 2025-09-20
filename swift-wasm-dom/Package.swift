// swift-tools-version: 6.2
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "swift-wasm-dom",
    dependencies: [
        .package(url: "https://github.com/swiftwasm/JavaScriptKit", from: "0.36.0")
    ],
    targets: [
        // Targets are the basic building blocks of a package, defining a module or a test suite.
        // Targets can depend on other targets in this package and products from dependencies.
        .executableTarget(
            name: "swift-wasm-dom",
            dependencies: [
                .product(name: "JavaScriptKit", package: "JavaScriptKit")
            ]
        )
    ]
)

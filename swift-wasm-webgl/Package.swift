// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "swift-wasm",
    products: [
        .executable(name: "swift-wasm", targets: ["swift-wasm"])
    ],
    dependencies: [
        .package(url: "https://github.com/swiftwasm/carton", from: "1.0.0"),
        .package(url: "https://github.com/swiftwasm/JavaScriptKit", from: "0.18.0"),
    ],
    targets: [
        .executableTarget(
            name: "swift-wasm",
            dependencies: [
                .product(name: "JavaScriptKit", package: "JavaScriptKit"),
                .product(name: "JavaScriptEventLoop", package: "JavaScriptKit"),
            ]
        )
    ]
)

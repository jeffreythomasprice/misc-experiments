// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Experiment",
    products: [
        .executable(name: "Experiment", targets: ["Experiment"])
    ],
    dependencies: [
        .package(url: "https://github.com/swiftwasm/carton", from: "1.0.0"),
        .package(url: "https://github.com/swiftwasm/JavaScriptKit", from: "0.18.0"),
    ],
    targets: [
        .executableTarget(
            name: "Experiment",
            dependencies: [
                .product(name: "JavaScriptKit", package: "JavaScriptKit"),
                .product(name: "JavaScriptEventLoop", package: "JavaScriptKit"),
            ]
        )
    ]
)

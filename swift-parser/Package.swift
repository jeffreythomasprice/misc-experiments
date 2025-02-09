// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Experiment",
    targets: [
        .target(
            name: "Experiment"
        ),
        .testTarget(
            name: "ExperimentTests",
            dependencies: ["Experiment"],
            path: "Tests"
        ),
    ]
)

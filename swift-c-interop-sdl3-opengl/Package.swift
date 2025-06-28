// swift-tools-version: 6.1

import PackageDescription

let package = Package(
    name: "Experiment",
    products: [
        .executable(
            name: "Experiment",
            targets: ["Experiment"],
        )
    ],
    targets: [
        .executableTarget(
            name: "Experiment",
            dependencies: [
                "CLib",
                "CSDL",
                "COpenGL",
            ],
        ),
        .testTarget(
            name: "ExperimentTests",
            dependencies: [
                "Experiment"
            ]
        ),
        .target(
            name: "CLib",
            publicHeadersPath: "./",
        ),
        .target(
            name: "CSDL",
            publicHeadersPath: ".",
            cSettings: [
                .unsafeFlags(["-L."])
            ],
        ),
        .target(
            name: "COpenGL",
            publicHeadersPath: ".",
        ),
    ]
)

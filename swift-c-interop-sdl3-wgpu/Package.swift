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
                "CWGPU",
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
            publicHeadersPath: ".",
        ),
        .target(
            name: "CSDL",
            publicHeadersPath: ".",
            // TODO cleanup cSettings?
            // cSettings: [
            // .headerSearchPath("-I.deps/SDL3-3.2.22/include")
            // .unsafeFlags([
            //     "-I../../.deps/SDL3/SDL3-3.2.16/include",
            //     "-L../../.deps/SDL3/SDL3-3.2.16/lib/x64",
            // ]),
            // ],
        ),
        .target(
            name: "CWGPU",
            publicHeadersPath: ".",
        ),
    ]
)

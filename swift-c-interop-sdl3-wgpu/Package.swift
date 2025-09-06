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
            resources: [
                .embedInCode("../../assets/shader.wsgl")
            ],
            cSettings: [
                .headerSearchPath("../../.deps/SDL3-3.2.22/include"),
                .headerSearchPath("../../.deps/wgpu-linux-x86_64-release/include"),
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
            linkerSettings: [
                .unsafeFlags(["-L", ".deps/SDL3-build"]),
                .linkedLibrary("SDL3"),
            ],
        ),
        .target(
            name: "CWGPU",
            publicHeadersPath: ".",
            linkerSettings: [
                .unsafeFlags(["-L", ".deps/wgpu-linux-x86_64-release/lib"]),
                .linkedLibrary("wgpu_native"),
            ],
        ),
    ]
)

// swift-tools-version: 6.1

import PackageDescription

let package = Package(
    name: "swift-c-interop",
    products: [
        .executable(
            name: "swift-c-interop",
            targets: ["swift-c-interop"],
        )
    ],
    targets: [
        .executableTarget(
            name: "swift-c-interop",
            dependencies: [
                "CLib",
                "CSDL",
                "COpenGL",
            ],
        ),
        .testTarget(
            name: "swift-c-interopTests",
            dependencies: [
                "swift-c-interop"
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

// swift-tools-version:5.9
import PackageDescription

let package = Package(
	name: "experiment",
	platforms: [
		.macOS(.v13)
	],
	dependencies: [
		.package(url: "https://github.com/vapor/vapor.git", from: "4.83.1"),
		.package(url: "https://github.com/vapor/leaf.git", from: "4.2.4"),
		.package(url: "https://github.com/vapor/jwt.git", from: "4.0.0"),
		.package(url: "https://github.com/stephencelis/SQLite.swift.git", from: "0.14.1"),
	],
	targets: [
		.executableTarget(
			name: "App",
			dependencies: [
				.product(name: "Leaf", package: "leaf"),
				.product(name: "Vapor", package: "vapor"),
				.product(name: "SQLite", package: "SQLite.swift"),
				.product(name: "JWT", package: "jwt"),
			]
		),
		.testTarget(
			name: "AppTests",
			dependencies: [
				.target(name: "App"),
				.product(name: "XCTVapor", package: "vapor"),

				// Workaround for https://github.com/apple/swift-package-manager/issues/6940
				.product(name: "Vapor", package: "vapor"),
				.product(name: "Leaf", package: "leaf"),
			]),
	]
)

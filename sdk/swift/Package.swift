// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "GateorixAdapter",
    platforms: [.macOS(.v13)],
    products: [
        .library(name: "GateorixAdapter", targets: ["GateorixAdapter"]),
    ],
    targets: [
        .target(
            name: "GateorixAdapter",
            path: "Sources/GateorixAdapter"
        ),
        .testTarget(
            name: "GateorixAdapterTests",
            dependencies: ["GateorixAdapter"],
            path: "Tests/GateorixAdapterTests"
        ),
    ]
)

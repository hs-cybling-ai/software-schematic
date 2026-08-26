// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "DiagramStudio",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "DiagramStudio", targets: ["DiagramStudio"])],
    targets: [
        .systemLibrary(
            name: "CSQLite",
            pkgConfig: "sqlite3",
            providers: [.brew(["sqlite3"]), .apt(["libsqlite3-dev"])]
        ),
        .executableTarget(
            name: "DiagramStudio",
            dependencies: ["CSQLite"],
            path: "DiagramStudio/Sources/DiagramStudio",
            resources: [.copy("Resources/Web")]
        ),
        .testTarget(
            name: "DiagramStudioTests",
            dependencies: ["DiagramStudio"],
            path: "DiagramStudio/Tests/DiagramStudioTests"
        )
    ]
)

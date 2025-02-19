use crate::core::graph::Graph;


pub(crate) async fn generate_package_swift(_graph: &Graph) -> String {
    format!(r#"// swift-tools-version:5.5

import PackageDescription

let package = Package(
    name: "Teo",
    platforms: [
        .macOS(.v12),
        .iOS(.v15)
    ],
    products: [
        .library(
            name: "Teo",
            targets: ["Teo"]),
    ],
    targets: [
        .target(
            name: "API",
            dependencies: [])
    ]
)
"#)
}

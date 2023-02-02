pub mod kotlin;
pub mod swift;
pub mod typescript;
pub mod dart;
pub mod csharp;

use std::path::{Path};

use async_trait::async_trait;
use crate::core::app::conf::ClientGeneratorConf;
use crate::generator::lib::generator::Generator;
use crate::core::graph::Graph;
use crate::generator::client::csharp::CSharpClientGenerator;
use crate::generator::client::dart::DartClientGenerator;
use crate::generator::client::kotlin::KotlinClientGenerator;
use crate::generator::client::swift::SwiftClientGenerator;
use crate::generator::client::typescript::TypeScriptClientGenerator;
use crate::generator::lib::path::relative_to_absolute;
use crate::parser::ast::client::{ClientLanguage};

#[async_trait]
pub(crate) trait ClientGenerator {
    fn module_directory_in_package(&self, client: &ClientGeneratorConf) -> String;
    async fn generate_module_files(&self, graph: &Graph, client: &ClientGeneratorConf, generator: &Generator) -> std::io::Result<()>;
    async fn generate_package_files(&self, graph: &Graph, client: &ClientGeneratorConf, generator: &Generator) -> std::io::Result<()>;
    async fn generate_main(&self, graph: &Graph, client: &ClientGeneratorConf, generator: &Generator) -> std::io::Result<()>;
}

pub(crate) async fn generate_client(graph: &Graph, client: &ClientGeneratorConf) -> std::io::Result<()> {
    match client.provider {
        ClientLanguage::TypeScript => generate_client_typed(TypeScriptClientGenerator::new(), graph, client).await,
        ClientLanguage::Swift => generate_client_typed(SwiftClientGenerator::new(), graph, client).await,
        ClientLanguage::Kotlin => generate_client_typed(KotlinClientGenerator::new(), graph, client).await,
        ClientLanguage::CSharp => generate_client_typed(CSharpClientGenerator::new(), graph, client).await,
        ClientLanguage::Dart => generate_client_typed(DartClientGenerator::new(), graph, client).await,
    }
}

async fn generate_client_typed<T: ClientGenerator>(client_generator: T, graph: &Graph, client: &ClientGeneratorConf) -> std::io::Result<()> {
    let dest = relative_to_absolute(&client.dest);
    let package = client.package;
    let mut module_dest = dest.clone();
    if package {
        let package_generator = Generator::new(dest);
        client_generator.generate_package_files(graph, client, &package_generator).await?;
        module_dest.push(Path::new(client_generator.module_directory_in_package(client).as_str()));
    }
    let module_generator = Generator::new(module_dest);
    client_generator.generate_module_files(graph, client, &module_generator).await?;
    client_generator.generate_main(graph, client, &module_generator).await?;
    Ok(())
}

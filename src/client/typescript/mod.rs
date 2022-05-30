use std::fmt::Error;
use crate::client::shared::{clear_directory, ensure_directory, generate_file};
use crate::client::typescript::src::index_ts::generate_index_ts;
use crate::core::graph::Graph;

pub mod src;
pub mod r#type;

pub async fn generate_typescript_package(graph: &'static Graph) -> std::io::Result<()> {
    ensure_directory("client").await?;
    clear_directory("client/typescript").await?;
    generate_file("client/typescript/index.ts", generate_index_ts(graph).await).await
}

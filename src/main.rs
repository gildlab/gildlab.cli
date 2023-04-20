pub mod author;
pub mod cli;
pub mod evm;
pub mod ipfs;
pub mod subgraph;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::main().await
}

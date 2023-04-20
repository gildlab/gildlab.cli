use crate::evm::network::Network;
use crate::ipfs::IPFSCID;
use crate::subgraph::Subgraph;
use clap::ArgMatches;
use strum::IntoEnumIterator;
use crate::author::AUTHORS;

pub static NAME: &str = "pins";
pub static ABOUT: &str = "Fetches all pins from all authors from all known subgraphs.";

pub async fn pins(_matches: &ArgMatches) -> anyhow::Result<()> {
    let authors: Vec<String> = AUTHORS.into_iter().map(|author| author.to_lowercase().into()).collect();
    tracing::info!("{:?}", authors);

    let tasks = Network::iter()
        .map(|network| crate::subgraph::pin::pins_from_subgraph(Subgraph { network }, authors.clone()));
    let pins: Vec<IPFSCID> = futures::future::join_all(tasks)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<Vec<IPFSCID>>>>()?
        .into_iter()
        .flatten()
        .collect();
    dbg!(pins.len());
    Ok(())
}

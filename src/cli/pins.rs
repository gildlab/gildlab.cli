use crate::author::authors::get_authors;
use crate::evm::network::Network;
use crate::ipfs::IPFSCID;
use crate::subgraph::Subgraph;
use clap::ArgMatches;
use std::collections::HashSet;
use strum::IntoEnumIterator;

pub static NAME: &str = "pins";
pub static ABOUT: &str = "Fetches all pins from all authors from all known subgraphs.";

pub async fn pins(_matches: &ArgMatches) -> anyhow::Result<()> {
    let authors_future = get_authors();

    // Await the authors future to get the result
    let authors_val = authors_future.await?;

    let authors: Vec<String> = authors_val
        .into_iter()
        .map(|author| author.to_lowercase().into())
        .collect();
    tracing::info!("{:?}", authors);

    let tasks = Network::iter().map(|network| {
        crate::subgraph::pin::pins_from_subgraph(Subgraph { network }, authors.clone())
    });
    let pins: HashSet<IPFSCID> = futures::future::join_all(tasks)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<Vec<IPFSCID>>>>()?
        .into_iter()
        .flatten()
        .collect();
    for pin in pins {
        println!("{}", bs58::encode(pin.to_bytes()).into_string());
    }
    Ok(())
}

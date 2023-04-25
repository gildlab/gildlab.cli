pub mod pin;
use once_cell::sync::Lazy;
use url::Url;

use crate::evm::network::Network;

static BASE_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://api.thegraph.com/subgraphs/name/gildlab/").unwrap());

static SUBGRAPH_PREFIX: &str = "offchainassetvault";

pub struct Subgraph {
    pub network: Network,
}

impl Subgraph {
    pub fn url(&self) -> anyhow::Result<Url> {
        Ok(BASE_URL.join(&format!("{}-{}", SUBGRAPH_PREFIX, self.network))?)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    pub fn subgraph_url() {
        assert_eq!(
            Subgraph {
                network: Network::Ethereum
            }
            .url()
            .unwrap(),
            Url::parse(
                "https://api.thegraph.com/subgraphs/name/gildlab/offchainassetvault-ethereum"
            )
            .unwrap()
        );
    }
}

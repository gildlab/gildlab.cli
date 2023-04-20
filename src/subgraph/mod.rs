pub mod pin;
use once_cell::sync::Lazy;
use url::Url;

use crate::evm::network::Network;

static BASE_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://api.thegraph.com/subgraphs/name/gildlab/offchainassetvault-").unwrap()
});

pub struct Subgraph {
    pub network: Network,
}

impl Subgraph {
    pub fn url(&self) -> anyhow::Result<Url> {
        Ok(BASE_URL.join(&self.network.to_string())?)
    }
}

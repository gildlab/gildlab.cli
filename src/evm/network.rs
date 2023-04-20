use strum_macros::{Display, EnumIter};

#[derive(Display, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Network {
    Goerli,
    Ethereum,
    Mumbai,
    Polygon,
}

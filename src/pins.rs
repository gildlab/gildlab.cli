use graphql_client::GraphQLQuery;
use graphql_client::Response;
use multihash::MultihashGeneric;

lazy_static! {
    static ref DEPLOYERS: Vec<String> =
        vec![
        "0x8058ad7c22fdc8788fe4cb1dac15d6e976127324".into(),
        "0xc0D477556c25C9d67E1f57245C7453DA776B51cf".into(),
        "0x6E37d34e35a5fF2f896eD9e76EC43e728adA1d18".into(),
        "0x2cb21fb0a2cebb57434b1a2b89c81e5f49cd484a".into(),
        "0xaa1decefc2b32ca6390c9773e4ecffe69a644ff7".into(),
        "0x627a12ce1f6d42c9305e03e83fe044e8c3c1a32c".into()
    ];
}

static URL: &str = "https://api.thegraph.com/subgraphs/name/gildlab/offchainassetvault-mumbai";

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema/schema.json",
    query_path = "src/graphql/pins.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct PinQuery;

type IPFSCID = MultihashGeneric<32>;

pub async fn pins() -> anyhow::Result<Vec<IPFSCID>> {
    let variables = pin_query::Variables {
        ids: Some((*DEPLOYERS).iter().map(|s| s.to_lowercase()).collect())
    };
    let request_body = PinQuery::build_query(variables);
    let client = reqwest::Client::new();
    let res = client.post(URL).json(&request_body).send().await?;
    let response_body: Response<pin_query::ResponseData> = res.json().await?;
    match response_body {
        Response { data: Some(pin_query::ResponseData{ hashes }), .. } => {
            Ok(hashes.into_iter().filter_map(|pin_query_hashes| {
                // Decode and drop any data that doesn't cleanly convert to a
                // multihash.
                bs58::decode(pin_query_hashes.hash).into_vec()
                .map_err(anyhow::Error::from)
                .and_then(|data| IPFSCID::from_bytes(&data).map_err(anyhow::Error::from)).ok()
        }).collect())
        },
        _ => {
            Ok(vec![])
        }
    }
}
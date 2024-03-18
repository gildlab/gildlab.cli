use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_cbor::from_slice;
use hex;
use serde_json::{Value};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    payload: Vec<u8>,
    magic_number: String,
}

async fn fetch_subgraph_dt(url: &str, query: &str) -> Result<serde_json::Value> {
    let client = Client::new();
    let req = client.post(url)
        .header("Content-Type", "application/json")
        .body(serde_json::json!({ "query": query }).to_string())
        .send()
        .await?;
    let text = req.text().await?;
    Ok(serde_json::from_str(&text)?)
}

async fn get_data(url: &str, query: &str) -> Result<serde_json::Value> {
    let data = fetch_subgraph_dt(url, query).await?;
    if let Some(errors) = data.get("errors") {
        println!("{:?}", errors);
    }
    Ok(data)
}

fn cbor_decode(encoded_str: &str) -> Result<Vec<u8>>{
    let extracted_substring = &encoded_str[18..]; //Remove rain meta magic_number
    let encoded = hex::decode(extracted_substring).expect("Error decoding hex string");
    let decoded: Payload = from_slice(&encoded)?;

    let addresses: Vec<String> = decoded.payload.chunks(20)
                              .map(|chunk| hex::encode(chunk))
                              .collect();

    println!("Decoded Ethereum addresses:");
    for address in addresses {
        println!("0x{}", address);
    }

   Ok(encoded)
}

pub async fn get_authors() -> Result<Vec<u8>> {
    let query = r#"
        query {
          metaV1S {
            meta
          }
        }
    "#;

    let url = "https://api.thegraph.com/subgraphs/name/ninokeldishvili/rain-metaboard";
    let res = get_data(url, query).await?;
    if let Some(Value::String(meta)) = res.get("data")
        .and_then(|data| data.get("metaV1S"))
        .and_then(|meta_v1s| meta_v1s.get(0))
        .and_then(|first_meta_v1| first_meta_v1.get("meta")) {
        let test = "0xff0a89c674ee7874a2677061796c6f616498a01880185818ad187c182218fd18c81878188f18e418cb181d18ac1518d618e91876121873182418c018d418771855186c182518c918d6187e181f18571824185c1874185318da1877186b185118cf186e183718d3184e183518a518ff182f1889186e18d918e7186e18c4183e1872188a18da181d1818182c18b2181f18b018a218ce18bb18571843184b181a182b188918c8181e185f184918cd1848184a18aa181d18ec18ef18c218b3182c18a618390c1897187318e418ec18ff18e6189a1864184f18f71862187a1218ce181f186d184218c91830185e0318e8183f18e0184418e818c318c118a3182c18be1418c818f31832183918db189618991842182b183718f0189a18a8186d189318bb188f18f618ba18a318e318dd186e18ee18bf188718af183918fc183518ee18cc18df121853187d18b5156c6d616769635f6e756d62657272307866666232363337363038633039653338";

        let accounts: Vec<u8> = cbor_decode(&test)?;

        Ok(accounts)
    } else {
        Err(anyhow!("Unable to fetch authors"))
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

#[tokio::test]
async fn test_fetch_subgraph_dt() {
    // Arrange
    let _m = mock("POST", "/graphql")
        .match_header("Content-Type", "application/json")
        .with_body(r#"{"query":"test query"}"#)
        .create();

    // Act
    let result = fetch_subgraph_dt(&mockito::server_url(), "test query").await;

    // Assert
    assert!(result.is_ok());
    let json_value = result.unwrap();
    assert_eq!(json_value, json!({ "test": "response" }));
}

#[tokio::test]
async fn test_get_data() {
    // Mocking fetch_subgraph_dt
    let fetch_mock = mock("POST", "/graphql")
        .match_header("Content-Type", "application/json")
        .with_body(r#"{"data": { "key": "value" }}"#)
        .create();

    // Act
    let result = get_data(&mockito::server_url(), "test query").await;
    println!("{:?}", result);

    // Assert
    assert!(result.is_ok());
    let json_value = result.unwrap();
    assert_eq!(json_value, json!({ "key": "value" }));

    // Assert that the expected request was made
    fetch_mock.assert();
}

#[test]
fn test_cbor_decode() {
    // Test case for valid input
    let input = "12345";
    let result = cbor_decode(input);
    println!("{:?}", result);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![49, 50, 51, 52, 53]); // ASCII values of "12345"

    // Test case for invalid input
    let input = "abcde";
    let result = cbor_decode(input);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_authors() {
    // Mocking get_data
    let data_mock = mock("POST", "/graphql")
        .match_header("Content-Type", "application/json")
        .with_body(r#"{"data": { "metaV1S": [{ "meta": "12345" }] }}"#)
        .create();

    // Act
    let result = get_authors().await;
    println!("{:?}", result);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec![49, 50, 51, 52, 53]); // ASCII values of "12345"

    // Assert that the expected request was made
    data_mock.assert();
}
}
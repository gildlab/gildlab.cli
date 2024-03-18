use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_cbor::from_slice;
use serde_json::json;
use mockito::mock;
use hex;
use serde_json::Value;


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

pub fn cbor_decode(data: &str) -> Result<Vec<u8>> {
dbg!(&data);
let test = "0xff0a89c674ee7874a200d84058a08058ad7c22fdc8788fe4cb1dac15d6e976127324c0d477556c25c9d67e1f57245c7453da776b51cf6e37d34e35a5ff2f896ed9e76ec43e728ada1d182cb21fb0a2cebb57434b1a2b89c81e5f49cd484aaa1decefc2b32ca6390c9773e4ecffe69a644ff7627a12ce1f6d42c9305e03e83fe044e8c3c1a32cbe14c8f33239db9699422b37f09aa86d93bb8ff6baa3e3dd6eeebf87af39fc35eeccdf12537db515011bffb2637608c0a000";
    let hex_decoded = hex::decode(&test.as_bytes()[2..])?;

    let decoded: serde_cbor::Value = from_slice(&hex_decoded[8..])?;
    dbg!(&hex_decoded[14..]);

        for chunk in hex_decoded[14..].chunks(20) {
            println!("Chunk: {:?}", chunk);
        }
    Ok(hex_decoded)
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

        let accounts: Vec<u8> = cbor_decode(&meta)?;
//         println!("{:?}", accounts);
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
    println!("{:?}", json_value);
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
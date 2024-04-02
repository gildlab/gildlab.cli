use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_cbor::from_slice;
use mockito::mock;
use serde_json::json;
use hex;
use serde_json::{Value};
use hex::FromHex;
use serde::{Serialize, Deserialize};
use rain_metadata::meta::{
                             RainMetaDocumentV1Item, ContentType, ContentEncoding, ContentLanguage,
                             KnownMagic
                         };
use rain_metadata::error::Error;


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

   Ok(encoded)
}

pub async fn get_authors() -> Result<Vec<String>> {
    let query = r#"
        query {
          metaV1S {
            meta
          }
        }
    "#;

    let url = "https://api.thegraph.com/subgraphs/name/ninokeldishvili/rain-metaboard";
    let res = get_data(url, query).await?;


//   let res = json!({
//         "data": {
//             "metaV1S": [
//                 {
//                     "meta": "0xff0a89c674ee7874a3005501c0d477556c25c9d67e1f57245c7453da776b51cf011bffb2637608c09e3802706170706c69636174696f6e2f63626f72"
//                 },
//                 {
//                     "meta": "0xff0a89c674ee7874a3007901573078383035386164376332326664633837383866653463623164616331356436653937363132373332342c3078633044343737353536633235433964363745316635373234354337343533444137373642353163662c3078364533376433346533356135664632663839366544396537364543343365373238616441316431382c3078326362323166623061326365626235373433346231613262383963383165356634396364343834612c3078616131646563656663326233326361363339306339373733653465636666653639613634346666372c3078363237613132636531663664343263393330356530336538336665303434653863336331613332632c3078626531346338663333323339646239363939343232623337663039616138366439336262386666362c307862616133653364643665656562663837616633396663333565656363646631323533376462353135011bffb2637608c0a00002706170706c69636174696f6e2f6a736f6e"
//                 },
//                 {
//                     "meta": "0xff0a89c674ee7874a3005501aa1decefc2b32ca6390c9773e4ecffe69a644ff7011bffb2637608c09e3802706170706c69636174696f6e2f63626f72"
//                 }
//             ]
//         }
//     });

   let mut addresses: Vec<String> = Vec::new();
   let mut addresses2: Vec<String> = Vec::new();
if let Some(meta_v1s) = res["data"]["metaV1S"].as_array() {
        for item in meta_v1s {
            if let Some(meta_value) = item["meta"].as_str() {
                let extracted_substring = &meta_value[18..]; // Remove rain meta magic_number
                let bytes_array_meta = hex::decode(extracted_substring)?;

                match RainMetaDocumentV1Item::cbor_decode(&bytes_array_meta) {
                    Ok(cbor_decoded) => {
                        let payload = &cbor_decoded[0].payload;

                        if payload[0] == 1 {
                            let address_str: String = hex::encode(payload);
                            let modified_address = format!("0x{}", &address_str[2..]);
                            addresses.push(modified_address);
                        }
                    }
                    Err(err) => {
                        dbg!(&err);
                        continue;
                    }
                }
            }
        }
    }
        Ok(addresses)
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
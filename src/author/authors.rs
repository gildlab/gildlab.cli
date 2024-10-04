use anyhow::Result;
use hex;
use rain_metadata::meta::RainMetaDocumentV1Item;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    payload: Vec<u8>,
    magic_number: String,
}

async fn fetch_subgraph_dt(url: &str, query: &str) -> Result<serde_json::Value> {
    let client = Client::new();
    let req = client
        .post(url)
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

pub async fn get_authors() -> Result<Vec<String>> {
    let query = r#"
        query {
          metaV1S {
            meta
          }
        }
    "#;

    let url = env::var("ADDRESSES_SUBGRAPH_URL").expect("FETCH_URL not set");
    let res = get_data(&url, query).await?;

    let mut addresses: Vec<String> = Vec::new();
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
                            dbg!(&addresses);
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
